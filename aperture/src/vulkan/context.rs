use crate::vulkan::debug::DebugInfo;
use crate::vulkan::image::Image;
use crate::vulkan::surface::Surface;

use ash::{Device, Entry, Instance, vk};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Swapchain;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use std::ffi::CString;
use std::ops::Drop;
use std::sync::Arc;

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = std::mem::zeroed();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

pub struct Context {
    pub entry: Entry,
    pub instance: Instance,
    pub logical_device: Arc<Device>,
    pub window: Window,
    pub debug: DebugInfo,

    pub physical_device: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface: Surface,

    pub swapchain_loader: Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub command_pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: Image,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub draw_commands_reuse_fence: vk::Fence,
    pub setup_commands_reuse_fence: vk::Fence,
}

impl Context {
    pub fn new(title: &str, width: u32, height: u32) -> (EventLoop<()>, Arc<Self>) {
        unsafe {
            let event_loop = EventLoop::new();

            let window = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    f64::from(width),
                    f64::from(height),
                ))
                .build(&event_loop)
                .unwrap();
            
            let entry = Entry::new().unwrap();

            let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            let layers_names_raw: Vec<*const i8> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let surface_extensions = ash_window::enumerate_required_extensions(&window).unwrap();
            let mut extension_names_raw = surface_extensions
                .iter()
                .map(|ext| ext.as_ptr())
                .collect::<Vec<_>>();
            extension_names_raw.push(DebugUtils::name().as_ptr());

            let app_name = CString::new(title).unwrap();
            let app_info = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(0)
                .engine_name(&app_name)
                .engine_version(0)
                .api_version(vk::make_api_version(0, 1, 0, 0));

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names_raw);

            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            let debug = DebugInfo::new(&entry, &instance);

            let mut surface = Surface::new(&entry, &instance, &window);
            
            let physical_devices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");

            let (physical_device, queue_family_index) = physical_devices
                .iter()
                .map(|physical_device| {
                    instance
                        .get_physical_device_queue_family_properties(*physical_device)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, info)| {
                            let supports_graphics = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                            let supports_surface = surface.ash_handle.get_physical_device_surface_support(
                                *physical_device,
                                index as u32, 
                                surface.vk_handle,
                            )
                            .unwrap();

                            if supports_graphics && supports_surface {
                                Some((*physical_device, index))
                            } else {
                                None
                            }
                        })
                        .next()
                })
                .flatten()
                .next()
                .expect("Couldn't find suitable device.");

            surface.init_properties(&physical_device, width, height);

            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities)
                .build()];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let logical_device = Arc::new(
                instance
                    .create_device(physical_device, &device_create_info, None)
                    .unwrap()
            );

            let present_queue = logical_device.get_device_queue(queue_family_index as u32, 0);
            
            let present_modes = surface.ash_handle
                .get_physical_device_surface_present_modes(physical_device, surface.vk_handle)
                .unwrap();

            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            let swapchain_loader = Swapchain::new(&instance, &logical_device);

            let surface_properties = surface.properties.as_ref().unwrap();
            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface.vk_handle)
                .min_image_count(surface_properties.desired_image_count)
                .image_color_space(surface_properties.format.color_space)
                .image_format(surface_properties.format.format)
                .image_extent(surface_properties.resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(surface_properties.pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let command_pool = logical_device.create_command_pool(&pool_create_info, None).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = logical_device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();
            let setup_command_buffer = command_buffers[0];
            let draw_command_buffer = command_buffers[1];

            let present_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
            let present_image_views: Vec<vk::ImageView> = present_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface.format().unwrap().format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    logical_device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect();

            let device_memory_properties = instance.get_physical_device_memory_properties(physical_device);
            
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_commands_reuse_fence = logical_device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.");
            let setup_commands_reuse_fence = logical_device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.");

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore = logical_device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = logical_device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            let depth_image = Image::new(
                logical_device.clone(),
                device_memory_properties,
                vk::ImageType::TYPE_2D,
                vk::Format::D16_UNORM,
                vk::Extent3D {
                    width: surface_properties.resolution.width,
                    height: surface_properties.resolution.height,
                    depth: 1,
                },
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::SampleCountFlags::TYPE_1,
                vk::ImageAspectFlags::DEPTH,
            )
            .with_transition(
                setup_command_buffer,
                setup_commands_reuse_fence,
                present_queue,
                vk::ImageAspectFlags::DEPTH,
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            );

            depth_image.transition();

            (
                event_loop,
                Arc::new(
                    Self {
                        entry,
                        instance,
                        logical_device,
                        queue_family_index,
                        physical_device,
                        device_memory_properties,
                        window,
                        debug,
                        surface, 
                        present_queue,
                        swapchain_loader,
                        swapchain,
                        present_images,
                        present_image_views,
                        command_pool,
                        draw_command_buffer,
                        setup_command_buffer,
                        depth_image,
                        present_complete_semaphore,
                        rendering_complete_semaphore,
                        draw_commands_reuse_fence,
                        setup_commands_reuse_fence,
                    },
                ),  
            )
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.device_wait_idle().unwrap();
            self.logical_device.destroy_semaphore(self.present_complete_semaphore, None);
            self.logical_device.destroy_semaphore(self.rendering_complete_semaphore, None);
            self.logical_device.destroy_fence(self.setup_commands_reuse_fence, None);
            self.logical_device.destroy_fence(self.draw_commands_reuse_fence, None);

            for &image_view in self.present_image_views.iter() {
                self.logical_device.destroy_image_view(image_view, None);
            }

            self.logical_device.destroy_command_pool(self.command_pool, None);
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            self.logical_device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

pub fn record_submit_command_buffer<F: FnOnce(&Device, vk::CommandBuffer)>(
    l_device: &Device,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        l_device
            .wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX)
            .unwrap();

        l_device
            .reset_fences(&[command_buffer_reuse_fence])
            .unwrap();

        l_device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .unwrap();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        l_device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .unwrap();

        f(l_device, command_buffer);

        l_device
            .end_command_buffer(command_buffer)
            .unwrap();

        let command_buffers = vec![command_buffer];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        l_device
            .queue_submit(
                submit_queue,
                &[submit_info.build()],
                command_buffer_reuse_fence,
            )
            .unwrap();
    }
}
