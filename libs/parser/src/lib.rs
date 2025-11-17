pub mod parser;

#[cfg(test)]
mod test {
    use super::*;
    use lexer::tokenize;

    #[test]
    fn test_parser() {
        let source = include_str!("../../../examples/basic/kitchen_sink.fo");

        let (lexer, _) = tokenize(source);
        let (program, _) = parser::parse(lexer);
        dbg!(program);
    }
}
