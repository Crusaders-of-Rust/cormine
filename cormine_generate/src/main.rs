use std::{
    collections::HashMap,
    fs::{
        File,
        OpenOptions,
    },
    io::{
        Result,
        Write,
    },
    path::Path,
    sync::LazyLock,
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

struct Serializer<'file> {
    file: &'file mut File,
}

impl<'file> Serializer<'file> {
    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        let byte = [byte];
        self.file.write_all(&byte)?;
        Ok(())
    }

    pub fn write_bytes<const N: usize>(&mut self, bytes: [u8; N]) -> Result<()> {
        self.file.write_all(&bytes)?;
        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write_bytes(u32::to_le_bytes(val))
    }

    pub fn write_leb128_signed(&mut self, value: i64) -> Result<()> {
        leb128::write::signed(&mut self.file, value)?;
        Ok(())
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VoxelKind {
    #[default]
    Air = 255,
    Stone = 0,
    Grass = 1,
    Water = 2,
    Snow = 3,
    Dirt = 4,
    Bedrock = 5,
}

#[derive(Debug)]
struct WorldData {
    seed: u32,
    blocks: HashMap<(i32, i32, i32), VoxelKind>,
}

impl WorldData {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let mut ser = Serializer { file: &mut f };
        ser.write_u32(self.seed)?;
        for (&(x, y, z), &vox) in self.blocks.iter() {
            ser.write_leb128_signed(x as _)?;
            ser.write_leb128_signed(y as _)?;
            ser.write_leb128_signed(z as _)?;
            ser.write_byte(vox as u8)?;
        }
        Ok(())
    }
}

const FONT_BYTES: &[u8] = include_bytes!("../pixeloid.ttf");
static FONT: LazyLock<Font> =
    LazyLock::new(|| Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap());

fn rasterize(string: &str, size: f32) -> (Vec<(u32, u32)>, (u32, u32)) {
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
                    positions.push((x, y));
                    max_x = u32::max(x, max_x);
                    max_y = u32::max(y, max_y);
                }
            }
        }
    }
    (positions, (max_x, max_y))
}

// Adds a string to the world and returns the maximum X and Y positions of it
fn add_string_to_world(
    string: &str,
    (start_x, start_y, start_z): (i32, i32, i32),
    world: &mut WorldData,
    block: VoxelKind,
) -> (i32, i32) {
    let (positions, (max_x, max_y)) = rasterize(string, 9.0);
    for (x, y) in positions {
        world.blocks.insert(
            (
                x.overflowing_add_signed(start_x).0 as _,
                y.overflowing_add_signed(start_y).0 as _,
                start_z,
            ),
            block,
        );
    }
    (
        max_x.overflowing_add_signed(start_x).0 as _,
        max_y.overflowing_add_signed(start_y).0 as _,
    )
}

fn add_box_to_world(
    start: (i32, i32, i32),
    end: (i32, i32, i32),
    world: &mut WorldData,
    block: VoxelKind,
    filled: bool,
) {
    assert!(end.0 > start.0);
    assert!(end.1 > start.1);
    assert!(end.2 > start.2);
    for x in start.0..=end.0 {
        for y in start.1..=end.1 {
            for z in start.2..=end.2 {
                if filled
                    || (x == start.0
                        || x == end.0
                        || y == start.1
                        || y == end.1
                        || z == start.2
                        || z == end.2)
                {
                    world.blocks.insert((x, y, z), block);
                }
            }
        }
    }
}

fn challenge1() -> (&'static str, WorldData) {
    let seed = rand::random();
    let name = "cormine1";
    let array = HashMap::new();
    let mut wd = WorldData {
        seed,
        blocks: array,
    };

    let start = (-40, 80, 0);
    let (end_x, end_y) = add_string_to_world("corCTF{wallhacks}", start, &mut wd, VoxelKind::Stone);

    // To avoid bugs with headglitching, make box extra thick
    for box_sz in [10, 11, 12] {
        let box_start = (start.0 - box_sz, start.1 - box_sz, start.2 - box_sz);
        let box_end = (end_x + box_sz, end_y + box_sz, start.2 + box_sz);
        add_box_to_world(box_start, box_end, &mut wd, VoxelKind::Bedrock, false);
    }
    (name, wd)
}

fn main() {
    for (name, world) in [challenge1].map(|f| f()) {
        world
            .save_to_file(format!("{name}.cms"))
            .unwrap_or_else(|e| panic!("serializing {name}: `{e:?}`"))
    }
}
