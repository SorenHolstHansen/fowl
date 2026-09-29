use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use fowlc_manifest::{Manifest, ManifestDependency, PathDependency};
use fowlc_package_manager::Package;
use semver::Version;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use yansi::Paint;

#[derive(ClapParser, Debug)]
#[command(name = "fowl", about = "Fowl", version)]
pub struct FowlCli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, ClapParser)]
pub struct BuildOptions {
    #[arg(long, global = true)]
    /// Dump the token stream of the specified file before parsing.
    /// e.g. `--dump-tokens src/my_file.fo`
    dump_tokens: Option<String>,

    #[arg(long, global = true)]
    /// Dump the parsed AST of the specified file before code generation.
    /// e.g. `--dump-ast src/my_file.fo`
    dump_ast: Option<String>,

    #[arg(long, global = true)]
    /// Target triple for cross-compilation (e.g., wasm32-unknown-unknown, thumbv7m-none-eabi)
    target: Option<String>,
}

impl From<BuildOptions> for fowlc_common::BuildOptions {
    fn from(value: BuildOptions) -> Self {
        fowlc_common::BuildOptions {
            dump_tokens: value.dump_tokens,
            dump_ast: value.dump_ast,
            target: value.target,
        }
    }
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
    pub build_options: BuildOptions,
}

impl RunCommand {
    fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let root = locate_project_root(&cwd)?;
        let fowlc_manifest = {
            let fowl_jsonc_src = std::fs::read_to_string(root.join(FOWL_JSONC_NAME))?;
            let mut manifest = Manifest::parse_str(&fowl_jsonc_src)?;
            manifest.add_dependency(
                "std",
                ManifestDependency::Path(PathDependency {
                    path: fowl_std_path()?,
                }),
            );
            manifest
        };
        let now = Instant::now();

        let packages = [
            Package {
                name: "std".to_string(),
                path: fowl_std_path()?,
                version: Version::new(0, 1, 0),
            },
            Package {
                name: fowlc_manifest.name().to_string(),
                path: root.to_path_buf(),
                version: fowlc_manifest.version().clone(),
            },
        ];
        let largest_dep_name = packages.iter().map(|p| p.name.len()).max().unwrap();
        let build_options: fowlc_common::BuildOptions = self.build_options.into();
        for package in packages {
            println!(
                "[{}] {name:width$} v{version}",
                "Compiling".bright_cyan().bold(),
                name = package.name,
                version = package.version,
                width = largest_dep_name
            );
            fowlc_hir::lower_to_hir(package, &build_options);
        }

        println!(
            "[{}] in {:.2}s",
            "Finished ".bright_cyan().bold(),
            (now.elapsed().as_millis() as f64) / 1000.0
        );

        println!("[{}] <output>", "Running  ".bright_cyan().bold(),);

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

fn compiler_root_dir() -> Result<PathBuf> {
    let relative_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    Ok(relative_dir.canonicalize()?)
}

fn fowl_std_path() -> Result<PathBuf> {
    let compiler_root = compiler_root_dir()?;

    Ok(compiler_root.join("std").canonicalize()?)
}
