use crate::render::shaders::*;
use crate::vulkan::Pipeline;

use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice, PhysicalDeviceType};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass};
use vulkano::swapchain::{
    self, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreationError,
};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

pub struct VulkanBase {
    // Vulkan
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface<Window>>,
    pub device: Arc<Device>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    pub queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    // TODO do we need pre-load all pipelines?
    pub pipeline_type: Pipeline,
    pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub environment_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    pub shaders: Shaders,

    pub recreate_swapchain: bool,
}

impl VulkanBase {
    pub fn new(title: String, width: u32, height: u32) -> (Self, EventLoop<()>) {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, Version::V1_1, &extensions, None).unwrap()
        };

        let event_loop = EventLoop::new();

        let surface = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(width, height))
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        // We need a `Swapchain` for rendering to a surface.
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        // Choose which physical device to use.
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| {
                // Ensure device list supports our extensions.
                DeviceExtensions::supported_by_device(p).intersection(&device_extensions)
                    == device_extensions
            })
            .filter_map(|p| {
                // Select a queue family that supports graphics operations, and surface rendering.
                p.queue_families()
                    .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                    .map(|q| (p, q))
            })
            // Assign the devices that pass the filters a score, and pick the lowest.
            .min_by_key(|(p, _)| match p.properties().device_type.unwrap() {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .unwrap();

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name.as_ref().unwrap(),
            physical_device.properties().device_type.unwrap(),
        );

        // Initialise the device. To do this we need to pass:
        //
        // - The physical device to connect to
        // - A list of optional features and extensions that we need.
        // - The list of queues that we are going to use.
        //
        // This then returns the device and a list of creates queues.
        let (device, mut queues) = Device::new(
            physical_device,
            &Features {
                fill_mode_non_solid: true,
                ..Features::none()
            },
            // Add any extensions that are required by the device to the extensions we want to enable.
            &DeviceExtensions::required_extensions(physical_device).union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();

        // We are only using one queue, so get the first element of the `queues` iterator.
        let queue = queues.next().unwrap();

        // Create the swapchain.
        // The swapchain allocates the color buffers that will contain the image visible on the screen.
        // These images are then returned alongside the swapchain.
        let (swapchain, images) = {
            // Get the surface capabilities.
            let caps = surface.capabilities(physical_device).unwrap();

            // Get the alpha mode.
            // Here, the window is opaque.
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

            // Choose the internal format of the images.
            let format = caps.supported_formats[0].0;
            println!("Formats: {:?}", caps.supported_formats);

            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .format(format)
                .dimensions([width, height])
                .usage(ImageUsage::color_attachment())
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .build()
                .unwrap()
        };

        // Create a render pass.
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        );

        // Load the shaders.
        let shaders = Shaders::new(device.clone());

        // Initialise the render pipeline and framebuffer for the current window size.
        //
        // This is also called whenever the window size changes, to make sure the render result
        // covers the entire window.
        //
        // The pipeline describes how the render should take place, detailing the shaders,
        // vertex buffer and attribute format, face culling, and viewports.
        //
        // The framebuffer is the render target.
        let pipeline_type = Pipeline::Shaded;
        let (pipeline, environment_pipeline, framebuffers) = window_size_dependent_setup(
            device.clone(),
            &shaders,
            &images,
            render_pass.clone(),
            pipeline_type,
        )
        .unwrap();

        (
            Self {
                instance,
                surface,
                device,
                swapchain,
                swapchain_images: images,
                render_pass,
                queue,
                pipeline_type,
                pipeline,
                environment_pipeline,
                framebuffers,
                shaders,
                recreate_swapchain: false,
            },
            event_loop,
        )
    }

    pub fn acquire_next_swapchain_image(
        &mut self,
    ) -> Option<(usize, SwapchainAcquireFuture<Window>)> {
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(swapchain::AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return None;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        Some((image_num, acquire_future))
    }

    pub fn resize_setup(&mut self) {
        let (new_swapchain, new_swapchain_images) = match self
            .swapchain
            .recreate()
            .dimensions(self.dimensions())
            .build()
        {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => return,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };

        self.swapchain = new_swapchain;

        if let Some((new_pipeline, new_environment_pipeline, new_framebuffers)) = window_size_dependent_setup(
            self.device.clone(),
            &self.shaders,
            &new_swapchain_images,
            self.render_pass.clone(),
            self.pipeline_type,
        ) {
            self.pipeline = new_pipeline;
            self.environment_pipeline = new_environment_pipeline;
            self.framebuffers = new_framebuffers;
            self.recreate_swapchain = false;
        } else {
            return;
        }
    }

    pub fn dimensions(&self) -> [u32; 2] {
        let size = self.surface.window().inner_size();
        [size.width, size.height]
    }
}

/// Called during initialisation, and whenever the window is resized.
fn window_size_dependent_setup(
    device: Arc<Device>,
    shaders: &Shaders,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    pipeline: Pipeline,
) -> Option<(
    Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
)> {
    let dimensions = images[0].dimensions();

    let depth_buffer = ImageView::new(
        match AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm) {
            Err(_) => return None,
            Ok(image) => image,
        },
    )
    .unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(view)
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>();

    let pipeline = pipeline.create(device.clone(), dimensions, shaders, render_pass.clone());
    let environment_pipeline = Pipeline::Cubemap.create(device.clone(), dimensions, shaders, render_pass.clone());

    Some((pipeline, environment_pipeline, framebuffers))
}
