pub use ash::{Device, EntryCustom, Instance};

use ash::{vk, Entry};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use std::borrow::Cow;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::ops::Drop;
use std::sync::Arc;

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::zeroes();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

pub struct SurfaceProperties {
    pub format: vk::SurfaceFormatKHR,
    pub resolution: vk::Extent2D,
    pub desired_image_count: u32,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
}

pub struct Context {
    pub instance: Instance,
    pub surface_loader: Surface,
    pub surface: vk::SurfaceKHR,
    pub surface_properties: SurfaceProperties,
    pub window: Window,
    pub debug_utils_loader: DebugUtils,
    pub debug_callback: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: Device,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: Swapchain,
    pub command_pool: vk::CommandPool,
    pub setup_command_buffer: vk::CommandBuffer,
    pub draw_command_buffer: vk::CommandBuffer,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,
    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,
    pub setup_commands_reuse_fence: vk::Fence,
    pub draw_commands_reuse_fence: vk::Fence,
    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,
}

impl Context {
    pub fn new(title: String, width: u32, height: u32) -> (EventLoop<()>, Arc<Self>) {
        unsafe {
            let entry = Entry::new().unwrap();

            let event_loop = EventLoop::<()>::new();
            let window = create_window(&event_loop, &title, width, height);

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

            let instance = create_instance(&entry, &title, layers_names_raw, extension_names_raw);
            let surface = ash_window::create_surface(&entry, &instance, &window, None).unwrap();
            let surface_loader = Surface::new(&entry, &instance);
            let (debug_utils_loader, debug_callback) = create_debug(entry, instance.clone());

            let (physical_device, queue_family_index) = select_device(&instance, &surface, &surface_loader);
            let (logical_device, present_queue) = create_logical_device_and_queue(
                &instance, 
                &physical_device, 
                queue_family_index,
            );

            let surface_properties = get_surface_properties(
                &surface, 
                &surface_loader, 
                &physical_device, 
                width, 
                height
            );

            let (swapchain, swapchain_loader) = create_swapchain(
                &instance, 
                &physical_device,
                &logical_device,
                &surface, 
                &surface_loader, 
                &surface_properties,
            );

            let command_pool = create_command_pool(queue_family_index, &logical_device);
            let (setup_command_buffer, draw_command_buffer) = create_command_buffers(&logical_device, &command_pool);

            let (present_images, present_image_views) = create_present_images(
                &swapchain_loader, 
                &swapchain, 
                &logical_device, 
                surface_properties.format.format,
            );
            
            let (setup_commands_reuse_fence, draw_commands_reuse_fence) = create_fences(&logical_device);
            let (present_complete_semaphore, rendering_complete_semaphore) = create_semaphores(&logical_device);
        
            let device_memory_properties = instance.get_physical_device_memory_properties(physical_device);

            let (depth_image, depth_image_view, depth_image_memory) = create_depth_images(
                &logical_device,
                &setup_command_buffer,
                &setup_commands_reuse_fence,
                &present_queue,
                &surface_properties,
                &device_memory_properties,
            );

            (
                event_loop,
                Arc::new(
                    Self {
                        instance,
                        surface_loader,
                        surface,
                        surface_properties,
                        window,
                        debug_utils_loader,
                        debug_callback,
                        physical_device,
                        logical_device: logical_device,
                        device_memory_properties,
                        queue_family_index,
                        present_queue,
                        swapchain,
                        swapchain_loader,
                        command_pool,
                        setup_command_buffer,
                        draw_command_buffer,
                        present_images,
                        present_image_views,
                        depth_image,
                        depth_image_view,
                        depth_image_memory,
                        setup_commands_reuse_fence,
                        draw_commands_reuse_fence,
                        present_complete_semaphore,
                        rendering_complete_semaphore,
                    }
                )
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
            self.logical_device.free_memory(self.depth_image_memory, None);
            self.logical_device.destroy_image_view(self.depth_image_view, None);
            self.logical_device.destroy_image(self.depth_image, None);

            for &image_view in self.present_image_views.iter() {
                self.logical_device.destroy_image_view(image_view, None);
            }

            self.logical_device.destroy_command_pool(self.command_pool, None);
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.logical_device.destroy_device(None);
            self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_callback, None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_instance(
    entry: &Entry, 
    title: &str, 
    layers_names_raw: Vec<*const i8>, 
    extensions_names_raw: Vec<*const i8>,
) -> Instance {
    let name = CString::new(title.clone()).unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&name)
        .application_version(0)
        .engine_name(&name)
        .engine_version(0)
        .api_version(vk::make_version(1, 0, 0));

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extensions_names_raw);

    unsafe {
        entry
            .create_instance(&instance_create_info, None)
            .unwrap()
    }
}

fn create_window(event_loop: &EventLoop<()>, title: &str, width: u32, height: u32) -> Window {
    WindowBuilder::new()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(width as f64, height as f64))
        .build(&event_loop)
        .unwrap()
}

fn create_debug<E, I>(entry: E, instance: I) -> (DebugUtils, vk::DebugUtilsMessengerEXT)
where
    E: EntryV1_0,
    I: InstanceV1_0,
{
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR |
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .pfn_user_callback(Some(vulkan_debug_callback));

    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_callback =  unsafe { 
        debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .unwrap()
    };

    (debug_utils_loader, debug_callback)
}

fn select_device(instance: &Instance, surface: &vk::SurfaceKHR, surface_loader: &Surface) -> (vk::PhysicalDevice, u32) {
    unsafe {
        let p_devices = instance.enumerate_physical_devices().unwrap();
    
        let (p_device, queue_family_index) = p_devices
            .iter()
            .map(|p_device| {
                instance
                    .get_physical_device_queue_family_properties(*p_device)
                    .iter()
                    .enumerate()
                    .filter_map(|(index, info)| {
                        let supports_graphics = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                        let supports_surface = surface_loader
                            .get_physical_device_surface_support(
                                *p_device,
                                index as u32,
                                *surface,
                            )
                            .unwrap();
                        
                        if supports_graphics && supports_surface {
                            Some((*p_device, index))
                        } else {
                            None
                        }
                    }) 
                    .next()
            })
            .flatten()
            .next()
            .expect("No suitable device found");

        (p_device, queue_family_index as u32)
    }
}

fn create_logical_device_and_queue(
    instance: &Instance, 
    p_device: &vk::PhysicalDevice, 
    queue_family_index: u32,
) -> (Device, vk::Queue) {
    let device_extension_names_raw = [Swapchain::name().as_ptr()];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let priorities = [1.0];

    let queue_info = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)
            .build()
    ];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);
    
    let device = unsafe {
        instance
            .create_device(*p_device, &device_create_info, None)
            .unwrap()
    };

    let present_queue = unsafe {
        device.get_device_queue(queue_family_index as u32, 0)
    };

    (device, present_queue)
}

fn get_surface_properties(
    surface: &vk::SurfaceKHR, 
    surface_loader: &Surface, 
    p_device: &vk::PhysicalDevice,
    width: u32,
    height: u32,
) -> SurfaceProperties {
    unsafe {
        let surface_format = surface_loader
           .get_physical_device_surface_formats(*p_device, *surface)
           .unwrap()[0];

        let surface_capabilities = surface_loader
            .get_physical_device_surface_capabilities(*p_device, *surface)
            .unwrap();

        let mut desired_image_count = surface_capabilities.min_image_count + 1;

        if surface_capabilities.max_image_count > 0 
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width,
                height,
            },
            _ => surface_capabilities.current_extent,
        };

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };

        SurfaceProperties {
            format: surface_format,
            resolution: surface_resolution,
            desired_image_count,
            pre_transform,
        }
    }
}

fn create_swapchain<D>(
    instance: &Instance, 
    p_device: &vk::PhysicalDevice,
    l_device: &D, 
    surface: &vk::SurfaceKHR, 
    surface_loader: &Surface,
    surface_properties: &SurfaceProperties,
) -> (vk::SwapchainKHR, Swapchain)
where
    D: DeviceV1_0,
{
    unsafe {
        let present_modes = surface_loader
            .get_physical_device_surface_present_modes(*p_device, *surface)
            .unwrap();

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_loader = Swapchain::new(instance, l_device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
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

        (
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap(),
            swapchain_loader
        )
    }
}

fn create_command_pool(queue_family_index: u32, logical_device: &Device) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);

    unsafe {
        logical_device
            .create_command_pool(&command_pool_create_info, None)
            .unwrap()
    }
}

fn create_command_buffers(logical_device: &Device, command_pool: &vk::CommandPool) -> (vk::CommandBuffer, vk::CommandBuffer) {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(2)
        .command_pool(*command_pool)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffers = unsafe {
        logical_device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .unwrap()
    };

    (command_buffers[0], command_buffers[1])
}

fn create_present_images(
    swapchain_loader: &Swapchain, 
    swapchain: &vk::SwapchainKHR, 
    l_device: &Device,
    format: vk::Format,
) -> (Vec<vk::Image>, Vec<vk::ImageView>) {
    unsafe {
        let present_images = swapchain_loader.get_swapchain_images(*swapchain).unwrap();

        let present_image_views = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
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

                l_device.create_image_view(&create_view_info, None).unwrap()
            })
            .collect::<Vec<_>>();

        (present_images, present_image_views)
    }
}

fn create_depth_images(
    l_device: &Device,
    setup_command_buffer: &vk::CommandBuffer,
    setup_commands_reuse_fence: &vk::Fence,
    present_queue: &vk::Queue,
    surface_properties: &SurfaceProperties,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
    unsafe {
        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(vk::Extent3D { 
                width: surface_properties.resolution.width,
                height: surface_properties.resolution.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = l_device.create_image(&depth_image_create_info, None).unwrap();
        let depth_image_memory_req = l_device.get_image_memory_requirements(depth_image);
        let depth_image_memory_index = find_memory_type_index(
            &depth_image_memory_req,
            memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .unwrap();

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(depth_image_memory_req.size)
            .memory_type_index(depth_image_memory_index);

        let depth_image_memory = l_device
            .allocate_memory(&depth_image_allocate_info, None)
            .unwrap();

        l_device.bind_image_memory(depth_image, depth_image_memory, 0).unwrap();

        record_submit_command_buffer(
            l_device,
            *setup_command_buffer, 
            *setup_commands_reuse_fence,
            *present_queue,
            &[],
            &[],
            &[],
            |device, setup_command_buffer| {
                let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                    .image(depth_image)
                    .dst_access_mask(
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ 
                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    )
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .layer_count(1)
                            .level_count(1)
                            .build()
                    );

                device.cmd_pipeline_barrier(
                    setup_command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barriers.build()],
                );
            },
        );

        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = l_device
            .create_image_view(&depth_image_view_info, None)
            .unwrap();

        (depth_image, depth_image_view, depth_image_memory)
    }
}

fn create_fences(l_device: &Device) -> (vk::Fence, vk::Fence) {
    let fence_create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    unsafe {
        let setup_commands_reuse_fence = l_device
            .create_fence(&fence_create_info, None)
            .unwrap();

        let draw_commands_reuse_fence = l_device
            .create_fence(&fence_create_info, None)
            .unwrap();

        (setup_commands_reuse_fence, draw_commands_reuse_fence)
    }
}

fn create_semaphores(l_device: &Device) -> (vk::Semaphore, vk::Semaphore) {
    let semaphore_create_info = vk::SemaphoreCreateInfo::default();

    unsafe {
        let present_complete_semaphore = l_device
            .create_semaphore(&semaphore_create_info, None)
            .unwrap();

        let rendering_complete_semaphore = l_device
            .create_semaphore(&semaphore_create_info, None)
            .unwrap();
        
        (present_complete_semaphore, rendering_complete_semaphore)
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

pub fn find_memory_type_index(
    memory_req: &vk::MemoryRequirements,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_properties.memory_types[..memory_properties.memory_type_count as _]
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
