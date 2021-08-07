use cgmath::Matrix4;

use renderer_common::{Transform, VPosNorm};
use renderer_mesh::Mesh;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

use std::sync::{Arc, Mutex};

pub struct DrawInfo {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[VPosNorm]>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    pub transform: Arc<Mutex<Transform>>,
}

impl DrawInfo {
    pub fn has_indices(&self) -> bool {
        self.index_buffer.is_some()
    }

    pub fn composed_transform(&self) -> Matrix4<f32> {
        self.transform.lock().expect("poisoned_lock").clone().compose()
    }
}

pub fn generate_from_mesh(device: Arc<Device>, mesh: &Mesh) -> Vec<DrawInfo> {
    mesh.primitives.iter().map(|p| {
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
                .unwrap()
            )
        } else {
            None
        };

        DrawInfo { 
            vertex_buffer, 
            index_buffer,
            transform: p.transform.clone(),
        }
    })
    .collect()
}
