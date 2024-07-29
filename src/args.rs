use argh::FromArgs;
use bevy::prelude::Resource;

/// CoRmine.
#[derive(FromArgs)]
pub struct Arguments {
    /// disable vsync
    #[argh(switch)]
    pub disable_vsync: bool,

    #[argh(subcommand)]
    pub commands: Option<ArgumentsCommands>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum ArgumentsCommands {
    Generate(ArgumentsGenerate),
    Load(ArgumentsLoad),
}

impl Default for ArgumentsCommands {
    fn default() -> Self {
        ArgumentsCommands::Generate(ArgumentsGenerate::default())
    }
}

/// Generate a new world
#[derive(FromArgs, Resource)]
#[argh(subcommand, name = "generate")]
pub struct ArgumentsGenerate {
    /// seed to use for world generation
    #[argh(option)]
    pub seed: Option<u32>,
}

impl Default for ArgumentsGenerate {
    fn default() -> Self {
        Self { seed: None }
    }
}

/// Load an existing save file
#[derive(FromArgs, Resource)]
#[argh(subcommand, name = "load")]
pub struct ArgumentsLoad {
    /// save file path
    #[argh(positional)]
    pub path: String,
}
