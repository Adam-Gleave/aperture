pub mod light;

use aperture_mesh::*;
use cgmath::Point3;

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

use self::light::PointLight;

#[derive(Default)]
pub struct World {
    pub meshes: HashMap<String, Mesh>,
    pub materials: HashMap<String, Material>,
    pub textures: HashMap<String, Texture>,
    pub default_material: Material,
    pub lights: Vec<PointLight>,
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

        self.lights = vec![
            PointLight {
                position: Point3::new(20.0, 60.0, 70.0),
                color: [1.0, 1.0, 1.0],
            },
            PointLight {
                position: Point3::new(-9.0, 2.0, -4.0),
                color: [1.0, 1.0, 1.0],
            },
            PointLight {
                position: Point3::new(-4.0, -6.0, 5.0),
                color: [1.0, 1.0, 1.0],
            },
            PointLight {
                position: Point3::new(2.0, 9.0, -3.0),
                color: [1.0, 1.0, 1.0],
            },
        ];

        self.default_material = Material::default();
    }
}
