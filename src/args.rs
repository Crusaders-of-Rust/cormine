use argh::FromArgs;
use bevy::prelude::Resource;

/// CoRmine.
#[derive(FromArgs)]
pub struct Arguments {
    #[argh(subcommand)]
    pub commands: ArgumentsCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum ArgumentsCommands {
    Generate(ArgumentsGenerate),
    Load(ArgumentsLoad),
}

/// Generate a new world
#[derive(FromArgs, Resource)]
#[argh(subcommand, name = "generate")]
pub struct ArgumentsGenerate {
    /// seed to use for world generation
    #[argh(option)]
    pub seed: Option<u32>,

    /// width and length of the world (in chunks)
    #[argh(option)]
    pub size: Option<usize>,
}

/// Load an existing save file
#[derive(FromArgs, Resource)]
#[argh(subcommand, name = "load")]
pub struct ArgumentsLoad {
    /// save file path
    #[argh(positional)]
    pub _path: String,
}
