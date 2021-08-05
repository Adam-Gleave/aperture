use std::{fmt::Debug, path::Path};

use renderer_common::VPosNorm;

mod error;
mod obj;

pub use error::Error;

pub struct Mesh {
    pub vertices: Vec<VPosNorm>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn from_obj<P>(path: P) -> Result<Mesh, Error>
    where
        P: AsRef<Path> + Clone + Debug,
    {
        obj::load(path)
    }
}
