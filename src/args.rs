use argh::FromArgs;
use std::path::PathBuf;

/// CoRmine.
#[derive(FromArgs)]
pub struct Arguments {
    /// disable vsync
    #[argh(switch)]
    pub disable_vsync: bool,
    /// save file to load
    #[argh(option, long = "load")]
    pub save_file: Option<PathBuf>,
    /// world seed to use
    #[argh(option)]
    pub seed: Option<u32>,
    /// radius in which to render chunks
    #[argh(option, default = "16")]
    pub load_distance: usize,
}
