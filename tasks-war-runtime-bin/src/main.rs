use anyhow::Ok;

use clap::*;

use tracing::{event, Level};
use tracing_subscriber::filter::LevelFilter;

use atty::Stream;

mod cli;
mod replay;
mod run;

use cli::Args;

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
        Some(cli::Subcommands::Run(run_args)) => run::execute_run_command(run_args),
        Some(cli::Subcommands::Replay(replay_args)) => replay::execute_replay_command(replay_args),

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
