use crate::ast::{
    BinaryOp, Block, Declaration, Expr, ExprKind, Function, Ident, Program, Statement, Type,
    TypeKind, Use, Vis,
};
use chumsky::{
    IterParser, ParseResult, Parser,
    error::{EmptyErr, Rich},
    extra,
    input::{Stream, ValueInput},
    prelude::*,
    select,
    span::SimpleSpan,
};
use error::{Diagnostic, emit_diagnostics};
use lexer::{Lexer, Token, TokenKind};

pub fn parse<'src>(lexer: Lexer<'src>) -> ParseResult<Statement<'src>, EmptyErr> {
    let parser = program_parser();
    let mut errors: Vec<Diagnostic<'src>> = Vec::new();

    let token_stream = Stream::from_iter(lexer.filter_map(move |t| match t {
        Ok(t) => Some(t),
        Err(e) => {
            errors.push((&e).into());
            None
        }
    }));
    // TODO: emit lexer errors

    let (program, errors) = parser.parse(token_stream).into_output_errors();
    let errors = errors.iter().map(|e| {
        let t = e.found().unwrap();
        Diagnostic::error(t.span, e.to_string()).with_error_label(t.span, e.to_string())
    });
    emit_diagnostics(errors);

    todo!()
}

fn parse_token_kind<'src, I>(
    kind: TokenKind<'src>,
) -> impl Parser<'src, I, Token<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token { kind: k, span } if kind == k => Token { kind: k, span }
    }
}

fn parse_ident<'src, I>()
-> impl Parser<'src, I, Ident<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token { kind: TokenKind::Ident(s), span } => Ident { inner: s, span }
    }
}

fn parse_use<'src, I>()
-> impl Parser<'src, I, Use<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    parse_token_kind(TokenKind::Use)
        .ignore_then(
            parse_ident()
                .separated_by(parse_token_kind(TokenKind::Dot))
                .collect::<Vec<_>>(),
        )
        .then_ignore(parse_token_kind(TokenKind::Semicolon))
        .map(|import| Use {
            span: import
                .first()
                .unwrap()
                .span
                .merge(import.last().unwrap().span),
            import,
        })
}

fn parse_vis<'src, I>() -> impl Parser<'src, I, Vis, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    choice((
        parse_token_kind(TokenKind::Public).map(|_| Vis::Public),
        parse_token_kind(TokenKind::Private).map(|_| Vis::Private),
        parse_token_kind(TokenKind::Internal).map(|_| Vis::Internal),
    ))
}

fn parse_type<'src, I>()
-> impl Parser<'src, I, Type<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    choice((
        parse_token_kind(TokenKind::Int).map(|t| Type {
            span: t.span,
            kind: TypeKind::Int,
        }),
        parse_token_kind(TokenKind::Float).map(|t| Type {
            span: t.span,
            kind: TypeKind::Float,
        }),
        parse_token_kind(TokenKind::String).map(|t| Type {
            span: t.span,
            kind: TypeKind::String,
        }),
        parse_token_kind(TokenKind::Bool).map(|t| Type {
            span: t.span,
            kind: TypeKind::Bool,
        }),
        parse_token_kind(TokenKind::Void).map(|t| Type {
            span: t.span,
            kind: TypeKind::Void,
        }),
    ))
}

fn parse_binary_op<'src, I>()
-> impl Parser<'src, I, BinaryOp, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    choice((
        parse_token_kind(TokenKind::Eq).to(BinaryOp::Eq),
        parse_token_kind(TokenKind::Neq).to(BinaryOp::Ne),
    ))
}

fn parse_statement<'src, I>()
-> impl Parser<'src, I, Statement<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    let mut stmt = Recursive::declare();
    let mut expr = Recursive::declare();

    let block_parser = stmt
        .clone()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(
            parse_token_kind(TokenKind::LBrace),
            parse_token_kind(TokenKind::RBrace),
        )
        .map(|statements: Vec<Statement<'src>>| Block {
            span: statements
                .first()
                .unwrap()
                .span()
                .merge(*statements.last().unwrap().span()),
            statements,
        });

    expr.define({
            let ident_parser = parse_ident().map(|i| Expr { kind: ExprKind::Ident(i), span: i.span });

            let binary_parser = expr.clone().then(parse_binary_op()).then(expr.clone()).map(|((left, op), right): ((Expr<'src>, BinaryOp), Expr<'src>)| {
                Expr {
                    span: left.span.merge(right.span),
                    kind: ExprKind::Binary { op, left: Box::new(left), right: Box::new(right) },
                }
            });

            let int_literal_parser = select! {
                Token { kind: TokenKind::IntLiteral(i), span } => Expr {span, kind: ExprKind::IntLiteral(i)}
            };



            let if_parser = parse_token_kind(TokenKind::If)
                .then(expr.clone())
                .then(block_parser)
                .map(|((if_keyword, expr), block)| Expr {
                    span: if_keyword.span.merge(block.span),
                    kind: ExprKind::If {
                        cond: Box::new(expr),
                        then: block,
                        else_if_blocks: Vec::new(),
                        else_block: None,
                    },
                });

            choice((binary_parser, if_parser, int_literal_parser, ident_parser)).boxed()
        });

    stmt.define({
        let let_parser = parse_token_kind(TokenKind::Let)
            .then(parse_token_kind(TokenKind::Mut).or_not())
            .then(parse_ident())
            .then(
                parse_token_kind(TokenKind::Colon)
                    .ignore_then(parse_type())
                    .or_not(),
            )
            .then_ignore(parse_token_kind(TokenKind::Eq))
            .then(expr.clone())
            .then_ignore(parse_token_kind(TokenKind::Semicolon))
            .map(
                |((((let_keyword, maybe_mut), name), ty), expr)| Statement::Let {
                    span: let_keyword.span.merge(expr.span),
                    name,
                    ty,
                    expr,
                    mutable: maybe_mut.is_some(),
                },
            );

        choice((let_parser, expr.map(Statement::Expr))).boxed()
    });

    stmt
}

fn parse_block<'src, I>()
-> impl Parser<'src, I, Block<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    parse_statement()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(
            parse_token_kind(TokenKind::LBrace),
            parse_token_kind(TokenKind::RBrace),
        )
        .map(|statements| Block {
            span: statements
                .first()
                .unwrap()
                .span()
                .merge(*statements.last().unwrap().span()),
            statements,
        })
}

fn parse_fn<'src, I>()
-> impl Parser<'src, I, Function<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    parse_vis()
        .or_not()
        .then(parse_token_kind(TokenKind::Fn))
        .then(parse_ident())
        .then_ignore(parse_token_kind(TokenKind::LParen))
        .then_ignore(parse_token_kind(TokenKind::RParen))
        .then(parse_type())
        .then(parse_block())
        .then_ignore(parse_token_kind(TokenKind::Semicolon))
        .map(|((((vis, fn_keyword), name), ty), block)| Function {
            span: fn_keyword.span.merge(name.span),
            name,
            params: Vec::new(),
            body: block,
            ret_ty: ty,
            vis: vis.unwrap_or(Vis::Private),
        })
}

fn program_parser<'src, I>()
-> impl Parser<'src, I, Program<'src>, extra::Err<Rich<'src, Token<'src>, SimpleSpan>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    let use_parser = parse_use().map(Declaration::Use);
    let fn_parser = parse_fn().map(Declaration::Function);

    choice((use_parser, fn_parser))
        .repeated()
        .collect::<Vec<_>>()
        .map(|declarations| Program { declarations })
}

#[cfg(test)]
mod test {
    use super::*;
    use lexer::tokenize;
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn test_chumsky() {
        let path = PathBuf::from_str("../../../examples/simple/src/main.fo").unwrap();
        let source = include_str!("../../../examples/simple/src/main.fo");

        let lexer = tokenize(source, &path);
        let res = parse(lexer);
        dbg!(res);
    }
}
