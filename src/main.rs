use std::process;

use bake::{project, Command, Config};
use clap::Parser;

fn main() {
    if let Err(err) = run() {
        eprintln!("\x1b[1;31mError:\x1b[0m {err}");
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("\x1b[1;90mCaused By:\x1b[0m {cause}"));
        process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    match config.command {
        Command::New { name } => project::new_project(&name),
        Command::Build { mode } => project::build_project(mode),
        Command::Run { mode } => project::run_project(mode),
    }
}
