mod base;

pub mod shaders;

use base::VulkanBase;
use vulkano::{command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, sync::{self, GpuFuture}};
use winit::event_loop::EventLoop;

use crate::world::World;

pub struct Renderer {
    pub base: VulkanBase,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl Renderer {
    pub fn new(title: String, width: u32, height: u32) -> (Self, EventLoop<()>) {
        let (base, event_loop) = VulkanBase::new(title, width, height);
        let previous_frame_end = Some(sync::now(base.device.clone()).boxed());

        (
            Self { 
                base,
                previous_frame_end,
            }, 
            event_loop,
        )
    }

    pub fn notify_resized(&mut self) {
        self.base.recreate_swapchain = true;
    }

    pub fn render(&mut self, world: &World) {
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

        // Start building the command buffer.
        let mut builder = AutoCommandBufferBuilder::primary(
            self.base.device.clone(),
            self.base.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Just clear the screen for now.
        builder
            .begin_render_pass(
                self.base.framebuffers[image_num].clone(),
                SubpassContents::Inline,
                vec![[0.1, 0.1, 0.1, 1.0].into(), 1f32.into()],
            )
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.base.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.base.queue.clone(), self.base.swapchain.clone(), image_num)
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