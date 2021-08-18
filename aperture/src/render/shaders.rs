use vulkano::device::Device;

use std::sync::Arc;

pub mod cube_vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../data/shaders/skybox.vert"
    }
}

pub mod cube_frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../data/shaders/skybox.frag"
    }
}

pub mod offscreen_cube_vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../data/shaders/offscreen_hdri.vert"
    }
}

pub mod offscreen_cube_frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../data/shaders/offscreen_hdri.frag"
    }
}

pub mod vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../data/shaders/pbr.vert"
    }
}

pub mod frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../data/shaders/pbr.frag"
    }
}

pub mod depth {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../data/shaders/depth.glsl"
    }
}


pub struct Shaders {
    pub cubemap_vert: cube_vert::Shader,
    pub cubemap_frag: cube_frag::Shader,
    pub offscreen_cube_vert: offscreen_cube_vert::Shader,
    pub offscreen_cube_frag: offscreen_cube_frag::Shader,
    pub vertex: vert::Shader,
    pub fragment: frag::Shader,
    pub depth: depth::Shader,
}

impl Shaders {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            cubemap_vert: cube_vert::Shader::load(device.clone()).unwrap(),
            cubemap_frag: cube_frag::Shader::load(device.clone()).unwrap(),
            offscreen_cube_vert: offscreen_cube_vert::Shader::load(device.clone()).unwrap(),
            offscreen_cube_frag: offscreen_cube_frag::Shader::load(device.clone()).unwrap(),
            vertex: vert::Shader::load(device.clone()).unwrap(),
            fragment: frag::Shader::load(device.clone()).unwrap(),
            depth: depth::Shader::load(device).unwrap(),
        }
    }
}
