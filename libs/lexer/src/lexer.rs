use crate::{
    Token,
    lexer_error::{LexerError, LexerErrorKind},
    lexing::YYC_INIT,
    token::TokenKind,
};
use std::{collections::VecDeque, ops::Range};

#[derive(Clone, Debug)]
pub struct Lexer<'src> {
    pub(crate) input: &'src str,
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
    pub fn new(source: &'src str) -> Self {
        Lexer {
            input: source,
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

    pub(crate) fn span(&self) -> Range<usize> {
        self.token..self.cursor
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
                | TokenKind::Ident(_)
                | TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::BoolLiteral(_)
                | TokenKind::StringInterpolationEnd
                | TokenKind::RParen
                | TokenKind::RBrace
                | TokenKind::RBracket => {
                    self.force_next_token = Some(Token {
                        kind: TokenKind::Semicolon,
                        span: self.cursor..(self.cursor + 1),
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

    pub(crate) fn int(&mut self) -> Result<Token<'src>, LexerError<'src>> {
        let token_text = self.token_text();
        self.token(TokenKind::IntLiteral(token_text))
    }

    pub(crate) fn float(&mut self) -> Result<Token<'src>, LexerError<'src>> {
        let token_text = self.token_text().replace("_", "");
        // expecting here, since the regex should only match for things that can actually be parsed.
        // Also, it is not a user error, but a bad regex on our part
        let f = token_text
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("Could not parse '{}' as float", token_text));
        self.token(TokenKind::FloatLiteral(f))
    }

    pub(crate) fn ident(&mut self) -> Result<Token<'src>, LexerError<'src>> {
        self.token(TokenKind::Ident(self.token_text()))
    }
}
