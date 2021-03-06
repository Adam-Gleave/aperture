use aperture_common::{Transform, VPosNormTex};

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

mod error;
mod material;
mod obj;

pub mod gltf;

pub use error::Error;
pub use material::{ImageFormat, Material, Texture, TextureSet};

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            primitives: vec![],
        }
    }
}

impl Mesh {
    pub(crate) fn add_primitive(&mut self, mut p: Primitive) {
        p.index = self.primitives.len();
        self.primitives.push(p);
    }
}

#[derive(Debug, Default)]
pub struct Primitive {
    pub index: usize,
    pub material_name: Option<String>,
    pub vertices: Vec<VPosNormTex>,
    pub indices: Vec<u32>,
    pub transform: Arc<Mutex<Transform>>,
}

impl Primitive {
    pub fn set_transform(&self, transform: Transform) {
        *self.transform.lock().expect("poisoned lock") = transform;
    }
}
