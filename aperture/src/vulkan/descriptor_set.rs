use crate::render::shaders::{frag, vert};

use vulkano::buffer::{DeviceLocalBuffer, TypedBufferAccess};
use vulkano::descriptor::DescriptorSet as VkDescriptorSet;

use std::sync::Arc;

pub struct DescriptorSet {
    pub set: Arc<dyn VkDescriptorSet + Send + Sync>,
    pub vertex_uniform_buffer:
        Option<Arc<dyn TypedBufferAccess<Content = vert::ty::Data> + Send + Sync>>,
    pub fragment_uniform_buffer:
        Option<Arc<dyn TypedBufferAccess<Content = frag::ty::Data> + Send + Sync>>,
}

impl DescriptorSet {
    pub fn new(set: Arc<dyn VkDescriptorSet + Send + Sync>) -> Self {
        Self {
            set,
            vertex_uniform_buffer: None,
            fragment_uniform_buffer: None,
        }
    }

    pub fn with_vertex_uniform_buffer(
        mut self,
        buffer: Arc<DeviceLocalBuffer<vert::ty::Data>>,
    ) -> Self {
        self.vertex_uniform_buffer = Some(buffer);
        self
    }

    pub fn with_fragment_uniform_buffer(
        mut self,
        buffer: Arc<DeviceLocalBuffer<frag::ty::Data>>,
    ) -> Self {
        self.fragment_uniform_buffer = Some(buffer);
        self
    }
}
