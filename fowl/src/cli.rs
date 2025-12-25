use anyhow::Result;
use clap::{Parser, Subcommand};
use codegen::{CodegenOptions, build_executable};
use error::emit_diagnostics;
use fowl_jsonc::{FowlJsonc, parse_fowl_jsonc};
use lexer::tokenize;
use parser::parser::parse;
use std::{
    collections::HashMap,
    io::Write,
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
        "{}:\npackage: {}\nversion: {}",
        "Building".green(),
        fowl_jsonc.name(),
        fowl_jsonc.version()
    );
    let now = std::time::Instant::now();

    let output = compile_pipeline(&root, &fowl_jsonc, settings)?;

    println!(
        "\n{}:\nin: {:.2}s",
        "Finished".green(),
        (now.elapsed().as_millis() as f64) / 1000.0
    );

    println!("\n{}:\npath: {:?}", "Running".green(), output);
    execute_binary(&output);

    Ok(())
}

fn compile_pipeline(
    root: &Path,
    fowl_jsonc: &FowlJsonc,
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
    // Find the std lib
    let std_lib_path = "../../std/src";
    let mut std_lib_files = Vec::new();
    for entry in WalkDir::new(std_lib_path) {
        let entry = entry?;
        let path = entry.path();
        match path.extension() {
            Some(ext) if ext == "fo" => {}
            _ => continue,
        }
        let src = std::fs::read_to_string(path)?;
        std_lib_files.push((path.to_path_buf(), src));
    }

    let mut has_errors = false;
    let mut parsed_files: HashMap<String, parser::ast::Program<'_>> = HashMap::new();
    for (path, src) in &files {
        let lexer = tokenize(src, path);
        if settings.dump_tokens {
            println!("\n== {:?} Tokens ==", path);
            println!("{}", lexer.clone().pretty_string());
        }

        let (program, parser_errors) = parse(lexer);
        has_errors = !parser_errors.is_empty();
        emit_diagnostics(parser_errors);
        if settings.dump_ast {
            println!("\n== AST ==");
            println!("{:#?}", program);
        }

        let module_name = path_to_module_name(path, root, fowl_jsonc.name());
        parsed_files.insert(module_name, program);
    }
    for (path, src) in &std_lib_files {
        let lexer = tokenize(src, path);
        if settings.dump_tokens {
            println!("\n== {:?} Tokens ==", path);
            println!("{}", lexer.clone().pretty_string());
        }
        let (program, parser_errors) = parse(lexer);
        has_errors = !parser_errors.is_empty();
        emit_diagnostics(parser_errors);
        if settings.dump_ast {
            println!("\n== AST ==");
            println!("{:#?}", program);
        }
        let p = path
            .strip_prefix(std_lib_path)
            .unwrap()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let module_name = format!("std.{}", p.split("/").collect::<Vec<_>>().join("."));
        parsed_files.insert(module_name, program);
    }

    // Type checker step
    let (program, analyzer_errors) = analyzer::analyzer(parsed_files, fowl_jsonc.name());
    if !analyzer_errors.is_empty() {
        has_errors = true;
    }
    emit_diagnostics(analyzer_errors);
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

fn execute_binary(path: &Path) {
    let mut command = std::process::Command::new(path);

    println!("\n");
    let output = command.output().unwrap();
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
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
