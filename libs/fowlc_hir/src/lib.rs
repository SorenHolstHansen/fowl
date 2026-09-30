mod ast;
use ast as hir_ast;
use fowlc_common::BuildOptions;
use fowlc_lexer::{Lexer, TokenKind};
use fowlc_package_manager::Package;
use fowlc_parser::{
    Parser,
    ast::{self as parser_ast, AstChildren, Declaration},
    print_with_source,
};
use syntree::{FlavorDefault, Tree};
use walkdir::WalkDir;

pub fn lower_to_hir(package: Package, build_options: &BuildOptions) {
    let mut ctx = LoweringContext {};
    let src_path = package.path.join("src");
    for entry in WalkDir::new(&src_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let src_main = std::fs::read_to_string(path).unwrap();
            let lexer = Lexer::new(&src_main, path);
            if build_options.should_dump_tokens_for_file(path) {
                let lexer = lexer.clone();
                let mut s = Vec::new();
                lexer.print(&mut s).unwrap();
                let s = String::from_utf8(s).unwrap();
                println!("{s}");
            }
            let parser = Parser::new(lexer);
            let tree = parser.parse();
            if build_options.should_dump_ast_for_file(path) {
                let mut s = Vec::new();
                print_with_source(&mut s, &tree, &src_main).unwrap();
                let s = String::from_utf8(s).unwrap();
                println!("{s}");
            }

            ctx.lower_declarations(tree);
        }
    }
}

struct LoweringContext {}

impl LoweringContext {
    fn lower_declarations(&mut self, tree: Tree<TokenKind, FlavorDefault>) {
        let declarations: AstChildren<'_, Declaration> = AstChildren::new(tree.children());
        for declaration in declarations {
            self.lower_declaration(declaration);
        }
    }

    fn lower_declaration(&mut self, declaration: Declaration) {
        match declaration {
            Declaration::Fn(fun) => {
                dbg!(fun.vis());
            }
        }
    }

    fn lower_funtion_definition(
        &mut self,
        fn_def: parser_ast::FnDef<'_>,
    ) -> hir_ast::FunctionDefinition {
        hir_ast::FunctionDefinition {
            name: fn_def.name(),
        }
    }
}
