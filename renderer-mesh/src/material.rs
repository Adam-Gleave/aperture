use cgmath::{Vector3, Vector4};

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub base_color_factor: Vector4<f32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_factor: Vector3<f32>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "<Unnamed>".to_string(),
            base_color_factor: Vector4::new(1.0, 1.0, 1.0, 1.0),
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            emissive_factor: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}
