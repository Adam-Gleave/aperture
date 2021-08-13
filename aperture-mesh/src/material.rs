use cgmath::{Vector3, Vector4};

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub base_color_factor: Vector4<f32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub reflectance: f32,
    pub emissive_factor: Vector3<f32>,
    pub textures: TextureSet,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            base_color_factor: Vector4::new(1.0, 1.0, 1.0, 1.0),
            metallic_factor: 0.0,
            roughness_factor: 0.4,
            reflectance: 0.5,
            emissive_factor: Vector3::new(0.0, 0.0, 0.0),
            textures: TextureSet::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct TextureSet {
    pub base_color: Option<String>,
    pub normal: Option<String>,
    pub metallic_roughness: Option<String>,
    pub ao: Option<String>,
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
            name: "Unnamed".to_string(),
            format: ImageFormat::R8G8B8A8,
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
