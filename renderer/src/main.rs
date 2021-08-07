mod draw_info;
mod pipelines;
mod shaders;

use self::pipelines::Pipeline;
use self::shaders::*;
use renderer_common::VPosNorm;

use cgmath::{Matrix4, Point3, Rad, Vector3};
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, SubpassContents,
};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::format::Format;
use vulkano::image::{view::ImageView, AttachmentImage, ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice, PhysicalDeviceType};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass};
use vulkano::swapchain::{self, Swapchain, SwapchainCreationError};
use vulkano::sync::{self, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;
use std::time::Instant;

const SCALE_STEP: f32 = 1.25;

#[tokio::main]
async fn main() {
    let (meshes, mut materials) = renderer_mesh::gltf::load("data/gltf/MetalRoughSpheresNoTextures.glb").unwrap();
    // let (meshes, mut materials) = renderer_mesh::gltf::load("data/gltf/Box.glb").unwrap();

    println!(
        "Loaded {} meshes, {} materials",
        meshes.len(),
        materials.len()
    );

    // Use the first material for now.
    let material = materials.remove(0);
    println!("Using material: {:?}", material);

    let ev_loop = EventLoop::new();
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, Version::V1_1, &extensions, None).unwrap()
    };

    let surface = WindowBuilder::new()
        .build_vk_surface(&ev_loop, instance.clone())
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

    let mut dimensions: [u32; 2] = surface.window().inner_size().into();

    // Create the swapchain.
    // The swapchain allocates the color buffers that will contain the image visible on the screen.
    // These images are then returned alongside the swapchain.
    let (mut swapchain, images) = {
        // Get the surface capabilities.
        let caps = surface.capabilities(physical_device).unwrap();

        // Get the alpha mode.
        // Here, the window is opaque.
        let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

        // Choose the internal format of the images.
        let format = caps.supported_formats[0].0;

        // The dimensions of the window, only used to initially set up the swapchain.
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::start(device.clone(), surface.clone())
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(composite_alpha)
            .build()
            .unwrap()
    };

    let mut draw_infos = vec![];
    for m in meshes {
        draw_infos.extend(draw_info::generate_from_mesh(device.clone(), &m));
    }

    // Now create a uniform buffer for the vertex shader.
    let uniform_buffer =
        CpuBufferPool::<vert::ty::Data>::new(device.clone(), BufferUsage::uniform_buffer());

    let frag_buffer =
        CpuBufferPool::<frag::ty::Data>::new(device.clone(), BufferUsage::uniform_buffer());

    // Create the shader modules.
    let vs = vert::Shader::load(device.clone()).unwrap();
    let fs = frag::Shader::load(device.clone()).unwrap();
    let depth = depth::Shader::load(device.clone()).unwrap();

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
                    store: Store,
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

    let mut pipeline_type = Pipeline::Shaded;
    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(
        device.clone(),
        &vs,
        &fs,
        &depth,
        &images,
        render_pass.clone(),
        pipeline_type,
    );
    let mut recreate_swapchain = false;
    let mut update_pipeline = false;

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
    let rotation_start = Instant::now();
    let mut scale = 1.0;

    ev_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }
            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(KeyboardInput {
                        scancode: _,
                        state: ElementState::Pressed,
                        virtual_keycode: Some(code),
                        ..
                    }),
                ..
            } => match code {
                VirtualKeyCode::D => {
                    pipeline_type = Pipeline::Depth;
                    update_pipeline = true;
                }
                VirtualKeyCode::S => {
                    pipeline_type = Pipeline::Shaded;
                    update_pipeline = true;
                }
                VirtualKeyCode::W => {
                    pipeline_type = Pipeline::Wireframe;
                    update_pipeline = true;
                }
                _ => {}
            },
            Event::DeviceEvent { event: DeviceEvent::MouseWheel { delta }, .. } => match delta {
                MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                    if delta_x > 0.0 || delta_y > 0.0 {
                        scale *= SCALE_STEP;
                    } else if delta_x < 0.0 || delta_y < 0.0 {
                        scale /= SCALE_STEP;
                    }
                }
                MouseScrollDelta::PixelDelta(delta) => {
                    if delta.x > 0.0 || delta.y > 0.0 {
                        scale *= SCALE_STEP;
                    } else if delta.x < 0.0 || delta.y < 0.0 {
                        scale /= SCALE_STEP;
                    }
                }
            }
            Event::RedrawEventsCleared => {
                // Update model rotations
                let elapsed = rotation_start.elapsed();
                let rotation =
                    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
                let rotation = Matrix4::from_angle_y(Rad(rotation as f32));

                for draw_info in &draw_infos {
                    draw_info.transform.lock().expect("poisoned lock").rotation = rotation;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if update_pipeline {
                    pipeline = pipeline_type.create(
                        device.clone(),
                        dimensions,
                        &vs,
                        &fs,
                        &depth,
                        render_pass.clone(),
                    );
                    update_pipeline = false;
                }

                if recreate_swapchain {
                    dimensions = surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match swapchain.recreate().dimensions(dimensions).build() {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    swapchain = new_swapchain;
                    let (new_pipeline, new_frame_buffers) = window_size_dependent_setup(
                        device.clone(),
                        &vs,
                        &fs,
                        &depth,
                        &new_images,
                        render_pass.clone(),
                        pipeline_type,
                    );

                    pipeline = new_pipeline;
                    framebuffers = new_frame_buffers;
                    recreate_swapchain = false;
                }

                let eye = [0.0003, 0.0003, 0.001];
                // let eye = [0.3, 0.3, 1.0];

                let uniform_buffer_subbuffer = {
                    // Note: flipping the cube here, since it was made for OpenGL.
                    let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                    let proj = cgmath::perspective(
                        Rad(std::f32::consts::FRAC_PI_2),
                        aspect_ratio,
                        0.00001,
                        100.0,
                    );

                    let view = Matrix4::look_at_rh(
                        Point3::new(eye[0], eye[1], eye[2]),
                        Point3::new(0.0, 0.0, 0.0),
                        Vector3::new(0.0, -1.0, 0.0),
                    );
                    let scale_matrix = Matrix4::from_scale(scale);

                    let uniform_data = vert::ty::Data {
                        view: Matrix4::from(view * scale_matrix).into(),
                        proj: proj.into(),
                    };

                    uniform_buffer.next(uniform_data).unwrap()
                };

                let frag_buffer_subbufer = {
                    let frag_data = frag::ty::Data { 
                        rotation: rotation.into(),
                        view_pos: eye,
                    };

                    frag_buffer.next(frag_data).unwrap()
                };

                let layout = pipeline.layout().descriptor_set_layout(0).unwrap();
                let set = Arc::new(
                    PersistentDescriptorSet::start(layout.clone())
                        .add_buffer(uniform_buffer_subbuffer)
                        .unwrap()
                        .add_buffer(frag_buffer_subbufer)
                        .unwrap()
                        .build()
                        .unwrap(),
                );

                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(swapchain::AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    device.clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    .begin_render_pass(
                        framebuffers[image_num].clone(),
                        SubpassContents::Inline,
                        vec![[0.05, 0.05, 0.05, 1.0].into(), 1f32.into()],
                    )
                    .unwrap();

                for draw_info in &draw_infos {
                    let push_constants = vert::ty::PushConstants {
                        model: draw_info.composed_transform().into(),
                    };

                    if draw_info.has_indices() {
                        builder
                            .draw_indexed(
                                pipeline.clone(),
                                &DynamicState::none(),
                                vec![draw_info.vertex_buffer.clone()],
                                draw_info.index_buffer.as_ref().unwrap().clone(),
                                set.clone(),
                                push_constants,
                                vec![],
                            )
                            .unwrap();
                    } else {
                        builder
                            .draw(
                                pipeline.clone(),
                                &DynamicState::none(),
                                vec![draw_info.vertex_buffer.clone()],
                                set.clone(),
                                push_constants,
                                vec![],
                            )
                            .unwrap();
                    }
                }
                
                builder.end_render_pass().unwrap();

                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(sync::FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => (),
        }
    });
}

/// Called during initialisation, and whenever the window is resized.
fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vert::Shader,
    fs: &frag::Shader,
    depth: &depth::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    pipeline: Pipeline,
) -> (
    Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
) {
    let dimensions = images[0].dimensions();

    let depth_buffer = ImageView::new(
        AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap(),
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

    let pipeline = pipeline.create(device, dimensions, vs, fs, depth, render_pass);

    (pipeline, framebuffers)
}
