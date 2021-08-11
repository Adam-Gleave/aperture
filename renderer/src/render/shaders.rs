use vulkano::device::Device;

use std::sync::Arc;

pub mod vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../data/shaders/vert.glsl"
    }
}

pub mod frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../data/shaders/frag.glsl"
    }
}

pub mod depth {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../data/shaders/depth.glsl"
    }
}

pub struct Shaders {
    pub vertex: vert::Shader,
    pub fragment: frag::Shader,
    pub depth: depth::Shader,
}

impl Shaders {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            vertex: vert::Shader::load(device.clone()).unwrap(),
            fragment: frag::Shader::load(device.clone()).unwrap(),
            depth: depth::Shader::load(device).unwrap(),
        }
    }
}
