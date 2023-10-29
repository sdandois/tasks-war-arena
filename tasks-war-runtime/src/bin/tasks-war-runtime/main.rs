use anyhow::Ok;
use rand::{rngs, SeedableRng, Rng};
use tasks_war_runtime::game::{GameConfig, BoardSize};
use std::io::Read;
use std::{fs::File, path::PathBuf};
use tasks_war_runtime::game_replay::GameReplay;
use tasks_war_runtime::runtime::GameRunner;
use tasks_war_runtime::{game_memento::GameMemento, runtime::bots::WasmBotFactory};

use clap::*;

use tracing::{event, Level};
use tracing_subscriber::filter::LevelFilter;

use atty::Stream;

mod cli;

use cli::{Args, ReplayCommandArgs, RunCommandArgs};

fn execute_run_command(run_args: RunCommandArgs) -> anyhow::Result<()> {
    let path0: PathBuf = run_args.player_0_module_path;
    let path1: PathBuf = run_args.player_1_module_path;

    let factory = WasmBotFactory::new(path0, path1)?;

    let mut seed_generator = rngs::SmallRng::from_entropy(); 

    let seed: u64 = seed_generator.gen();
    let config = GameConfig {
        board_size: BoardSize(run_args.board_size, run_args.board_size),
        seed,
    };

    let runner = GameRunner::with_config(factory, config);

    let _result = runner.run_game();

    if let Some(replay_file) = run_args.replay_output {
        let f = File::create(replay_file)?;

        let memento = _result.dump();

        serde_json::to_writer(f, &memento)?;
    };

    Ok(())
}

fn execute_replay_command(replay_args: ReplayCommandArgs) -> anyhow::Result<()> {
    let path = replay_args.replay_input;

    let f = File::open(path)?;

    let memento: GameMemento = serde_json::from_reader(f)?;

    let mut game_replay = GameReplay::from(memento);

    while let Some(()) = game_replay.advance_skipping_looks() {
        println!("{}", game_replay.current());

        println!("Press ENTER to continue...");
        let buffer = &mut [0u8];

        if replay_args.interactive {
            std::io::stdin().read_exact(buffer)?;
        }
    }

    println!("Replay finished");

    Ok(())
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
    event!(Level::INFO, "Starting execution...");

    let result = match args.command {
        Some(cli::Subcommands::Run(run_args)) => execute_run_command(run_args),
        Some(cli::Subcommands::Replay(replay_args)) => execute_replay_command(replay_args),

        None => Err(anyhow::anyhow!("No command...exiting")),
    };

    if result.is_err() {
        let err: anyhow::Error = result.unwrap_err();
        let s = err.to_string();
        event!(Level::ERROR, "{s}");

        Err(err)
    } else {
        Ok(())
    }
}
