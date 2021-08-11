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
