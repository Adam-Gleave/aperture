use renderer_common::VPosNorm;

use renderer_mesh::Mesh;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

use std::sync::Arc;

pub struct DrawInfo {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[VPosNorm]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
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

        let index_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::index_buffer(),
            false,
            p.indices.iter().cloned(),
        )
        .unwrap();

        DrawInfo { vertex_buffer, index_buffer }
    })
    .collect()
}
