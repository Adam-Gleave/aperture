#[derive(Default, Debug, Clone)]
pub struct VPosNorm {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

vulkano::impl_vertex!(VPosNorm, position, normal);
