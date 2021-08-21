use crate::vulkan::context::Context;

use winit::event_loop::EventLoop;

pub struct Renderer {
    pub title: String,
    pub width: u32,
    pub height: u32,

    pub event_loop: EventLoop<()>,
    pub vk_context: Context,
}

impl Renderer {
    pub fn new(title: String, width: u32, height: u32) -> Self {
        let (event_loop, vk_context) = Context::new(title.clone(), width, height);

        Self {
            title,
            width,
            height,
            event_loop,
            vk_context,
        }
    }

    pub fn render(&self) {}
}