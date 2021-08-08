use crate::{Error, Material, Mesh, Primitive};

use cgmath::{Matrix4, Quaternion};

use renderer_common::{Transform, VPosNorm};

use std::{fmt::Debug, path::Path};

pub fn load<P>(path: P) -> Result<(Vec<Mesh>, Vec<Material>), Error>
where
    P: AsRef<Path> + Clone + Debug,
{
    let (document, buffers, _images) = gltf::import(path.clone())
        .map_err(|_| Error::NoSuchFile(path.as_ref().as_os_str().to_owned()))?;

    let materials = load_materials(&document).collect::<Vec<_>>();
    let meshes = load_nodes(&document, &buffers)?.collect::<Vec<_>>();

    Ok((meshes, materials))
}

fn load_materials<'a>(gltf: &'a gltf::Document) -> impl Iterator<Item = Material> + 'a {
    gltf.materials().map(|m| {
        let mut material = Material::default();

        if let Some(name) = m.name() {
            material.name = name.to_string();
        }

        let pbr = m.pbr_metallic_roughness();
        material.base_color_factor = pbr.base_color_factor().into();
        material.metallic_factor = pbr.metallic_factor();
        material.roughness_factor = pbr.roughness_factor();
        material.emissive_factor = m.emissive_factor().into();

        material
    })
}

fn load_nodes<'a>(
    gltf: &'a gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> Result<impl Iterator<Item = Mesh> + 'a, Error> {
    Ok(gltf
        .nodes()
        .filter(|n| n.mesh().is_some())
        .map(|n| {
            let m = n.mesh().unwrap();

            let n_transform = n.transform().decomposed();
            let transform = Transform {
                translation: Matrix4::from_translation(n_transform.0.into()),
                // GLTF quaternions are (x, y, z, w), but cgmath quaternions are (w, x, y, z).
                rotation: Matrix4::from(Quaternion::new(
                    n_transform.1[3],
                    n_transform.1[0],
                    n_transform.1[1],
                    n_transform.1[2],
                )),
                scale: Matrix4::from_nonuniform_scale(
                    n_transform.2[0],
                    n_transform.2[1],
                    n_transform.2[2],
                ),
            };

            let primitives = m
                .primitives()
                .map(|p| {
                    let reader = p.reader(|buffer| Some(&buffers[buffer.index()]));

                    let positions = reader
                        .read_positions()
                        .map_or(Err(Error::NoVerticesFound), |p| Ok(p.collect::<Vec<_>>()))?;

                    let normals =
                        reader
                            .read_normals()
                            .map_or(
                                vec![[0.0, 0.0, 0.0]; positions.len()],
                                |n| n.collect(),
                            );

                    if positions.len() != normals.len() {
                        return Err(Error::MismatchedVerticesNormals);
                    }

                    let vertices = positions
                        .iter()
                        .zip(normals.iter())
                        .map(|(p, n)| VPosNorm {
                            position: *p,
                            normal: *n,
                        })
                        .collect::<Vec<_>>();

                    let indices = reader
                        .read_indices()
                        .take()
                        .map_or(vec![], |i| i.into_u32().collect());

                    let mut primitive = Primitive::default();
                    primitive.vertices = vertices;
                    primitive.indices = indices;
                    primitive.material_index = p.material().index();
                    
                    primitive.set_transform(transform.clone());

                    Ok(primitive)
                })
                .collect::<Result<Vec<_>, Error>>()?;

            let mut mesh = Mesh::default();
            if let Some(name) = m.name() {
                mesh.name = name.to_string();
            }

            for p in primitives {
                mesh.add_primitive(p);
            }

            Ok(mesh)
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter())
}
