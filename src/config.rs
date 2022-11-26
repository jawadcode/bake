use std::{
    fmt::{self, Display},
    ops::ControlFlow,
};

/// `argv` in a more structured form
#[derive(Debug, Clone)]
enum Config {
    /// Print help message
    Help,
    /// Create a new project
    New { name: String },
    /// Build the project in the current working directory, in either release or debug mode, depending on `mode`
    Build { mode: BuildMode },
    /// Run the project in the current working directory, in either release or debug mode, depending on `mode`
    Run { mode: BuildMode },
}

/// The optimisation level to be used for compilation
#[derive(Debug, Clone)]
enum BuildMode {
    Debug,
    Release,
}

#[derive(Debug, Clone)]
enum ArgsError {
    InvalidSubcommand(String),
    InvalidFlag(String),
    MissingArg { arg: String, subcommand: String },
}

impl Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSubcommand(subcommand) => {
                write!(f, "'{subcommand}' is not a valid subcommand")
            }
            Self::InvalidFlag(flag) => write!(f, "'{flag}' is not a valid flag"),
            Self::MissingArg { arg, subcommand } => {
                write!(f, "Missing arg '{arg}' for 'bake {subcommand}'")
            }
        }
    }
}

impl Config {
    pub fn new() -> Result<Self, ArgsError> {
        let mut args = std::env::args().peekable();
        let mut config = Self::Help;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "help" | "--help" => (),
                "new" | "init" => {
                    config = Self::New {
                        name: args.next().ok_or_else(|| ArgsError::MissingArg {
                            arg: "project_name".to_string(),
                            subcommand: arg,
                        })?,
                    }
                }
                "build" => {
                    config = Self::Build {
                        mode: match args.next().map(|mode_str| match mode_str.as_str() {
                            "--debug" => ControlFlow::Continue(BuildMode::Debug),
                            "--release" => ControlFlow::Continue(BuildMode::Release),
                            _ => ControlFlow::Break(ArgsError::InvalidFlag(mode_str)),
                        }) {
                            Some(ControlFlow::Continue(mode)) => mode,
                            Some(ControlFlow::Break(err)) => return Err(err),
                            None => BuildMode::Debug,
                        },
                    }
                }
                "run" => {
                    config = Self::Run {
                        mode: match args.next().map(|mode_str| match mode_str.as_str() {
                            "--debug" => ControlFlow::Continue(BuildMode::Debug),
                            "--release" => ControlFlow::Continue(BuildMode::Release),
                            _ => ControlFlow::Break(ArgsError::InvalidFlag(mode_str)),
                        }) {
                            Some(ControlFlow::Continue(mode)) => mode,
                            Some(ControlFlow::Break(err)) => return Err(err),
                            None => BuildMode::Debug,
                        },
                    }
                }
                _ => return Err(ArgsError::InvalidSubcommand(arg)),
            }
        }
        Ok(config)
    }
}
