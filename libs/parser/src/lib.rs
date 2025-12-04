pub mod ast;
pub mod new_parser;
pub mod parser;

#[cfg(test)]
mod test {
    use super::*;
    use lexer::tokenize;
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn test_parser() {
        let path = PathBuf::from_str("../../../examples/kitchen_sink.fo").unwrap();
        let source = include_str!("../../../examples/kitchen_sink.fo");

        let lexer = tokenize(source, &path);
        let (program, _) = parser::parse(lexer);
        dbg!(program);
    }
}
