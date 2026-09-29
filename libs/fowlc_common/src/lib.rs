use std::path::Path;

#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Dump the token stream of the specified file before parsing.
    /// e.g. `--dump-tokens src/my_file.fo`
    pub dump_tokens: Option<String>,
    /// Dump the parsed AST of the specified file before code generation.
    /// e.g. `--dump-ast src/my_file.fo`
    pub dump_ast: Option<String>,
    /// Target triple for cross-compilation (e.g., wasm32-unknown-unknown, thumbv7m-none-eabi)
    pub target: Option<String>,
}

impl BuildOptions {
    pub fn should_dump_tokens_for_file(&self, file: &Path) -> bool {
        if let Some(path) = &self.dump_tokens {
            return file.ends_with(path);
        }

        false
    }

    pub fn should_dump_ast_for_file(&self, file: &Path) -> bool {
        if let Some(path) = &self.dump_ast {
            return file.ends_with(path);
        }

        false
    }
}
