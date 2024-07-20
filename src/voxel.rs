use bevy::math::{IVec3, Vec3};

use crate::chunk::{ChunkPosition, CHUNK_SIZE, MAX_HEIGHT};

/// X, Y and Z coordinate of voxel withun the world
#[derive(Copy, Clone, Debug, Hash)]
pub struct VoxelPosition(IVec3);

impl VoxelPosition {
    pub fn new(position: IVec3) -> Self {
        Self(position)
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn y(&self) -> i32 {
        self.0.y
    }

    pub fn z(&self) -> i32 {
        self.0.z
    }

    pub fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x() as _, self.y() as _, self.z() as _)
    }
}

/// Position of a voxel within a chunk. Will all be within [0, CHUNK_DIMENSION_SIZE]
#[derive(Copy, Clone, Debug, Hash)]
pub struct LocalVoxelPosition {
    x: u8,
    y: u32,
    z: u8,
}

impl LocalVoxelPosition {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        debug_assert!(
            matches!(x as _, 0..CHUNK_SIZE)
                && matches!(y as _, 0..MAX_HEIGHT)
                && matches!(z as _, 0..CHUNK_SIZE),
            "({x}, {y}, {z}) is not within chunk ranges"
        );
        Self {
            x: x as _,
            y,
            z: z as _,
        }
    }

    pub fn x(&self) -> u32 {
        self.x.into()
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn z(&self) -> u32 {
        self.z.into()
    }
}

impl From<VoxelPosition> for LocalVoxelPosition {
    fn from(position: VoxelPosition) -> Self {
        let chunk_pos: ChunkPosition = position.into();
        let x = position.x() - chunk_pos.x();
        let y = position.y();
        let z = position.z() - chunk_pos.z();
        debug_assert!(
            x >= 0 && y >= 0 && z >= 0,
            "({x}, {y}, {z}) is not positive"
        );

        LocalVoxelPosition::new(x as _, y as _, z as _)
    }
}

impl From<LocalVoxelPosition> for [usize; 3] {
    fn from(position: LocalVoxelPosition) -> Self {
        [position.x as _, position.y as _, position.z as _]
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Voxel {
    pub kind: VoxelKind,
}

impl Voxel {
    pub const AIR: Self = Self {
        kind: VoxelKind::Air,
    };

    pub const STONE: Self = Self {
        kind: VoxelKind::Stone,
    };

    pub const GRASS: Self = Self {
        kind: VoxelKind::Grass,
    };

    pub const WATER: Self = Self {
        kind: VoxelKind::Water,
    };

    pub fn should_mesh(&self) -> bool {
        !matches!(self.kind, VoxelKind::Air)
    }

    pub fn kind(&self) -> VoxelKind {
        self.kind
    }

    pub fn clear(&mut self) {
        self.kind = VoxelKind::Air;
    }
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(u8)]
pub enum VoxelKind {
    #[default]
    Air = 255,
    Stone = 0,
    Grass = 1,
    Water = 2,
    Snow = 3,
    Dirt = 4,
}
