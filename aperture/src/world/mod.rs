use aperture_mesh::*;

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

#[derive(Default)]
pub struct World {
    pub meshes: HashMap<String, Mesh>,
    pub materials: HashMap<String, Material>,
    pub textures: HashMap<String, Texture>,
    pub default_material: Material,
}

impl World {
    pub fn load_gltf<P>(&mut self, path: P)
    where
        P: AsRef<Path> + Clone + Debug,
    {
        let (meshes, materials, textures) = gltf::load(path).unwrap();

        for mesh in meshes {
            self.meshes.insert(mesh.name.clone(), mesh);
        }

        for material in materials {
            self.materials.insert(material.name.clone(), material);
        }

        for texture in textures {
            self.textures.insert(texture.name.clone(), texture);
        }

        println!("Meshes: {:?}", self.meshes.keys());
        println!("Materials: {:?}", self.materials);
        println!("Textures: {:?}", self.textures.keys().collect::<Vec<_>>());

        self.default_material = Material::default();
    }
}
