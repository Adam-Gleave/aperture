pub mod resource;
pub mod stage;

use crate::vulkan::context::Context; 
use crate::vulkan::swapchain::Swapchain;
use self::resource::Resource;
use self::stage::GraphicsStage;

use ash::vk;

use std::sync::Arc;

pub struct RenderGraph {
    pub swapchain: Swapchain,
    pub context: Arc<Context>,
    pub command_pool: vk::CommandPool,
    pub stages: Vec<GraphicsStage>,
    pub reads: Vec<Resource>,
    pub writes: Vec<Resource>,
}

impl RenderGraph {
    pub fn new(
        swapchain: Swapchain,
        context: Arc<Context>,
        command_pool: vk::CommandPool,
    ) -> Self {
        Self {
            swapchain,
            context,
            command_pool,
            stages: vec![],
            reads: vec![],
            writes: vec![],
        }
    }

    pub fn recreate_swapchain(&mut self) {
        // TODO
    }
    
    pub fn reads_from(&mut self, resource: Resource) -> usize {
        let idx = self.reads.len();
        self.reads.push(resource);
        idx
    }

    pub fn writes_to(&mut self, resource: Resource) -> usize {
        let idx = self.writes.len();
        self.writes.push(resource);
        idx
    }
}
