use anyhow::Result;
use clap::{Parser, Subcommand};
use codegen::{CodegenOptions, build_executable};
use error::emit_diagnostics;
use fowl_jsonc::{FowlJsonc, parse_fowl_jsonc};
use lexer::tokenize;
use parser::parser::parse;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};
use walkdir::WalkDir;
use yansi::Paint;

#[derive(Parser, Debug)]
#[command(name = "fowl", about = "Fowl", version)]
pub struct FowlCli {
    #[arg(long, global = true)]
    /// Dump the token stream before parsing.
    dump_tokens: bool,

    #[arg(long, global = true)]
    /// Dump the parsed AST before code generation.
    dump_ast: bool,

    #[arg(long, global = true)]
    /// Target triple for cross-compilation (e.g., wasm32-unknown-unknown, thumbv7m-none-eabi)
    target: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a fowl project
    Run,
}

pub fn run() -> Result<()> {
    let cli = FowlCli::parse();
    let settings = CompilerSettings::from(&cli);

    match &cli.command {
        Command::Run => handle_run(settings),
    }
}

const FOWL_JSONC_NAME: &str = "fowl.jsonc";

fn locate_project_root(from_path: PathBuf) -> Result<PathBuf> {
    let mut project_root = from_path;
    while !std::fs::exists(project_root.join(FOWL_JSONC_NAME))? {
        if !project_root.pop() {
            return Err(anyhow::anyhow!(
                "Could not find fowl.jsonc in any parent directory"
            ));
        }
    }

    Ok(project_root)
}

fn handle_run(settings: CompilerSettings) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let root = locate_project_root(current_dir)?;
    let fowl_jsonc = {
        let fowl_jsonc_src = std::fs::read_to_string(root.join(FOWL_JSONC_NAME))?;
        parse_fowl_jsonc(&fowl_jsonc_src)?
    };
    println!(
        "{:>12} '{}@{}'",
        "Building".green(),
        fowl_jsonc.name(),
        fowl_jsonc.version()
    );
    let now = std::time::Instant::now();

    let path = root.join("src/main.fo");
    let source = std::fs::read_to_string(&path)?;
    let output = compile_pipeline(&path, &root, &fowl_jsonc, &source, settings)?;

    println!(
        "{:>12} in {:.2}s",
        "Finished".green(),
        (now.elapsed().as_millis() as f64) / 1000.0
    );

    println!("{:>12} {:?}", "Running".green(), output);
    execute_binary(&output);

    Ok(())
}

fn compile_pipeline(
    path: &Path,
    root: &Path,
    fowl_jsonc: &FowlJsonc,
    source: &str,
    settings: CompilerSettings,
) -> Result<PathBuf> {
    // Read all src/**/*.fo files
    let mut files = Vec::new();
    for entry in WalkDir::new(root.join("src")) {
        let entry = entry?;
        let path = entry.path();
        match path.extension() {
            Some(ext) if ext == "fo" => {}
            _ => continue,
        }
        let src = std::fs::read_to_string(path)?;
        files.push((path.to_path_buf(), src));
    }

    let mut has_errors = false;
    let parsed_files = files
        .iter()
        .map(|(path, src)| {
            let lexer = tokenize(src);
            if settings.dump_tokens {
                println!("\n== {:?} Tokens ==", path);
                println!("{}", lexer.clone().pretty_string());
            }

            let (program, parser_errors) = parse(lexer);
            has_errors = !parser_errors.is_empty();
            emit_diagnostics(parser_errors.into_iter().map(|e| e.with_file(path)), src);
            if settings.dump_ast {
                println!("\n== AST ==");
                println!("{:#?}", program);
            }

            let module_name = path_to_module_name(path, root, fowl_jsonc.name());
            (module_name, program)
        })
        .collect::<HashMap<_, _>>();

    // Module step
    // resolve_modules(&program)?;

    // Type checker step
    let (program, typecheck_errors) = typecheck::typecheck(parsed_files, fowl_jsonc.name());
    if !typecheck_errors.is_empty() {
        has_errors = true;
    }
    emit_diagnostics(
        typecheck_errors.into_iter().map(|e| e.with_file(path)),
        source,
    );
    if settings.dump_ast {
        println!("\n== TYPED AST ==");
        println!("{:#?}", program);
    }

    if has_errors {
        std::process::exit(1);
    }

    // Codegen step
    let codegen_options = settings.codegen_options()?;
    let output = root.join(format!(".fowl/{}", fowl_jsonc.name()));
    build_executable(&program, &output, &codegen_options)?;

    Ok(output)
}

fn path_to_module_name(path: &Path, root: &Path, package_name: &str) -> String {
    let other = path
        .strip_prefix(root.join("src"))
        .unwrap()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let a = other.split("/").collect::<Vec<_>>().join(".");
    format!("{}.{}", package_name, a)
}

fn resolve_modules<'source>(program: &parser::ast::Program<'source>) -> Result<()> {
    for declaration in &program.declarations {
        if let parser::ast::Declaration::Use { import } = declaration {
            let namespace = import.first();
        }
    }

    Ok(())
}

fn execute_binary(path: &Path) {
    let mut command = std::process::Command::new(path);

    let output = command.output().unwrap();
    std::process::exit(output.status.code().unwrap_or(0));
}

pub struct CompilerSettings {
    dump_tokens: bool,
    dump_ast: bool,
    target: Option<String>,
}

impl From<&FowlCli> for CompilerSettings {
    fn from(value: &FowlCli) -> Self {
        CompilerSettings {
            dump_tokens: value.dump_tokens,
            dump_ast: value.dump_ast,
            target: value.target.clone(),
        }
    }
}

impl CompilerSettings {
    fn codegen_options(&self) -> Result<CodegenOptions> {
        Ok(CodegenOptions {
            target: self
                .target
                .as_ref()
                .map(|target| codegen::Triple::from_str(target))
                .transpose()?,
        })
    }
}
