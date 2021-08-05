use crate::{Error, Mesh};

use renderer_common::VPosNorm;

use cgmath::{InnerSpace, Vector3};

use std::{fmt::Debug, path::Path};

pub fn load<P>(path: P) -> Result<Mesh, Error> 
where
    P: AsRef<Path> + Clone + Debug,
{
    let model = tobj::load_obj(
        path.clone(),
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
    );

    let (models, _) = match model {
        Ok(result) => result,
        Err(_) => Err(Error::LoadError(path.as_ref().as_os_str().to_owned()))?,
    };

    let mesh = &models.first().unwrap().mesh;

    let positions = (0..mesh.positions.len() / 3).map(|i| {
        [
            mesh.positions[i * 3],
            mesh.positions[i * 3 + 1],
            mesh.positions[i * 3 + 2],
        ]
    })
    .collect::<Vec<_>>();

    let mut normals = vec![[0f32; 3]; positions.len()];
    if !mesh.normals.is_empty() {
        for i in 0..mesh.normals.len() / 3 {
            let n = [
                mesh.normals[i * 3],
                mesh.normals[i * 3 + 1],
                mesh.normals[i * 3 + 2],
            ];

            normals[i] = n;
        }
    } else {
        for i in 0..mesh.indices.len() / 3 {
            let a = positions[mesh.indices[i * 3] as usize];
            let b = positions[mesh.indices[i * 3 + 1] as usize];
            let c = positions[mesh.indices[i * 3 + 2] as usize];

            let v_a = Vector3::new(a[0], a[1], a[2]);
            let v_b = Vector3::new(b[0], b[1], b[2]);
            let v_c = Vector3::new(c[0], c[1], c[2]);
            
            let n = (v_b - v_a).cross(v_c - v_a).normalize();

            normals[mesh.indices[i * 3] as usize] = [n.x, n.y, n.z];
            normals[mesh.indices[i * 3 + 1] as usize] = [n.x, n.y, n.z];
            normals[mesh.indices[i * 3 + 2] as usize] = [n.x, n.y, n.z];
        }
    }

    let vertices = (0..positions.len()).map(|i| {
        VPosNorm {
            position: positions[i],
            normal: normals[i],
        }
    })
    .collect::<Vec<_>>();

    Ok(Mesh {
        vertices,
        indices: mesh.indices.clone(),
    })
}