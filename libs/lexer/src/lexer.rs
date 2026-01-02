use std::{collections::VecDeque, path::Path};

use crate::{
    Token,
    lexer_error::{LexerError, LexerErrorKind},
    lexing::YYC_INIT,
    token::TokenKind,
};
use span::Span;

#[derive(Clone, Debug)]
pub struct Lexer<'src> {
    pub(crate) input: &'src str,
    pub(crate) path: &'src Path,
    pub(crate) token: usize,
    pub(crate) cursor: usize,
    pub(crate) marker: usize,
    pub(crate) cond: usize,
    pub(crate) interpolation_depth: usize,
    pub(crate) eof: bool,
    pub(crate) peek_queue: VecDeque<Result<Token<'src>, LexerError<'src>>>,
    pub(crate) force_next_token: Option<Result<Token<'src>, LexerError<'src>>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, path: &'src Path) -> Self {
        Lexer {
            input: source,
            path,
            token: 0,
            cursor: 0,
            marker: 0,
            cond: YYC_INIT,
            interpolation_depth: 0,
            eof: false,
            peek_queue: VecDeque::new(),
            force_next_token: None,
        }
    }
}

impl<'src> Lexer<'src> {
    pub fn peek(&mut self) -> &Result<Token<'src>, LexerError<'src>> {
        if !self.peek_queue.is_empty() {
            return self.peek_queue.front().unwrap();
        }

        let next = self.next();
        self.peek_queue.push_back(next);

        self.peek_queue.front().unwrap()
    }

    pub fn peek_more(&mut self) -> Option<&Result<Token<'src>, LexerError<'src>>> {
        let next = self.next_internal(false);
        self.peek_queue.push_back(next);

        self.peek_queue.back()
    }

    pub fn catch_up(&mut self) {
        self.peek_queue.clear();
    }

    pub(crate) fn span(&self) -> Span<'src> {
        Span::new(self.token, self.cursor, self.path, self.input)
    }

    pub(crate) fn error(
        &mut self,
        kind: LexerErrorKind<'src>,
    ) -> Result<Token<'src>, LexerError<'src>> {
        Err(LexerError {
            span: self.span(),
            kind,
        })
    }

    pub(crate) fn token(&mut self, kind: TokenKind<'src>) -> Result<Token<'src>, LexerError<'src>> {
        let res = Ok(Token {
            kind,
            span: self.span(),
        });
        if kind == TokenKind::Eof {
            return res;
        }
        let rest = &self.input[self.cursor..];
        let mut next_is_newline = false;
        for char in rest.chars() {
            if char == '\n' {
                next_is_newline = true;
                break;
            } else if char.is_whitespace() {
                continue;
            } else {
                break;
            }
        }
        if next_is_newline {
            match kind {
                TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::None
                | TokenKind::Int
                | TokenKind::Float
                | TokenKind::String
                | TokenKind::Bool
                | TokenKind::Void
                | TokenKind::Ident(_)
                | TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::BoolLiteral(_)
                | TokenKind::StringInterpolationEnd
                | TokenKind::RParen
                | TokenKind::RBrace
                | TokenKind::RBracket => {
                    self.force_next_token = Some(Ok(Token {
                        kind: TokenKind::Semicolon,
                        span: Span::new(self.cursor, self.cursor + 1, self.path, self.input),
                    }))
                }
                _ => {}
            }
        }

        res
    }

    pub(crate) fn token_text(&self) -> &'src str {
        &self.input[self.token..self.cursor]
    }

    pub(crate) fn int(&mut self) -> Result<Token<'src>, LexerError<'src>> {
        let token_text = self.token_text();
        // expecting here, since the regex should only match for things that can actually be parsed.
        // Also, it is not a user error, but a bad regex on out part
        let i = token_text
            .parse::<i64>()
            .unwrap_or_else(|_| panic!("Could not parse '{}' as float", token_text));
        self.token(TokenKind::IntLiteral(i))
    }

    pub(crate) fn float(&mut self) -> Result<Token<'src>, LexerError<'src>> {
        let token_text = self.token_text();
        // expecting here, since the regex should only match for things that can actually be parsed.
        // Also, it is not a user error, but a bad regex on out part
        let f = token_text
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("Could not parse '{}' as float", token_text));
        self.token(TokenKind::FloatLiteral(f))
    }

    pub(crate) fn ident(&mut self) -> Result<Token<'src>, LexerError<'src>> {
        self.token(TokenKind::Ident(self.token_text()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn test_lex2() {
        let path = PathBuf::from_str("../../../examples/kitchen_sink.fo").unwrap();
        let source = include_str!("../../../examples/kitchen_sink.fo");

        let mut lexer = Lexer::new(source, &path);
        loop {
            let token = lexer.next();
            match token {
                Ok(t) => {
                    println!("TOKEN {} {}", t.span, t.kind);
                }
                Err(e) => {
                    println!("ERROR {}", e.kind);
                }
            }
        }
    }
}
