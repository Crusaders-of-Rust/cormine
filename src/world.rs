use crate::chunk::{Chunk, ChunkPosition};
use crate::voxel::{Voxel, VoxelPosition};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Default, Resource)]
pub struct World {
    chunk_map: HashMap<ChunkPosition, Entity>,
}

impl World {
    pub fn add_chunk(&mut self, pos: ChunkPosition, entity: Entity) {
        assert!(
            self.chunk_map.insert(pos, entity).is_none(),
            "Overwriting chunk in map"
        );
    }

    pub fn chunk_containing(&self, pos: VoxelPosition) -> Option<Entity> {
        let chunk_base: ChunkPosition = pos.into();
        self.chunk_map.get(&chunk_base).copied()
    }

    pub fn voxel_at<'a>(&self, pos: VoxelPosition, chunks: &'a Query<&Chunk>) -> Option<&'a Voxel> {
        let chunk_base: ChunkPosition = pos.into();
        let chunk = self.chunk_map.get(&chunk_base).copied()?;
        let chunk = chunks.get(chunk).unwrap();
        let local_coord = pos.into();

        Some(chunk.voxel(local_coord))
    }
}
