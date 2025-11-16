use itertools::{Either, Itertools};
use span::Span;
use std::{iter::Peekable, vec::IntoIter};
mod token;
use crate::lexer_error::LexerError;
pub use token::Token;
pub mod lexer_error;
use logos::Logos;

#[derive(Clone)]
pub struct Lexer<'source> {
    tokens: Peekable<IntoIter<(Token<'source>, Span)>>,
    previous_span: Option<Span>,
}

impl<'source> std::iter::Iterator for Lexer<'source> {
    type Item = (Token<'source>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.tokens.next();
        self.previous_span = next.as_ref().map(|n| n.1);

        next
    }
}

impl<'source> Lexer<'source> {
    pub fn new(tokens: Vec<(Token<'source>, Span)>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
            previous_span: None,
        }
    }

    pub fn peek(&mut self) -> Option<(&Token<'source>, Span)> {
        match self.tokens.peek() {
            Some((token, span)) => Some((token, *span)),
            None => None,
        }
    }
}

impl<'source> std::fmt::Display for Lexer<'source> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (token, span) in self.clone() {
            writeln!(f, "{:8} {}", span.to_string(), token)?
        }

        Ok(())
    }
}

#[allow(clippy::type_complexity)]
pub fn tokenize<'source>(
    source: &'source str,
) -> (Lexer<'source>, Vec<(LexerError<'source>, Span)>) {
    let a = Token::lexer(source);

    let (tokens, errors): (Vec<_>, Vec<_>) = a.spanned().partition_map(|(t, span)| match t {
        Ok(t) => Either::Left((t, Span::from(span))),
        Err(e) => Either::Right((e, span.into())),
    });

    (Lexer::new(tokens), errors)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = include_str!("../../../examples/basic/kitchen_sink.fo");

        let (lexer, errors) = tokenize(source);
        assert!(errors.is_empty(), "Lexer had errors: {:?}", errors);

        for ((_, span1), (_, span2)) in lexer.tuple_windows() {
            assert!(
                !span1.overlaps(span2),
                "Two spans overlapped. {} {}",
                span1,
                span2
            );
        }
    }
}
