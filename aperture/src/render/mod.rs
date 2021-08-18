mod base;
mod camera;
mod environment;
mod world_render;

pub mod shaders;

use crate::render::environment::Environment;
use crate::render::world_render::WorldRender; 
use crate::state::InputState;
use crate::world::World;
use crate::world::light::Light;

use base::VulkanBase;
use camera::Camera;
use shaders::*;

use cgmath::{Deg, Matrix4, Point3, Vector3, perspective};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, SubpassContents};
use vulkano::sync::{self, GpuFuture};
use winit::event_loop::EventLoop;

use std::convert::TryInto;

pub struct Renderer {
    pub base: VulkanBase,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,

    pub world_render: WorldRender,
    pub camera: Camera,
}

impl Renderer {
    pub fn new(title: String, width: u32, height: u32) -> (Self, EventLoop<()>) {
        let (base, event_loop) = VulkanBase::new(title, width, height);
        let previous_frame_end = Some(sync::now(base.device.clone()).boxed());

        (
            Self {
                base,
                previous_frame_end,
                world_render: WorldRender::default(),
                camera: Camera::new(
                    Point3::new(2.0, 0.5, 2.0),
                    Point3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                ),
            },
            event_loop,
        )
    }

    pub fn notify_resized(&mut self) {
        self.base.recreate_swapchain = true;
    }

    pub fn load_world(&mut self, world: &World) {
        self.world_render.update(
            world.meshes.values(),
            world.materials.values(),
            world.textures.values(),
            self.base.pipeline_type,
            self.base.pipeline.clone(),
            self.base.device.clone(),
            self.base.queue.clone(),
        );

        self.world_render.update_environment(
            self.base.environment_pipeline.clone(),
            &self.base.shaders,
            self.base.device.clone(),
            self.base.queue.clone(),
        );
    }

    pub fn update(&mut self, input_state: &InputState) {
        if let Some(delta) = input_state.position_delta {
            if input_state.mouse_left_down {
                let dimensions = self.base.dimensions();
                let theta_x = (2.0 * std::f32::consts::PI) / dimensions[0] as f32;
                let theta_y = std::f32::consts::PI / dimensions[1] as f32;
                let delta_x = delta[0] * theta_x;
                let delta_y = delta[1] * theta_y;

                self.camera.orbit(delta_x, delta_y);
            } else if input_state.mouse_right_down {
                self.camera.translate(delta[0], delta[1]);
            }
        }

        if let Some(delta) = input_state.wheel_delta {
            self.camera.zoom(delta);
        }
    }

    pub fn render(&mut self, world: &World) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Don't render anything if the window is minimised.
        let dimensions = self.base.dimensions();
        if dimensions[0] == 0 || dimensions[1] == 0 {
            return;
        }

        // Recreate the swapchain, pipeline and framebuffers if the window has been resized.
        if self.base.recreate_swapchain {
            self.base.resize_setup();
        }

        // Retrieve the index of the next available presentable image, and its future.
        // If there are none available, break out of this iteration of the render loop.
        let (image_num, acquire_future) = match self.base.acquire_next_swapchain_image() {
            Some((image_num, acquire_future)) => (image_num, acquire_future),
            None => return,
        };

        let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
        let proj = cgmath::perspective(Deg(60.0), aspect_ratio, 0.1, 100.0);
        let view = self.camera.view_matrix();

        // Start building the command buffer.
        let mut builder = AutoCommandBufferBuilder::primary(
            self.base.device.clone(),
            self.base.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // TODO why are we doing this for every primitive?
        // Can we do this in some dynamic buffer?
        let mut point_lights = [
            frag::ty::PointLight { 
                position: [0.0, 0.0, 0.0, 0.0], 
                color: [0.0, 0.0, 0.0, 0.0],
                power: [0, 0, 0, 0],
            }; 
            255 
        ];

        for (i, l) in world.lights.iter().enumerate() {
            point_lights[i] = 
                frag::ty::PointLight {
                    position: l.position(),
                    color: l.color(),
                    power: l.power(),
                };
        }

        // Update environment uniform buffers.
        if let Some(environment) = &self.world_render.environment {
            builder
                .update_buffer(
                    environment.skybox_uniform_buffer.clone(),
                    std::sync::Arc::new(cube_vert::ty::Data {
                        proj: proj.into(),
                        view: view.into(),
                    }),
                )
                .unwrap();

            let proj = perspective(Deg(90.0), 1.0, 0.1, 10.0);
            let views: [[[f32; 4]; 4]; 6] = [
                Matrix4::look_at_rh([0.0, 0.0, 0.0].into(), [ 1.0,  0.0,  0.0].into(), [0.0, -1.0,  0.0].into()).into(),
                Matrix4::look_at_rh([0.0, 0.0, 0.0].into(), [-1.0,  0.0,  0.0].into(), [0.0, -1.0,  0.0].into()).into(),
                Matrix4::look_at_rh([0.0, 0.0, 0.0].into(), [ 0.0,  1.0,  0.0].into(), [0.0,  0.0,  1.0].into()).into(),
                Matrix4::look_at_rh([0.0, 0.0, 0.0].into(), [ 0.0, -1.0,  0.0].into(), [0.0,  0.0, -1.0].into()).into(),
                Matrix4::look_at_rh([0.0, 0.0, 0.0].into(), [ 0.0,  0.0,  1.0].into(), [0.0, -1.0,  0.0].into()).into(),
                Matrix4::look_at_rh([0.0, 0.0, 0.0].into(), [ 0.0,  0.0, -1.0].into(), [0.0, -1.0,  0.0].into()).into(),
            ];

            builder
                .update_buffer(
                    environment.offscreen_cube_uniform_buffer.clone(),
                    std::sync::Arc::new(offscreen_cube_vert::ty::Data {
                        proj: proj.into(),
                        views,
                    }),
                )
                .unwrap();
        }

        // Update uniform buffers.
        for draw_info in &self.world_render.primitive_info {
            let set = self.world_render.material_info[draw_info.material_name.as_ref().unwrap()]
                .descriptor_set
                .clone();

            builder
                .update_buffer(
                    set.vertex_uniform_buffer.as_ref().unwrap().clone(),
                    std::sync::Arc::new(vert::ty::Data {
                        proj: proj.into(),
                        view: view.into(),
                    }),
                )
                .unwrap()
                .update_buffer(
                    set.fragment_uniform_buffer.as_ref().unwrap().clone(),
                    std::sync::Arc::new(frag::ty::Data {
                        view_pos: [
                            self.camera.eye.x,
                            self.camera.eye.y,
                            self.camera.eye.z,
                            0.0,
                        ],
                        lights: point_lights,
                    }),
                )
                .unwrap();
        }

        // Project the HDRI environment map to a cube.
        if let Some(environment) = &self.world_render.environment {
            for i in 0..Environment::CUBE_IMAGE_LAYERS {
                builder
                    .begin_render_pass(
                        environment.offscreen_framebuffer.clone(),
                        SubpassContents::Inline,
                        vec![[0.1, 0.1, 0.1, 1.0].into(), 1f32.into()],
                    )
                    .unwrap();

                let push_constants = offscreen_cube_vert::ty::VertPushConstants {
                    index: i,
                };

                builder.draw(
                    environment.offscreen_cube_pipeline.clone(),
                    &DynamicState::none(),
                    vec![environment.offscreen_cube_vertex_buffer.clone()],
                    environment.offscreen_cube_set.clone(),
                    push_constants,
                    vec![],
                )
                .unwrap();

                builder
                    .end_render_pass()
                    .unwrap();
                
                builder
                    .copy_image(
                        environment.framebuffer_image.clone(),
                        [0, 0, 0],
                        0, 
                        0,
                        environment.cubemap_image.clone(),
                        [0, 0, 0],
                        i,
                        0,
                        [Environment::CUBE_DIMENSIONS[0], Environment::CUBE_DIMENSIONS[1], 1],
                        1,
                    )
                    .unwrap();
            }
        }

        builder
            .begin_render_pass(
                self.base.framebuffers[image_num].clone(),
                SubpassContents::Inline,
                vec![[0.1, 0.1, 0.1, 1.0].into(), 1f32.into()],
            )
            .unwrap();

        for draw_info in &self.world_render.primitive_info {
            let vert_push_constants = vert::ty::VertPushConstants {
                model: draw_info.composed_transform().into(),
            };

            let material = if let Some(name) = &draw_info.material_name {
                &world.materials[name.as_str()]
            } else {
                &world.default_material
            };

            let frag_push_constants = frag::ty::FragPushConstants {
                _dummy0: [0u8; 64],
                base_color: material.base_color_factor.into(),
                metalness: material.metallic_factor,
                roughness: material.roughness_factor,
                reflectance: material.reflectance,
                point_light_count: world.lights.len() as u32,
            };

            // FIXME
            let vert_data = unsafe {
                std::mem::transmute::<vert::ty::VertPushConstants, [u8; 64]>(vert_push_constants)
            };

            let frag_data = unsafe {
                std::mem::transmute::<frag::ty::FragPushConstants, [u8; 96]>(frag_push_constants)
            };

            let mut data_vec = vert_data.to_vec();
            data_vec.extend(frag_data.iter().skip(64));
            let push_constants: [u8; 96] = data_vec.try_into().unwrap();

            let set = self.world_render.material_info[draw_info.material_name.as_ref().unwrap()]
                .descriptor_set
                .clone();

            if draw_info.has_indices() {
                builder
                    .draw_indexed(
                        self.base.pipeline.clone(),
                        &DynamicState::none(),
                        vec![draw_info.vertex_buffer.clone()],
                        draw_info.index_buffer.as_ref().unwrap().clone(),
                        set.set.clone(),
                        push_constants,
                        vec![],
                    )
                    .unwrap();
            } else {
                builder
                    .draw(
                        self.base.pipeline.clone(),
                        &DynamicState::none(),
                        vec![draw_info.vertex_buffer.clone()],
                        set.set.clone(),
                        push_constants,
                        vec![],
                    )
                    .unwrap();
            }
        }

        // Draw the environment cube.
        if let Some(environment) = &self.world_render.environment {
            builder
                .draw(
                    self.base.environment_pipeline.clone(),
                    &DynamicState::none(),
                    vec![environment.skybox_vertex_buffer.clone()],
                    environment.skybox_set.clone(),
                    (),
                    vec![],
                )
                .unwrap();
        }

        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.base.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.base.queue.clone(),
                self.base.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(sync::FlushError::OutOfDate) => {
                self.base.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.base.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.base.device.clone()).boxed());
            }
        }
    }
}
