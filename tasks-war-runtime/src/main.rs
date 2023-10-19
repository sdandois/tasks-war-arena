use std::{fs::File, path::PathBuf};
use tasks_war_runtime::runtime::bots::WasmBotFactory;
use tasks_war_runtime::runtime::GameRunner;

use clap::{arg, Parser};

use tracing_subscriber::filter::LevelFilter;

use atty::Stream;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    player_0_module_path: PathBuf,
    player_1_module_path: PathBuf,

    #[arg(short = 'o', long = "replay-file")]
    replay_file: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_ansi(atty::is(Stream::Stdout))
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .with_level(false)
        .with_max_level(LevelFilter::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let path0: PathBuf = args.player_0_module_path;
    let path1: PathBuf = args.player_1_module_path;

    let factory = WasmBotFactory::new(path0, path1)?;

    let runner = GameRunner::new(factory);

    let _result = runner.run_game();

    if let Some(replay_file) = args.replay_file {
        let f = File::create(replay_file)?;

        let memento = _result.dump();

        serde_json::to_writer(f, &memento)?;
    }

    Ok(())
}
