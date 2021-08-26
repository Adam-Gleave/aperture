use crate::vulkan::debug::DebugInfo;
use crate::vulkan::image::Image;
use crate::vulkan::logical_device::LogicalDevice;
use crate::vulkan::physical_device::PhysicalDevice;
use crate::vulkan::surface::Surface;
use crate::vulkan::swapchain::Swapchain;

use ash::{Entry, Instance, vk};
use ash::extensions::ext::DebugUtils;
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
    pub logical_device: Arc<LogicalDevice>,
    pub physical_device: PhysicalDevice,
    pub window: Window,
    pub surface: Surface,
    pub debug: DebugInfo,
    pub swapchain: Swapchain,

    pub command_pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,
    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,
    pub draw_commands_reuse_fence: vk::Fence,
    pub setup_commands_reuse_fence: vk::Fence,

    pub depth_image: Image,
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

            let instance_create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names_raw);

            let instance: Instance = entry
                .create_instance(&instance_create_info, None)
                .expect("Instance creation error");

            let debug = DebugInfo::new(&entry, &instance);

            let mut surface = Surface::new(&entry, &instance, &window); 

            let physical_device = PhysicalDevice::new(&instance, &surface);
            
            surface.init_properties(&physical_device, width, height);
            let surface_properties = surface.properties.as_ref().unwrap();

            let logical_device = Arc::new(LogicalDevice::new(&instance, &physical_device));
            
            let swapchain = Swapchain::new(
                &instance, 
                &surface, 
                logical_device.clone(), 
                &physical_device,
            );

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(physical_device.present_queue_family_index);

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
                physical_device.memory_properties,
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
                        physical_device,
                        window,
                        debug,
                        surface, 
                        swapchain,
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

            self.logical_device.destroy_command_pool(self.command_pool, None);
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

pub fn record_submit_command_buffer<F: FnOnce(&LogicalDevice, vk::CommandBuffer)>(
    logical_device: &LogicalDevice,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        logical_device
            .wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX)
            .unwrap();

        logical_device
            .reset_fences(&[command_buffer_reuse_fence])
            .unwrap();

        logical_device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .unwrap();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        logical_device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .unwrap();

        f(logical_device, command_buffer);

        logical_device
            .end_command_buffer(command_buffer)
            .unwrap();

        let command_buffers = vec![command_buffer];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        logical_device
            .queue_submit(
                submit_queue,
                &[submit_info.build()],
                command_buffer_reuse_fence,
            )
            .unwrap();
    }
}
