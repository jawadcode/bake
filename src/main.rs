use std::{
    env,
    error::Error,
    fmt::{self, Display},
    fs,
    path::PathBuf,
    process::exit,
};

use anyhow::Context;
use config::{BuildMode, Config};
use lazy_static::lazy_static;
use regex::Regex;

mod config;

#[derive(Clone, Debug)]
enum BakeError {
    ProjectName(String),
    InvalidProject(PathBuf),
}

impl Display for BakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BakeError::ProjectName(name) => write!(f, "Invalid project name '{name}'"),
            BakeError::InvalidProject(path) => {
                write!(f, "'{}' is not a valid project", path.display())
            }
        }
    }
}

impl Error for BakeError {}

fn main() {
    if let Err(err) = run() {
        eprintln!("\x1b[1;31mError:\x1b[0m {err}");
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("\x1b[1;90mCaused By:\x1b[0m {cause}"));
        exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mut args = env::args();
    let _exec_name = args.next();
    let config = Config::new(args)?;
    match config {
        Config::Help => Ok(print_help_msg()),
        Config::New { name } => new_project(name),
        Config::Build { mode } => build_project(mode),
        Config::Run { mode } => todo!(),
    }
}

lazy_static! {
    static ref PROJECT_NAME_REGEX: Regex = Regex::new("^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
}

fn new_project(name: String) -> anyhow::Result<()> {
    if !PROJECT_NAME_REGEX.is_match(&name) {
        return Err(anyhow::Error::new(BakeError::ProjectName(name)));
    }
    let mut path = env::current_dir().context("Failed to get current directory")?;
    path.push(&name);
    fs::create_dir(&path).with_context(|| {
        format!(
            "Failed to create project directory at '{}'",
            path.to_string_lossy()
        )
    })?;
    path.push("bake.toml");
    fs::write(&path, &format!("[package]\nname=\"{}\"\n", &name))
        .context("Failed to create bake.toml")?;
    path.pop();
    path.push("src");
    fs::create_dir(&path).with_context(|| {
        path.pop();
        format!(
            "Failed to create source directory for '{name}' at '{}'",
            path.to_string_lossy()
        )
    })?;
    path.push("main.c");
    fs::write(&path, "#include <stdio.h>\n\nint main(int argc, char *argv[]) {\n    puts(\"Hello World\");\n    return 0;\n}\n").with_context(|| {
        path.pop();
        path.pop();
        format!(
            "Failed to create src/main.c for '{name}' at '{}'",
            path.display()
        )
    })?;
    path.pop();
    path.pop();
    path.push("bin");
    fs::create_dir(&path).with_context(|| {
        path.pop();
        format!(
            "Failed to create bin directory for '{name}' at '{}'",
            path.display()
        )
    })?;
    path.pop();
    println!(
        "\x1b[1;32mSuccess:\x1b[0m Created project '{name}' at '{}'",
        path.display()
    );
    Ok(())
}

fn build_project(mode: BuildMode) -> anyhow::Result<()> {
    let mut path = env::current_dir().context("Failed to get current directory")?;
    path.push("bake.toml");
    if !path.exists() {
        return Err(anyhow::Error::new(BakeError::InvalidProject({
            path.pop();
            path
        })));
    }

    path.pop();
    build_project_inner(&mut path, mode).with_context(|| "")?;

    Ok(())
}

fn build_project_inner(path: &mut PathBuf, mode: BuildMode) -> anyhow::Result<()> {
    path.push("bin");
    Ok(())
}

fn print_help_msg() {
    println!(
        r#"bake 0.1.0
Jawad Ahmed <jawad.w.ahmed@gmail.com>

A simple build system for C/C++.

USAGE:
        bake [--help | help]             Print this help message

        bake (init | new) name           Create a new bake project called 'name'

        bake build [--debug | --release] Build the project in the CWD with the
                                         default build mode being '--debug'

        bake run [--debug | --release]   Build the project in the CWD and run it

INFO:
        project name must match '^[A-Za-z_][A-Za-z0-9_]*$'
"#
    )
}
