use crate::vulkan::context::*;

use ash::util::read_spv;
use ash::vk;

use std::io::Cursor;
use std::sync::Arc;

pub struct ShaderModule {
    pub vk_handle: vk::ShaderModule,
    pub vk_context: Arc<Context>,
}

impl ShaderModule {
    pub fn new(spirv: &mut Cursor<&[u8]>, vk_context: Arc<Context>) -> Self {
        let source = read_spv(spirv).unwrap();
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&source);
        let shader_module = unsafe {
            vk_context
                .logical_device
                .create_shader_module(&create_info, None)
                .unwrap()
        };

        Self {
            vk_handle: shader_module,
            vk_context,
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.vk_context
                .logical_device
                .destroy_shader_module(self.vk_handle, None);
        }
    }
}
