use crate::{depth, frag, vert, VPosNorm};

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

#[derive(Clone, Copy)]
pub enum Pipeline {
    Depth,
    Shaded,
    Wireframe,
}

impl Pipeline {
    pub fn create(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        vs: &vert::Shader,
        fs: &frag::Shader,
        depth: &depth::Shader,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        match self {
            Self::Depth => self.depth(device, dimensions, vs, depth, render_pass),
            Self::Shaded => self.shaded(device, dimensions, vs, fs, render_pass),
            Self::Wireframe => self.wireframe(device, dimensions, vs, fs, render_pass),
        }
    }

    fn depth(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        vs: &vert::Shader,
        depth: &depth::Shader,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleBufferDefinition::<VPosNorm>::new())
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(depth.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        )
    }

    fn shaded(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        vs: &vert::Shader,
        fs: &frag::Shader,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let pipeline_layout_desc = {
            let stages = vec![
                vs.main_entry_point(),
                fs.main_entry_point(),
            ];

            let desc = stages
                .into_iter()
                .fold(PipelineLayoutDesc::empty(), |total, stage| {
                    total.union(stage.layout_desc())
                });

            desc.union(&PipelineLayoutDesc::new(
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
                        size: 24,
                        stages: ShaderStages {
                            fragment: true,
                            ..ShaderStages::none()
                        }
                    },
                ]
            )
            .unwrap())
        };

        let pipeline_layout = Arc::new(PipelineLayout::new(device.clone(), pipeline_layout_desc).unwrap());

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleBufferDefinition::<VPosNorm>::new())
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .with_pipeline_layout(device.clone(), pipeline_layout)
                .unwrap(),
        )
    }

    fn wireframe(
        &self,
        device: Arc<Device>,
        dimensions: [u32; 2],
        vs: &vert::Shader,
        fs: &frag::Shader,
        render_pass: Arc<RenderPass>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleBufferDefinition::<VPosNorm>::new())
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .polygon_mode_line()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        )
    }
}