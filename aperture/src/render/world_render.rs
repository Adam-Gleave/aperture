use crate::render::shaders::*;
use crate::vulkan::{DescriptorSet, Pipeline};

use aperture_common::{Transform, VPosNormTex};
use aperture_mesh::{Material, Mesh, Texture};

use cgmath::Matrix4;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::{image::ImageViewAbstract, sampler::Sampler};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::environment::Environment;

#[derive(Default)]
pub struct WorldRender {
    pub primitive_info: Vec<PrimitiveInfo>,
    pub material_info: HashMap<String, MaterialInfo>,
    pub image_samplers: HashMap<String, ImageData>,
    pub environment: Option<Environment>,
}

impl WorldRender {
    pub fn update<'a>(
        &mut self,
        meshes: impl Iterator<Item = &'a Mesh>,
        materials: impl Iterator<Item = &'a Material>,
        textures: impl Iterator<Item = &'a Texture<u8>>,
        pipeline_type: Pipeline,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) {
        self.gen_samplers(textures, device.clone(), queue.clone());
        self.gen_primitive_info(meshes, device.clone());
        self.gen_material_info(materials, pipeline_type, pipeline.clone(), device.clone());
    }

    pub fn update_environment(
        &mut self,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        shaders: &Shaders,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) {
        self.environment = Some(Environment::new(
            pipeline, 
            shaders,
            device, 
            queue,
        ));
    }

    fn gen_primitive_info<'a>(
        &mut self,
        meshes: impl Iterator<Item = &'a Mesh>,
        device: Arc<Device>,
    ) {
        for mesh in meshes {
            self.primitive_info
                .extend(PrimitiveInfo::generate_from_mesh(&mesh, device.clone()));
        }
    }

    fn gen_material_info<'a>(
        &mut self,
        materials: impl Iterator<Item = &'a Material>,
        pipeline_type: Pipeline,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        device: Arc<Device>,
    ) {
        for material in materials {
            if !self.material_info.contains_key(&material.name) {
                self.material_info.insert(
                    material.name.clone(),
                    MaterialInfo::new(
                        &material,
                        &self.image_samplers,
                        pipeline_type,
                        pipeline.clone(),
                        device.clone(),
                    ),
                );
            }
        }
    }

    fn gen_samplers<'a>(
        &mut self,
        textures: impl Iterator<Item = &'a Texture<u8>>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) {
        for texture in textures {
            if !self.image_samplers.contains_key(&texture.name) {
                self.image_samplers.insert(
                    texture.name.clone(),
                    ImageData::new(
                        &texture, 
                        // FIXME
                        Format::R8G8B8A8Unorm,
                        device.clone(),
                        queue.clone(),
                    ),
                );
            }
        }
    }
}

pub struct PrimitiveInfo {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[VPosNormTex]>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    pub transform: Arc<Mutex<Transform>>,
    pub material_name: Option<String>,
}

impl PrimitiveInfo {
    pub fn has_indices(&self) -> bool {
        self.index_buffer.is_some()
    }

    pub fn composed_transform(&self) -> Matrix4<f32> {
        self.transform
            .lock()
            .expect("poisoned_lock")
            .clone()
            .compose()
    }

    pub fn generate_from_mesh(mesh: &Mesh, device: Arc<Device>) -> Vec<PrimitiveInfo> {
        mesh.primitives
            .iter()
            .map(|p| {
                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::vertex_buffer(),
                    false,
                    p.vertices.iter().cloned(),
                )
                .unwrap();

                let index_buffer = if !p.indices.is_empty() {
                    Some(
                        CpuAccessibleBuffer::from_iter(
                            device.clone(),
                            BufferUsage::index_buffer(),
                            false,
                            p.indices.iter().cloned(),
                        )
                        .unwrap(),
                    )
                } else {
                    None
                };

                PrimitiveInfo {
                    vertex_buffer,
                    index_buffer,
                    transform: p.transform.clone(),
                    material_name: p.material_name.clone(),
                }
            })
            .collect()
    }
}

pub struct MaterialInfo {
    pub descriptor_set: Arc<DescriptorSet>,
}

impl MaterialInfo {
    pub fn new(
        material: &Material,
        image_samplers: &HashMap<String, ImageData>,
        pipeline_type: Pipeline,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        device: Arc<Device>,
    ) -> Self {
        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();

        let dummy_color = "DUMMY_COLOR".to_string();
        let dummy_normal = "DUMMY_NORMAL".to_string();
        let dummy_metal_rough = "DUMMY_METAL_ROUGH".to_string();
        let dummy_ao = "DUMMY_AO".to_string();

        let set = match pipeline_type {
            Pipeline::Shaded => {
                let color_data = image_samplers[material
                    .textures
                    .base_color
                    .as_ref()
                    .unwrap_or(&dummy_color)
                    .as_str()]
                .clone();
                let normal_data = image_samplers[material
                    .textures
                    .normal
                    .as_ref()
                    .unwrap_or(&dummy_normal)
                    .as_str()]
                .clone();
                let metal_rough_data = image_samplers[material
                    .textures
                    .metallic_roughness
                    .as_ref()
                    .unwrap_or(&dummy_metal_rough)
                    .as_str()]
                .clone();
                let ao_data = image_samplers
                    [material.textures.ao.as_ref().unwrap_or(&dummy_ao).as_str()]
                .clone();

                let vertex_uniform_buffer = DeviceLocalBuffer::<vert::ty::Data>::new(
                    device.clone(),
                    BufferUsage::uniform_buffer_transfer_destination(),
                    device.active_queue_families(),
                )
                .unwrap();

                let fragment_uniform_buffer = DeviceLocalBuffer::<frag::ty::Data>::new(
                    device.clone(),
                    BufferUsage::uniform_buffer_transfer_destination(),
                    device.active_queue_families(),
                )
                .unwrap();

                let vk_set = PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(vertex_uniform_buffer.clone())
                    .unwrap()
                    .add_sampled_image(color_data.view, color_data.sampler)
                    .unwrap()
                    .add_sampled_image(normal_data.view, normal_data.sampler)
                    .unwrap()
                    .add_sampled_image(metal_rough_data.view, metal_rough_data.sampler)
                    .unwrap()
                    .add_sampled_image(ao_data.view, ao_data.sampler)
                    .unwrap()
                    .add_buffer(fragment_uniform_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap();

                DescriptorSet::new(Arc::new(vk_set))
                    .with_vertex_uniform_buffer(vertex_uniform_buffer.clone())
                    .with_fragment_uniform_buffer(fragment_uniform_buffer.clone())
            }
            // TODO probably need to re-work this.
            _ => unimplemented!()
        };

        Self {
            descriptor_set: Arc::new(set),
        }
    }
}

#[derive(Clone)]
pub struct ImageData {
    pub view: Arc<dyn ImageViewAbstract + Send + Sync>,
    pub sampler: Arc<Sampler>,
}

impl ImageData {
    fn new(
        texture: &Texture<u8>,
        format: Format,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self {
        let (image, _) = ImmutableImage::from_iter(
            texture.pixels.iter().cloned(),
            ImageDimensions::Dim2d {
                width: texture.width,
                height: texture.height,
                array_layers: 1,
            },
            MipmapsCount::One,
            format,
            queue.clone(),
        )
        .unwrap();

        let view = ImageView::new(image).unwrap();

        let sampler = Sampler::simple_repeat_linear_no_mipmap(device);

        Self {
            view: Arc::new(view),
            sampler,
        }
    }
}
