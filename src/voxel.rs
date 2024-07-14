#[derive(Default, Copy, Clone, Debug)]
pub struct Voxel {
    kind: VoxelKind,
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
}
