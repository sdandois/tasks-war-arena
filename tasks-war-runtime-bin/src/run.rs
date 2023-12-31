use anyhow::Ok;
use rand::{rngs, Rng, SeedableRng};
use tracing::{event, Level};
use std::io::Read;
use std::{fs::File, path::PathBuf};
use tasks_war_runtime::game::{BoardSize, GameConfig};
use tasks_war_runtime::game_replay::GameReplay;
use tasks_war_runtime::runtime::bots::WasmBotFactory;
use tasks_war_runtime::runtime::GameRunner;

use crate::cli::RunCommandArgs;

pub fn execute_run_command(run_args: RunCommandArgs) -> anyhow::Result<()> {
    let path0: PathBuf = run_args.player_0_module_path;
    let path1: PathBuf = run_args.player_1_module_path;

    let factory = WasmBotFactory::new(path0, path1)?;

    let seed: u64 = run_args.seed.unwrap_or_else(|| {
        let mut seed_generator = rngs::SmallRng::from_entropy();

        seed_generator.gen()
    });
    let config = GameConfig {
        board_size: BoardSize(run_args.board_size, run_args.board_size),
        seed,
    };

    let runner = GameRunner::with_config(factory, config);

    let _result = runner.run_game();
    let memento = _result.dump();

    if let Some(replay_file) = run_args.replay_output {
        let f = File::create(replay_file)?;

        serde_json::to_writer(f, &memento)?;
    };

    if run_args.play_replay {
        event!(Level::INFO, "Playing replay of game");
        let mut game_replay = GameReplay::from(memento);

        while let Some(entry) = game_replay.advance_skipping_looks() {
            println!("{}", game_replay.current());

            println!("{entry}");

            println!("Press ENTER to continue...");
            let buffer = &mut [0u8];
            std::io::stdin().read_exact(buffer)?;
        }
    }

    Ok(())
}
