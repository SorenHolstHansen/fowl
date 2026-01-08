use super::ast::{
    Block, Call, Declaration, Enum, EnumVariant, ExprKind, Function, Ident, Op, Param, Program,
    Statement, Struct, Type, TypeKind,
};
use crate::ast::{BinaryOp, CallArg, Expr, Use, Vis};
use error::Diagnostic;
use lexer::{Lexer, Token, TokenKind};

pub fn parse<'src>(lexer: Lexer<'src>) -> (Program<'src>, Vec<Diagnostic<'src>>) {
    let mut parser = Parser {
        lexer,
        errors: vec![],
    };

    let program = parser.parse();

    (program, parser.errors)
}

struct Parser<'src> {
    lexer: Lexer<'src>,
    errors: Vec<Diagnostic<'src>>,
}

impl<'src> Parser<'src> {
    fn next_token(&mut self) -> Token<'src> {
        match self.lexer.next() {
            Ok(t) => t,
            Err(e) if e.is_eof() => panic!("Eof should have been handled by now"),
            Err(e) => {
                self.errors.push((&e).into());
                self.next_token()
            }
        }
    }

    fn peek_token(&mut self) -> &Token<'src> {
        while let Err(e) = self.lexer.peek() {
            self.errors.push(e.into());
            self.next_token();
        }
        match self.lexer.peek() {
            Ok(t) => t,
            _ => unreachable!(),
        }
    }

    fn eat_if_token(&mut self, token: TokenKind<'src>) {
        let peeked = self.peek_token();
        if peeked.kind == token {
            self.next_token();
        }
    }

    fn expect_token(&mut self, token: TokenKind<'src>) -> Result<Token<'src>, Diagnostic<'src>> {
        match self.next_token() {
            t if token == t.kind => Ok(t),
            t => Err(Diagnostic::error(
                t.span,
                format!("Syntax error: Expected '{}', found '{}'", token, t.kind),
            )
            .with_error_label(t.span, "here")),
        }
    }

    fn expect_one_of_token(
        &mut self,
        tokens: &[TokenKind<'src>],
    ) -> Result<Token<'src>, Diagnostic<'src>> {
        match self.next_token() {
            t if tokens.contains(&t.kind) => Ok(t),
            t => Err(Diagnostic::error(
                t.span,
                format!(
                    "Syntax error: Expected one of {}, found '{}'",
                    tokens
                        .iter()
                        .map(|t| format!("'{}'", t))
                        .collect::<Vec<_>>()
                        .join(", "),
                    t.kind
                ),
            )
            .with_error_label(t.span, "here")),
        }
    }

    fn parse(&mut self) -> Program<'src> {
        let mut declarations = Vec::new();

        loop {
            let peeked = self.peek_token();
            if matches!(
                peeked.kind,
                TokenKind::RBrace | TokenKind::RParen | TokenKind::Eof
            ) {
                break;
            };
            match self.parse_declaration() {
                Ok(d) => declarations.push(d),
                Err(e) => {
                    self.errors.push(e);
                    return Program { declarations };
                }
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
            Token {
                kind: TokenKind::Int,
                span,
            } => Ok(Type {
                kind: TypeKind::Int,
                span,
            }),
            Token {
                kind: TokenKind::Float,
                span,
            } => Ok(Type {
                kind: TypeKind::Float,
                span,
            }),
            Token {
                kind: TokenKind::Bool,
                span,
            } => Ok(Type {
                kind: TypeKind::Bool,
                span,
            }),
            Token {
                kind: TokenKind::String,
                span,
            } => Ok(Type {
                kind: TypeKind::String,
                span,
            }),
            Token {
                kind: TokenKind::Void,
                span,
            } => Ok(Type {
                kind: TypeKind::Void,
                span,
            }),
            Token {
                kind: TokenKind::Eof,
                span,
            } => Err(Diagnostic::error(span, "Unexpected EOF")),
            Token { kind: _, span } => {
                Err(Diagnostic::error(span, "expected type").with_error_label(span, "here"))
            }
        }
    }

    fn parse_vis(&mut self) -> Result<Vis, Diagnostic<'src>> {
        match self.peek_token() {
            Token {
                kind: TokenKind::Public,
                ..
            } => {
                self.next_token();
                Ok(Vis::Public)
            }
            Token {
                kind: TokenKind::Internal,
                ..
            } => {
                self.next_token();
                Ok(Vis::Internal)
            }
            Token {
                kind: TokenKind::Private,
                ..
            } => {
                self.next_token();
                Ok(Vis::Private)
            }
            _ => Ok(Vis::Private),
        }
    }

    fn parse_ident(&mut self) -> Result<Ident<'src>, Diagnostic<'src>> {
        match self.next_token() {
            Token {
                kind: TokenKind::Ident(name),
                span,
            } => Ok(Ident { inner: name, span }),
            Token {
                kind: TokenKind::Eof,
                span,
            } => Err(Diagnostic::error(span, "Unexpected EOF")),
            Token { kind: _, span } => {
                Err(Diagnostic::error(span, "expected identifier").with_error_label(span, "here"))
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration<'src>, Diagnostic<'src>> {
        let vis = self.parse_vis()?;

        let t = self.peek_token();
        let kind = t.kind;
        let span = t.span;

        match kind {
            TokenKind::Struct => {
                // Skip the peeked "struct"
                self.next_token();

                // Check if there is a struct name
                let name = self.parse_ident().map_err(|e| {
                    e.with_help("Expected struct name")
                        .with_note("try giving the struct a name: `struct MyStruct {{}}`")
                })?;

                self.expect_token(TokenKind::LBrace).map_err(|e| {
                    e.with_note(format!(
                        "try giving the struct a body: `struct {} {{}}`",
                        name.inner,
                    ))
                })?;

                let mut struct_span = span;

                let mut fields: Vec<(Ident<'src>, Type<'src>)> = Vec::new();
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

                    let param_name = self.parse_ident().map_err(|e| {
                        e.with_help("Expected struct field name")
                            .with_note("try giving the field a name: `my_field: ...`")
                    })?;
                    self.expect_token(TokenKind::Colon)?;

                    let ty = self.parse_type()?;
                    fields.push((param_name, ty));

                    self.expect_token(TokenKind::Semicolon)?;
                    if let Token {
                        kind: TokenKind::RBrace,
                        span,
                    } = self.peek_token()
                    {
                        struct_span = struct_span.merge(*span);
                        self.next_token();
                        self.eat_if_token(TokenKind::Semicolon);
                        break;
                    }
                }

                Ok(Declaration::Struct(Struct {
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
                let name = self.parse_ident().map_err(|e| {
                    e.with_help("Expected enum name")
                        .with_note("try giving the enum a name: `enum MyEnum {{}}`")
                })?;

                self.expect_token(TokenKind::LBrace).map_err(|e| {
                    e.with_note(format!(
                        "try giving the enum a body: `enum {} {{}}`",
                        name.inner,
                    ))
                })?;

                let mut enum_span = span;

                let mut variants: Vec<EnumVariant> = Vec::new();
                loop {
                    if matches!(
                        self.peek_token(),
                        Token {
                            kind: TokenKind::RBrace,
                            ..
                        }
                    ) {
                        // enum end
                        self.next_token();
                        break;
                    }

                    let variant_name = self.parse_ident().map_err(|e| {
                        e.with_help("Expected enum variant name")
                            .with_note("try giving the variant a name: `MyVariant ...`")
                    })?;
                    self.expect_token(TokenKind::Semicolon)?;

                    variants.push(EnumVariant {
                        span: variant_name.span,
                        name: variant_name,
                        fields: Vec::new(),
                    });

                    if let Token {
                        kind: TokenKind::RBrace,
                        span,
                    } = self.peek_token()
                    {
                        enum_span = enum_span.merge(*span);
                        // Eat '}'
                        self.next_token();
                        self.eat_if_token(TokenKind::Semicolon);
                        self.next_token();
                        break;
                    }
                }

                Ok(Declaration::Enum(Enum {
                    variants,
                    span: enum_span,
                    name,
                    vis,
                }))
            }
            TokenKind::Fn => Ok(Declaration::Function(self.parse_function()?.set_vis(vis))),
            TokenKind::Use => {
                // Skip the peeked "use"
                self.next_token();

                let namespace = self.parse_ident()?;
                let mut import = vec![namespace];

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
                            let submodule = self.parse_ident()?;
                            import.push(submodule);
                            continue;
                        }
                        Token { kind: _, span } => {
                            return Err(Diagnostic::error(*span, "Unexpected token")
                                .with_error_label(*span, "here"));
                        }
                    }
                }

                Ok(Declaration::Use(Use {
                    import,
                    // TODO:
                    span,
                }))
            }
            TokenKind::Let => Err(Diagnostic::error(span, "Unexpected let binding")),
            TokenKind::Return => Err(Diagnostic::error(span, "Unexpected return statement")),
            x => Err(Diagnostic::error(span, format!("Unexpected '{}'", x))
                .with_error_label(span, "here")),
        }
    }

    fn parse_function(&mut self) -> Result<Function<'src>, Diagnostic<'src>> {
        // Skip the peeked "fn"
        let t = self.next_token();
        let mut function_span = t.span;

        let peeked = self.peek_token();
        let on = match peeked.kind {
            TokenKind::On => {
                self.next_token();
                let ty = self.parse_type()?;
                Some(ty)
            }
            _ => None,
        };

        let name = self.parse_ident().map_err(|e| {
            e.with_help("Expected function name")
                .with_note("try giving the function a name: `fn my_function() ...`")
        })?;

        let params = self.parse_function_parameters(on)?;

        let ret_ty = self
            .parse_type()
            .map_err(|e| e.with_note("expected function return type"))?;

        let Token {
            span: lbrace_span, ..
        } = self.expect_token(TokenKind::LBrace)?;

        let statements = self.parse_statements()?;

        let Token {
            span: rbrace_span, ..
        } = self.expect_token(TokenKind::RBrace)?;
        self.eat_if_token(TokenKind::Semicolon);
        let block = Block {
            statements,
            span: lbrace_span.merge(rbrace_span),
        };
        function_span = function_span.merge(rbrace_span);

        Ok(Function {
            span: function_span,
            on,
            name,
            ret_ty,
            params,
            body: block,
            vis: Vis::Private,
        })
    }

    fn parse_function_parameters(
        &mut self,
        function_on: Option<Type<'src>>,
    ) -> Result<Vec<Param<'src>>, Diagnostic<'src>> {
        self.expect_token(TokenKind::LParen)?;
        let mut parameters = Vec::new();

        if matches!(
            self.peek_token(),
            Token {
                kind: TokenKind::RParen,
                ..
            }
        ) {
            // immediate parameter list end
            self.next_token();
            return Ok(parameters);
        }

        // The first param is allowed to be a `self`
        let peeked = self.peek_token();
        let peeked_span = peeked.span;
        if peeked.kind == TokenKind::Self_ {
            self.next_token();
            let Some(function_on) = function_on else {
                return Err(Diagnostic::error(
                    peeked_span,
                    "Function can only accept a `self` parameter if it is a funciton on a type",
                )
                .with_error_label(
                    peeked_span,
                    "Function can only accept a `self` parameter if it is a funciton on a type",
                )
                .with_help("Consider making this a function on some type: `fn on MyType ...`"));
            };
            self.expect_token(TokenKind::Comma)?;
            parameters.push(Param {
                span: peeked_span,
                name: Ident {
                    inner: "self",
                    span: peeked_span,
                },
                label_ignored: false,
                ty: function_on,
                default: None,
            });
        }

        loop {
            let peeked = self.peek_token();
            let span = peeked.span;
            let label_ignored = if peeked.kind == TokenKind::Underscore {
                self.lexer.next().unwrap();
                true
            } else {
                false
            };
            let name = self.parse_ident()?;
            self.expect_token(TokenKind::Colon)?;

            let ty = self.parse_type()?;

            parameters.push(Param {
                span: span.merge(ty.span),
                label_ignored,
                name,
                ty,
                default: None,
            });

            let token = self
                .expect_one_of_token(&[TokenKind::RParen, TokenKind::Comma])
                .map_err(|e| {
                    e.with_note("Expected new parameter or a ) to end the parameter list")
                })?;

            if token.kind == TokenKind::RParen {
                break;
            }
        }

        Ok(parameters)
    }

    fn parse_statement(&mut self) -> Result<Statement<'src>, Diagnostic<'src>> {
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

                let ident = self.parse_ident()?;

                // Try and get type
                let ty = if let Token {
                    kind: TokenKind::Colon,
                    ..
                } = self.peek_token()
                {
                    self.next_token();
                    let ty = self.parse_type()?;
                    Some(ty)
                } else {
                    None
                };

                self.expect_token(TokenKind::Eq)?;

                let expression = self.parse_expression(0)?;

                let Token { span, .. } = self.expect_token(TokenKind::Semicolon)?;

                Ok(Statement::Let {
                    span: token_span.merge(span),
                    name: ident,
                    mutable,
                    ty,
                    expr: expression,
                })
            }
            TokenKind::Return => {
                // Skip the peeked "return"
                self.next_token();

                let rhs = self.parse_expression(1)?;
                let Token { span, .. } = self.expect_token(TokenKind::Semicolon)?;
                Ok(Statement::Return {
                    span: token_span.merge(span),
                    expr: Some(rhs),
                })
            }
            TokenKind::For => {
                // Skip the peeked "for"
                let Token { span, .. } = self.next_token();
                let (cond, lbrace) = match self.next_token() {
                    t if t.kind == TokenKind::LBrace => (None, t),
                    t if t.kind == TokenKind::LParen => {
                        let cond = self.parse_expression(0)?;
                        self.expect_token(TokenKind::RParen)?;
                        let lbrace = self.expect_token(TokenKind::LBrace)?;

                        (Some(cond), lbrace)
                    }
                    t => {
                        return Err(Diagnostic::error(
                            span,
                            format!("Expected a '(' or '{{', found '{}'", t.kind),
                        )
                        .with_error_label(span, "here"));
                    }
                };
                let stmts = self.parse_statements()?;
                let rbrace = self.expect_token(TokenKind::RBrace)?;
                self.eat_if_token(TokenKind::Semicolon);

                Ok(Statement::ForLoop {
                    span: span.merge(rbrace.span),
                    cond,
                    block: Block {
                        span: lbrace.span.merge(rbrace.span),
                        statements: stmts,
                    },
                })
            }
            TokenKind::If => {
                // Skip the peeked "if"
                self.next_token();
                self.expect_token(TokenKind::LParen)?;
                let cond = self.parse_expression(0)?;
                self.expect_token(TokenKind::RParen)?;
                let then_lbrace = self.expect_token(TokenKind::LBrace)?;
                let then = self.parse_statements()?;
                let then_rbrace = self.expect_token(TokenKind::RBrace)?;
                let mut else_if_blocks = Vec::new();
                let mut else_block = None;
                while let Token {
                    kind: TokenKind::Else,
                    ..
                } = self.peek_token()
                {
                    // Skip the peeked 'else'
                    self.next_token();

                    if let Token {
                        kind: TokenKind::If,
                        ..
                    } = self.peek_token()
                    {
                        // Skip the peeked 'if'
                        self.next_token();
                        self.expect_token(TokenKind::LParen)?;
                        let cond = self.parse_expression(0)?;
                        self.expect_token(TokenKind::RParen)?;
                        let then_lbrace = self.expect_token(TokenKind::LBrace)?;
                        let then = self.parse_statements()?;
                        let then_rbrace = self.expect_token(TokenKind::RBrace)?;
                        else_if_blocks.push((
                            cond,
                            Block {
                                span: then_lbrace.span.merge(then_rbrace.span),
                                statements: then,
                            },
                        ));
                    } else {
                        let then_lbrace = self.expect_token(TokenKind::LBrace)?;
                        let then = self.parse_statements()?;
                        let then_rbrace = self.expect_token(TokenKind::RBrace)?;
                        else_block = Some(Block {
                            span: then_lbrace.span.merge(then_rbrace.span),
                            statements: then,
                        });
                        break;
                    }
                }

                self.expect_token(TokenKind::Semicolon)?;

                Ok(Statement::Expr(Expr {
                    span: token_span,
                    kind: ExprKind::If {
                        cond: Box::new(cond),
                        then: Block {
                            span: then_lbrace.span.merge(then_rbrace.span),
                            statements: then,
                        },
                        else_if_blocks,
                        else_block,
                    },
                }))
            }
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
                        let _ = self.lexer.next().unwrap();
                        // Skip the peeked '='
                        let _ = self.lexer.next().unwrap();
                        let expr = self.parse_expression(0)?;
                        self.expect_token(TokenKind::Semicolon)?;

                        return Ok(Statement::Assign {
                            name: Ident {
                                inner: ident,
                                span: token_span,
                            },
                            span: token_span.merge(expr.span),
                            expr,
                        });
                    }
                    _ => {
                        let expr = self.parse_expression(0)?;
                        self.expect_token(TokenKind::Semicolon)?;
                        return Ok(Statement::Expr(expr));
                    }
                };
                // Skip the peeked ident
                let _ = self.lexer.next().unwrap();
                // Skip the peeked op
                let _ = self.lexer.next().unwrap();
                let expr = self.parse_expression(0)?;
                let name = Ident {
                    inner: ident,
                    span: token_span,
                };
                let expr = Expr {
                    span: name.span.merge(expr.span),
                    kind: ExprKind::Binary {
                        op,
                        left: Box::new(Expr {
                            kind: ExprKind::Ident(name),
                            span: name.span,
                        }),
                        right: Box::new(expr),
                    },
                };
                self.expect_token(TokenKind::Semicolon)?;
                Ok(Statement::Assign {
                    name,
                    span: token_span.merge(expr.span),
                    expr,
                })
            }
            TokenKind::Break => {
                let _ = self.lexer.next();
                self.expect_token(TokenKind::Semicolon)?;
                Ok(Statement::Break { span: token_span })
            }
            TokenKind::Continue => {
                let _ = self.lexer.next();
                self.expect_token(TokenKind::Semicolon)?;
                Ok(Statement::Continue { span: token_span })
            }
            _ => {
                let e = self.parse_expression(0)?;
                self.expect_token(TokenKind::Semicolon)?;
                Ok(Statement::Expr(e))
                //     Err(Diagnostic::error(
                //     statement_span,
                //     format!("Unexpected token {:?}. Expected statement start.", t),
                // )
                // .with_error_label(statement_span, "here"))},
            }
        }
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement<'src>>, Diagnostic<'src>> {
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

            let s = self.parse_statement()?;
            statements.push(s);
        }

        Ok(statements)
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
            t => Err(Diagnostic::error(
                expr_span,
                format!("Unexpected token {}", t),
            )),
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
            t => {
                return Err(
                    Diagnostic::error(expr_span, format!("Unexpected token '{}'", t))
                        .with_error_label(expr_span, format!("unexpected token '{}'", t)),
                );
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
                t => {
                    return Err(Diagnostic::error(
                        op_token.span,
                        format!("expected an infix operator, got {}", t),
                    )
                    .with_error_label(op_token.span, "here"));
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
                                return Err(Diagnostic::error(expr_span, "Expected field name"));
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
                                left: Box::new(lhs),
                                right: Box::new(rhs),
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
