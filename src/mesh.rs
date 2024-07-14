use crate::chunk::Chunk;
use bevy::math::vec3;
use std::ops::Add;

use crate::voxel::{Voxel, VoxelKind};
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues::Float32x3;
use bevy::render::mesh::{MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::VertexFormat;
use ndarray::Array3;

#[derive(Component)]
pub struct HasMesh;

pub const VOXEL_VERTEX_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Data", 0x3bbb0d7d, VertexFormat::Uint32);

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
    let mut vertex_data = Vec::new();

    fn add_face(
        chunk: &Chunk,
        vertices: &mut Vec<[f32; 3]>,
        vertex_data: &mut Vec<u32>,
        material: VoxelKind,
        pos: Vec3,
    ) {
        let adjacent = get_adjacent_voxels(chunk.array(), pos);
        for (i, (face, adj)) in FACES.into_iter().zip(adjacent.iter()).enumerate() {
            let mut per_vertex_data = VertexData::new();
            per_vertex_data.set_normal_idx(i);
            per_vertex_data.set_material(material);
            // Don't render faces touching a solid voxel
            if adj.should_mesh() {
                continue;
            }
            let verts = face.map(|f| (pos + VERTICES[f]).to_array());
            // TODO: It uses less memory (40 vs 24 bytes per face) to use vertices only and no indexes
            // However, it should use less if we were to share vertices across the whole chunk
            for idx in [2, 1, 0, 3, 2, 0] {
                // TODO: vertex_data should hold more than just the normal index
                vertex_data.push(per_vertex_data.to_u32());
                vertices.push(verts[idx]);
            }
        }
    }

    for ((x, y, z), v) in chunk.iter().filter(|(_, v)| v.should_mesh()) {
        let pos = vec3(x as f32, y as f32, z as f32);
        add_face(&chunk, &mut vertices, &mut vertex_data, v.kind(), pos);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));
    mesh.insert_attribute(
        VOXEL_VERTEX_DATA,
        VertexAttributeValues::Uint32(vertex_data),
    );

    mesh
}

#[repr(transparent)]
#[derive(Copy, Clone)]
struct VertexData(u32);

impl VertexData {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set_normal_idx(&mut self, idx: usize) {
        assert!(idx <= 5);
        self.0 |= idx as u32;
    }

    pub fn set_material(&mut self, kind: VoxelKind) {
        self.0 |= (kind as u32) << 3;
    }

    pub fn to_u32(self) -> u32 {
        self.0
    }
}
