use crate::chunk::Chunk;
use bevy::math::vec3;
use std::ops::Add;

use crate::voxel::Voxel;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh::VertexAttributeValues::Float32x3;
use bevy::render::render_asset::RenderAssetUsages;
use ndarray::Array3;

#[derive(Component)]
pub struct HasMesh;

pub fn from_chunk(chunk: &Chunk) -> Mesh {
    trace!("Generating chunk mesh @ {}", chunk.position());
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    /// The 8 vertices making up a cube
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

    /// The indices into [`VERTICES`] making up each face of the cube
    const FACES: [[usize; 4]; 6] = [
        [0, 1, 2, 3], // front
        [5, 4, 7, 6], // back
        [4, 0, 3, 7], // left
        [1, 5, 6, 2], // right
        [4, 5, 1, 0], // bottom
        [3, 2, 6, 7], // top
    ];

    /// Get the 6 directly adjacent voxels, returning [`Voxel::AIR`] if on a chunk boundary
    fn get_adjacent_voxels(arr: &Array3<Voxel>, pos: Vec3) -> [Voxel; 6] {
        let pos = pos.as_ivec3();
        fn get_adjacent_voxel(arr: &Array3<Voxel>, pos: IVec3, dir: IVec3) -> Voxel {
            let pos = pos.add(dir);
            let usize_pos = pos.to_array().map(|x| x as usize);
            *arr.get(usize_pos).unwrap_or(&Voxel::AIR)
        }
        [
            get_adjacent_voxel(arr, pos, IVec3::NEG_Z),
            get_adjacent_voxel(arr, pos, IVec3::Z),
            get_adjacent_voxel(arr, pos, IVec3::NEG_X),
            get_adjacent_voxel(arr, pos, IVec3::X),
            get_adjacent_voxel(arr, pos, IVec3::NEG_Y),
            get_adjacent_voxel(arr, pos, IVec3::Y),
        ]
    }

    let mut vertices = Vec::new();

    fn add_face(chunk: &Chunk, vertices: &mut Vec<[f32; 3]>, pos: Vec3) {
        let adjacent = get_adjacent_voxels(chunk.array(), pos);
        for (face, adj) in FACES.into_iter().zip(adjacent.iter()) {
            // Don't render faces touching a solid voxel
            if adj.should_mesh() {
                continue;
            }
            let verts = face.map(|f| (pos + VERTICES[f]).to_array());
            // TODO: It uses less memory (40 vs 24 bytes per face) to use vertices only and no indexes
            // However, it should use less if we were to share vertices across the whole chunk
            for idx in [2, 1, 0, 3, 2, 0] {
                vertices.push(verts[idx]);
            }
        }
    }

    for ((x, y, z), _) in chunk.iter().filter(|(_, v)| v.should_mesh()) {
        let pos = vec3(x as f32, y as f32, z as f32);
        add_face(&chunk, &mut vertices, pos);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));

    mesh
}
