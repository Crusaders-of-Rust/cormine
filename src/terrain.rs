use std::{cmp::Ordering, collections::HashMap};

use bevy::prelude::*;
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    BasicMulti, Perlin, ScalePoint,
};
use rand::{thread_rng, Rng};

use crate::{
    args::{ArgumentsGenerate, ArgumentsLoad},
    chunk::{Chunk, ChunkPosition, CHUNK_SIZE, MAX_HEIGHT},
    save::SaveData,
    voxel::{LocalVoxelPosition, VoxelKind, VoxelPosition},
    WorldSize,
};

pub fn generate_noise_map(width: usize, height: usize, seed: u32) -> NoiseMap {
    let mut basic_multi = BasicMulti::<Perlin>::new(seed);
    basic_multi.octaves = 4;
    basic_multi.persistence = 0.5;
    basic_multi.lacunarity = 2.0;

    let mut scaled_bm: ScalePoint<_> = ScalePoint::new(basic_multi);
    scaled_bm.x_scale = 4.5;
    scaled_bm.y_scale = 4.5;

    PlaneMapBuilder::<_, 3>::new(&scaled_bm)
        .set_size(width, height)
        .build()
}

pub fn ground_height_to_voxel(height: usize, is_top_level: bool) -> VoxelKind {
    if height > 100 && is_top_level {
        return VoxelKind::Snow;
    }
    if !(50..=96).contains(&height) {
        return VoxelKind::Stone;
    }
    if is_top_level {
        VoxelKind::Grass
    } else {
        VoxelKind::Dirt
    }
}

pub fn generate_chunks(
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
    options: Res<ArgumentsGenerate>,
    size: Res<WorldSize>,
) {
    let seed = options.seed.unwrap_or_else(|| {
        let mut rng = thread_rng();
        rng.gen()
    });
    world.set_seed(seed);
    let chunk_count = size.width;
    let noise_map = generate_noise_map(1024, 1024, seed);
    for chunk_x in 0..chunk_count {
        for chunk_z in 0..chunk_count {
            let chunk_pos =
                ChunkPosition::new((chunk_x * CHUNK_SIZE) as i32, (chunk_z * CHUNK_SIZE) as i32);
            let mut chunk = Chunk::new().with_position(chunk_pos);

            for x in 0..CHUNK_SIZE {
                for y in 0..MAX_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        let local_pos = LocalVoxelPosition::new(x as _, y as _, z as _);
                        let global_pos = chunk_pos + local_pos;
                        chunk.voxel_mut(local_pos).kind = block_at_position(global_pos, &noise_map);
                    }
                }
            }

            world.add_chunk(chunk_pos, commands.spawn((Name::new("Chunk"), chunk)).id());
        }
    }
}

pub fn block_at_position(pos: VoxelPosition, noise: &NoiseMap) -> VoxelKind {
    let noise_val = noise.get_value(pos.x() as usize, pos.z() as usize);
    let mut height = (noise_val.powf(2.0) * MAX_HEIGHT as f64) as usize;
    let y = pos.y() as usize;
    height += 64;
    if height == 64 {
        let water_floor: usize = ((noise_val.powf(1.2) * MAX_HEIGHT as f64) as usize).max(32);
        if y < water_floor {
            VoxelKind::Stone
        } else if y < height + 2 {
            VoxelKind::Water
        } else {
            VoxelKind::Air
        }
    } else {
        if height > MAX_HEIGHT - 1 {
            height = MAX_HEIGHT - 1;
        }
        match y.cmp(&height) {
            Ordering::Less => ground_height_to_voxel(y, false),
            Ordering::Equal => ground_height_to_voxel(height, true),
            Ordering::Greater => VoxelKind::Air,
        }
    }
}

pub fn load_chunks(
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
    options: Res<ArgumentsLoad>,
    size: Res<WorldSize>,
) {
    let save = SaveData::from_file(&options.path).unwrap();
    let changes = save
        .voxels
        .iter()
        .map(|(pos, vox)| (*pos, *vox))
        .collect::<HashMap<_, _>>();
    let seed = save.seed;
    world.set_seed(seed);
    let chunk_count = size.width;
    let noise_map = generate_noise_map(1024, 1024, seed);
    for chunk_x in 0..chunk_count {
        for chunk_z in 0..chunk_count {
            let chunk_pos =
                ChunkPosition::new((chunk_x * CHUNK_SIZE) as i32, (chunk_z * CHUNK_SIZE) as i32);
            let mut chunk = Chunk::new().with_position(chunk_pos);

            for x in 0..CHUNK_SIZE {
                for y in 0..MAX_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        let local_pos = LocalVoxelPosition::new(x as _, y as _, z as _);
                        let global_pos = chunk_pos + local_pos;
                        let voxel_kind = if let Some(change) = changes.get(&global_pos.as_ivec3()) {
                            change.kind
                        } else {
                            block_at_position(global_pos, &noise_map)
                        };
                        chunk.voxel_mut(local_pos).kind = voxel_kind;
                    }
                }
            }

            world.add_chunk(chunk_pos, commands.spawn((Name::new("Chunk"), chunk)).id());
        }
    }
}
