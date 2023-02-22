use std::{
    env,
    ffi::OsStr,
    fs,
    path::Path,
    process::{self, Command},
    str::FromStr,
};

use anyhow::{bail, Context};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;

use crate::BuildMode;

const DEFAULT_MAIN_C: &str = "#include <stdio.h>\n\nint main(int argc, char *argv[]) {\n    puts(\"Hello World\");\n    return 0;\n}\n";

lazy_static! {
    static ref PROJECT_NAME_REGEX: Regex = Regex::new("^[A-Za-z_][A-Za-z0-9-_]*$").unwrap();
}

pub fn new_project(name: &str) -> anyhow::Result<()> {
    if !PROJECT_NAME_REGEX.is_match(name) {
        bail!("{name} is not a valid project name");
    }

    let cwd = env::current_dir().context("Failed to get current directory")?;
    let proj_dir = cwd.join(name);
    fs::create_dir(&proj_dir).with_context(|| {
        format!(
            "Failed to create project directory at '{}'",
            proj_dir.display()
        )
    })?;
    fs::write(proj_dir.join("bake.toml"), format!("name=\"{}\"\n", name))
        .context("Failed to create 'bake.toml'")?;
    fs::create_dir(proj_dir.join("src")).context("Failed to create 'src/'")?;
    fs::write(proj_dir.join("src").join("main.c"), DEFAULT_MAIN_C)
        .context("Failed to create 'main.c'")?;
    fs::write(proj_dir.join(".gitignore"), "bin/\n").context("Failed to create '.gitignore'")?;
    println!(
        "\x1b[1;32mSuccess:\x1b[0m Created project '{}'",
        proj_dir.display()
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct ProjConfig {
    pub name: String,
}

static CC: Lazy<String> = Lazy::new(|| env::var("CC").unwrap_or_else(|_| "cc".to_string()));
static CPP: Lazy<String> = Lazy::new(|| env::var("CXX").unwrap_or_else(|_| "c++".to_string()));

enum Lang {
    C,
    Cpp,
}

impl Lang {
    fn get_compiler(&self) -> &str {
        match self {
            Lang::C => &CC,
            Lang::Cpp => &CPP,
        }
    }
}

impl FromStr for Lang {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "c" => Ok(Self::C),
            "cc" | "cxx" | "cpp" | "c++" => Ok(Self::Cpp),
            _ => Err(()),
        }
    }
}

pub fn build_project(mode: BuildMode) -> anyhow::Result<()> {
    let cwd = env::current_dir().context("Failed to get current dir")?;
    let config: ProjConfig = {
        let config_str =
            fs::read_to_string(cwd.join("bake.toml")).context("Failed to read 'bake.toml'")?;
        toml::from_str(&config_str).context("Failed to parse 'bake.toml'")?
    };
    build_project_inner(&config, mode, cwd)
}

pub fn run_project(mode: BuildMode) -> anyhow::Result<()> {
    let cwd = env::current_dir().context("Failed to get current dir")?;
    let config: ProjConfig = {
        let config_str =
            fs::read_to_string(cwd.join("bake.toml")).context("Failed to read 'bake.toml'")?;
        toml::from_str(&config_str).context("Failed to parse 'bake.toml'")?
    };
    build_project_inner(&config, mode, &cwd)?;
    println!(
        "    \x1b[1;32mRunning\x1b[0m {}",
        cwd.join("bin")
            .join(mode.to_string())
            .join(&config.name)
            .display()
    );
    process::Command::new(cwd.join("bin").join(mode.to_string()).join(&config.name))
        .spawn()
        .with_context(|| format!("Failed to run '{}'", &config.name))?
        .wait()?;
    Ok(())
}

pub fn build_project_inner(
    config: &ProjConfig,
    mode: BuildMode,
    path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let opt_level = mode.to_flag();
    let src_dir = path.as_ref().join("src");
    let bin_dir = path.as_ref().join("bin").join(mode.to_string());
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("Failed to create 'bin/' in '{}'", path.as_ref().display()))?;
    for source in fs::read_dir(&src_dir).context("Failed to read 'src/'")? {
        let source = source.with_context(|| {
            format!(
                "Failed to access directory entry in '{}'",
                src_dir.display()
            )
        })?;
        let source_path = source.path();
        let source_metadata = source
            .metadata()
            .with_context(|| format!("Failed to read metadata of '{}'", source_path.display()))?;
        if !source_metadata.is_file() {
            continue;
        }
        let Some(lang) = source_path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(|ext| ext.parse::<Lang>().ok())
        else {
            continue;
        };
        let object_path = {
            let file_name = source_path
                .file_name()
                .with_context(|| format!("'{}' is missing file stem", source_path.display()))?
                .to_string_lossy()
                .to_string();
            bin_dir.join(file_name + ".o")
        };
        if !object_path.exists()
            || source_metadata.modified()? > object_path.metadata()?.modified()?
        {
            if Command::new(lang.get_compiler())
                .args([
                    opt_level,
                    "-g",
                    "-c",
                    source_path.to_str().unwrap(),
                    "-o",
                    object_path.to_str().unwrap(),
                ])
                .status()?
                .success()
            {
                println!("    \x1b[1;32mCompiled\x1b[0m {}", source_path.display());
            } else {
                bail!("Failed to compile file '{}'", source_path.display());
            }
        }
    }
    let mut object_files = Vec::new();
    for source in
        fs::read_dir(&bin_dir).with_context(|| format!("Failed to read '{}'", bin_dir.display()))?
    {
        let source = source.with_context(|| {
            format!(
                "Failed to access directory entry in '{}'",
                src_dir.display()
            )
        })?;
        let source_path = source.path();
        let source_metadata = source
            .metadata()
            .with_context(|| format!("Failed to read metadata of '{}'", source_path.display()))?;
        if !source_metadata.is_file() {
            continue;
        }
        let Some(ext) = source_path
            .extension()
        else {
            continue;
        };
        if ext != OsStr::new("o") {
            continue;
        }
        object_files.push(source_path.to_string_lossy().to_string());
    }
    if !Command::new(&*CC)
        .args(object_files.iter().map(AsRef::as_ref).chain([
            opt_level,
            "-o",
            bin_dir.join(&config.name).to_str().unwrap(),
        ]))
        .status()?
        .success()
    {
        bail!("Failed to link executable")
    }
    println!("    \x1b[1;32mCompiled\x1b[0m '{}'", config.name);
    Ok(())
}
