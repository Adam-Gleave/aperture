use crate::vulkan::buffer::Buffer;
use crate::vulkan::context::Context;
use crate::vulkan::shader_module::ShaderModule;

use aperture_common::VPosCol;

use ash::version::DeviceV1_0;
use ash::vk;
use winit::event_loop::EventLoop;

use std::io::Cursor;
use std::sync::Arc;

pub struct Renderer {
    pub title: String,
    pub width: u32,
    pub height: u32,

    pub event_loop: EventLoop<()>,
    pub vk_context: Arc<Context>,
}

impl Renderer {
    pub fn new(title: String, width: u32, height: u32) -> Self {
        let (event_loop, vk_context) = Context::new(title.clone(), width, height);

        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: vk_context.surface_properties.format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];

        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: 
                vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()
        ];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let renderpass = unsafe {
            vk_context
                .logical_device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let framebuffers = vk_context
            .present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, vk_context.depth_image_view];

                let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(vk_context.surface_properties.resolution.width)
                    .height(vk_context.surface_properties.resolution.height)
                    .layers(1);

                unsafe {
                    vk_context
                        .logical_device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .unwrap()
                }
            })
            .collect::<Vec<_>>();

        let index_buffer_data = [0u32, 1, 2];

        let index_buffer = Buffer::new(
            index_buffer_data.len() as _,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk_context.clone(),
        );

        index_buffer.upload(&index_buffer_data, 0);

        let vertex_buffer_data = [
            VPosCol {
                position: [1.0, 1.0, 0.0, 1.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            VPosCol {
                position: [1.0, 1.0, 0.0, 1.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
            VPosCol {
                position: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        let vertex_buffer = Buffer::new(
            vertex_buffer_data.len() as _,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk_context.clone()
        );

        vertex_buffer.upload(&vertex_buffer_data, 0);

        let vert_shader_module = ShaderModule::new(
            &mut Cursor::new(&include_bytes!("../../../data/shaders/gen/triangle.vert.spv")[..]),
            vk_context.clone(),
        );

        let frag_shader_module = ShaderModule::new(
            &mut Cursor::new(&include_bytes!("../../../data/shaders/gen/triangle.frag.spv")[..]),
            vk_context.clone(),
        );

        Self {
            title,
            width,
            height,
            event_loop,
            vk_context,
        }
    }

    pub fn render(&self) {

    }
}