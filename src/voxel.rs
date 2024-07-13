#[derive(Default, Copy, Clone, Debug)]
pub struct Voxel {
    kind: VoxelKind,
}

impl Voxel {
    pub const GRASS: Self = Self {
        kind: VoxelKind::Grass,
    };

    pub fn should_mesh(&self) -> bool {
        !matches!(self.kind, VoxelKind::Air)
    }
}

#[derive(Default, Copy, Clone, Debug)]
enum VoxelKind {
    #[default]
    Air,
    Grass,
}
