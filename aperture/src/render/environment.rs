use crate::render::shaders::*;
use crate::world::cube::Cube;

use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sampler::Sampler;

use std::sync::Arc;

pub struct Environment {
    pub cube: Cube,
    pub vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    pub uniform_buffer: Arc<dyn TypedBufferAccess<Content = cube_vert::ty::Data> + Send + Sync>,
    pub set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl Environment {
    pub fn new(
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
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

        let view = ImageView::new(image).unwrap();
        let sampler = Sampler::simple_repeat_linear_no_mipmap(device.clone());
        
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            Cube::VERTICES.iter().cloned(),
        )
        .unwrap();

        let uniform_buffer = DeviceLocalBuffer::<cube_vert::ty::Data>::new(
            device.clone(),
            BufferUsage::uniform_buffer_transfer_destination(),
            device.active_queue_families(),
        )
        .unwrap();

        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();

        let set = Arc::new(PersistentDescriptorSet::start(layout.clone())
            .add_buffer(uniform_buffer.clone())
            .unwrap()
            .add_sampled_image(view.clone(), sampler.clone())
            .unwrap()
            .build()
            .unwrap()
        );

        Self {
            cube,
            vertex_buffer,
            uniform_buffer,
            set,
        }
    }
}