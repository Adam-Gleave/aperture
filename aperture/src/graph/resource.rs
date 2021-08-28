use crate::vulkan::buffer::Buffer;
use crate::vulkan::image::Image;

use ash::vk;

pub enum Resource {
    VertexBuffer(VertexBufferResource),
    Buffer(Buffer),
    Image(Image),
}

pub struct VertexBufferResource {
    pub buffer: Buffer,
    pub vertex_binding: vk::VertexInputBindingDescription,
    pub attribute_bindings: Vec<vk::VertexInputAttributeDescription>,
}

impl VertexBufferResource {
    // TODO multiple vertex input bindings.
    pub fn new<T>(buffer: Buffer) -> Self {
        let vertex_binding = vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<T>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };

        Self {
            buffer,
            vertex_binding,
            attribute_bindings: vec![],
        }
    }

    pub fn with_attribute_binding(mut self, format: vk::Format, binding: u32, offset: u32) -> Self {
        let attribute_binding = vk::VertexInputAttributeDescription {
            location: self.attribute_bindings.len() as _,
            binding,
            format,
            offset,
        };

        self.attribute_bindings.push(attribute_binding);
        self
    }
}