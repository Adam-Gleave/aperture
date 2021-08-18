use crate::render::shaders::*;
use crate::world::cube::Cube;

use aperture_common::VPos;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess};
use vulkano::descriptor::descriptor::ShaderStages;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewType};
use vulkano::image::{AttachmentImage, ImageAccess, ImageCreateFlags, ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount, SampleCount, StorageImage};
use vulkano::pipeline::depth_stencil::{DepthBounds, DepthStencil};
use vulkano::pipeline::layout::{PipelineLayout, PipelineLayoutDesc, PipelineLayoutDescPcRange};
use vulkano::pipeline::shader::EntryPointAbstract;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::sampler::{Compare, Sampler};

use std::iter;
use std::sync::Arc;

pub struct Environment {
    pub cube: Cube,
    
    pub skybox_vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    pub skybox_uniform_buffer: Arc<dyn TypedBufferAccess<Content = cube_vert::ty::Data> + Send + Sync>,
    pub skybox_set: Arc<dyn DescriptorSet + Send + Sync>,

    pub cube_texture_target: Arc<dyn ImageAccess + Send + Sync>,
    pub offscreen_cube_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub offscreen_cube_vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    pub offscreen_cube_uniform_buffer: Arc<dyn TypedBufferAccess<Content = offscreen_cube_vert::ty::Data> + Send + Sync>,  
    pub offscreen_cube_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub offscreen_render_pass: Arc<RenderPass>,
    pub offscreen_framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
}

impl Environment {
    pub const CUBE_DIMENSIONS: [u32; 2] = [1024, 1024];

    pub const CUBE_IMAGE_LAYERS: u32 = 6;

    pub fn new(
        skybox_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        shaders: &Shaders,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self {
        let cube = Cube::textured();

        let (image, _) = ImmutableImage::from_iter(
            cube.texture.pixels.iter().cloned(),
            ImageDimensions::Dim2d {
                width: cube.texture.width,
                height: cube.texture.height,
                array_layers: 1,
            },
            MipmapsCount::One,
            Format::R32G32B32A32Sfloat,
            queue.clone(),
        )
        .unwrap();

        let hdri_view = ImageView::new(image).unwrap();
        let hdri_sampler = Sampler::simple_repeat_linear_no_mipmap(device.clone());
        
        let skybox_vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            Cube::VERTICES.iter().cloned(),
        )
        .unwrap();

        let skybox_uniform_buffer = DeviceLocalBuffer::<cube_vert::ty::Data>::new(
            device.clone(),
            BufferUsage::uniform_buffer_transfer_destination(),
            device.active_queue_families(),
        )
        .unwrap();

        let offscreen_render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    cube: {
                        load: Clear,
                        store: Store,
                        format: Format::R32G32B32A32Sfloat,
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [cube],
                    depth_stencil: {depth}
                }
            ).unwrap()
        );

        let cube_texture_target = StorageImage::with_usage(
            device.clone(),
            ImageDimensions::Dim2d {
                width: Self::CUBE_DIMENSIONS[0],
                height: Self::CUBE_DIMENSIONS[1],
                array_layers: Self::CUBE_IMAGE_LAYERS,
            },
            Format::R32G32B32A32Sfloat,
            ImageUsage {
                color_attachment: true,
                transfer_destination: true,
                sampled: true,
                ..ImageUsage::none()
            },
            ImageCreateFlags {
                cube_compatible: true,
                ..ImageCreateFlags::none()
            },
            device.active_queue_families(),
        )
        .unwrap();

        let cubemap_image_view = ImageView::start(cube_texture_target.clone())
            .with_type(ImageViewType::Cubemap)
            .build()
            .unwrap();

        let offscreen_framebuffer = {
            let depth_buffer = ImageView::new(
                AttachmentImage::multisampled_with_usage_with_layers(
                    device.clone(), 
                    Self::CUBE_DIMENSIONS, 
                    Self::CUBE_IMAGE_LAYERS,
                    SampleCount::Sample1,
                    Format::D16Unorm,
                    ImageUsage {
                        depth_stencil_attachment: true,
                        ..ImageUsage::none()
                    },
                )
                .unwrap()
            )
            .unwrap();

            Arc::new(
                Framebuffer::start(offscreen_render_pass.clone())
                    .add(cubemap_image_view.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        };

        let cube_texture_sampler = Sampler::simple_repeat_linear(device.clone());

        let stages = vec![
            shaders.offscreen_cube_vert.main_entry_point(),
            shaders.offscreen_cube_frag.main_entry_point(),
        ];

        let offscreen_cube_pipeline_layout = Arc::new(PipelineLayout::new(
            device.clone(),
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
                                size: 4,
                                stages: ShaderStages {
                                    vertex: true,
                                    ..ShaderStages::none()
                                }
                            }
                        ]
                    )
                    .unwrap()
                )
        ).unwrap());

        let offscreen_cube_pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleBufferDefinition::<VPos>::new())
                .vertex_shader(shaders.offscreen_cube_vert.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [Self::CUBE_DIMENSIONS[0] as f32, Self::CUBE_DIMENSIONS[1] as f32],
                    depth_range: 0.0..0.99,
                }))
                .fragment_shader(shaders.offscreen_cube_frag.main_entry_point(), ())
                .depth_stencil(DepthStencil { 
                    depth_compare: Compare::LessOrEqual,
                    depth_write: true, 
                    depth_bounds_test: DepthBounds::Disabled,
                    stencil_front: Default::default(),
                    stencil_back: Default::default(), 
                })
                .render_pass(Subpass::from(offscreen_render_pass.clone(), 0).unwrap())
                .with_pipeline_layout(device.clone(), offscreen_cube_pipeline_layout)
                .unwrap(),
        );

        let offscreen_cube_vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            Cube::VERTICES.iter().cloned(),
        )
        .unwrap();

        let offscreen_cube_uniform_buffer = DeviceLocalBuffer::<offscreen_cube_vert::ty::Data>::new(
            device.clone(),
            BufferUsage::uniform_buffer_transfer_destination(),
            device.active_queue_families(),
        )
        .unwrap();

        let layout = offscreen_cube_pipeline.layout().descriptor_set_layout(0).unwrap();
        let offscreen_cube_set = Arc::new(PersistentDescriptorSet::start(layout.clone())
            .add_buffer(offscreen_cube_uniform_buffer.clone())
            .unwrap()
            .add_sampled_image(hdri_view.clone(), hdri_sampler.clone())
            .unwrap()
            .build()
            .unwrap()
        );

        let layout = skybox_pipeline.layout().descriptor_set_layout(0).unwrap();
        let skybox_set = Arc::new(PersistentDescriptorSet::start(layout.clone())
            .add_buffer(skybox_uniform_buffer.clone())
            .unwrap()
            .add_sampled_image(cubemap_image_view.clone(), cube_texture_sampler.clone())
            .unwrap()
            .build()
            .unwrap()
        );

        Self {
            cube,
            skybox_vertex_buffer,
            skybox_uniform_buffer,
            skybox_set,
            cube_texture_target,
            offscreen_cube_pipeline,
            offscreen_cube_vertex_buffer,
            offscreen_cube_uniform_buffer,
            offscreen_cube_set,
            offscreen_render_pass,
            offscreen_framebuffer,
        }
    }
}