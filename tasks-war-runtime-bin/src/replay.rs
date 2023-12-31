use anyhow::Ok;
use std::fs::File;
use std::io::Read;

use tasks_war_runtime::game_replay::GameReplay;

use tasks_war_runtime::game_memento::GameMemento;

use crate::cli::ReplayCommandArgs;

pub fn execute_replay_command(replay_args: ReplayCommandArgs) -> anyhow::Result<()> {
    let path = replay_args.replay_input;

    let f = File::open(path)?;

    let memento: GameMemento = serde_json::from_reader(f)?;

    let mut game_replay = GameReplay::from(memento);

    println!("{:?}\n\n", game_replay.current().get_config());

    if replay_args.show_all {
        println!("{}", game_replay.current());

        if replay_args.interactive {
            println!("Press ENTER to continue...");
            let buffer = &mut [0u8];
            std::io::stdin().read_exact(buffer)?;
        }

        while let Some(entry) = game_replay.advance() {
            println!("{entry}");

            if replay_args.interactive && !entry.is_look() {
                println!("{}", game_replay.current());
                println!("Press ENTER to continue...");
                let buffer = &mut [0u8];
                std::io::stdin().read_exact(buffer)?;
            }
        }
    } else {
        while let Some(entry) = game_replay.advance_skipping_looks() {
            println!("{}", game_replay.current());

            println!("{entry}");

            if replay_args.interactive {
                println!("Press ENTER to continue...");
                let buffer = &mut [0u8];
                std::io::stdin().read_exact(buffer)?;
            }
        }
    }

    println!("Replay finished");

    Ok(())
}
