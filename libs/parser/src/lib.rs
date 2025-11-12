pub mod parser;

#[cfg(test)]
mod test {
    use super::*;
    use lexer::tokenize;

    #[test]
    fn test_parser() {
        let source = include_str!("../../../examples/basic/kitchen_sink.fo");

        let (tokens, _) = tokenize(source);
        let (program, parser_errors) = parser::parse(tokens);
    }
}
