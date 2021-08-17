use crate::render::shaders::*;

use aperture_common::{VPos, VPosNormTex};

use vulkano::descriptor::descriptor::ShaderStages;
use vulkano::device::Device;
use vulkano::pipeline::layout::{PipelineLayout, PipelineLayoutDesc, PipelineLayoutDescPcRange};
use vulkano::pipeline::shader::EntryPointAbstract;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::render_pass::{RenderPass, Subpass};

use std::iter;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pipeline {
    Cubemap,
    Shaded,
}

impl Pipeline {
    pub fn create(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        shaders: &Shaders,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        match self {
            Self::Shaded => self.shaded(device, dimensions, shaders, render_pass),
            Self::Cubemap => self.cubemap(device, dimensions, shaders, render_pass),
        }
    }

    fn shaded(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        shaders: &Shaders,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let pipeline_layout_desc = {
            let stages = vec![
                shaders.vertex.main_entry_point(),
                shaders.fragment.main_entry_point(),
            ];

            stages
                .into_iter()
                .fold(PipelineLayoutDesc::empty(), |total, stage| {
                    total.union(stage.layout_desc())
                })
                .union(
                    &PipelineLayoutDesc::new(
                        vec![],
                        vec![
                            PipelineLayoutDescPcRange {
                                offset: 0,
                                size: 64,
                                stages: ShaderStages {
                                    vertex: true,
                                    ..ShaderStages::none()
                                },
                            },
                            PipelineLayoutDescPcRange {
                                offset: 64,
                                size: 28,
                                stages: ShaderStages {
                                    fragment: true,
                                    ..ShaderStages::none()
                                },
                            },
                        ],
                    )
                    .unwrap(),
                )
        };

        let pipeline_layout =
            Arc::new(PipelineLayout::new(device.clone(), pipeline_layout_desc).unwrap());

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleBufferDefinition::<VPosNormTex>::new())
                .vertex_shader(shaders.vertex.main_entry_point(), ())
                .polygon_mode_fill()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..0.99,
                }))
                .fragment_shader(shaders.fragment.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .cull_mode_back()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .with_pipeline_layout(device.clone(), pipeline_layout)
                .unwrap(),
        )
    }

    fn cubemap(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        shaders: &Shaders,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let stages = vec![
            shaders.cubemap_vert.main_entry_point(),
            shaders.cubemap_frag.main_entry_point(),
        ];

        let pipeline_layout = Arc::new(PipelineLayout::new(
            device.clone(),
            stages
                .into_iter()
                .fold(PipelineLayoutDesc::empty(), |total, stage| {
                    total.union(stage.layout_desc())
                }),
        ).unwrap());

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleBufferDefinition::<VPos>::new())
                .vertex_shader(shaders.cubemap_vert.main_entry_point(), ())
                .polygon_mode_fill()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..0.99,
                }))
                .fragment_shader(shaders.cubemap_frag.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .cull_mode_back()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .with_pipeline_layout(device.clone(), pipeline_layout)
                .unwrap(),
        )
    }
}
