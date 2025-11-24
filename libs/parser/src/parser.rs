use super::ast::{
    Block, Call, Declaration, Enum, EnumVariant, ExprKind, Function, Ident, Op, Param, Program,
    Statement, Struct, Type, TypeKind,
};
use crate::ast::Expr;
use error::Diagnostic;
use lexer::{Lexer, Token, TokenKind};
use span::Span;

pub fn parse<'source>(lexer: Lexer<'source>) -> (Program<'source>, Vec<Diagnostic>) {
    let mut parser = Parser {
        lexer,
        errors: vec![],
    };

    let program = parser.parse();

    (program, parser.errors)
}

struct Parser<'source> {
    lexer: Lexer<'source>,
    errors: Vec<Diagnostic>,
}

impl<'source> Parser<'source> {
    fn next_token(&mut self) -> Option<Token<'source>> {
        match self.lexer.next() {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => {
                self.errors.push((&e).into());
                self.next_token()
            }
            None => None,
        }
    }

    fn peek_token(&mut self) -> Option<&Token<'source>> {
        while let Some(Err(e)) = self.lexer.peek() {
            self.errors.push(e.into());
            self.next_token();
        }
        match self.lexer.peek() {
            Some(Ok(t)) => Some(t),
            None => None,
            _ => unreachable!(),
        }
    }

    fn parse(&mut self) -> Program<'source> {
        let mut declarations = Vec::new();

        loop {
            if matches!(
                self.peek_token().map(|t| t.kind),
                None | Some(TokenKind::RBrace | TokenKind::RParen | TokenKind::Eof)
            ) {
                self.next_token();
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

    fn expect_token(&mut self, token: TokenKind<'source>) -> Result<Token<'source>, Diagnostic> {
        match self.next_token() {
            Some(t) if token == t.kind => Ok(t),
            Some(t) => {
                // panic!("HHH");
                Err(Diagnostic::error(
                    t.span,
                    format!("Syntax error: Expected '{}', found '{}'", token, t.kind),
                )
                .with_error_label(t.span, "here"))
            }
            None => todo!("Need an EOF span. Expected '{}'", token),
        }
    }

    fn expect_one_of_token(
        &mut self,
        tokens: &[TokenKind<'source>],
    ) -> Result<Token<'source>, Diagnostic> {
        match self.next_token() {
            Some(t) if tokens.contains(&t.kind) => Ok(t),
            Some(t) => Err(Diagnostic::error(
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
            None => todo!("Need an EOF span"),
        }
    }

    fn parse_type(&mut self) -> Result<Type<'source>, Diagnostic> {
        match self.next_token().expect("Unexpected EOF") {
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
            Token { kind: _, span } => Err(Diagnostic::error(span, "expected type")),
        }
    }

    fn parse_maybe_vis(&mut self) -> bool {
        match self.peek_token() {
            Some(Token {
                kind: TokenKind::Public,
                ..
            }) => {
                self.lexer.next();
                true
            }
            Some(Token {
                kind: TokenKind::Internal,
                ..
            }) => {
                // TODO: change this
                self.lexer.next();
                true
            }
            _ => false,
        }
    }

    fn parse_ident(&mut self) -> Result<Ident<'source>, Diagnostic> {
        match self.next_token().expect("Unexpected EOF") {
            Token {
                kind: TokenKind::Ident(name),
                span,
            } => Ok(Ident { inner: name, span }),
            Token {
                kind: TokenKind::Eof,
                span,
            } => Err(Diagnostic::error(span, "Unexpected EOF")),
            Token { kind: _, span } => Err(Diagnostic::error(span, "expected identifier")),
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration<'source>, Diagnostic> {
        let is_pub = self.parse_maybe_vis();

        let t = self.peek_token().expect("Unexpected EOF");
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

                let mut fields: Vec<(Ident<'source>, Type<'source>)> = Vec::new();
                loop {
                    if matches!(
                        self.peek_token(),
                        Some(Token {
                            kind: TokenKind::RBrace,
                            ..
                        })
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
                    if let Some(Token {
                        kind: TokenKind::RBrace,
                        span,
                    }) = self.peek_token()
                    {
                        struct_span = struct_span.merge(*span);
                        self.next_token();
                        if let Some(Token {
                            kind: TokenKind::Semicolon,
                            ..
                        }) = self.peek_token()
                        {
                            self.lexer.next();
                        }
                        break;
                    }
                }

                Ok(Declaration::Struct(Struct {
                    span: struct_span,
                    name,
                    fields,
                    public: is_pub,
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
                        Some(Token {
                            kind: TokenKind::RBrace,
                            ..
                        })
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

                    if let Some(Token {
                        kind: TokenKind::RBrace,
                        span,
                    }) = self.peek_token()
                    {
                        enum_span = enum_span.merge(*span);
                        // Eat '}'
                        self.next_token();
                        if let Some(Token {
                            kind: TokenKind::Semicolon,
                            ..
                        }) = self.peek_token()
                        {
                            self.lexer.next();
                        }
                        self.next_token();
                        break;
                    }
                }

                Ok(Declaration::Enum(Enum {
                    variants,
                    span: enum_span,
                    name,
                    public: is_pub,
                }))
            }
            TokenKind::Fn => Ok(Declaration::Function(
                self.parse_function()?.set_public(is_pub),
            )),
            TokenKind::Use => {
                // Skip the peeked "use"
                self.next_token();

                let namespace = self.parse_ident()?;
                let mut import = vec![namespace];

                // repeatedly parse `.submodule`
                loop {
                    match self.peek_token() {
                        None => todo!("Need an EOF"),
                        Some(Token {
                            kind: TokenKind::Semicolon,
                            ..
                        }) => {
                            self.next_token();
                            break;
                        }
                        Some(Token {
                            kind: TokenKind::Dot,
                            ..
                        }) => {
                            self.next_token();
                            let submodule = self.parse_ident()?;
                            import.push(submodule);
                            continue;
                        }
                        Some(Token { kind: _, span }) => {
                            return Err(Diagnostic::error(*span, "Unexpected token")
                                .with_error_label(*span, "here"));
                        }
                    }
                }

                Ok(Declaration::Use { import })
            }
            TokenKind::Let => Err(Diagnostic::error(span, "Unexpected let binding")),
            TokenKind::Return => Err(Diagnostic::error(span, "Unexpected return statement")),
            x => Err(Diagnostic::error(span, format!("Unexpected '{}'", x))
                .with_error_label(span, "here")),
        }
    }

    fn parse_function(&mut self) -> Result<Function<'source>, Diagnostic> {
        // Skip the peeked "fn"
        let t = self.next_token().unwrap();
        let mut function_span = t.span;

        // TODO: Handle anon functions
        let name = self.parse_ident().map_err(|e| {
            e.with_help("Expected function name")
                .with_note("try giving the function a name: `fn my_function() ...`")
        })?;

        let params = self.parse_function_parameters()?;

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
        if let Some(Token {
            kind: TokenKind::Semicolon,
            ..
        }) = self.peek_token()
        {
            self.lexer.next();
        }
        let block = Block {
            statements,
            span: lbrace_span.merge(rbrace_span),
        };
        function_span = function_span.merge(rbrace_span);

        Ok(Function {
            span: function_span,
            name,
            ret_ty,
            params,
            body: block,
            public: false,
        })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Param<'source>>, Diagnostic> {
        self.expect_token(TokenKind::LParen)?;
        let mut parameters = Vec::new();

        if matches!(
            self.peek_token(),
            Some(Token {
                kind: TokenKind::RParen,
                ..
            })
        ) {
            // immediate parameter list end
            self.next_token();
            return Ok(parameters);
        }

        loop {
            let param_name = self
                .parse_ident()
                .map_err(|e| e.with_note("expected parameter name"))?;
            let param_span = param_name.span;

            self.expect_token(TokenKind::Colon)?;

            let ty = self.parse_type()?;

            parameters.push(Param {
                span: param_span.merge(ty.span),
                name: param_name,
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

    fn parse_statement(&mut self) -> Result<Statement<'source>, Diagnostic> {
        let Some(token) = self.peek_token() else {
            todo!("Need an EOF span")
        };
        let statement_span = token.span;

        match token.kind {
            TokenKind::Let => {
                // Skip the peeked 'let'
                self.next_token();

                // Check if there is a mut
                let mutable = if let Some(Token {
                    kind: TokenKind::Mut,
                    ..
                }) = self.peek_token()
                {
                    self.next_token();
                    true
                } else {
                    false
                };

                let ident = self.parse_ident()?;

                // Try and get type
                let ty = if let Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) = self.peek_token()
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
                    span: statement_span.merge(span),
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
                    span: statement_span.merge(span),
                    expr: Some(rhs),
                })
            }
            TokenKind::Ident(_) => {
                let expr = self.parse_expression(0)?;
                self.expect_token(TokenKind::Semicolon)?;
                Ok(Statement::Expr(expr))
            }
            t => Err(Diagnostic::error(
                statement_span,
                format!("Unexpected token {:?}. Expected statement start.", t),
            )),
        }
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement<'source>>, Diagnostic> {
        let mut statements = Vec::new();

        loop {
            if matches!(
                self.peek_token(),
                None | Some(Token {
                    kind: TokenKind::RBrace,
                    ..
                })
            ) {
                break;
            };

            let s = self.parse_statement()?;
            statements.push(s);
        }

        Ok(statements)
    }

    fn parse_string_literal_or_interpolation(&mut self) -> Result<Expr<'source>, Diagnostic> {
        let mut exprs = Vec::new();
        while !matches!(
            self.peek_token(),
            Some(Token {
                kind: TokenKind::StringInterpolationEnd,
                ..
            })
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
            kind: ExprKind::StringInterpolation(exprs),
            // TODO:
            span: Span::new(0, 0),
        })
    }

    fn parse_string_interpolation_expr(&mut self, min_bp: u8) -> Result<Expr<'source>, Diagnostic> {
        let Some(token) = self.next_token() else {
            todo!("Need an EOF span")
        };
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

    fn parse_expression(&mut self, min_bp: u8) -> Result<Expr<'source>, Diagnostic> {
        let Some(token) = self.next_token() else {
            todo!("Need an EOF span")
        };
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
            t => {
                return Err(
                    Diagnostic::error(expr_span, format!("Unexpected token '{}'", t))
                        .with_error_label(expr_span, format!("unexpected token '{}'", t)),
                );
            }
        };

        loop {
            let Some(op_token) = self.peek_token() else {
                break;
            };
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
                TokenKind::Lt => Op::Lt,
                TokenKind::LtEq => Op::LtEq,
                TokenKind::Gt => Op::Gt,
                TokenKind::GtEq => Op::GtEq,
                TokenKind::LParen => Op::Call,
                TokenKind::LBrace => Op::StructInstance,
                TokenKind::Dot => Op::FieldIndex,
                TokenKind::StringInterpolationEnd => {
                    self.lexer.next();
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
                                op: op.try_into()?,
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

    fn parse_fn_call_arguments(&mut self) -> Result<Vec<Expr<'source>>, Diagnostic> {
        let mut arguments = Vec::new();

        // parent has already eaten left paren as the operator

        if matches!(
            self.peek_token(),
            Some(Token {
                kind: TokenKind::RParen,
                ..
            })
        ) {
            // immediate argument list end
            self.next_token();
        } else {
            loop {
                // let param_name = self.parse_ident()?;
                // self.expect_token(TokenKind::Equals)?;

                let value = self.parse_expression(0)?;
                arguments.push(value);

                let token = self.expect_one_of_token(&[TokenKind::RParen, TokenKind::Comma])?;

                if token.kind == TokenKind::RParen {
                    break;
                }
            }
        }

        Ok(arguments)
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(Ident<'source>, Expr<'source>)>, Diagnostic> {
        let mut fields = Vec::new();

        // parent has already eaten left brace as the operator

        if matches!(
            self.peek_token(),
            Some(Token {
                kind: TokenKind::RBrace,
                ..
            })
        ) {
            // immediate argument list end
            self.next_token();
        } else {
            loop {
                if matches!(
                    self.peek_token(),
                    Some(Token {
                        kind: TokenKind::RBrace,
                        ..
                    })
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
