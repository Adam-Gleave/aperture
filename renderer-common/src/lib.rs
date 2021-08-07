use cgmath::{Matrix4, Zero};

#[derive(Default, Debug, Clone)]
pub struct VPosNorm {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

vulkano::impl_vertex!(VPosNorm, position, normal);

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Matrix4<f32>,
    pub rotation: Matrix4<f32>,
    pub scale: Matrix4<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: Matrix4::zero(),
            rotation: Matrix4::zero(),
            scale: Matrix4::zero(),
        }
    }

    pub fn compose(&self) -> Matrix4<f32> {
        // Row-major order
        self.scale * self.rotation * self.translation
    }
}
