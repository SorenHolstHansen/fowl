use std::assert_matches;

use crate::errors::SyntaxError;
use error::{Diagnostic, IntoDiagnostic, ResultExt};
use lexer::{Lexer, Token, TokenKind, lexer_error::LexerError};

#[derive(Clone, Copy, Debug)]
pub enum Syntax {
    Declaration,
    Fn,
    Vis,
    Public,
    Internal,
    Private,
    Ident,
    Whitespace,
    Comment,
    Type,
    FnParameters,
    FnParam,
    Self_,
}

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    tree: syntree::Builder<Syntax>,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Parser<'src> {
        Parser {
            lexer,
            tree: syntree::Builder::new(),
        }
    }

    pub fn parse(mut self) -> syntree::Tree<Syntax, syntree::FlavorDefault> {
        self.parse_internal();

        self.tree.build().unwrap()
    }

    fn parse_internal(&mut self) {
        loop {
            let peeked = self.peek_token();
            if matches!(
                peeked.kind,
                TokenKind::RBrace | TokenKind::RParen | TokenKind::Eof
            ) {
                break;
            };
            self.parse_declaration();
        }

        assert_matches!(
            self.lexer.next(),
            Err(LexerError::EofAlreadyReached)
                | Ok(Token {
                    kind: TokenKind::Eof,
                    ..
                }),
            "The parser should have reached the Eof"
        );
    }

    fn parse_vis(&mut self) {
        match self.peek_token() {
            Token {
                kind: TokenKind::Public,
                ..
            } => {
                self.tree.open(Syntax::Vis).unwrap();
                self.tree.token(Syntax::Public, 6).unwrap();
                self.tree.close().unwrap();
            }
            Token {
                kind: TokenKind::Internal,
                ..
            } => {
                self.tree.open(Syntax::Vis).unwrap();
                self.tree.token(Syntax::Internal, 8).unwrap();
                self.tree.close().unwrap();
            }
            _ => {
                self.tree.open(Syntax::Vis).unwrap();
                self.tree.token(Syntax::Private, 0).unwrap();
                self.tree.close().unwrap();
            }
        }
    }

    fn parse_ident(&mut self) {
        match self.next_token() {
            Token {
                kind: TokenKind::Ident(i),
                ..
            } => {
                self.tree.token(Syntax::Ident, i.len()).unwrap();
            }
            Token { kind: _, span } => {
                SyntaxError {
                    span,
                    expected: "a name".into(),
                }
                .into_diagnostic()
                .emit();
            }
        };
    }

    fn parse_type(&mut self) -> Result<(), Diagnostic<'src>> {
        match self.peek_token() {
            Token {
                kind: TokenKind::Ident(i),
                ..
            } => {
                // Skip the peeked ident
                self.next_token();
                self.tree.token(Syntax::Type, i.len()).unwrap();
            }
            Token { span, .. } => {
                return Err(SyntaxError {
                    span,
                    expected: "a type".into(),
                }
                .into_diagnostic());
            }
        }

        Ok(())
    }

    fn parse_delim_seq_to_end(
        &mut self,
        close: TokenKind<'src>,
        delim: TokenKind<'src>,
        mut f: impl FnMut(&mut Parser<'src>) -> Result<(), Diagnostic<'src>>,
    ) {
        loop {
            let peek = self.peek_token();
            if peek.kind == close {
                // skip the peeked token
                self.next_token();
                break;
            }

            match f(self) {
                Ok(_) => {}
                Err(d) => {
                    d.emit();
                    break;
                }
            }

            if let Err(e) = self.expect_token(delim) {
                e.emit();
                break;
            }
        }
    }

    fn parse_enclosed_delim_seq(
        &mut self,
        open: TokenKind<'src>,
        close: TokenKind<'src>,
        delim: TokenKind<'src>,
        f: impl FnMut(&mut Parser<'src>) -> Result<(), Diagnostic<'src>>,
    ) {
        match self.expect_token(open) {
            Ok(_) => self.parse_delim_seq_to_end(close, delim, f),
            Err(e) => {
                // Didn't match opening token, won't try and parse the rest then
                e.emit();
            }
        }
    }

    fn parse_fn_param(&mut self) -> Result<(), Diagnostic<'src>> {
        self.tree.open(Syntax::FnParam).unwrap();

        let peeked = self.peek_token();
        match peeked.kind {
            TokenKind::Self_ => {
                self.next_token();
                self.tree.token(Syntax::Self_, 4).unwrap();
            }
            TokenKind::Ident(i) => {
                self.next_token();
                self.tree.token(Syntax::Ident, i.len()).unwrap();
                self.expect_token(TokenKind::Colon)?;
                self.parse_type()?;
            }
            _ => {
                self.tree.close().unwrap();
                return Err(SyntaxError {
                    span: peeked.span,
                    expected: "a parameter".into(),
                }
                .into_diagnostic());
            }
        }

        self.tree.close().unwrap();

        Ok(())
    }

    fn parse_fn_parameters(&mut self) {
        self.tree.open(Syntax::FnParameters).unwrap();

        self.parse_enclosed_delim_seq(
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Comma,
            Parser::parse_fn_param,
        );

        self.tree.close().unwrap();
    }

    fn parse_function(&mut self) {
        let _ = self.next_token();
        self.tree.open(Syntax::Fn).unwrap();
        self.tree.token(Syntax::Fn, 2).unwrap();

        let peeked = self.peek_token();
        if peeked.kind == TokenKind::On {
            // Skip the peeked "on"
            let _ = self.next_token();
            self.parse_type().emit_ok();
        };

        self.parse_ident();

        self.parse_fn_parameters();

        self.tree.close().unwrap();
    }

    fn parse_declaration(&mut self) {
        self.tree.open(Syntax::Declaration).unwrap();
        self.parse_vis();

        let t = self.peek_token();
        let kind = t.kind;

        match kind {
            TokenKind::Fn => self.parse_function(),
            _ => todo!(),
        }

        self.tree.close().unwrap();
    }

    /// Eats the next token and returns it
    fn next_token(&mut self) -> Token<'src> {
        match self.lexer.next() {
            Ok(Token {
                kind: TokenKind::Whitespace(ws),
                ..
            }) => {
                self.tree.token(Syntax::Whitespace, ws.len()).unwrap();
                self.next_token()
            }
            Ok(Token {
                kind: TokenKind::Comment(ws),
                ..
            }) => {
                self.tree.token(Syntax::Comment, ws.len()).unwrap();
                self.next_token()
            }
            Ok(t) => t,
            Err(e) => {
                e.into_diagnostic().emit();
                self.next_token()
            }
        }
    }

    /// Peeks at the next `Ok` token, and eats all `Err` tokens up till that
    fn peek_token(&mut self) -> Token<'src> {
        let peeked = self.lexer.peek().clone();
        match peeked {
            Err(e) => {
                e.into_diagnostic().emit();
                let _ = self.next_token();
                self.peek_token()
            }
            Ok(Token {
                kind: TokenKind::Whitespace(ws),
                ..
            }) => {
                let _ = self.lexer.next();
                self.tree.token(Syntax::Whitespace, ws.len()).unwrap();
                self.peek_token()
            }
            Ok(Token {
                kind: TokenKind::Comment(ws),
                ..
            }) => {
                let _ = self.lexer.next();
                self.tree.token(Syntax::Comment, ws.len()).unwrap();
                self.peek_token()
            }
            Ok(t) => t,
        }
    }

    /// peeks at the next token, and eats if it it matches `token`, otherwise it returns a `Diagnostic`
    fn expect_token(&mut self, token: TokenKind<'src>) -> Result<Token<'src>, Diagnostic<'src>> {
        match self.peek_token() {
            t if token == t.kind => {
                self.next_token();
                Ok(t)
            }
            t => Err(crate::errors::SyntaxError {
                span: t.span,
                expected: format!("'{}'", token).into(),
            }
            .into_diagnostic()),
        }
    }

    fn expect_one_of_token(
        &mut self,
        tokens: &[TokenKind<'src>],
    ) -> Result<Token<'src>, Diagnostic<'src>> {
        match self.peek_token() {
            t if tokens.contains(&t.kind) => {
                let _ = self.next_token();
                Ok(t)
            }
            t => Err(crate::errors::SyntaxError {
                span: t.span,
                expected: format!(
                    "one of {}",
                    tokens
                        .iter()
                        .map(|t| format!("'{}'", t))
                        .collect::<Vec<_>>()
                        .join(",")
                )
                .into(),
            }
            .into_diagnostic()),
        }
    }
}
