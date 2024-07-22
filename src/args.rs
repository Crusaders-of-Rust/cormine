use argh::FromArgs;
use bevy::prelude::Resource;

/// CoRmine.
#[derive(FromArgs)]
pub struct Arguments {
    #[argh(subcommand)]
    pub commands: ArgumentsCommands,
    /// width of the world in chunks
    #[argh(option, default = "16")]
    pub width: usize,
    /// length of the world in chunks
    #[argh(option, default = "16")]
    pub length: usize,
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
}

/// Load an existing save file
#[derive(FromArgs, Resource)]
#[argh(subcommand, name = "load")]
pub struct ArgumentsLoad {
    /// save file path
    #[argh(positional)]
    pub path: String,
}
