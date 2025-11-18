use super::ast::{
    Block, Call, Declaration, Enum, EnumVariant, Expr, Function, Ident, Op, Param, Program,
    Statement, Struct, Type, TypeKind,
};
use error::Diagnostic;
use lexer::{Lexer, Token};
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
    fn parse(&mut self) -> Program<'source> {
        let mut declarations = Vec::new();

        loop {
            if matches!(
                self.lexer.peek(),
                None | Some((Token::RBrace | Token::RParen, _))
            ) {
                self.lexer.next();
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

    fn expect_token(&mut self, token: Token) -> Result<(Token<'source>, Span), Diagnostic> {
        match self.lexer.next() {
            Some((t, span)) if token == t => Ok((t, span)),
            Some((t, span)) => Err(Diagnostic::error(
                span,
                format!("Syntax error: Expected '{}', found '{}'", token, t),
            )),
            None => todo!("Need an EOF span. Expected '{}'", token),
        }
    }

    fn expect_one_of_token(
        &mut self,
        tokens: &[Token],
    ) -> Result<(Token<'source>, Span), Diagnostic> {
        match self.lexer.next() {
            Some((t, span)) if tokens.contains(&t) => Ok((t, span)),
            Some((t, span)) => Err(Diagnostic::error(
                span,
                format!(
                    "Syntax error: Expected one of {}, found '{}'",
                    tokens
                        .iter()
                        .map(|t| format!("'{}'", t))
                        .collect::<Vec<_>>()
                        .join(", "),
                    t
                ),
            )),
            None => todo!("Need an EOF span"),
        }
    }

    fn parse_type(&mut self) -> Result<Type<'source>, Diagnostic> {
        match self.lexer.next() {
            Some((Token::Ident(i), span)) => Ok(Type {
                span,
                kind: TypeKind::Ident(Ident { inner: i, span }),
            }),
            Some((Token::Int, span)) => Ok(Type {
                kind: TypeKind::Int,
                span,
            }),
            Some((Token::Float, span)) => Ok(Type {
                kind: TypeKind::Float,
                span,
            }),
            Some((Token::Bool, span)) => Ok(Type {
                kind: TypeKind::Bool,
                span,
            }),
            Some((Token::String, span)) => Ok(Type {
                kind: TypeKind::String,
                span,
            }),
            Some((Token::Void, span)) => Ok(Type {
                kind: TypeKind::Void,
                span,
            }),
            Some((_, span)) => Err(Diagnostic::error(span, "expected type")),
            None => Err(Diagnostic::error(Span::new(0, 0), "expected type")),
        }
    }

    fn parse_maybe_pub(&mut self) -> bool {
        match self.lexer.peek() {
            Some((Token::Pub, _)) => {
                self.lexer.next();
                true
            }
            _ => false,
        }
    }

    fn parse_ident(&mut self) -> Result<Ident<'source>, Diagnostic> {
        match self.lexer.next() {
            Some((Token::Ident(name), span)) => Ok(Ident { inner: name, span }),
            Some((_, span)) => Err(Diagnostic::error(span, "expected identifier")),
            None => {
                // TODO: The tokens should know the last span
                todo!()
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration<'source>, Diagnostic> {
        let is_pub = self.parse_maybe_pub();

        let Some((token, span)) = self.lexer.peek() else {
            todo!("Need an EOF span")
        };

        match token {
            Token::Struct => {
                // Skip the peeked "struct"
                self.lexer.next();

                // Check if there is a struct name
                let name = self.parse_ident().map_err(|e| {
                    e.with_help("Expected struct name")
                        .with_suggestion("try giving the struct a name: `struct MyStruct {{}}`")
                })?;

                self.expect_token(Token::LBrace).map_err(|e| {
                    e.with_suggestion(format!(
                        "try giving the struct a body: `struct {} {{}}`",
                        name.inner,
                    ))
                })?;

                let mut struct_span = span;

                let mut fields: Vec<(Ident<'source>, Type<'source>)> = Vec::new();
                loop {
                    if matches!(self.lexer.peek(), Some((Token::RBrace, _))) {
                        // struct end
                        self.lexer.next();
                        break;
                    }

                    let param_name = self.parse_ident().map_err(|e| {
                        e.with_help("Expected struct field name")
                            .with_suggestion("try giving the field a name: `my_field: ...`")
                    })?;
                    self.expect_token(Token::Colon)?;

                    let ty = self.parse_type()?;
                    fields.push((param_name, ty));

                    self.expect_token(Token::Comma)?;
                    if let Some((Token::RBrace, span)) = self.lexer.peek() {
                        struct_span = struct_span.merge(span);
                        self.lexer.next();
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
            Token::Enum => {
                // Skip the peeked "enum"
                self.lexer.next();

                // Check if there is an enum name
                let name = self.parse_ident().map_err(|e| {
                    e.with_help("Expected enum name")
                        .with_suggestion("try giving the enum a name: `enum MyEnum {{}}`")
                })?;

                self.expect_token(Token::LBrace).map_err(|e| {
                    e.with_suggestion(format!(
                        "try giving the enum a body: `enum {} {{}}`",
                        name.inner,
                    ))
                })?;

                let mut enum_span = span;

                let mut variants: Vec<EnumVariant> = Vec::new();
                loop {
                    if matches!(self.lexer.peek(), Some((Token::RBrace, _))) {
                        // enum end
                        self.lexer.next();
                        break;
                    }

                    let variant_name = self.parse_ident().map_err(|e| {
                        e.with_help("Expected enum variant name")
                            .with_suggestion("try giving the variant a name: `MyVariant ...`")
                    })?;
                    self.expect_token(Token::Comma)?;

                    variants.push(EnumVariant {
                        span: variant_name.span,
                        name: variant_name,
                        fields: Vec::new(),
                    });

                    if let Some((Token::RBrace, span)) = self.lexer.peek() {
                        enum_span = enum_span.merge(span);
                        self.lexer.next();
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
            Token::Fn => Ok(Declaration::Function(
                self.parse_function()?.set_public(is_pub),
            )),
            Token::Let => Err(Diagnostic::error(span, "Unexpected let binding")),
            Token::Return => Err(Diagnostic::error(span, "Unexpected return statement")),
            x => Err(Diagnostic::error(span, format!("Unexpected '{}'", x))),
        }
    }

    fn parse_function(&mut self) -> Result<Function<'source>, Diagnostic> {
        // Skip the peeked "fn"
        let (_, span) = self.lexer.next().unwrap();
        let mut function_span = span;

        // TODO: Handle anon functions
        let name = self.parse_ident().map_err(|e| {
            e.with_help("Expected function name")
                .with_suggestion("try giving the function a name: `fn my_function() ...`")
        })?;

        let params = self.parse_function_parameters()?;

        let ret_ty = self
            .parse_type()
            .map_err(|e| e.with_suggestion("expected function return type"))?;

        let (_, lbrace_span) = self.expect_token(Token::LBrace)?;

        let statements = self.parse_statements()?;

        let (_, rbrace_span) = self.expect_token(Token::RBrace)?;
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
        self.expect_token(Token::LParen)?;
        let mut parameters = Vec::new();

        if matches!(self.lexer.peek(), Some((Token::RParen, _))) {
            // immediate parameter list end
            self.lexer.next();
            return Ok(parameters);
        }

        loop {
            let param_name = self
                .parse_ident()
                .map_err(|e| e.with_suggestion("expected parameter name"))?;
            let param_span = param_name.span;

            self.expect_token(Token::Colon)?;

            let ty = self.parse_type()?;

            parameters.push(Param {
                span: param_span.merge(ty.span),
                name: param_name,
                ty,
                default: None,
            });

            let (token, _) = self
                .expect_one_of_token(&[Token::RParen, Token::Comma])
                .map_err(|e| {
                    e.with_suggestion("Expected new parameter or a ) to end the parameter list")
                })?;

            if token == Token::RParen {
                break;
            }
        }

        Ok(parameters)
    }

    fn parse_statement(&mut self) -> Result<Statement<'source>, Diagnostic> {
        let Some((token, span)) = self.lexer.peek() else {
            todo!("Need an EOF span")
        };
        let statement_span = span;

        match token {
            Token::Let => {
                // Skip the peeked 'let'
                self.lexer.next();

                // Check if there is a mut
                let mutable = if let Some((Token::Mut, _)) = self.lexer.peek() {
                    self.lexer.next();
                    true
                } else {
                    false
                };

                let ident = self.parse_ident()?;

                // Try and get type
                let ty = if let Some((Token::Colon, _)) = self.lexer.peek() {
                    self.lexer.next();
                    let ty = self.parse_type()?;
                    Some(ty)
                } else {
                    None
                };

                self.expect_token(Token::Equals)?;

                let expression = self.parse_expression(0)?;

                let (_, span) = self.expect_token(Token::Semicolon)?;

                Ok(Statement::Let {
                    span: statement_span.merge(span),
                    name: ident,
                    mutable,
                    ty,
                    expr: expression,
                })
            }
            Token::Return => {
                // Skip the peeked "return"
                self.lexer.next();

                let rhs = self.parse_expression(1)?;
                let (_, span) = self.expect_token(Token::Semicolon)?;
                Ok(Statement::Return {
                    span: statement_span.merge(span),
                    expr: Some(rhs),
                })
            }
            Token::Ident(_) => {
                let expr = self.parse_expression(0)?;
                self.expect_token(Token::Semicolon)?;
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
            if matches!(self.lexer.peek(), None | Some((Token::RBrace, _))) {
                break;
            };

            let s = self.parse_statement()?;
            statements.push(s);
        }

        Ok(statements)
    }

    fn parse_string_literal_or_interpolation(
        &mut self,
        parts: Vec<(Token<'source>, Span)>,
    ) -> Result<Expr<'source>, Diagnostic> {
        if parts.is_empty() {
            return Ok(Expr::StringLiteral(None));
        }
        let mut string_interpolation_parser = Parser {
            lexer: Lexer::new(parts),
            errors: vec![],
        };

        let mut exprs = Vec::new();
        while string_interpolation_parser.lexer.peek().is_some() {
            let e = string_interpolation_parser.parse_string_interpolation_expr(0)?;
            exprs.push(e);
        }

        if exprs.len() == 1 {
            let first = exprs.first().expect("Must be there by the if check");
            if let Expr::StringLiteral(_) = first {
                return Ok(first.clone());
            }
        }

        Ok(Expr::StringInterpolation(exprs))
    }

    fn parse_string_interpolation_expr(&mut self, min_bp: u8) -> Result<Expr<'source>, Diagnostic> {
        let Some((token, span)) = self.lexer.next() else {
            todo!("Need an EOF span")
        };
        let expr_span = span;

        match token {
            Token::LBrace => {
                let expr = self.parse_expression(min_bp)?;
                self.lexer.next();
                Ok(expr)
            }
            Token::StringFragment(s) => Ok(Expr::StringLiteral(Some(s))),
            t => Err(Diagnostic::error(
                expr_span,
                format!("Unexpected token {}", t),
            )),
        }
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Expr<'source>, Diagnostic> {
        let Some((token, span)) = self.lexer.next() else {
            todo!("Need an EOF span")
        };
        let expr_span = span;

        // Create the initial lhs. Then we will, in a recursion like way update lhs to be the lhs with an op and a right side
        let mut lhs = match token {
            Token::IntLiteral(i) => Expr::IntLiteral(i),
            Token::FloatLiteral(f) => Expr::FloatLiteral(f),
            Token::BoolLiteral(b) => Expr::BoolLiteral(b),
            Token::Ident(ident) => Expr::Ident(Ident {
                inner: ident,
                span: expr_span,
            }),
            Token::StringLiteralOrInterpolation(s) => {
                self.parse_string_literal_or_interpolation(s)?
            }
            Token::StringFragment(s) => Expr::StringLiteral(Some(s)),
            t => {
                return Err(Diagnostic::error(
                    expr_span,
                    format!("Unexpected token {}", t),
                ));
            }
        };

        loop {
            let Some((op_token, span)) = self.lexer.peek() else {
                break;
            };

            let op = match op_token {
                Token::Semicolon | Token::RBrace | Token::Comma | Token::RParen => break,
                Token::Plus => Op::Add,
                Token::Minus => Op::Sub,
                Token::Star => Op::Mul,
                Token::Slash => Op::Div,
                Token::StarStar => Op::Exp,
                Token::Percent => Op::Mod,
                Token::And => Op::And,
                Token::Or => Op::Or,
                Token::Lt => Op::Lt,
                Token::LtEq => Op::LtEq,
                Token::Gt => Op::Gt,
                Token::GtEq => Op::GtEq,
                Token::LParen => Op::Call,
                Token::LBrace => Op::StructInstance,
                Token::Dot => Op::FieldIndex,
                _ => {
                    return Err(Diagnostic::error(span, "expected an infix operator"));
                }
            };

            if let Some((l_bp, ())) = op.postfix_binding_power() {
                if l_bp < min_bp {
                    break;
                }
                self.lexer.next();

                lhs = match op {
                    Op::Call => Expr::Call(Call {
                        callee: Box::new(lhs),
                        args: self.parse_fn_call_arguments()?,
                    }),
                    Op::StructInstance => {
                        let struct_name = self.parse_ident()?;
                        let struct_fields = self.parse_struct_fields()?;
                        Expr::StructInstance {
                            name: struct_name,
                            fields: struct_fields,
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
                self.lexer.next();

                lhs = match op {
                    Op::FieldIndex => {
                        let rhs = match self.parse_expression(r_bp)? {
                            Expr::Ident(ident) => ident,
                            _ => {
                                return Err(Diagnostic::error(expr_span, "Expected field name"));
                            }
                        };
                        Expr::Member {
                            object: Box::new(lhs),
                            field: rhs,
                        }
                    }
                    _ => {
                        let rhs = self.parse_expression(r_bp)?;
                        Expr::Binary {
                            op: op.try_into()?,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
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

        if matches!(self.lexer.peek(), Some((Token::RParen, _))) {
            // immediate argument list end
            self.lexer.next();
        } else {
            loop {
                // let param_name = self.parse_ident()?;
                // self.expect_token(Token::Equals)?;

                let value = self.parse_expression(0)?;
                arguments.push(value);

                let (token, _) = self.expect_one_of_token(&[Token::RParen, Token::Comma])?;

                if token == Token::RParen {
                    break;
                }
            }
        }

        Ok(arguments)
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(Ident<'source>, Expr<'source>)>, Diagnostic> {
        let mut fields = Vec::new();

        // parent has already eaten left brace as the operator

        if matches!(self.lexer.peek(), Some((Token::RBrace, _))) {
            // immediate argument list end
            self.lexer.next();
        } else {
            loop {
                if matches!(self.lexer.peek(), Some((Token::RBrace, _))) {
                    // struct end
                    self.lexer.next();
                    break;
                }

                let field_name = self.parse_ident()?;

                self.expect_token(Token::Equals)?;

                let value = self.parse_expression(0)?;

                fields.push((field_name, value));

                let (token, _) = self.expect_one_of_token(&[Token::RBrace, Token::Comma])?;

                if token == Token::RBrace {
                    break;
                }
            }
        }

        Ok(fields)
    }
}
