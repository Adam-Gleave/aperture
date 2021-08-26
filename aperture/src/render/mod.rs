use crate::offset_of;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::context::{record_submit_command_buffer, Context};
use crate::vulkan::shader_module::ShaderModule;

use aperture_common::VPosCol;

use ash::vk;
use winit::event_loop::EventLoop;

use std::ffi::CString;
use std::io::Cursor;
use std::sync::Arc;

pub struct Renderer {
    pub title: String,
    pub width: u32,
    pub height: u32,

    pub vk_context: Arc<Context>,

    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub renderpass: vk::RenderPass,
    pub graphics_pipeline: vk::Pipeline,
    pub graphics_pipeline_layout: vk::PipelineLayout,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub viewports: Vec<vk::Viewport>,
    pub scissors: Vec<vk::Rect2D>,
}

impl Renderer {
    pub fn new(title: String, width: u32, height: u32) -> (EventLoop<()>, Self) {
        unsafe {
            let (event_loop, vk_context) = Context::new(&title, 1920, 1080);

            let renderpass_attachments = [
                vk::AttachmentDescription {
                    format: vk_context.surface.format().unwrap().format,
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
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()];

            let renderpass_create_info = vk::RenderPassCreateInfo::builder()
                .attachments(&renderpass_attachments)
                .subpasses(&subpasses)
                .dependencies(&dependencies);

            let renderpass = vk_context
                .logical_device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap();

            let surface_resolution = vk_context.surface.properties.as_ref().unwrap().resolution;

            let framebuffers: Vec<vk::Framebuffer> = vk_context
                .swapchain
                .image_views
                .iter()
                .map(|&present_image_view| {
                    let framebuffer_attachments = [present_image_view, vk_context.depth_image.view];

                    let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(surface_resolution.width)
                        .height(surface_resolution.height)
                        .layers(1);

                    vk_context
                        .logical_device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                })
                .collect();

            let index_buffer_data = [0u32, 1, 2];

            let mut index_buffer = Buffer::new(
                (std::mem::size_of::<f32>() * index_buffer_data.len()) as _,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk_context.clone(),
            );

            index_buffer.upload(&index_buffer_data, 0);

            let vertex_buffer_data = [
                VPosCol {
                    position: [-1.0, 1.0, 0.0, 1.0],
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

            let mut vertex_buffer = Buffer::new(
                (std::mem::size_of::<VPosCol>() * vertex_buffer_data.len()) as _,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk_context.clone(),
            );

            vertex_buffer.upload(&vertex_buffer_data, 0);

            let vert_shader_module = ShaderModule::new(
                &mut Cursor::new(
                    &include_bytes!("../../../data/shaders/gen/triangle.vert.spv")[..],
                ),
                vk_context.clone(),
            );

            let frag_shader_module = ShaderModule::new(
                &mut Cursor::new(
                    &include_bytes!("../../../data/shaders/gen/triangle.frag.spv")[..],
                ),
                vk_context.clone(),
            );

            let layout_create_info = vk::PipelineLayoutCreateInfo::default();

            let pipeline_layout = vk_context
                .logical_device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();

            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage_create_infos = [
                vk::PipelineShaderStageCreateInfo {
                    module: vert_shader_module.vk_handle,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                    module: frag_shader_module.vk_handle,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ];

            let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<VPosCol>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];

            let vertex_input_attribute_descriptions = [
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(VPosCol, position) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(VPosCol, color) as u32,
                },
            ];

            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
                vertex_attribute_description_count: vertex_input_attribute_descriptions.len()
                    as u32,
                p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
                vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
                p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
                ..Default::default()
            };

            let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };

            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: surface_resolution.width as f32,
                height: surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_resolution,
            }];

            let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
                .scissors(&scissors)
                .viewports(&viewports);

            let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
                front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                polygon_mode: vk::PolygonMode::FILL,
                ..Default::default()
            };

            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
                rasterization_samples: vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            };

            let noop_stencil_state = vk::StencilOpState {
                fail_op: vk::StencilOp::KEEP,
                pass_op: vk::StencilOp::KEEP,
                depth_fail_op: vk::StencilOp::KEEP,
                compare_op: vk::CompareOp::ALWAYS,
                ..Default::default()
            };

            let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
                depth_test_enable: 1,
                depth_write_enable: 1,
                depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
                front: noop_stencil_state,
                back: noop_stencil_state,
                max_depth_bounds: 1.0,
                ..Default::default()
            };

            let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::all(),
            }];

            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);

            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

            let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stage_create_infos)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .depth_stencil_state(&depth_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .render_pass(renderpass);

            let graphics_pipelines = vk_context
                .logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info.build()],
                    None,
                )
                .expect("Unable to create graphics pipeline");

            let graphics_pipeline = graphics_pipelines[0];

            (
                event_loop,
                Self {
                    title,
                    width,
                    height,
                    vk_context,
                    vertex_buffer,
                    index_buffer,
                    renderpass,
                    graphics_pipeline,
                    graphics_pipeline_layout: pipeline_layout,
                    framebuffers,
                    viewports: viewports.to_vec(),
                    scissors: scissors.to_vec(),
                },
            )
        }
    }

    pub fn render(&self) {
        let (present_index, _) = unsafe {
            self.vk_context
                .swapchain
                .acquire_next_image(
                    self.vk_context.swapchain.vk_handle,
                    std::u64::MAX,
                    self.vk_context.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
        };

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.framebuffers[present_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.vk_context.surface.resolution().unwrap(),
            })
            .clear_values(&clear_values);

        unsafe {
            record_submit_command_buffer(
                &self.vk_context.logical_device,
                self.vk_context.draw_command_buffer,
                self.vk_context.draw_commands_reuse_fence,
                self.vk_context.logical_device.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[self.vk_context.present_complete_semaphore],
                &[self.vk_context.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );

                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.graphics_pipeline,
                    );

                    device.cmd_set_viewport(draw_command_buffer, 0, &self.viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &self.scissors);

                    device.cmd_bind_vertex_buffers(
                        draw_command_buffer,
                        0,
                        &[self.vertex_buffer.vk_handle],
                        &[0],
                    );

                    device.cmd_bind_index_buffer(
                        draw_command_buffer,
                        self.index_buffer.vk_handle,
                        0,
                        vk::IndexType::UINT32,
                    );

                    device.cmd_draw_indexed(draw_command_buffer, 3, 1, 0, 0, 1);

                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );

            let wait_semaphors = [self.vk_context.rendering_complete_semaphore];
            let swapchains = [self.vk_context.swapchain.vk_handle];
            let image_indices = [present_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphors)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.vk_context
                .swapchain
                .queue_present(self.vk_context.logical_device.present_queue, &present_info)
                .unwrap();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.vk_context
                .logical_device
                .destroy_pipeline(self.graphics_pipeline, None);

            self.vk_context
                .logical_device
                .destroy_pipeline_layout(self.graphics_pipeline_layout, None);

            for framebuffer in &self.framebuffers {
                self.vk_context
                    .logical_device
                    .destroy_framebuffer(*framebuffer, None);
            }

            self.vk_context
                .logical_device
                .destroy_render_pass(self.renderpass, None);
        }
    }
}
