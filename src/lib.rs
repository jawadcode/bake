use std::fmt::{self, Debug, Display};

use clap::{Parser, Subcommand, ValueEnum};

pub mod project;

#[derive(Parser, Debug)]
#[command(author, version, about = "A simple build system for C/C++", long_about = None)]
pub struct Config {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new bake project
    New { name: String },
    /// Build a bake project in the CWD
    Build {
        #[arg(short, long)]
        mode: BuildMode,
    },
    /// Build and run a bake project in the CWD
    Run {
        #[arg(short, long)]
        mode: BuildMode,
    },
}

/// The optimisation level to be used for compilation
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum BuildMode {
    Debug,
    Release,
}

impl BuildMode {
    pub fn to_flag(self) -> &'static str {
        match self {
            BuildMode::Debug => "-O0",
            BuildMode::Release => "-O3",
        }
    }
}

impl Display for BuildMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildMode::Debug => f.write_str("debug"),
            BuildMode::Release => f.write_str("release"),
        }
    }
}

impl Default for BuildMode {
    fn default() -> Self {
        BuildMode::Debug
    }
}
