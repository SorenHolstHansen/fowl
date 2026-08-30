pub mod ast;
mod errors;
mod parser;
pub use parser::Parser;

#[cfg(test)]
mod test {
    use super::*;
    use lexer::Lexer;

    #[test]
    fn test_parser() {
        let lexer = Lexer::new("fn", &std::path::Path::new(""));
        let mut parser = Parser::new(lexer);
        let ast = parser.parse();
        dbg!(ast);
    }
}
