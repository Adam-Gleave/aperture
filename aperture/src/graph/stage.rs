use crate::graph::resource::Resource;
use crate::vulkan::context::Context;
use crate::vulkan::shader_module::ShaderModule;

use ash::vk;

use std::ffi::CString;
use std::sync::Arc;

pub struct GraphicsStage {
    pub reads: Vec<Resource>,
    pub writes: Vec<Resource>,
    pub shader: ShaderModule,

    pub vk_context: Arc<Context>,
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
    reads: Vec<usize>,
    writes: Vec<usize>,
    shaders: Vec<ShaderModule>,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    push_constants: Vec<vk::PushConstantRange>,
    clear: bool,
}

impl GraphcisStageBuilder {
    pub fn reads_from(mut self, idx: usize) -> Self {
        self.reads.push(idx);
        self
    }

    pub fn writes_from(mut self, idx: usize) -> Self {
        self.writes.push(idx);
        self
    }

    pub fn uses_shader(mut self, shader: ShaderModule) -> Self {
        self.shaders.push(shader);
        self
    }

    pub fn uses_descriptor_set_layout(mut self, layout: vk::DescriptorSetLayout) -> Self {
        self.descriptor_set_layouts.push(layout);
        self
    }

    pub fn push_constants(mut self, push_constants: vk::PushConstantRange) -> Self {
        self.push_constants.push(push_constants);
        self
    }

    pub fn clear(mut self) -> Self {
        self.clear = true;
        self
    }

    // TODO should make this a result.
    pub fn build_render_pass(
        self, 
        vk_context: Arc<Context>,
        write_resources: &Vec<Resource>,
    ) -> Option<vk::RenderPass> {
        let mut attachments = vec![];
        let mut color_attachment_refs = vec![];
        let mut depth_attachment_refs = vec![];

        let mut attachment_idx = 0;

        for write_resource_idx in self.writes {
            let write_resource = &write_resources[write_resource_idx];

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
            vk_context
                .logical_device
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        Some(render_pass)
    }

    pub fn build_pipeline_layout(&self, vk_context: Arc<Context>) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&self.descriptor_set_layouts)
            .push_constant_ranges(&self.push_constants);

        unsafe {
            vk_context
                .logical_device
                .create_pipeline_layout(&create_info, None)
                .unwrap()
        }
    }

    pub fn build_graphics_pipeline(
        &self, 
        vk_context: Arc<Context>,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        read_resources: &Vec<Resource>,
    ) -> Option<vk::Pipeline> {
        let mut vertex_bindings = vec![];
        let mut attribute_bindings = vec![];

        for read_resource_idx in &self.reads {
            let read_resource = &read_resources[*read_resource_idx];

            if let Resource::VertexBuffer(buffer) = read_resource {
                vertex_bindings.push(buffer.vertex_binding);
                attribute_bindings.extend(buffer.attribute_bindings.clone());
            } else {
                continue;
            }
        }

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&attribute_bindings);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let shader_entry_name = CString::new("main").unwrap();
        let mut all_stages = vk::ShaderStageFlags::empty();
        let mut shader_stage_create_infos = vec![];

        for shader in &self.shaders {
            let create_info = vk::PipelineShaderStageCreateInfo {
                module: shader.vk_handle,
                p_name: shader_entry_name.as_ptr(),
                stage: shader.stage,
                ..Default::default()
            };

            shader_stage_create_infos.push(create_info);
            all_stages |= shader.stage;
        };

        if all_stages.contains(vk::ShaderStageFlags::VERTEX) 
            || !all_stages.contains(vk::ShaderStageFlags::FRAGMENT) 
        {
            return None;
        }

        let surface_resolution = vk_context.surface.properties.as_ref().unwrap().resolution;

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: surface_resolution.width as _,
            height: surface_resolution.height as _,
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
            .render_pass(render_pass);

        let graphics_pipelines = unsafe {
            vk_context
                .logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info.build()],
                    None,
                )
                .unwrap()
        };

        Some(graphics_pipelines[0])
    }
}
