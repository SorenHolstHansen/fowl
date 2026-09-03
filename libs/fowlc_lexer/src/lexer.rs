use fowlc_span::Span;

use crate::{Token, lexer_error::LexerError, lexing::YYC_INIT, token::TokenKind};
use std::{collections::VecDeque, path::Path};

#[derive(Clone)]
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
    pub(crate) force_next_token: Option<Token<'src>>,
}

impl<'src> Lexer<'src> {
    /// Creates a new lexer from the source.
    /// The lexer is sort of like an iterator, in that it doesn't have a `lex` method,
    /// rather, you advance the lexer by calling `next`
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
    /// Peeks ahead to the next token without consuming
    pub fn peek(&mut self) -> &Result<Token<'src>, LexerError<'src>> {
        if !self.peek_queue.is_empty() {
            return self.peek_queue.front().unwrap();
        }

        let next = self.next();
        self.peek_queue.push_back(next);

        self.peek_queue.front().unwrap()
    }

    /// Peeks one more ahead to the next token without consuming
    pub fn peek_more(&mut self) -> Option<&Result<Token<'src>, LexerError<'src>>> {
        let next = self.next_internal(false);
        self.peek_queue.push_back(next);

        self.peek_queue.back()
    }

    /// After peeking, this makes the lexer catch up to the peeked queue
    pub fn catch_up(&mut self) {
        self.peek_queue.clear();
    }

    /// Get the current lexer span
    pub(crate) fn span(&self) -> Span<'src> {
        Span::new(self.token, self.cursor, self.path, self.input)
    }

    /// Report an error
    pub(crate) fn error(
        &mut self,
        error: LexerError<'src>,
    ) -> Result<Token<'src>, LexerError<'src>> {
        Err(error)
    }

    pub(crate) fn token(&mut self, kind: TokenKind) -> Result<Token<'src>, LexerError<'src>> {
        // Add semicolons based on go-like heuristics
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
                | TokenKind::Ident
                | TokenKind::IntLiteral
                | TokenKind::FloatLiteral
                | TokenKind::BoolLiteral
                | TokenKind::StringInterpolationEnd
                | TokenKind::RParen
                | TokenKind::RBrace
                | TokenKind::RBracket => {
                    self.force_next_token = Some(Token {
                        kind: TokenKind::Semicolon,
                        span: Span::new(self.cursor, self.cursor + 1, self.path, self.input),
                    })
                }
                _ => {}
            }
        }

        res
    }

    /// In cases when we get weird characters like 'Ş' with special char boundaries, we can use this method to advance the cursor to the nearest boundary
    pub(crate) fn find_boundary(&mut self) {
        while !self.input.is_char_boundary(self.cursor) {
            self.cursor += 1;
        }
    }

    pub(crate) fn token_text(&self) -> &'src str {
        &self.input[self.token..self.cursor]
    }
}
