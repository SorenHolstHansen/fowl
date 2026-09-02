use std::assert_matches;

use crate::errors::SyntaxError;
use error::{Diagnostic, IntoDiagnostic, ResultExt};
use lexer::{Lexer, Token, TokenKind, lexer_error::LexerError};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    tree: syntree::Builder<TokenKind>,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Parser<'src> {
        Parser {
            lexer,
            tree: syntree::Builder::new(),
        }
    }

    pub fn parse(mut self) -> syntree::Tree<TokenKind, syntree::FlavorDefault> {
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
        self.tree.open(TokenKind::Vis).unwrap();
        match self.peek_token() {
            Token {
                kind: TokenKind::Public,
                ..
            } => {
                self.tree.token(TokenKind::Public, 6).unwrap();
            }
            Token {
                kind: TokenKind::Internal,
                ..
            } => {
                self.tree.token(TokenKind::Internal, 8).unwrap();
            }
            _ => {
                self.tree.token(TokenKind::Private, 0).unwrap();
            }
        }
        self.tree.close().unwrap();
    }

    fn parse_ident(&mut self) {
        self.expect_token(TokenKind::Ident).emit_ok();
    }

    fn parse_type(&mut self) -> Result<(), Diagnostic<'src>> {
        match self.peek_token() {
            Token {
                kind: TokenKind::Ident,
                span,
            } => {
                // Skip the peeked ident
                self.next_token();
                self.tree.token(TokenKind::Type, span.len()).unwrap();
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
        close: TokenKind,
        delim: TokenKind,
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

            match self.expect_one_of_token(&[close, delim]) {
                Err(e) => {
                    e.emit();
                    break;
                }
                Ok(t) => {
                    if t.kind == close {
                        break;
                    }
                }
            }
        }
    }

    fn parse_enclosed_delim_seq(
        &mut self,
        open: TokenKind,
        close: TokenKind,
        delim: TokenKind,
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
        self.tree.open(TokenKind::FnParameter).unwrap();

        let peeked = self.peek_token();
        match peeked.kind {
            TokenKind::Self_ => {
                self.next_token();
                self.tree.token(TokenKind::Self_, 4).unwrap();
            }
            TokenKind::Ident => {
                self.next_token();
                self.tree
                    .token(TokenKind::Ident, peeked.span.len())
                    .unwrap();
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
        self.tree.open(TokenKind::FnParameters).unwrap();

        self.parse_enclosed_delim_seq(
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Comma,
            Parser::parse_fn_param,
        );

        self.tree.close().unwrap();
    }

    fn parse_prefix_expression(&mut self, precedence: u8) -> Result<(), Diagnostic<'src>> {
        let token = self.peek_token();

        match token.kind {
            TokenKind::LParen => {
                self.tree.open(TokenKind::ParenExpr).unwrap();
                self.expect_token(TokenKind::LParen)?;
                self.parse_expression(0)?;
                self.expect_token(TokenKind::RParen)?;
                self.tree.close().unwrap();
            }
            TokenKind::LBrace => {
                self.parse_block();
            }
            TokenKind::IntLiteral => {
                self.expect_token(TokenKind::IntLiteral)?;
            }
            TokenKind::FloatLiteral => {
                self.expect_token(TokenKind::FloatLiteral)?;
            }
            TokenKind::BoolLiteral => {
                self.expect_token(TokenKind::BoolLiteral)?;
            }
            _ => todo!(),
        };

        Ok(())
    }

    fn parse_following_expression(&mut self) -> Result<(), Diagnostic<'src>> {
        let peeked = self.peek_token();
        if lexer::INFIX_OPERATORS.iter().any(|o| o == &peeked.kind) {
            self.tree.open(TokenKind::BinaryOp).unwrap();
            self.tree.token(peeked.kind, peeked.span.len()).unwrap();
            self.next_token();
            self.parse_expression(peeked.kind.precedence())?;
            self.tree.close().unwrap();
        }
        Ok(())
    }

    fn parse_expression(&mut self, precedence: u8) -> Result<(), Diagnostic<'src>> {
        let c = self.tree.checkpoint().unwrap();

        self.parse_prefix_expression(precedence)?;

        if self.peek_token().kind == TokenKind::Semicolon {
            return Ok(());
        }

        while precedence < self.peek_token().kind.precedence() {
            self.parse_following_expression()?;
        }

        self.tree.close_at(&c, TokenKind::Expression).unwrap();
        Ok(())
    }

    fn parse_statement(&mut self) -> Result<(), Diagnostic<'src>> {
        let c = self.tree.checkpoint().unwrap();

        let token = self.peek_token();
        match token.kind {
            TokenKind::Let => {
                // Skip the peeked 'let'
                self.next_token();
                self.tree.token(TokenKind::Let, 3).unwrap();

                self.eat_if_token(TokenKind::Mut);
                self.expect_token(TokenKind::Ident).emit_ok();
                self.expect_token(TokenKind::Eq).emit_ok();
                self.parse_expression(0).emit_ok();
            }
            _ => {
                todo!()
            }
        }

        self.tree.close_at(&c, TokenKind::Statement).unwrap();
        Ok(())
    }

    fn parse_block(&mut self) {
        self.tree.open(TokenKind::Block).unwrap();

        self.parse_enclosed_delim_seq(
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            Parser::parse_statement,
        );

        self.tree.close().unwrap();
    }

    fn parse_function(&mut self) {
        // Skip the peeked 'fn'
        let _ = self.next_token();
        let c = self.tree.checkpoint().unwrap();
        self.tree.token(TokenKind::Fn, 2).unwrap();

        let peeked = self.peek_token();
        if peeked.kind == TokenKind::On {
            // Skip the peeked "on"
            let _ = self.next_token();
            self.parse_type().emit_ok();
        };

        self.parse_ident();

        self.parse_fn_parameters();

        self.tree.open(TokenKind::ReturnType).unwrap();
        self.parse_type().emit_ok();
        self.tree.close().unwrap();

        self.parse_block();

        self.tree.close_at(&c, TokenKind::Fn).unwrap();
    }

    fn parse_declaration(&mut self) {
        let c = self.tree.checkpoint().unwrap();
        self.parse_vis();

        let t = self.peek_token();
        let kind = t.kind;

        match kind {
            TokenKind::Fn => self.parse_function(),
            _ => todo!(),
        }

        self.tree.close_at(&c, TokenKind::Declaration).unwrap();
    }

    /// Eats the next token and returns it
    fn next_token(&mut self) -> Token<'src> {
        match self.lexer.next() {
            Ok(Token {
                kind: TokenKind::Whitespace,
                span,
            }) => {
                self.tree.token(TokenKind::Whitespace, span.len()).unwrap();
                self.next_token()
            }
            Ok(Token {
                kind: TokenKind::Comment,
                span,
            }) => {
                self.tree.token(TokenKind::Comment, span.len()).unwrap();
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
                kind: TokenKind::Whitespace,
                span,
            }) => {
                let _ = self.lexer.next();
                self.tree.token(TokenKind::Whitespace, span.len()).unwrap();
                self.peek_token()
            }
            Ok(Token {
                kind: TokenKind::Comment,
                span,
            }) => {
                let _ = self.lexer.next();
                self.tree.token(TokenKind::Comment, span.len()).unwrap();
                self.peek_token()
            }
            Ok(t) => t,
        }
    }

    /// peeks at the next token, and eats if it it matches `token`, otherwise it returns a `Diagnostic`
    fn expect_token(&mut self, token: TokenKind) -> Result<Token<'src>, Diagnostic<'src>> {
        match self.peek_token() {
            t if token == t.kind => {
                self.next_token();
                self.tree.token(token, t.span.len()).unwrap();
                Ok(t)
            }
            t => Err(crate::errors::SyntaxError {
                span: t.span,
                expected: format!("'{}'", token).into(),
            }
            .into_diagnostic()),
        }
    }

    /// Eats the token if it is found, otherwise does nothing
    fn eat_if_token(&mut self, token: TokenKind) {
        let _ = self.expect_token(token);
    }

    fn expect_one_of_token(
        &mut self,
        tokens: &[TokenKind],
    ) -> Result<Token<'src>, Diagnostic<'src>> {
        match self.peek_token() {
            t if tokens.contains(&t.kind) => {
                let _ = self.expect_token(t.kind);
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
