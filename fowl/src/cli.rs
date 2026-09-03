use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use fowlc_lexer::Lexer;
use fowlc_manifest::Manifest;
use fowlc_parser::Parser;
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};
use yansi::Paint;

#[derive(ClapParser, Debug)]
#[command(name = "fowl", about = "Fowl", version)]
pub struct FowlCli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, ClapParser)]
pub struct BuildOptions {
    #[arg(long, global = true)]
    /// Dump the token stream before parsing.
    dump_tokens: bool,

    #[arg(long, global = true)]
    /// Dump the parsed AST before code generation.
    dump_ast: bool,

    #[arg(long, global = true)]
    /// Target triple for cross-compilation (e.g., wasm32-unknown-unknown, thumbv7m-none-eabi)
    target: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a fowl project
    Run(RunCommand),
}

#[derive(Debug, clap::Parser)]
#[command(name = "run", about = "Build and run a fowl project", long_about = None)]
struct RunCommand {
    #[command(flatten)]
    pub build: BuildOptions,
}

impl RunCommand {
    fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let root = locate_project_root(&cwd)?;
        let fowl_jsonc = {
            let fowl_jsonc_src = std::fs::read_to_string(root.join(FOWL_JSONC_NAME))?;
            Manifest::parse_str(&fowl_jsonc_src)?
        };
        println!(
            "[{}] {} v{}",
            "Building".bright_cyan().bold(),
            fowl_jsonc.name(),
            fowl_jsonc.version()
        );
        let now = Instant::now();

        println!(
            "[{}] in {:.2}s",
            "Finished".bright_cyan().bold(),
            (now.elapsed().as_millis() as f64) / 1000.0
        );

        println!("[{}] <output>", "Running ".bright_cyan().bold(),);

        Ok(())
    }
}

pub fn run() -> Result<()> {
    let cli = FowlCli::parse();

    match cli.command {
        Command::Run(cmd) => cmd.run(),
    }
}

const FOWL_JSONC_NAME: &str = "fowl.jsonc";

fn locate_project_root(from_path: &Path) -> Result<&Path> {
    for ancestor in from_path.ancestors() {
        if std::fs::exists(ancestor.join(FOWL_JSONC_NAME))? {
            return Ok(ancestor);
        }
    }
    Err(anyhow::anyhow!(
        "Could not find {FOWL_JSONC_NAME} in any parent directory"
    ))
}
