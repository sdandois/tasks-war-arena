use clap::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Subcommands>,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    Run(RunCommandArgs),
    Replay(ReplayCommandArgs),
}

#[derive(Args, Debug)]
pub struct RunCommandArgs {
    // Path of player 0 WASM module.
    pub player_0_module_path: PathBuf,
    // Path of player 1 WASM module.
    pub player_1_module_path: PathBuf,

    /// Path to create a replay file.
    #[arg(short = 'o', long = "replay-output")]
    pub replay_output: Option<PathBuf>,

    /// Specify the size of the board.
    ///
    /// Width and height will take the same value
    #[arg(short = 's', long = "board-size", default_value_t = 50)]
    pub board_size: usize,

    /// A seed for the pseudo-random number generator.
    #[arg(long = "seed")]
    pub seed: Option<u64>,
}

#[derive(Args, Debug)]

pub struct ReplayCommandArgs {
    pub replay_input: PathBuf,

    /// If active, key presses are needed for advancing replay.
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// If active, all commands are shown in replay.
    #[arg(short = 'a', long = "show-all")]
    pub show_all: bool,
}
