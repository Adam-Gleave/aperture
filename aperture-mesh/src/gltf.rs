use crate::{
    material::{ImageFormat, Texture, TextureSet},
    Error, Material, Mesh, Primitive,
};

use aperture_common::{Transform, VPosNormTex};
use gltf::mesh::util::ReadTexCoords;

use cgmath::{Matrix4, Quaternion};

use std::{collections::HashSet, fmt::Debug, path::Path};

pub fn load<P>(path: P) -> Result<(Vec<Mesh>, Vec<Material>, Vec<Texture<u8>>), Error>
where
    P: AsRef<Path> + Clone + Debug,
{
    let (document, buffers, images) = gltf::import(path.clone())
        .map_err(|_| Error::NoSuchFile(path.as_ref().as_os_str().to_owned()))?;

    let textures = load_textures(&document, &images);
    let materials = load_materials(&document, &textures);
    let meshes = load_nodes(&document, &buffers, &materials)?.collect::<Vec<_>>();

    Ok((meshes, materials, textures))
}

fn load_materials(gltf: &gltf::Document, textures: &[Texture<u8>]) -> Vec<Material> {
    gltf.materials()
        .map(|m| {
            let mut material = Material::default();

            if let Some(name) = m.name() {
                material.name = name.to_string();
            }

            let pbr = m.pbr_metallic_roughness();
            material.base_color_factor = pbr.base_color_factor().into();
            material.metallic_factor = pbr.metallic_factor();
            material.roughness_factor = pbr.roughness_factor();
            material.emissive_factor = m.emissive_factor().into();

            let mut texture_set = TextureSet::default();

            if let Some(t) = pbr.base_color_texture() {
                let i = t.texture().index();
                texture_set.base_color.replace(textures[i].name.clone());
            }

            if let Some(t) = m.normal_texture() {
                let i = t.texture().index();
                texture_set.normal.replace(textures[i].name.clone());
            }

            if let Some(t) = pbr.metallic_roughness_texture() {
                let i = t.texture().index();
                texture_set
                    .metallic_roughness
                    .replace(textures[i].name.clone());
            }

            if let Some(t) = m.occlusion_texture() {
                let i = t.texture().index();
                texture_set.ao.replace(textures[i].name.clone());
            }

            material.textures = texture_set;
            material
        })
        .collect()
}

fn load_textures(gltf: &gltf::Document, images: &[gltf::image::Data]) -> Vec<Texture<u8>> {
    let mut textures = vec![];
    let mut texture_names = HashSet::new();

    for t in gltf.textures() {
        let idx = t.source().index();
        let image = images
            .get(idx)
            .expect("could not find image given by texture index");

        let mut texture = Texture::default();

        if let Some(name) = t.name() {
            texture.name = name.to_string();
        }

        texture.format = format(image.format);
        texture.width = image.width;
        texture.height = image.height;

        let pixels_rgba = image
            .pixels
            .chunks(3)
            .map(|rgb| [rgb[0], rgb[1], rgb[2], 255]);

        let mut pixels_u8 = vec![];
        for rgba in pixels_rgba {
            pixels_u8.push(rgba[0]);
            pixels_u8.push(rgba[1]);
            pixels_u8.push(rgba[2]);
            pixels_u8.push(rgba[3]);
        }

        texture.pixels = pixels_u8;

        // FIXME unoptimised
        let mut count = 1;
        let mut name = texture.name.clone();

        while texture_names.contains(&name) {
            name = format!("{}_{}", &texture.name, count.to_string());
            count += 1;
        }

        texture_names.insert(name.clone());
        texture.name = name;

        textures.push(texture);
    }

    textures
}

fn format(f: gltf::image::Format) -> ImageFormat {
    match f {
        gltf::image::Format::R8 => ImageFormat::R8,
        gltf::image::Format::R8G8 => ImageFormat::R8G8,
        gltf::image::Format::R8G8B8 => ImageFormat::R8G8B8,
        gltf::image::Format::R8G8B8A8 => ImageFormat::R8G8B8A8,
        gltf::image::Format::B8G8R8 => ImageFormat::B8G8R8,
        gltf::image::Format::B8G8R8A8 => ImageFormat::B8G8R8A8,
        gltf::image::Format::R16 => ImageFormat::R16,
        gltf::image::Format::R16G16 => ImageFormat::R16G16,
        gltf::image::Format::R16G16B16 => ImageFormat::R16G16B16,
        gltf::image::Format::R16G16B16A16 => ImageFormat::R16G16B16A16,
    }
}

fn load_nodes<'a>(
    gltf: &'a gltf::Document,
    buffers: &[gltf::buffer::Data],
    materials: &[Material],
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

                    let normals = reader
                        .read_normals()
                        .map_or(vec![[0.0, 0.0, 0.0]; positions.len()], |n| n.collect());

                    if positions.len() != normals.len() {
                        return Err(Error::MismatchedVerticesNormals);
                    }

                    let coords = if let Some(coords) = reader.read_tex_coords(0) {
                        // FIXME: convert all to f32 for now.
                        match coords {
                            ReadTexCoords::U8(uv) => uv
                                .map(|arr| [arr[0] as f32, arr[1] as f32])
                                .collect::<Vec<_>>(),
                            ReadTexCoords::U16(uv) => uv
                                .map(|arr| [arr[0] as f32, arr[1] as f32])
                                .collect::<Vec<_>>(),
                            ReadTexCoords::F32(uv) => uv.collect::<Vec<_>>(),
                        }
                    } else {
                        vec![]
                    };

                    let vertices = positions
                        .iter()
                        .zip(normals.iter())
                        .zip(coords.iter())
                        .map(|((p, n), c)| VPosNormTex {
                            position: *p,
                            normal: *n,
                            uv_coord: *c,
                        })
                        .collect::<Vec<_>>();

                    let indices = reader
                        .read_indices()
                        .take()
                        .map_or(vec![], |i| i.into_u32().collect());

                    let mut primitive = Primitive::default();
                    primitive.vertices = vertices;
                    primitive.indices = indices;

                    if let Some(index) = p.material().index() {
                        primitive.material_name = Some(materials[index].name.clone());
                    }

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
