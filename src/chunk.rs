use std::ops::Add;

use crate::voxel::{
    LocalVoxelPosition,
    Voxel,
    VoxelPosition,
};

use bevy::prelude::*;
use ndarray::{
    Array3,
    ArrayView3,
    ArrayViewMut3,
    SliceInfo,
    SliceInfoElem,
};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I: i32 = CHUNK_SIZE as i32;
pub const MAX_HEIGHT: usize = 256;
const CHUNK_SHAPE: (usize, usize, usize) = (CHUNK_SIZE, MAX_HEIGHT, CHUNK_SIZE);

/// X and Z positions of a chunk. Will always be multiples of 16
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        Vec3::new(self.x() as _, 0.0, self.z() as _)
    }

    pub fn as_ivec3(&self) -> IVec3 {
        IVec3::new(self.x() as _, 0, self.z() as _)
    }
}

impl From<VoxelPosition> for ChunkPosition {
    fn from(voxel: VoxelPosition) -> Self {
        let mut pos = IVec2 {
            x: voxel.x(),
            y: voxel.z(),
        };
        pos -= pos.rem_euclid(IVec2::new(CHUNK_SIZE as _, CHUNK_SIZE as _));
        Self(pos)
    }
}

impl Add<LocalVoxelPosition> for ChunkPosition {
    type Output = VoxelPosition;

    fn add(self, rhs: LocalVoxelPosition) -> Self::Output {
        VoxelPosition::new(self.as_ivec3() + rhs.as_ivec3())
    }
}

impl Add<IVec3> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: IVec3) -> Self::Output {
        let pos = self.as_ivec3() + rhs;
        ChunkPosition::new(pos.x, pos.z)
    }
}

#[derive(Component, Clone)]
pub struct Chunk {
    voxels: Array3<Voxel>,
    position: ChunkPosition,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            voxels: Array3::from_elem(CHUNK_SHAPE, Voxel::default()),
            position: default(),
        }
    }

    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// Set the position of the (0, 0, 0) voxel
    pub fn with_position(self, position: ChunkPosition) -> Self {
        Self { position, ..self }
    }

    pub fn array(&self) -> &Array3<Voxel> {
        &self.voxels
    }

    /// Iterate over voxels, returning their local index as a tuple
    pub fn iter(&self) -> impl Iterator<Item = ((usize, usize, usize), &Voxel)> {
        self.voxels.indexed_iter()
    }

    /// Iterate over voxels, returning their index as a [`LocalVoxelPosition`]
    pub fn iter_local_pos(&self) -> impl Iterator<Item = (LocalVoxelPosition, &Voxel)> {
        self.iter()
            .map(|((x, y, z), vox)| (LocalVoxelPosition::new(x as _, y as _, z as _), vox))
    }

    /// Iterate over voxels, returning their index as a [`VoxelPosition`]
    pub fn iter_pos(&self) -> impl Iterator<Item = (VoxelPosition, &Voxel)> {
        self.iter_local_pos()
            .map(|(lp, vox)| (self.position + lp, vox))
    }

    pub fn voxel(&self, position: LocalVoxelPosition) -> &Voxel {
        let coord: [usize; 3] = position.into();
        &self.voxels[coord]
    }

    pub fn voxel_mut(&mut self, position: LocalVoxelPosition) -> &mut Voxel {
        let coord: [usize; 3] = position.into();
        &mut self.voxels[coord]
    }

    pub fn slice(
        &self,
        index: SliceInfo<[SliceInfoElem; 3], ndarray::Ix3, ndarray::Ix3>,
    ) -> ArrayView3<Voxel> {
        self.voxels.slice(index)
    }

    pub fn slice_mut(
        &mut self,
        index: SliceInfo<[SliceInfoElem; 3], ndarray::Ix3, ndarray::Ix3>,
    ) -> ArrayViewMut3<Voxel> {
        self.voxels.slice_mut(index)
    }
}
