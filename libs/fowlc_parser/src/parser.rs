use std::assert_matches;

use crate::errors::{SyntaxError, Unimplemented};
use fowlc_error::{Diagnostic, IntoDiagnostic, ResultExt};
use fowlc_lexer::{Lexer, Token, TokenKind, lexer_error::LexerError};

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
                TokenKind::RightBrace | TokenKind::RightParenthesis | TokenKind::Eof
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
        self.tree.open(TokenKind::Visibility).unwrap();
        match self.peek_token() {
            Token {
                kind: TokenKind::Public,
                ..
            } => {
                let _ = self.expect_token(TokenKind::Public);
            }
            Token {
                kind: TokenKind::Internal,
                ..
            } => {
                let _ = self.expect_token(TokenKind::Internal);
            }
            Token {
                kind: TokenKind::Private,
                ..
            } => {
                let _ = self.expect_token(TokenKind::Private);
            }
            _ => {
                self.tree.token(TokenKind::Private, 0).unwrap();
            }
        }
        self.tree.close().unwrap();
    }

    fn parse_ident(&mut self) {
        let t = self.peek_token();
        match t.kind {
            TokenKind::Identifier(_) => {
                self.next_token();
                self.tree.token(t.kind, t.span.len()).unwrap();
            }
            _ => {
                crate::errors::SyntaxError {
                    span: t.span,
                    expected: format!("'{}'", t.kind).into(),
                }
                .into_diagnostic()
                .emit();
            }
        }
    }

    fn parse_type(&mut self) -> Result<(), Diagnostic<'src>> {
        match self.peek_token() {
            Token {
                kind: TokenKind::Identifier(_),
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
                let _ = self.expect_token(close);
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
                self.expect_token(TokenKind::Self_).unwrap();
            }
            TokenKind::Underscore => {
                self.expect_token(TokenKind::Underscore).unwrap();
                self.parse_ident();
                self.expect_token(TokenKind::Colon)?;
                self.parse_type()?;
            }
            TokenKind::Identifier(_) => {
                self.parse_ident();
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
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::Comma,
            Parser::parse_fn_param,
        );

        self.tree.close().unwrap();
    }

    fn parse_string_literal_or_interpolation(&mut self) {
        // TODO: Create a tree node
        self.expect_token(TokenKind::StringInterpolationStart)
            .unwrap();

        loop {
            let peeked = self.peek_token();
            match peeked.kind {
                TokenKind::StringLiteral => {
                    self.expect_token(TokenKind::StringLiteral).emit_ok();
                }
                TokenKind::LeftBrace => {
                    self.expect_token(TokenKind::LeftBrace).emit_ok();
                    self.parse_expression(0).emit_ok();
                    self.expect_token(TokenKind::RightBrace).emit_ok();
                }
                TokenKind::StringInterpolationEnd => break,
                _ => {
                    SyntaxError {
                        span: peeked.span,
                        expected: "a string or '{'".into(),
                    }
                    .into_diagnostic()
                    .emit();
                }
            };
        }

        self.expect_token(TokenKind::StringInterpolationEnd)
            .emit_ok();
    }

    fn parse_prefix_expression(&mut self, _precedence: u8) -> Result<(), Diagnostic<'src>> {
        let token = self.peek_token();

        match token.kind {
            TokenKind::LeftParenthesis => {
                self.tree.open(TokenKind::ParenExpr).unwrap();
                self.expect_token(TokenKind::LeftParenthesis)?;
                self.parse_expression(0)?;
                self.expect_token(TokenKind::RightParenthesis)?;
                self.tree.close().unwrap();
            }
            TokenKind::LeftBrace => {
                self.parse_block();
            }
            TokenKind::IntegerLiteral => {
                self.expect_token(TokenKind::IntegerLiteral)?;
            }
            TokenKind::FloatLiteral => {
                self.expect_token(TokenKind::FloatLiteral)?;
            }
            TokenKind::BoolLiteral => {
                self.expect_token(TokenKind::BoolLiteral)?;
            }
            TokenKind::Identifier(_) => {
                self.parse_ident();
                self.parse_ident_expression();
            }
            TokenKind::StringInterpolationStart => {
                self.parse_string_literal_or_interpolation();
            }
            x => {
                Unimplemented {
                    span: token.span,
                    in_function: "parse_prefix_expression",
                    token: token.kind,
                }
                .into_diagnostic()
                .emit();
                panic!("parse_prefix_expression not implemented for {x}")
            }
        };

        Ok(())
    }

    fn parse_ident_expression(&mut self) {
        let peeked = self.peek_token();

        match peeked.kind {
            TokenKind::LeftParenthesis => {
                self.parse_call();
            }
            TokenKind::RightBrace | TokenKind::Semicolon => {
                #[allow(clippy::needless_return)]
                return;
            }
            _ => {}
        }
    }

    fn parse_call_arguments(&mut self) {
        self.parse_enclosed_delim_seq(
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::Comma,
            |parser| Parser::parse_expression(parser, 0),
        );
    }

    fn parse_call(&mut self) {
        let c = self.tree.checkpoint().unwrap();

        // TODO: could be somthing like my.lib.some_function(), and not just some_function
        // self.parse_path();
        self.parse_call_arguments();

        self.tree.close_at(&c, TokenKind::CallExpression).unwrap();
    }

    fn parse_following_expression(&mut self) -> Result<(), Diagnostic<'src>> {
        let peeked = self.peek_token();
        if fowlc_lexer::INFIX_OPERATORS
            .iter()
            .any(|o| o == &peeked.kind)
        {
            self.tree.open(TokenKind::BinaryOperator).unwrap();
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

    fn parse_expression_statement(&mut self) -> Result<(), Diagnostic<'src>> {
        let c = self.tree.checkpoint().unwrap();

        self.parse_expression(0)?;

        self.expect_token(TokenKind::Semicolon).emit_ok();

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
                self.parse_ident();
                self.expect_token(TokenKind::Equal).emit_ok();
                self.parse_expression(0).emit_ok();
            }
            TokenKind::Return => {
                // Skip the peeked 'return'
                self.next_token();

                self.tree.token(TokenKind::Return, 6).unwrap();
                self.parse_expression(0).emit_ok();
            }
            _ => {
                self.parse_expression_statement().emit_ok();
            }
        }

        self.tree.close_at(&c, TokenKind::Statement).unwrap();
        Ok(())
    }

    fn parse_block(&mut self) {
        self.tree.open(TokenKind::Block).unwrap();

        self.parse_enclosed_delim_seq(
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
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
            _ => {
                Unimplemented {
                    span: t.span,
                    in_function: "parse_declaration",
                    token: t.kind,
                }
                .into_diagnostic()
                .emit();
                todo!()
            }
        }

        self.eat_if_token(TokenKind::Semicolon);

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
