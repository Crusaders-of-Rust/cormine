use std::path::Path;

use crate::{
    chunk::Chunk,
    world::World,
};

use bevy::prelude::*;
use cormine_shared::save::SaveData as SaveDataInner;

#[derive(Resource)]
pub struct SaveData(SaveDataInner);

impl SaveData {
    pub fn from_world(query: Query<&Chunk>, world: &World) -> Self {
        let noise_map = crate::terrain::generate_noise_map(1024, 1024, world.seed);
        let mut voxels = Vec::new();
        for (_, chunk) in world.iter() {
            let chunk = query.get(chunk).expect("invalid chunk in world");
            for (vox_pos, vox) in chunk.iter_pos() {
                if crate::terrain::block_at_position(vox_pos, &noise_map) != vox.kind() {
                    voxels.push((vox_pos.as_ivec3(), vox.kind()));
                }
            }
        }
        Self(SaveDataInner {
            seed: world.seed,
            voxels,
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        Self(SaveDataInner::from_file(path).expect("loading save from file"))
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P, replace: bool) {
        self.0.to_file(path, replace).expect("saving game to disk");
    }
}
