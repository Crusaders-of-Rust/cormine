use std::collections::HashMap;
use std::ops::Add;

use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues::Float32x3;
use bevy::render::mesh::{MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::VertexFormat;
use bit_field::BitField;

use crate::chunk::Chunk;
use crate::voxel::{Voxel, VoxelKind};

#[derive(Component)]
/// Marker component indicating a mesh is present and up to date
pub struct HasMesh;

pub const VOXEL_VERTEX_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Data", 0x3bbb0d7d, VertexFormat::Uint32);

pub fn from_chunk(chunk: Chunk, adj_chunks: Vec<Chunk>) -> Mesh {
    trace!("Generating chunk mesh @ {:?}", chunk.position());
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

    /// The indices into [`VERTICES`] making up each face of the cube, as
    /// well as the direction of the face
    const FACES: [([usize; 4], IVec3); 6] = [
        ([0, 1, 2, 3], IVec3::Z),     // front
        ([5, 4, 7, 6], IVec3::NEG_Z), // back
        ([4, 0, 3, 7], IVec3::NEG_X), // left
        ([1, 5, 6, 2], IVec3::X),     // right
        ([4, 5, 1, 0], IVec3::NEG_Y), // bottom
        ([3, 2, 6, 7], IVec3::Y),     // top
    ];

    fn get_adjacent_voxel(map: &HashMap<[i32; 3], Voxel>, pos: IVec3, dir: IVec3) -> Voxel {
        let pos = pos.add(dir);
        let item = map.get(&pos.to_array());
        *item.unwrap_or(&Voxel::AIR)
    }

    /// Get the 6 directly adjacent voxels, returning [`Voxel::AIR`] if on a chunk boundary
    fn get_adjacent_voxels(map: &HashMap<[i32; 3], Voxel>, pos: Vec3) -> [Voxel; 6] {
        let pos = pos.as_ivec3();
        [
            get_adjacent_voxel(map, pos, IVec3::NEG_Z),
            get_adjacent_voxel(map, pos, IVec3::Z),
            get_adjacent_voxel(map, pos, IVec3::NEG_X),
            get_adjacent_voxel(map, pos, IVec3::X),
            get_adjacent_voxel(map, pos, IVec3::NEG_Y),
            get_adjacent_voxel(map, pos, IVec3::Y),
        ]
    }

    let mut vertices = Vec::new();
    let mut vertex_data = Vec::new();

    let base_voxels = chunk.array().clone();
    let mut voxels: HashMap<[i32; 3], Voxel> = base_voxels
        .indexed_iter()
        .map(|((x, y, z), v)| {
            let pos = [x as i32, y as i32, z as i32];
            (pos, *v)
        })
        .collect();

    for adj_chunk in adj_chunks {
        for (adj_pos, adj_voxel) in adj_chunk.iter() {
            let new_pos = [
                (adj_chunk.position().x() + adj_pos.0 as i32) - chunk.position().x(),
                adj_pos.1 as i32,
                (adj_chunk.position().z() + adj_pos.2 as i32) - chunk.position().z(),
            ];
            voxels.insert(new_pos, *adj_voxel);
        }
    }

    fn add_cube(
        voxels: &HashMap<[i32; 3], Voxel>,
        vertices: &mut Vec<[f32; 3]>,
        vertex_data: &mut Vec<u32>,
        material: VoxelKind,
        pos: Vec3,
    ) {
        let adjacent = get_adjacent_voxels(voxels, pos);
        #[cfg(feature = "short_blocks")]
        let height = match material.height() {
            1.0 => 1.0,
            height => {
                let above_pos = (pos.as_ivec3() + IVec3::Y).to_array();
                match voxels.get(&above_pos) {
                    Some(above) if above.kind() != material => height,
                    _ => 1.0,
                }
            }
        };
        #[cfg(not(feature = "short_blocks"))]
        let height = 1.0;

        let voxel_size = Vec3::new(1.0, height, 1.0);

        fn face_neighbour_offsets(direction: IVec3) -> [IVec3; 8] {
            let (perp1, perp2) = match direction {
                IVec3::NEG_Z => (IVec3::Y, IVec3::X),
                IVec3::Z => (IVec3::Y, IVec3::NEG_X),

                IVec3::NEG_Y => (IVec3::X, IVec3::Z),
                IVec3::Y => (IVec3::X, IVec3::NEG_Z),

                IVec3::NEG_X => (IVec3::Y, IVec3::NEG_Z),
                IVec3::X => (IVec3::Y, IVec3::Z),
                _ => unreachable!(),
            };

            [
                direction + perp1,
                direction + perp1 + perp2,
                direction + perp2,
                direction - perp1 + perp2,
                direction - perp1,
                direction - perp1 - perp2,
                direction - perp2,
                direction + perp1 - perp2,
            ]
        }

        // Calculate the 4 AO values for a face. See:
        // https://0fps.net/2013/07/03/ambient-occlusion-for-minecraft-like-worlds/
        // https://playspacefarer.com/ambient-occlusion/
        // FIXME: Some of the values in this are wrong, leading to the AO looking a bit wonky
        fn ao_values_for_face(
            map: &HashMap<[i32; 3], Voxel>,
            pos: IVec3,
            face_direction: IVec3,
        ) -> [u32; 4] {
            // Offsets to neighbouring voxels of a face - starting at the 'middle left'
            // and continuing anti-clockwise

            let offsets = face_neighbour_offsets(face_direction);
            let voxels = offsets.map(|off| get_adjacent_voxel(map, pos, off));
            [[2, 3, 4], [0, 1, 2], [6, 7, 0], [4, 5, 6]]
                .map(|[a, b, c]| [voxels[a], voxels[b], voxels[c]])
                .map(|voxels| voxels.map(|v| v.casts_shadow()))
                .map(|[s1, corner, s2]| match (s1, corner, s2) {
                    (true, _, true) => 0,
                    (true, true, false) | (false, true, true) => 1,
                    (false, false, false) => 3,
                    _ => 2,
                })
        }

        for (i, ((face_vertices, face_direction), adj)) in
            FACES.into_iter().zip(adjacent.iter()).enumerate()
        {
            let mut per_vertex_data = VertexData::new();
            per_vertex_data.set_normal_idx(i as u32);
            per_vertex_data.set_material(material);
            // Don't render faces touching a solid voxel
            if !adj.transparent() && voxel_size.y == 1.0 {
                continue;
            }
            // Don't render faces between multiple transparent blocks of the same type
            if adj.transparent() && adj.kind() == material {
                continue;
            }

            let verts = face_vertices.map(|f| (pos + VERTICES[f] * voxel_size).to_array());
            let ao_vals = ao_values_for_face(voxels, pos.as_ivec3(), face_direction);
            // TODO: It uses less memory (40 vs 24 bytes per face) to use vertices only and no indexes
            // However, it should use less if we were to share vertices across the whole chunk
            for idx in [2, 1, 0, 3, 2, 0] {
                per_vertex_data.set_uv(idx as u32);
                let vertex = verts[idx];
                per_vertex_data.set_neighbours(ao_vals[idx]);
                vertices.push(vertex);
                vertex_data.push(per_vertex_data.to_u32());
            }
        }
    }

    for ((x, y, z), v) in chunk.iter().filter(|(_, v)| v.should_mesh()) {
        let pos = vec3(x as f32, y as f32, z as f32);
        add_cube(&voxels, &mut vertices, &mut vertex_data, v.kind(), pos);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));
    mesh.insert_attribute(
        VOXEL_VERTEX_DATA,
        VertexAttributeValues::Uint32(vertex_data),
    );

    mesh
}

// TODO: use proper bitfields
#[repr(transparent)]
#[derive(Copy, Clone)]
struct VertexData(u32);

impl VertexData {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set_normal_idx(&mut self, idx: u32) {
        assert!(idx <= 5);
        self.0.set_bits(0..3, idx);
    }

    pub fn set_material(&mut self, kind: VoxelKind) {
        self.0.set_bits(3..6, kind as u32);
    }

    pub fn set_uv(&mut self, uv: u32) {
        self.0.set_bits(6..8, uv);
    }

    pub fn set_neighbours(&mut self, neighbours: u32) {
        self.0.set_bits(8..10, neighbours);
    }

    pub fn to_u32(self) -> u32 {
        self.0
    }
}
