use std::{
    fs::OpenOptions,
    path::Path,
    sync::LazyLock,
};

use anyhow::Result;
use cormine_shared::{
    save::{
        SaveData,
        Serializer,
    },
    voxel::VoxelKind,
};
use fontdue::{
    layout::{
        CoordinateSystem,
        Layout,
        TextStyle,
    },
    Font,
    FontSettings,
};
use glam::{
    IVec2,
    IVec3,
    UVec2,
};

const FONT_BYTES: &[u8] = include_bytes!("../pixeloid.ttf");
static FONT: LazyLock<Font> =
    LazyLock::new(|| Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap());

fn rasterize(string: &str, size: f32) -> (Vec<UVec2>, UVec2) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    layout.append(&[&*FONT], &TextStyle::new(string, size, 0));
    let mut positions = Vec::new();

    let mut max_x = 0;
    let mut max_y = 0;
    for glyph in layout.glyphs() {
        let x_start = glyph.x as u32;
        let y_start = glyph.y as u32;
        let (_, bitmap) = FONT.rasterize(glyph.parent, size);
        for (yb, y) in (y_start..y_start + glyph.height as u32).enumerate() {
            // Hack; Glyphs are upside down for some reason so just invert this
            let y = glyph.height as u32 - y;
            for (xb, x) in (x_start..x_start + glyph.width as u32).enumerate() {
                if bitmap[xb + yb * glyph.width] > 127 {
                    positions.push(UVec2::new(x, y));
                    max_x = u32::max(x, max_x);
                    max_y = u32::max(y, max_y);
                }
            }
        }
    }
    (positions, UVec2::new(max_x, max_y))
}

// Adds a string to the world and returns the maximum X and Y positions of it
fn add_string_to_world(
    string: &str,
    start: IVec3,
    world: &mut SaveData,
    block: VoxelKind,
) -> IVec2 {
    let (positions, max) = rasterize(string, 9.0);
    for pos in positions {
        world
            .voxels
            .push((start + pos.as_ivec2().extend(start.z), block));
    }
    start.truncate() + max.as_ivec2()
}

fn add_box_to_world(
    start: IVec3,
    end: IVec3,
    world: &mut SaveData,
    block: VoxelKind,
    filled: bool,
) {
    assert!(IVec3::cmpgt(end, start).all());
    for x in start.x..=end.x {
        for y in start.y..=end.y {
            for z in start.z..=end.z {
                if filled
                    || (x == start.x
                        || x == end.x
                        || y == start.y
                        || y == end.y
                        || z == start.z
                        || z == end.z)
                {
                    world.voxels.push((IVec3::new(x, y, z), block));
                }
            }
        }
    }
}

fn challenge1() -> (&'static str, SaveData) {
    let seed = rand::random();
    let name = "cormine1";
    let mut wd = SaveData {
        seed,
        width: 16,
        length: 8,
        voxels: Vec::new(),
    };

    let start = IVec3::new(0, 90, 8);
    let end =
        add_string_to_world("corCTF{w4llh4cks}", start, &mut wd, VoxelKind::Stone).extend(start.z);

    // To avoid bugs with headglitching, make box extra thick
    for box_sz in [10, 11, 12] {
        let box_start = start - box_sz;
        let box_end = end + box_sz;
        add_box_to_world(box_start, box_end, &mut wd, VoxelKind::Bedrock, false);
    }
    (name, wd)
}

fn save_data_to_file<P: AsRef<Path>>(data: &SaveData, path: P) -> Result<()> {
    let f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let mut ser = Serializer::new(f);
    ser.write_u32(data.seed)?;
    ser.write_leb128_unsigned(data.width as _)?;
    ser.write_leb128_unsigned(data.length as _)?;
    for &(pos, vox) in data.voxels.iter() {
        ser.write_leb128_signed(pos.x as _)?;
        ser.write_leb128_signed(pos.y as _)?;
        ser.write_leb128_signed(pos.z as _)?;
        ser.write_byte(vox as u8)?;
    }
    Ok(())
}

fn main() {
    for (name, world) in [challenge1].map(|f| f()) {
        save_data_to_file(&world, format!("{name}.cms"))
            .unwrap_or_else(|e| panic!("serializing {name}: `{e:?}`"))
    }
}
