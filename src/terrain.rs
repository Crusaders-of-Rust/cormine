use bevy::prelude::*;
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    BasicMulti, Perlin, ScalePoint,
};
use rand::{thread_rng, Rng};

use crate::voxel::VoxelKind;
use crate::{
    chunk::{Chunk, ChunkPosition, CHUNK_SIZE, MAX_HEIGHT},
    voxel::LocalVoxelPosition,
};

fn generate_noise_map(width: usize, height: usize) -> NoiseMap {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen();

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
    if height < rand::thread_rng().gen_range(49..51) || height > 96 {
        return VoxelKind::Stone;
    }
    return if is_top_level {
        VoxelKind::Grass
    } else {
        VoxelKind::Dirt
    };
}

pub fn generate_chunks(mut commands: Commands, mut world: ResMut<crate::world::World>) {
    let noise_map = generate_noise_map(1024, 1024);

    let chunk_count: i32 = 16;
    for chunk_x in 0..chunk_count {
        for chunk_z in 0..chunk_count {
            let chunk_pos =
                ChunkPosition::new(chunk_x * CHUNK_SIZE as i32, chunk_z * CHUNK_SIZE as i32);
            let mut chunk = Chunk::new().with_position(chunk_pos);

            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let noise_val = noise_map.get_value(
                        (chunk_x as usize * CHUNK_SIZE) + x,
                        (chunk_z as usize * CHUNK_SIZE) as usize + z,
                    );
                    let mut height: usize = (noise_val.powf(2.0) * MAX_HEIGHT as f64) as usize;
                    height += 64;
                    if height == 64 {
                        let water_floor: usize = ((noise_val.powf(1.2) * MAX_HEIGHT as f64)
                            as usize)
                            .max(rand::thread_rng().gen_range(31..33));
                        for y in 0..water_floor {
                            chunk
                                .voxel_mut(LocalVoxelPosition::new(x as _, y as _, z as _))
                                .kind = VoxelKind::Stone;
                        }
                        for y in water_floor..=(height + 1) {
                            chunk
                                .voxel_mut(LocalVoxelPosition::new(x as _, y as _, z as _))
                                .kind = VoxelKind::Water;
                        }
                    } else {
                        if height > MAX_HEIGHT - 1 {
                            height = MAX_HEIGHT - 1;
                        }
                        chunk
                            .voxel_mut(LocalVoxelPosition::new(x as _, height as _, z as _))
                            .kind = ground_height_to_voxel(height, true);
                        for y in 0..height {
                            chunk
                                .voxel_mut(LocalVoxelPosition::new(x as _, y as _, z as _))
                                .kind = ground_height_to_voxel(y, false);
                        }
                    }
                }
            }

            world.add_chunk(chunk_pos, commands.spawn((Name::new("Chunk"), chunk)).id());
        }
    }
}
