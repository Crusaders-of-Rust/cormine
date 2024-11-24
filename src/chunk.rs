use std::ops::Add;

use crate::{
    octree::{
        Octant,
        OctantPos,
        Octree,
    },
    voxel::{
        LocalVoxelPosition,
        Voxel,
        VoxelPosition,
    },
};

use bevy::{
    math::{
        ivec2,
        ivec3,
        vec3,
    },
    prelude::*,
};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I: i32 = CHUNK_SIZE as i32;
pub const MAX_HEIGHT: usize = 256;

/// X and Z positions of a chunk. Will always be multiples of [`CHUNK_SIZE`]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ChunkPosition(IVec2);

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> Self {
        debug_assert!(x % CHUNK_SIZE_I == 0);
        debug_assert!(z % CHUNK_SIZE_I == 0);
        Self(IVec2 { x, y: z })
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn z(&self) -> i32 {
        self.0.y
    }

    pub fn as_vec3(&self) -> Vec3 {
        vec3(self.x() as _, 0.0, self.z() as _)
    }

    pub fn as_ivec3(&self) -> IVec3 {
        ivec3(self.x() as _, 0, self.z() as _)
    }

    // Calculate the 4 possible chunks neighbouring this one
    pub fn neighbouring_chunks(&self) -> NeighbouringChunks {
        let neg_x = self + IVec3::NEG_X * CHUNK_SIZE_I;
        let x = self + IVec3::X * CHUNK_SIZE_I;
        let neg_z = self + IVec3::NEG_Z * CHUNK_SIZE_I;
        let z = self + IVec3::Z * CHUNK_SIZE_I;

        NeighbouringChunks { neg_x, x, neg_z, z }
    }

    pub fn spawn_distance(&self) -> usize {
        std::cmp::max(self.x().abs(), self.z().abs()) as usize
    }

    pub fn in_range_of_spawn(&self, range: usize) -> bool {
        self.x() <= range as i32 && self.z() <= range as i32
    }
}

impl From<VoxelPosition> for ChunkPosition {
    fn from(voxel: VoxelPosition) -> Self {
        let mut pos = IVec2 {
            x: voxel.x(),
            y: voxel.z(),
        };
        pos -= pos.rem_euclid(ivec2(CHUNK_SIZE as _, CHUNK_SIZE as _));
        Self(pos)
    }
}

impl From<Vec3> for ChunkPosition {
    fn from(pos: Vec3) -> Self {
        pos.as_ivec3().into()
    }
}

impl From<IVec3> for ChunkPosition {
    fn from(pos: IVec3) -> Self {
        let mut pos = IVec2 { x: pos.x, y: pos.z };
        pos -= pos.rem_euclid(ivec2(CHUNK_SIZE as _, CHUNK_SIZE as _));
        Self(pos)
    }
}

impl Add<LocalVoxelPosition> for &ChunkPosition {
    type Output = VoxelPosition;

    fn add(self, rhs: LocalVoxelPosition) -> Self::Output {
        VoxelPosition::new(self.as_ivec3() + rhs.as_ivec3())
    }
}

impl Add<IVec2> for &ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: IVec2) -> Self::Output {
        ChunkPosition::new(self.x() + rhs.x, self.z() + rhs.y)
    }
}

impl Add<IVec3> for &ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: IVec3) -> Self::Output {
        let pos = self.as_ivec3() + rhs;
        ChunkPosition::new(pos.x, pos.z)
    }
}

/// The 4 chunks bordering a given chunk
pub struct NeighbouringChunks {
    pub neg_x: ChunkPosition,
    pub x: ChunkPosition,
    pub neg_z: ChunkPosition,
    pub z: ChunkPosition,
}

impl NeighbouringChunks {
    pub fn all(&self) -> [ChunkPosition; 4] {
        [self.neg_x, self.x, self.neg_z, self.z]
    }
}

#[derive(Component, Clone, Default)]
pub struct ChunkVoxels {
    // Stack of CHUNK_SIZE^3 cubes; starting at Y=0 and ending at Y=MAX
    voxels: [Octree<CHUNK_SIZE, Voxel>; MAX_HEIGHT / CHUNK_SIZE],
}

impl ChunkVoxels {
    pub fn new() -> Self {
        Self::default()
    }

    /// Iterate over voxels, returning their local index as a tuple
    pub fn iter(&self) -> impl Iterator<Item = ((usize, usize, usize), &Voxel)> {
        self.voxels.iter().enumerate().flat_map(|(y_off, octree)| {
            octree.iter().map(move |(pos, voxel)| {
                (
                    (pos + OctantPos::new(0, y_off * CHUNK_SIZE, 0)).into(),
                    voxel,
                )
            })
        })
    }

    /// Iterate over the internal octants in no specific order
    pub fn iter_octants(&self) -> impl Iterator<Item = Octant<Voxel>> + '_ {
        self.voxels.iter().enumerate().flat_map(|(y_off, octree)| {
            octree.iter_octants().map(move |octant| Octant {
                position: octant.position + OctantPos::new(0, y_off * CHUNK_SIZE, 0),
                ..*octant
            })
        })
    }
    /// Iterate over voxels, returning their [`LocalVoxelPosition`]
    pub fn iter_local_pos(&self) -> impl Iterator<Item = (LocalVoxelPosition, &Voxel)> {
        self.iter()
            .map(|((x, y, z), vox)| (LocalVoxelPosition::new(x as _, y as _, z as _), vox))
    }

    /// Iterate over voxels, returning their [`VoxelPosition`]
    pub fn iter_world_pos(
        &self,
        chunk_pos: ChunkPosition,
    ) -> impl Iterator<Item = (VoxelPosition, &Voxel)> {
        self.iter_local_pos()
            .map(move |(pos, vox)| (&chunk_pos + pos, vox))
    }

    pub fn voxel(&self, position: LocalVoxelPosition) -> &Voxel {
        let (idx, pos) = lvp_to_octree_idx(position);
        self.voxels[idx].get(pos)
    }

    pub fn voxel_mut(&mut self, position: LocalVoxelPosition) -> &mut Voxel {
        let (idx, pos) = lvp_to_octree_idx(position);
        self.voxels[idx].get_mut(pos)
    }
}

fn lvp_to_octree_idx(lvp: LocalVoxelPosition) -> (usize, OctantPos) {
    let y_idx = lvp.y() as usize / CHUNK_SIZE;
    (
        y_idx,
        OctantPos::new_u32(lvp.x(), lvp.y() % CHUNK_SIZE as u32, lvp.z()),
    )
}
