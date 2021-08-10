use cgmath::{Vector3, Vector4};

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub base_color_factor: Vector4<f32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub reflectance: f32,
    pub emissive_factor: Vector3<f32>,
    pub textures: Textures,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "<Unnamed>".to_string(),
            base_color_factor: Vector4::new(1.0, 1.0, 1.0, 1.0),
            metallic_factor: 0.0,
            roughness_factor: 0.4,
            reflectance: 0.5,
            emissive_factor: Vector3::new(0.0, 0.0, 0.0),
            textures: Textures::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Textures {
    pub base_color: Option<usize>,
    pub normal: Option<usize>,
    pub metallic_roughness: Option<usize>,
}

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub format: ImageFormat,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Default for Texture {
    fn default() -> Self {
        Self { 
            name: "<Unnamed>".to_string(),
            format: ImageFormat::R16G16B16,
            pixels: vec![],
            width: 0,
            height: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    R8,
    R8G8,
    R8G8B8,
    R8G8B8A8,
    B8G8R8,
    B8G8R8A8,
    R16,
    R16G16,
    R16G16B16,
    R16G16B16A16,
}
