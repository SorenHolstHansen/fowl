use anyhow::Result;
use clap::{Parser, Subcommand};
use codegen::{CodegenOptions, build_executable};
use error::emit_diagnostics;
use lexer::{lexer_error::lexer_error_to_diagnostic, tokenize};
use logging::init_logging;
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
    Run { path: PathBuf },
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
        Command::Run { path } => handle_run(path, settings),
        _ => todo!(),
    }
}

fn handle_run(path: &Path, settings: CompilerSettings) -> Result<()> {
    init_logging();
    let source = std::fs::read_to_string(path)?;
    compile_pipeline(path, &source, settings)?;

    Ok(())
}

fn compile_pipeline(path: &Path, source: &str, settings: CompilerSettings) -> Result<()> {
    // Lexing step
    tracing::info!(?path, "Lexing");
    let (lexer, lexer_errors) = tokenize(source);
    emit_diagnostics(
        lexer_errors
            .iter()
            .map(|(e, span)| lexer_error_to_diagnostic(e, *span, path)),
        source,
    );
    if settings.dump_tokens {
        println!("\n== Tokens ==");
        println!("{}", lexer);
    }

    // Parsing step
    tracing::info!(?path, "Parsing");
    let (program, parser_errors) = parse(lexer);
    emit_diagnostics(parser_errors.into_iter().map(|e| e.with_file(path)), source);
    if settings.dump_ast {
        println!("\n== AST ==");
        println!("{:#?}", program);
    }

    // Module step

    // Type checker step

    // Codegen step
    tracing::info!(?path, "Codegen");
    let codegen_options = settings.codegen_options()?;
    let output = PathBuf::from("./.fowl/tmp_binary");
    build_executable(&program, &output, &codegen_options)?;
    execute_binary(&output);

    Ok(())
}

fn execute_binary(path: &Path) {
    tracing::info!(?path, "Running binary");
    let mut command = std::process::Command::new(path);

    let status = command.status().unwrap();
    tracing::info!(?status, "Binary ran");
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
