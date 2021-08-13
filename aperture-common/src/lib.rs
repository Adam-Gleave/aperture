use cgmath::{Matrix4, One};

#[derive(Default, Debug, Clone)]
pub struct VPosNormTex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv_coord: [f32; 2],
}

vulkano::impl_vertex!(VPosNormTex, position, normal, uv_coord);

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
            translation: Matrix4::one(),
            rotation: Matrix4::one(),
            scale: Matrix4::one(),
        }
    }

    pub fn compose(&self) -> Matrix4<f32> {
        // Column-major order
        self.translation * self.rotation * self.scale
    }
}
