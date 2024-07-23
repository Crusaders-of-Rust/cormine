use std::{
    io::{
        Read,
        Seek,
        Write,
    },
    path::Path,
};

use anyhow::Result;
use bevy::{
    math::IVec3,
    prelude::Query,
};

use crate::{
    chunk::Chunk,
    voxel::Voxel,
    world::World,
};

pub struct SaveData {
    pub seed: u32,
    pub voxels: Vec<(IVec3, Voxel)>,
}

impl SaveData {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path)?;
        Self::from_reader(Serializer { cursor: file })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_reader(Serializer {
            cursor: std::io::Cursor::new(bytes),
        })
    }

    fn from_reader<Cursor>(mut reader: Serializer<Cursor>) -> Result<Self>
    where
        Cursor: Seek + Read,
    {
        let seed = reader.read_u32()?;
        let mut voxels = Vec::new();
        loop {
            let Ok(x) = reader.read_leb128_signed() else {
                break;
            };
            let y = reader.read_leb128_signed()?;
            let z = reader.read_leb128_signed()?;
            let kind = reader.read_byte().and_then(TryInto::try_into)?;
            voxels.push((IVec3::new(x as _, y as _, z as _), Voxel { kind }))
        }
        Ok(Self { seed, voxels })
    }

    pub fn from_world(query: Query<&Chunk>, world: &World) -> Self {
        let seed = world.seed();
        let noise_map = crate::terrain::generate_noise_map(1024, 1024, seed);
        let mut voxels = Vec::new();
        for (_, chunk) in world.iter() {
            let chunk = query.get(chunk).expect("invalid chunk in world");
            for (vox_pos, vox) in chunk.iter_pos() {
                if crate::terrain::block_at_position(vox_pos, &noise_map) != vox.kind() {
                    voxels.push((vox_pos.as_ivec3(), *vox));
                }
            }
        }
        Self { seed, voxels }
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P, replace: bool) -> Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.read(false).write(true);
        if replace {
            options.create(true);
            options.truncate(true);
        } else {
            options.create_new(true);
        }
        let file = options.open(path)?;
        self.to_writer(Serializer { cursor: file })
    }

    fn to_writer<Cursor>(&self, mut writer: Serializer<Cursor>) -> Result<()>
    where
        Cursor: Seek + Write,
    {
        writer.write_u32(self.seed)?;
        for (vox_pos, vox) in &self.voxels {
            writer.write_leb128_signed(vox_pos.x as _)?;
            writer.write_leb128_signed(vox_pos.y as _)?;
            writer.write_leb128_signed(vox_pos.z as _)?;
            writer.write_byte(vox.kind() as u8)?;
        }
        Ok(())
    }
}

struct Serializer<Cursor> {
    cursor: Cursor,
}

impl<Cursor> Serializer<Cursor>
where
    Cursor: Seek + Read,
{
    pub fn read_byte(&mut self) -> Result<u8> {
        let mut byte = [0];
        self.cursor.read_exact(&mut byte)?;
        Ok(byte[0])
    }

    pub fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut bytes = [0; N];
        self.cursor.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_bytes()?))
    }

    pub fn read_leb128_signed(&mut self) -> Result<i64> {
        Ok(leb128::read::signed(&mut self.cursor)?)
    }
}

impl<Cursor> Serializer<Cursor>
where
    Cursor: Seek + Write,
{
    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        let byte = [byte];
        self.cursor.write_all(&byte)?;
        Ok(())
    }

    pub fn write_bytes<const N: usize>(&mut self, bytes: [u8; N]) -> Result<()> {
        self.cursor.write_all(&bytes)?;
        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write_bytes(u32::to_le_bytes(val))
    }

    pub fn write_leb128_signed(&mut self, value: i64) -> Result<()> {
        leb128::write::signed(&mut self.cursor, value)?;
        Ok(())
    }
}
