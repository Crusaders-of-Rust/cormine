use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::voxel::Voxel;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Default, Resource)]
pub struct World {
    chunk_map: HashMap<IVec3, Entity>,
}

impl World {
    pub fn add_chunk(&mut self, pos: IVec3, entity: Entity) {
        debug_assert!(
            pos % CHUNK_SIZE as i32 == IVec3::ZERO,
            "Invalid chunk coordinate {pos}"
        );
        assert!(
            self.chunk_map.insert(pos, entity).is_none(),
            "Overwriting chunk in map"
        );
    }

    pub fn voxel_at<'a>(&self, pos: IVec3, chunks: &'a Query<&Chunk>) -> Option<&'a Voxel> {
        let chunk_base = pos
            - pos.rem_euclid(IVec3::new(
                CHUNK_SIZE as _,
                CHUNK_SIZE as _,
                CHUNK_SIZE as _,
            ));
        let chunk = self.chunk_map.get(&chunk_base).copied()?;
        let chunk = chunks.get(chunk).unwrap();
        let local_coord = (pos - chunk_base).to_array().map(|i| i as usize);

        Some(chunk.voxel(local_coord))
    }
}
