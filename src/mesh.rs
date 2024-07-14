use crate::chunk::Chunk;
use bevy::math::vec3;

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues::Float32x3;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Component)]
pub struct HasMesh;

pub fn from_chunk(chunk: &Chunk) -> Mesh {
    trace!("Generating chunk mesh @ {}", chunk.position());
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    const VERTICES: [Vec3; 8] = [
        vec3(0., 0., 0.),
        vec3(1., 0., 0.),
        vec3(1., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 0., 1.),
        vec3(1., 0., 1.),
        vec3(1., 1., 1.),
        vec3(0., 1., 1.),
    ];

    const FACES: [[usize; 4]; 6] = [
        [0, 1, 2, 3],
        [5, 4, 7, 6],
        [4, 0, 3, 7],
        [1, 5, 6, 2],
        [4, 5, 1, 0],
        [3, 2, 6, 7],
    ];

    for ((x, y, z), _) in chunk.iter().filter(|(_, v)| v.should_mesh()) {
        let pos = vec3(x as f32, y as f32, z as f32);
        for face in FACES {
            let start_idx = vertices.len();
            vertices.push((pos + VERTICES[face[0]]).to_array());
            vertices.push((pos + VERTICES[face[1]]).to_array());
            vertices.push((pos + VERTICES[face[2]]).to_array());
            vertices.push((pos + VERTICES[face[3]]).to_array());
            for idx in [2, 1, 0, 3, 2, 0] {
                indices.push(start_idx as u32 + idx);
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
