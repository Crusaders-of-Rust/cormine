use std::sync::Arc;

use crate::{
    chunk::{
        ChunkPosition,
        ChunkVoxels,
    },
    save,
    voxel::{
        Voxel,
        VoxelPosition,
    },
};
use bevy::{
    prelude::*,
    utils::HashMap,
};
use noise::utils::NoiseMap;

#[derive(Resource)]
pub struct World {
    pub seed: u32,
    chunk_map: HashMap<ChunkPosition, Entity>,
    pub noise_map: Arc<NoiseMap>,
}

impl World {
    pub fn from_seed(seed: u32) -> Self {
        let noise_map = crate::terrain::generate_noise_map(1024, 1024, seed);
        Self {
            seed,
            chunk_map: default(),
            noise_map: Arc::new(noise_map),
        }
    }

    pub fn add_chunk(&mut self, pos: ChunkPosition, entity: Entity) {
        assert!(
            self.chunk_map.insert(pos, entity).is_none(),
            "Overwriting chunk in map"
        );
    }

    pub fn chunk_at(&self, pos: ChunkPosition) -> Option<Entity> {
        self.chunk_map.get(&pos).copied()
    }

    pub fn chunk_containing(&self, pos: VoxelPosition) -> Option<Entity> {
        let chunk_base: ChunkPosition = pos.into();
        self.chunk_map.get(&chunk_base).copied()
    }

    pub fn voxel_at<'a>(
        &self,
        pos: VoxelPosition,
        chunks: &'a Query<&ChunkVoxels>,
    ) -> Option<&'a Voxel> {
        let chunk_base: ChunkPosition = pos.into();
        let chunk = self.chunk_map.get(&chunk_base).copied()?;
        let chunk = chunks.get(chunk).ok()?;
        let local_coord = pos.into();

        Some(chunk.voxel(local_coord))
    }

    /// Iterate over each chunk entity and it's position
    pub fn iter(&self) -> impl Iterator<Item = (ChunkPosition, Entity)> + '_ {
        self.chunk_map.iter().map(|(p, e)| (*p, *e))
    }
}

pub fn process_save_events(query: Query<&ChunkVoxels>, world: Res<World>) {
    let save = save::SaveData::from_world(query, &world);
    save.to_file("game.cms", true);
    info!("Saved to `game.cms`");
}
