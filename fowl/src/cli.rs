use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use codegen::{CodegenOptions, build_executable};
use error::emit_diagnostics;
use fowl_jsonc::parse_fowl_jsonc;
use lexer::tokenize;
use parser::parser::parse;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Parser, Debug)]
#[command(name = "fowl", about = "Fowl compiler")]
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
    /// Lexes, parses, and executes the specified source file via the cached native pipeline.
    Run,
    /// Builds a native executable from the specified source file.
    Build {
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Format Fowl source code.
    Fmt {
        /// Files to format (defaults to all .fo files in current directory)
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    // /// Profile Fowl programs (memory or performance)
    // Profile {
    //     #[command(subcommand)]
    //     subcommand: crate::tools::profiler::ProfileCommand,
    // },
}

pub fn run() -> Result<()> {
    let cli = FowlCli::parse();
    let settings = CompilerSettings::from(&cli);

    match &cli.command {
        Command::Run => handle_run(settings),
        _ => todo!(),
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
    println!("Building '{}@{}'", fowl_jsonc.name(), fowl_jsonc.version());

    let path = root.join("src/main.fo");
    let source = std::fs::read_to_string(&path)?;
    let output = compile_pipeline(&path, &source, settings)?;

    println!("Running '{}@{}'", fowl_jsonc.name(), fowl_jsonc.version());
    execute_binary(&output);

    Ok(())
}

fn compile_pipeline(path: &Path, source: &str, settings: CompilerSettings) -> Result<PathBuf> {
    // Lexing step
    let lexer = tokenize(source);
    if settings.dump_tokens {
        println!("\n== Tokens ==");
        println!("{}", lexer.clone().pretty_string());
    }

    // Parsing step
    let (program, parser_errors) = parse(lexer);
    let mut has_errors = !parser_errors.is_empty();
    emit_diagnostics(parser_errors.into_iter().map(|e| e.with_file(path)), source);
    if settings.dump_ast {
        println!("\n== AST ==");
        println!("{:#?}", program);
    }

    // Module step
    resolve_modules(&program)?;

    // Type checker step
    let (program, typecheck_errors) = typecheck::typecheck(program);
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
        bail!("Please address the issues");
    }
    // Codegen step
    let codegen_options = settings.codegen_options()?;
    let output = PathBuf::from("./.fowl/tmp_binary");
    build_executable(&program, &output, &codegen_options)?;

    Ok(output)
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
