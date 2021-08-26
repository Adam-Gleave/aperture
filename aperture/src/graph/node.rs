use crate::vulkan::buffer::Buffer;
use crate::vulkan::image::Image;
use crate::vulkan::logical_device::LogicalDevice;
use crate::vulkan::shader_module::ShaderModule;

use ash::vk;

use std::sync::Arc;

pub enum Resource {
    Buffer(Buffer),
    Image(Image),
}

pub struct GraphicsStage {
    pub reads: Vec<Resource>,
    pub writes: Vec<Resource>,
    pub shader: ShaderModule,

    pub logical_device: Arc<LogicalDevice>,
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl GraphicsStage {
    pub fn builder() -> GraphcisStageBuilder {
        GraphcisStageBuilder::default()
    }
}

#[derive(Default)]
pub struct GraphcisStageBuilder {
    reads: Vec<Resource>,
    writes: Vec<Resource>,
    shader: Option<ShaderModule>,
    clear: bool,
}

impl GraphcisStageBuilder {
    pub fn reads_from(mut self, resource: Resource) -> Self {
        self.reads.push(resource);
        self
    }

    pub fn writes_from(mut self, resource: Resource) -> Self {
        self.writes.push(resource);
        self
    }

    pub fn uses_shader(mut self, shader: ShaderModule) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn clear(mut self) -> Self {
        self.clear = true;
        self
    }

    // TODO should make this a result.
    pub fn build_render_pass(self, logical_device: Arc<LogicalDevice>) -> Option<vk::RenderPass> {
        let mut attachments = vec![];
        let mut color_attachment_refs = vec![];
        let mut depth_attachment_refs = vec![];

        let mut attachment_idx = 0;

        for write_resource in self.writes {
            let attachment = if let Resource::Image(image) = write_resource {
                let mut attachment = vk::AttachmentDescription {
                    format: image.format,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: if self.clear {
                        vk::AttachmentLoadOp::CLEAR
                    } else {
                        vk::AttachmentLoadOp::DONT_CARE
                    },
                    store_op: vk::AttachmentStoreOp::STORE,
                    ..Default::default()
                };

                if image.back_buffer {
                    if self.clear {
                        attachment.initial_layout = vk::ImageLayout::PRESENT_SRC_KHR;
                        attachment.load_op = vk::AttachmentLoadOp::LOAD;
                    }

                    attachment.final_layout = vk::ImageLayout::PRESENT_SRC_KHR;
                    color_attachment_refs.push(vk::AttachmentReference {
                        attachment: attachment_idx,
                        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    });
                } else if image
                    .usage
                    .contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                {
                    attachment.final_layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                    depth_attachment_refs.push(vk::AttachmentReference {
                        attachment: attachment_idx,
                        layout: attachment.final_layout,
                    });
                } else {
                    attachment.final_layout = image.transition.as_ref().unwrap().final_layout;
                    color_attachment_refs.push(vk::AttachmentReference {
                        attachment: attachment_idx,
                        layout: attachment.final_layout,
                    });
                }

                attachment_idx += 1;
                attachment
            } else {
                continue;
            };

            attachments.push(attachment);
        }

        // TODO should this always hold true?
        if color_attachment_refs.is_empty()
            || depth_attachment_refs.is_empty()
            || depth_attachment_refs.len() > 1
        {
            return None;
        }

        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_refs[0])
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe {
            logical_device
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        Some(render_pass)
    }
}
