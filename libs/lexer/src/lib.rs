use itertools::{Either, Itertools};
use span::Span;
use std::ops::Range;
mod token;
use crate::lexer_error::LexerError;
pub use token::Token;
pub mod lexer_error;
use logos::Logos;

#[allow(clippy::type_complexity)]
pub fn tokenize<'source>(
    source: &'source str,
) -> (
    Vec<(Token<'source>, Range<usize>)>,
    Vec<(LexerError<'source>, Span)>,
) {
    let a = Token::lexer(source);

    let (tokens, errors): (Vec<_>, Vec<_>) = a.spanned().partition_map(|(t, span)| match t {
        Ok(t) => Either::Left((t, span)),
        Err(e) => Either::Right((e, span.into())),
    });

    (tokens, errors)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = include_str!("../../../examples/basic/kitchen_sink.fo");

        let lexer = Token::lexer(source);
        for (result, span) in lexer.spanned() {
            assert!(result.is_ok(), "Lexer failed at {:?}", &source[span]);
        }
    }
}
