use argh::FromArgs;
use bevy::prelude::Resource;

/// CoRmine.
#[derive(FromArgs, Resource)]
pub struct Arguments {
    #[argh(subcommand)]
    pub commands: ArgumentsCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum ArgumentsCommands {
    Generate(ArgumentsGenerate),
}

/// Generate a new world
#[derive(FromArgs)]
#[argh(subcommand, name = "generate")]
pub struct ArgumentsGenerate {
    /// seed to use for world generation
    #[argh(option)]
    pub seed: Option<u32>,

    /// width and length of the world (in chunks)
    #[argh(option)]
    pub size: Option<usize>,
}
