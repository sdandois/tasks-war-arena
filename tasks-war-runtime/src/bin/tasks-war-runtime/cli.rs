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
    pub player_0_module_path: PathBuf,
    pub player_1_module_path: PathBuf,

    #[arg(short = 'o', long = "replay-output")]
    pub replay_output: Option<PathBuf>,

}

#[derive(Args, Debug)]

pub struct ReplayCommandArgs {
    pub replay_input: PathBuf,

    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,
}
