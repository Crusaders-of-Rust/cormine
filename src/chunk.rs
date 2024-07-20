use crate::voxel::Voxel;

use bevy::prelude::*;
use ndarray::{Array3, ArrayView3, ArrayViewMut3, SliceInfo, SliceInfoElem};

pub const CHUNK_SIZE: usize = 16;
pub const MAX_HEIGHT: usize = 256;
const CHUNK_SHAPE: (usize, usize, usize) = (CHUNK_SIZE, MAX_HEIGHT, CHUNK_SIZE);

#[derive(Component)]
pub struct Chunk {
    voxels: Array3<Voxel>,
    position: IVec3,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            voxels: Array3::from_elem(CHUNK_SHAPE, Voxel::default()),
            position: default(),
        }
    }

    pub fn position(&self) -> IVec3 {
        self.position
    }

    /// Set the position of the (0, 0, 0) voxel
    pub fn with_position(mut self, pos: IVec2) -> Self {
        self.position = IVec3 {
            x: pos.x,
            y: 0,
            z: pos.y,
        };
        self
    }

    pub fn array(&self) -> &Array3<Voxel> {
        &self.voxels
    }

    pub fn iter(&self) -> impl Iterator<Item = ((usize, usize, usize), &Voxel)> {
        self.voxels.indexed_iter()
    }

    pub fn voxel(&self, position: [usize; 3]) -> &Voxel {
        &self.voxels[position]
    }

    pub fn voxel_mut(&mut self, position: [usize; 3]) -> &mut Voxel {
        &mut self.voxels[position]
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
