mod errors;
mod parser;
pub use parser::Parser;

#[cfg(test)]
mod test {
    use super::*;
    use lexer::Lexer;

    #[test]
    fn test_parser() {
        let src = "fn my_function(gkjh: MyType) Void {
            let a = 1 + 2;
        }";
        let lexer = Lexer::new(src, &std::path::Path::new(""));
        let parser = Parser::new(lexer);
        let tree = parser.parse();

        let mut s = Vec::new();
        syntree::print::print_with_source(&mut s, &tree, src).unwrap();
        let s = String::from_utf8(s).unwrap();
        eprintln!("{s}");
    }
}
