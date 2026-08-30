use super::ast::{
    Block, Call, Declaration, Enum, EnumVariant, ExprKind, Function, Ident, Op, Param, Program,
    Statement, Struct, Type, TypeKind,
};
use crate::{
    ast::{BinaryOp, CallArg, Closure, Expr, Use, Vis},
    errors::{SelfParamInUnassociatedFunction, SyntaxError},
};
use error::{Diagnostic, Fmt, IntoDiagnostic, ResultExt};
use lexer::{Lexer, Token, TokenKind};

#[derive(Clone, Copy)]
pub enum Syntax {
    Fn,
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

    pub fn parse_new(&mut self) -> Result<(), syntree::Error> {
        let peeked = self.peek_token();
        match peeked.kind {
            TokenKind::Fn => {
                self.tree.open(Syntax::Fn)?;

                self.tree.close()?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    /// Eats the next token and returns it
    fn next_token(&mut self) -> Token<'src> {
        match self.lexer.next() {
            Ok(t) => t,
            Err(e) => {
                e.into_diagnostic().emit();
                self.next_token()
            }
        }
    }

    /// Peeks at the next `Ok` token, and eats all `Err` tokens up till that
    fn peek_token(&mut self) -> &Token<'src> {
        while let Err(e) = self.lexer.peek() {
            e.into_diagnostic().emit();
            self.next_token();
        }
        match self.lexer.peek() {
            Ok(t) => t,
            _ => unreachable!(),
        }
    }

    /// peeks at the next token, and eats if it it matches `token`, otherwise it returns a `Diagnostic`
    fn expect_token(&mut self, token: TokenKind<'src>) -> Result<Token<'src>, Diagnostic<'src>> {
        match *self.peek_token() {
            t if token == t.kind => {
                self.next_token();
                Ok(t)
            }
            t => Err(crate::errors::SyntaxError {
                span: t.span,
                expected: vec![format!("'{}'", token).into()],
            }
            .into_diagnostic()),
        }
    }

    fn expect_one_of_token(
        &mut self,
        tokens: &[TokenKind<'src>],
    ) -> Result<Token<'src>, Diagnostic<'src>> {
        match *self.peek_token() {
            t if tokens.contains(&t.kind) => {
                self.next_token();
                Ok(t)
            }
            t => Err(crate::errors::SyntaxError {
                span: t.span,
                expected: tokens
                    .iter()
                    .map(|t| format!("'{}'", t).into())
                    .collect::<Vec<_>>(),
            }
            .into_diagnostic()),
        }
    }

    pub fn parse(&mut self) -> Program<'src> {
        let mut declarations = Vec::new();

        loop {
            let peeked = self.peek_token();
            if matches!(
                peeked.kind,
                TokenKind::RBrace | TokenKind::RParen | TokenKind::Eof
            ) {
                break;
            };
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            }
        }

        Program { declarations }
    }

    fn parse_type(&mut self) -> Result<Type<'src>, Diagnostic<'src>> {
        match self.next_token() {
            Token {
                kind: TokenKind::Ident(i),
                span,
            } => Ok(Type {
                span,
                kind: TypeKind::Ident(Ident { inner: i, span }),
            }),
            Token { kind, span } => Err(SyntaxError {
                span,
                expected: vec!["a type".into()],
            }
            .into_diagnostic()),
        }
    }

    fn parse_vis(&mut self) -> Vis {
        match self.peek_token() {
            Token {
                kind: TokenKind::Public,
                ..
            } => {
                self.next_token();
                Vis::Public
            }
            Token {
                kind: TokenKind::Internal,
                ..
            } => {
                self.next_token();
                Vis::Internal
            }
            Token {
                kind: TokenKind::Private,
                ..
            } => {
                self.next_token();
                Vis::Private
            }
            _ => Vis::Private,
        }
    }

    fn parse_ident(&mut self) -> Result<Ident<'src>, Diagnostic<'src>> {
        match *self.peek_token() {
            Token {
                kind: TokenKind::Ident(name),
                span,
            } => {
                self.next_token();
                Ok(Ident { inner: name, span })
            }
            Token { kind, span } => Err(SyntaxError {
                span,
                expected: vec!["a name".into()],
            }
            .into_diagnostic()),
        }
    }

    fn parse_declaration(&mut self) -> Option<Declaration<'src>> {
        let vis = self.parse_vis();

        let t = self.peek_token();
        let kind = t.kind;
        let span = t.span;

        match kind {
            TokenKind::Struct => {
                // Skip the peeked "struct"
                self.next_token();

                let name = self
                    .parse_ident()
                    .add_help(format!(
                        "Try giving the struct a name: `struct {}`",
                        "MyStruct".fg(error::colors::PRIMARY)
                    ))
                    .emit_ok();

                self.expect_token(TokenKind::LParen)
                    .add_help(format!(
                        "try giving the struct a body: `struct {} {{}}`",
                        name.map(|n| n.inner).unwrap_or("Name"),
                    ))
                    .emit_ok();

                let mut struct_span = span;

                let mut fields: Vec<(Option<Ident<'src>>, Option<Type<'src>>)> = Vec::new();
                loop {
                    if self.expect_token(TokenKind::RParen).is_ok() {
                        break;
                    }

                    let param_name = self
                        .parse_ident()
                        .add_help(format!(
                            "try giving the field a name: `{}: ...`",
                            "my_field".fg(error::colors::PRIMARY),
                        ))
                        .emit_ok();
                    self.expect_token(TokenKind::Colon).emit_ok();

                    let ty = self.parse_type().emit_ok();
                    fields.push((param_name, ty));

                    self.expect_token(TokenKind::Semicolon).emit_ok();
                    if let Ok(t) = self.expect_token(TokenKind::RParen) {
                        struct_span = struct_span.merge(t.span);
                        let _ = self.expect_token(TokenKind::Semicolon);
                        break;
                    }
                }

                Some(Declaration::Struct(Struct {
                    span: struct_span,
                    name,
                    fields,
                    vis,
                }))
            }
            TokenKind::Enum => {
                // Skip the peeked "enum"
                self.next_token();

                // Check if there is an enum name
                let name = self
                    .parse_ident()
                    .add_help(format!(
                        "Try giving the enum a name: `enum {}`",
                        "MyEnum".fg(error::colors::PRIMARY)
                    ))
                    .emit_ok();

                self.expect_token(TokenKind::LParen).emit_ok();

                let mut enum_span = span;

                let mut variants: Vec<EnumVariant> = Vec::new();
                loop {
                    if self.expect_token(TokenKind::RParen).is_ok() {
                        break;
                    }

                    let variant_name = match self.parse_ident() {
                        Ok(n) => n,
                        Err(e) => {
                            e.add_help(format!(
                                "try giving the variant a name: `{} ...`",
                                "MyVariant".fg(error::colors::PRIMARY)
                            ))
                            .emit();
                            continue;
                        }
                    };
                    self.expect_token(TokenKind::Semicolon).emit_ok();

                    variants.push(EnumVariant {
                        span: variant_name.span,
                        name: variant_name,
                        fields: Vec::new(),
                    });

                    if let Ok(t) = self.expect_token(TokenKind::RParen) {
                        enum_span = enum_span.merge(t.span);
                        let _ = self.expect_token(TokenKind::Semicolon);
                        self.next_token();
                        break;
                    }
                }

                Some(Declaration::Enum(Enum {
                    variants,
                    span: enum_span,
                    name,
                    vis,
                }))
            }
            TokenKind::Fn => Some(Declaration::Function(self.parse_function().set_vis(vis))),
            TokenKind::Use => {
                // Skip the peeked "use"
                self.next_token();

                let mut import = vec![];
                if let Some(namespace) = self.parse_ident().emit_ok() {
                    import.push(namespace);
                }

                // repeatedly parse `.submodule`
                loop {
                    match self.peek_token() {
                        Token {
                            kind: TokenKind::Semicolon,
                            ..
                        } => {
                            self.next_token();
                            break;
                        }
                        Token {
                            kind: TokenKind::Dot,
                            ..
                        } => {
                            self.next_token();
                            if let Some(submodule) = self.parse_ident().emit_ok() {
                                import.push(submodule);
                            }
                            continue;
                        }
                        Token { kind: _, span } => {
                            SyntaxError {
                                span: *span,
                                expected: vec!["a '.module' or ';'".into()],
                            }
                            .into_diagnostic()
                            .emit();
                        }
                    }
                }

                Some(Declaration::Use(Use {
                    import,
                    // TODO:
                    span,
                }))
            }
            _ => {
                self.next_token();
                SyntaxError {
                    span,
                    expected: vec!["a declaration".into()],
                }
                .into_diagnostic()
                .emit();
                None
            }
        }
    }

    fn parse_function(&mut self) -> Function<'src> {
        // Skip the peeked "fn"
        let t = self.next_token();
        let mut function_span = t.span;

        let peeked = self.peek_token();
        let on = match peeked.kind {
            TokenKind::On => {
                self.next_token();

                self.parse_type()
                    .add_help(format!(
                        "try adding a type here: `on {} ...`",
                        "MyType".fg(error::colors::PRIMARY)
                    ))
                    .emit_ok()
            }
            _ => None,
        };

        let name = self
            .parse_ident()
            .add_help(format!(
                "try giving the function a name: `fn {}() ...`",
                "my_function".fg(error::colors::PRIMARY)
            ))
            .emit_ok();

        let params = self.parse_function_parameters(on);

        let ret_ty = self.parse_type().emit_ok();

        let mut spans = vec![];
        if let Some(lbrace) = self.expect_token(TokenKind::LBrace).emit_ok() {
            spans.push(lbrace.span);
        }

        let statements = self.parse_statements();
        if !statements.is_empty() {
            spans.push(*statements.first().unwrap().span());
            spans.push(*statements.last().unwrap().span());
        }

        if let Some(rbrace) = self.expect_token(TokenKind::RBrace).emit_ok() {
            spans.push(rbrace.span);
        }
        let block_span = spans.first().unwrap().merge(*spans.last().unwrap());

        let _ = self.expect_token(TokenKind::Semicolon);
        let block = Block {
            statements,
            span: block_span,
        };
        function_span = function_span.merge(block_span);

        Function {
            span: function_span,
            on,
            name,
            ret_ty,
            params,
            body: block,
            vis: Vis::Private,
        }
    }

    fn parse_function_parameters(&mut self, self_type: Option<Type<'src>>) -> Vec<Param<'src>> {
        self.expect_token(TokenKind::LParen).emit_ok();
        let mut parameters = Vec::new();

        if self.expect_token(TokenKind::RParen).is_ok() {
            // immediate parameter list end
            return parameters;
        }

        // The first param is allowed to be a `self`
        let peeked = self.peek_token();
        let peeked_span = peeked.span;
        if peeked.kind == TokenKind::Self_ {
            self.next_token();
            match self_type {
                None => {
                    SelfParamInUnassociatedFunction { span: peeked_span }
                        .into_diagnostic()
                        .emit();
                }
                Some(self_type) => {
                    self.expect_token(TokenKind::Comma).emit_ok();
                    parameters.push(Param {
                        span: peeked_span,
                        name: Some(Ident {
                            inner: "self",
                            span: peeked_span,
                        }),
                        label_ignored: false,
                        ty: Some(self_type),
                        default: None,
                    });
                }
            }
        }

        loop {
            let peeked = self.peek_token();
            let span = peeked.span;
            let label_ignored = if peeked.kind == TokenKind::Underscore {
                self.next_token();
                true
            } else {
                false
            };
            let name = self.parse_ident().emit_ok();
            self.expect_token(TokenKind::Colon).emit_ok();

            let ty = self.parse_type().emit_ok();
            let mut param_span = span;
            if let Some(ty) = ty {
                param_span = param_span.merge(ty.span);
            }

            parameters.push(Param {
                span: param_span,
                label_ignored,
                name,
                ty,
                default: None,
            });

            let token = self
                .expect_one_of_token(&[TokenKind::RParen, TokenKind::Comma])
                .emit_ok();

            if let Some(token) = token
                && token.kind == TokenKind::RParen
            {
                break;
            }
        }

        parameters
    }

    fn parse_statement(&mut self) -> Option<Statement<'src>> {
        let token = self.peek_token();
        let token_span = token.span;

        match token.kind {
            TokenKind::Let => {
                // Skip the peeked 'let'
                self.next_token();

                // Check if there is a mut
                let mutable = if let Token {
                    kind: TokenKind::Mut,
                    ..
                } = self.peek_token()
                {
                    self.next_token();
                    true
                } else {
                    false
                };

                let name = self.parse_ident().emit_ok();

                // Try and get type
                let ty = if let Token {
                    kind: TokenKind::Colon,
                    ..
                } = self.peek_token()
                {
                    self.next_token();

                    self.parse_type().emit_ok()
                } else {
                    None
                };

                self.expect_token(TokenKind::Eq).emit_ok();

                let expression = self.parse_expression(0).emit_ok();

                self.expect_token(TokenKind::Semicolon).emit_ok();

                Some(Statement::Let {
                    span: token_span,
                    name,
                    mutable,
                    ty,
                    expr: expression,
                })
            }
            TokenKind::Return => {
                // Skip the peeked "return"
                self.next_token();

                let rhs = self.parse_expression(1).emit_ok();
                let _ = self.expect_token(TokenKind::Semicolon);
                Some(Statement::Return {
                    span: rhs
                        .as_ref()
                        .map(|rhs| token_span.merge(rhs.span))
                        .unwrap_or(token_span),
                    expr: rhs,
                })
            }
            TokenKind::For => {
                // Skip the peeked "for"
                let Token { span, .. } = self.next_token();
                let (cond, lbrace) = match self.next_token() {
                    t if t.kind == TokenKind::LBrace => (None, Some(t)),
                    t if t.kind == TokenKind::LParen => {
                        let cond = self.parse_expression(0).emit_ok();
                        self.expect_token(TokenKind::RParen).emit_ok();
                        let lbrace = self.expect_token(TokenKind::LBrace).emit_ok();

                        (cond, lbrace)
                    }
                    _ => {
                        SyntaxError {
                            span,
                            expected: vec!["'('".into(), "{".into()],
                        }
                        .into_diagnostic()
                        .emit();
                        (None, None)
                    }
                };
                let stmts = self.parse_statements();
                let rbrace = self.expect_token(TokenKind::RBrace).emit_ok();
                let _ = self.expect_token(TokenKind::Semicolon);

                let block = if lbrace.is_none() && rbrace.is_none() && stmts.is_empty() {
                    None
                } else {
                    let mut spans = vec![];
                    if let Some(lbrace) = lbrace {
                        spans.push(lbrace.span);
                    };
                    if !stmts.is_empty() {
                        spans.push(*stmts.first().unwrap().span());
                        spans.push(*stmts.last().unwrap().span());
                    }
                    if let Some(rbrace) = rbrace {
                        spans.push(rbrace.span)
                    }
                    let span = spans.first().unwrap().merge(*spans.last().unwrap());
                    Some(Block {
                        span,
                        statements: stmts,
                    })
                };

                Some(Statement::ForLoop {
                    span: rbrace.map(|rbrace| span.merge(rbrace.span)).unwrap_or(span),
                    cond,
                    block,
                })
            }
            // TokenKind::If => {
            //     // Skip the peeked "if"
            //     self.next_token();
            //     self.expect_token(TokenKind::LParen).emit_ok();
            //     let cond = self.parse_expression(0).emit_ok();
            //     self.expect_token(TokenKind::RParen).emit_ok();
            //     let then_lbrace = self.expect_token(TokenKind::LBrace).emit_ok();
            //     let then = self.parse_statements();
            //     let then_rbrace = self.expect_token(TokenKind::RBrace).emit_ok();
            //     let mut else_if_blocks = Vec::new();
            //     let mut else_block = None;
            //     while let Token {
            //         kind: TokenKind::Else,
            //         ..
            //     } = self.peek_token()
            //     {
            //         // Skip the peeked 'else'
            //         self.next_token();

            //         if let Token {
            //             kind: TokenKind::If,
            //             ..
            //         } = self.peek_token()
            //         {
            //             // Skip the peeked 'if'
            //             self.next_token();
            //             self.expect_token(TokenKind::LParen)?;
            //             let cond = self.parse_expression(0)?;
            //             self.expect_token(TokenKind::RParen)?;
            //             let then_lbrace = self.expect_token(TokenKind::LBrace)?;
            //             let then = self.parse_statements()?;
            //             let then_rbrace = self.expect_token(TokenKind::RBrace)?;
            //             else_if_blocks.push((
            //                 cond,
            //                 Block {
            //                     span: then_lbrace.span.merge(then_rbrace.span),
            //                     statements: then,
            //                 },
            //             ));
            //         } else {
            //             let then_lbrace = self.expect_token(TokenKind::LBrace)?;
            //             let then = self.parse_statements()?;
            //             let then_rbrace = self.expect_token(TokenKind::RBrace)?;
            //             else_block = Some(Block {
            //                 span: then_lbrace.span.merge(then_rbrace.span),
            //                 statements: then,
            //             });
            //             break;
            //         }
            //     }

            //     self.expect_token(TokenKind::Semicolon)?;

            //     Ok(Statement::Expr(Expr {
            //         span: token_span,
            //         kind: ExprKind::If {
            //             cond: Box::new(cond),
            //             then: Block {
            //                 span: then_lbrace.span.merge(then_rbrace.span),
            //                 statements: then,
            //             },
            //             else_if_blocks,
            //             else_block,
            //         },
            //     }))
            // }
            TokenKind::Ident(ident) => {
                let more = self.lexer.peek_more();
                // TODO: Don't resolve the += (and similar) syntax sugar here, but in BIR
                let op = match more {
                    Some(Ok(Token {
                        kind: TokenKind::PlusEq,
                        ..
                    })) => BinaryOp::Add,
                    Some(Ok(Token {
                        kind: TokenKind::MinusEq,
                        ..
                    })) => BinaryOp::Sub,
                    Some(Ok(Token {
                        kind: TokenKind::StarEq,
                        ..
                    })) => BinaryOp::Mul,
                    Some(Ok(Token {
                        kind: TokenKind::SlashEq,
                        ..
                    })) => BinaryOp::Div,
                    Some(Ok(Token {
                        kind: TokenKind::Eq,
                        ..
                    })) => {
                        // This is an assignment

                        // Skip the peeked ident
                        let _ = self.next_token();
                        // Skip the peeked '='
                        let _ = self.next_token();
                        let expr = self.parse_expression(0).emit_ok();
                        self.expect_token(TokenKind::Semicolon).emit_ok();

                        return Some(Statement::Assign {
                            name: Ident {
                                inner: ident,
                                span: token_span,
                            },
                            span: expr
                                .as_ref()
                                .map(|e| token_span.merge(e.span))
                                .unwrap_or(token_span),
                            expr,
                        });
                    }
                    _ => {
                        let expr = self.parse_expression(0).emit_ok();
                        self.expect_token(TokenKind::Semicolon).emit_ok();
                        return expr.map(Statement::Expr);
                    }
                };
                // Skip the peeked ident
                let _ = self.next_token();
                // Skip the peeked op
                let _ = self.next_token();
                let expr = self.parse_expression(0).emit_ok();
                let name = Ident {
                    inner: ident,
                    span: token_span,
                };
                let expr = Expr {
                    span: expr
                        .as_ref()
                        .map(|e| name.span.merge(e.span))
                        .unwrap_or(name.span),
                    kind: ExprKind::Binary {
                        op,
                        left: Some(Box::new(Expr {
                            kind: ExprKind::Ident(name),
                            span: name.span,
                        })),
                        right: expr.map(Box::new),
                    },
                };
                self.expect_token(TokenKind::Semicolon).emit_ok();
                Some(Statement::Assign {
                    name,
                    span: token_span.merge(expr.span),
                    expr: Some(expr),
                })
            }
            TokenKind::Break => {
                let _ = self.lexer.next();
                self.expect_token(TokenKind::Semicolon).emit_ok();
                Some(Statement::Break { span: token_span })
            }
            TokenKind::Continue => {
                let _ = self.lexer.next();
                self.expect_token(TokenKind::Semicolon).emit_ok();
                Some(Statement::Continue { span: token_span })
            }
            _ => {
                let e = self.parse_expression(0).emit_ok();
                self.expect_token(TokenKind::Semicolon).emit_ok();
                e.map(Statement::Expr)
                //     Err(Diagnostic::error(
                //     statement_span,
                //     format!("Unexpected token {:?}. Expected statement start.", t),
                // )
                // .with_error_label(statement_span, "here"))},
            }
        }
    }

    fn parse_statements(&mut self) -> Vec<Statement<'src>> {
        let mut statements = Vec::new();

        loop {
            if matches!(
                self.peek_token(),
                Token {
                    kind: TokenKind::RBrace,
                    ..
                }
            ) {
                break;
            };

            if let Some(s) = self.parse_statement() {
                statements.push(s);
            }
        }

        statements
    }

    fn parse_string_literal_or_interpolation(&mut self) -> Result<Expr<'src>, Diagnostic<'src>> {
        let mut exprs = Vec::new();
        while !matches!(
            self.peek_token(),
            Token {
                kind: TokenKind::StringInterpolationEnd,
                ..
            }
        ) {
            let e = self.parse_string_interpolation_expr(0)?;
            exprs.push(e);
        }

        if exprs.len() == 1 {
            let first = exprs.first().expect("Must be there by the if check");
            if let ExprKind::StringLiteral(_) = first.kind {
                return Ok(first.clone());
            }
        }

        Ok(Expr {
            // TODO:
            span: exprs.first().unwrap().span,
            kind: ExprKind::StringInterpolation(exprs),
        })
    }

    fn parse_string_interpolation_expr(
        &mut self,
        min_bp: u8,
    ) -> Result<Expr<'src>, Diagnostic<'src>> {
        let token = self.next_token();
        let expr_span = token.span;

        match token.kind {
            TokenKind::LBrace => {
                let expr = self.parse_expression(min_bp)?;
                self.next_token();
                Ok(expr)
            }
            TokenKind::StringLiteral(s) => Ok(Expr {
                kind: ExprKind::StringLiteral(Some(s)),
                span: expr_span,
            }),
            _ => Err(SyntaxError {
                span: expr_span,
                expected: vec!["a string or '{'".into()],
            }
            .into_diagnostic()),
        }
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Expr<'src>, Diagnostic<'src>> {
        let token = self.next_token();
        let expr_span = token.span;

        // Create the initial lhs. Then we will, in a recursion like way update lhs to be the lhs with an op and a right side
        let mut lhs = match token.kind {
            TokenKind::IntLiteral(i) => Expr {
                kind: ExprKind::IntLiteral(i),
                span: expr_span,
            },
            TokenKind::FloatLiteral(f) => Expr {
                kind: ExprKind::FloatLiteral(f),
                span: expr_span,
            },
            TokenKind::BoolLiteral(b) => Expr {
                kind: ExprKind::BoolLiteral(b),
                span: expr_span,
            },
            TokenKind::Ident(ident) => Expr {
                kind: ExprKind::Ident(Ident {
                    inner: ident,
                    span: expr_span,
                }),
                span: expr_span,
            },
            TokenKind::StringInterpolationStart => self.parse_string_literal_or_interpolation()?,
            TokenKind::StringLiteral(s) => Expr {
                kind: ExprKind::StringLiteral(Some(s)),
                span: expr_span,
            },
            // groups
            TokenKind::LParen => {
                let lhs = self.parse_expression(0)?;
                self.expect_token(TokenKind::RParen)?;

                lhs
            }
            TokenKind::Self_ => Expr {
                kind: ExprKind::Ident(Ident {
                    inner: "self",
                    span: expr_span,
                }),
                span: expr_span,
            },
            TokenKind::Fn => {
                // TODO: Allow named closures if they are not assigned to a variable

                // TODO: Make a separate parse_closure_parameters.
                // We would wan't to not care about labels, and possibly param types
                let params = self.parse_function_parameters(None);
                let ty = self.parse_type()?;
                let lbrace = self.expect_token(TokenKind::LBrace)?;
                let body = self.parse_statements();
                let rbrace = self.expect_token(TokenKind::RBrace)?;
                Expr {
                    span: expr_span.merge(rbrace.span),
                    kind: ExprKind::Closure(Closure {
                        params,
                        ret_ty: ty,
                        body: Block {
                            span: lbrace.span.merge(rbrace.span),
                            statements: body,
                        },
                    }),
                }
            }
            _ => {
                return Err(SyntaxError {
                    span: expr_span,
                    expected: vec!["an item".into()],
                }
                .into_diagnostic());
            }
        };

        loop {
            let op_token = self.peek_token();
            let expr_span = expr_span.merge(op_token.span);

            let op = match op_token.kind {
                TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Comma | TokenKind::RParen => {
                    break;
                }
                TokenKind::Plus => Op::Add,
                TokenKind::Minus => Op::Sub,
                TokenKind::Star => Op::Mul,
                TokenKind::Slash => Op::Div,
                TokenKind::StarStar => Op::Exp,
                TokenKind::Percent => Op::Mod,
                TokenKind::And => Op::And,
                TokenKind::Or => Op::Or,
                TokenKind::EqEq => Op::Eq,
                TokenKind::Lt => Op::Lt,
                TokenKind::LtEq => Op::LtEq,
                TokenKind::Gt => Op::Gt,
                TokenKind::GtEq => Op::GtEq,
                TokenKind::LParen => Op::Call,
                TokenKind::LBrace => Op::StructInstance,
                TokenKind::Dot => Op::FieldIndex,
                TokenKind::StringInterpolationEnd => {
                    self.next_token();
                    return Ok(lhs);
                }
                _ => {
                    return Err(SyntaxError {
                        span: op_token.span,
                        expected: vec!["an infix operator".into()],
                    }
                    .into_diagnostic());
                }
            };

            if let Some((l_bp, ())) = op.postfix_binding_power() {
                if l_bp < min_bp {
                    break;
                }

                lhs = match op {
                    Op::Call => {
                        // Eat the '('
                        self.next_token();
                        let args = self.parse_fn_call_arguments()?;
                        // TODO: extend expr_span
                        Expr {
                            kind: ExprKind::Call(Call {
                                callee: Box::new(lhs),
                                args,
                            }),
                            span: expr_span,
                        }
                    }
                    Op::StructInstance => {
                        let struct_name = self.parse_ident()?;
                        let struct_fields = self.parse_struct_fields()?;
                        Expr {
                            kind: ExprKind::StructInstance {
                                name: struct_name,
                                fields: struct_fields,
                            },
                            span: expr_span,
                        }
                    }
                    _ => todo!(),
                };
                continue;
            }

            if let Some((l_bp, r_bp)) = op.infix_binding_power() {
                if l_bp < min_bp {
                    break;
                }
                // We only peeked before, consume it now.
                self.next_token();

                lhs = match op {
                    Op::FieldIndex => {
                        let expr = self.parse_expression(r_bp)?;
                        let rhs = match expr.kind {
                            ExprKind::Ident(ident) => ident,
                            _ => {
                                return Err(SyntaxError {
                                    span: expr_span,
                                    expected: vec!["a field name".into()],
                                }
                                .into_diagnostic());
                            }
                        };
                        Expr {
                            kind: ExprKind::Member {
                                object: Box::new(lhs),
                                field: rhs,
                            },
                            span: expr_span,
                        }
                    }
                    _ => {
                        let rhs = self.parse_expression(r_bp)?;
                        Expr {
                            kind: ExprKind::Binary {
                                op: BinaryOp::from_op(op).unwrap(),
                                left: Some(Box::new(lhs)),
                                right: Some(Box::new(rhs)),
                            },
                            span: expr_span,
                        }
                    }
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_fn_call_arguments(&mut self) -> Result<Vec<CallArg<'src>>, Diagnostic<'src>> {
        let mut arguments = Vec::new();

        // parent has already eaten left paren as the operator

        if matches!(
            self.peek_token(),
            Token {
                kind: TokenKind::RParen,
                ..
            }
        ) {
            // immediate argument list end
            self.next_token();
        } else {
            loop {
                let mut call_arg_span = None;
                let label = if let Token {
                    kind: TokenKind::Ident(i),
                    span,
                } = self.peek_token()
                {
                    call_arg_span = Some(*span);
                    Some(Ident {
                        inner: i,
                        span: *span,
                    })
                } else {
                    None
                };
                if label.is_some() {
                    let _ = self.lexer.next();
                    self.expect_token(TokenKind::Eq)?;
                }
                let value = self.parse_expression(0)?;
                arguments.push(CallArg {
                    span: call_arg_span
                        .map(|s| s.merge(value.span))
                        .unwrap_or(value.span),
                    label,
                    expr: value,
                });

                let token = self.expect_one_of_token(&[TokenKind::RParen, TokenKind::Comma])?;

                if token.kind == TokenKind::RParen {
                    break;
                }
            }
        }

        Ok(arguments)
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(Ident<'src>, Expr<'src>)>, Diagnostic<'src>> {
        let mut fields = Vec::new();

        // parent has already eaten left brace as the operator

        if matches!(
            self.peek_token(),
            Token {
                kind: TokenKind::RBrace,
                ..
            }
        ) {
            // immediate argument list end
            self.next_token();
        } else {
            loop {
                if matches!(
                    self.peek_token(),
                    Token {
                        kind: TokenKind::RBrace,
                        ..
                    }
                ) {
                    // struct end
                    self.next_token();
                    break;
                }

                let field_name = self.parse_ident()?;

                self.expect_token(TokenKind::Eq)?;

                let value = self.parse_expression(0)?;

                fields.push((field_name, value));

                let token = self.expect_one_of_token(&[TokenKind::RBrace, TokenKind::Comma])?;

                if token.kind == TokenKind::RBrace {
                    break;
                }
            }
        }

        Ok(fields)
    }
}
