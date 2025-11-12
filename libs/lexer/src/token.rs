use crate::lexer_error::LexerError;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(LexerError<'s>, LexerError::from_lexer))]
#[logos(skip r"[ \r\t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token<'source> {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("in")]
    In,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("match")]
    Match,
    #[token("print")]
    Print,
    #[token("None")]
    None,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("throw")]
    Throw,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("mut")]
    Mut,

    // Types
    #[token("int")]
    Int,
    #[token("float")]
    Float,
    #[token("string")]
    String,
    #[token("bool")]
    Bool,
    #[token("void")]
    Void,

    // Identifiers
    #[regex("[a-zA-Z_][a-zA-Z_0-9]*")]
    Ident(&'source str),

    // Literals
    #[regex("[+-]?[0-9]+", |lex| lex.slice().parse::<i64>())]
    IntLiteral(i64),
    #[regex(r"[+-]?[0-9]+\.[0-9]*", |lex| lex.slice().parse::<f64>())]
    FloatLiteral(f64),
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    BoolLiteral(bool),

    StringFragment(&'source str),
    #[token("\"", string_literal_definition)]
    StringLiteral(Vec<Token<'source>>),

    // Structural
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("?")]
    QuestionMark,

    // Operators
    #[token("=")]
    Equals,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("**")]
    StarStar,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("!")]
    Bang,

    // Assignment operators
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,

    // Range operator
    #[token("..")]
    DoubleDot,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::For => write!(f, "for"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::In => write!(f, "in"),
            Token::Use => write!(f, "use"),
            Token::Pub => write!(f, "pub"),
            Token::Match => write!(f, "match"),
            Token::Print => write!(f, "print"),
            Token::None => write!(f, "none"),
            Token::Try => write!(f, "try"),
            Token::Catch => write!(f, "catch"),
            Token::Throw => write!(f, "throw"),
            Token::Struct => write!(f, "struct"),
            Token::Enum => write!(f, "enum"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Mut => write!(f, "mut"),
            Token::Int => write!(f, "int"),
            Token::Float => write!(f, "float"),
            Token::String => write!(f, "string"),
            Token::Bool => write!(f, "bool"),
            Token::Void => write!(f, "void"),
            Token::Ident(i) => write!(f, "{}", i),
            Token::IntLiteral(i) => write!(f, "{}", i),
            Token::FloatLiteral(fl) => write!(f, "{}", fl),
            Token::BoolLiteral(b) => write!(f, "{}", b),
            Token::StringFragment(s) => write!(f, "{}", s),
            Token::StringLiteral(tokens) => write!(
                f,
                "{}",
                tokens
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::QuestionMark => write!(f, "?"),
            Token::Equals => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::LtEq => write!(f, "<="),
            Token::GtEq => write!(f, ">="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::StarStar => write!(f, "**"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Bang => write!(f, "!"),
            Token::PlusEq => write!(f, "+="),
            Token::MinusEq => write!(f, "-="),
            Token::StarEq => write!(f, "*="),
            Token::SlashEq => write!(f, "/="),
            Token::DoubleDot => write!(f, ".."),
        }
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
enum StringContext<'source> {
    #[token("\"")]
    Quote,
    #[regex(r#"[^"{}]+"#)]
    Content,
    #[token("{", evaluate_interpolation)]
    InterpolationStart(Vec<Token<'source>>),
}

fn evaluate_interpolation<'source>(
    lex: &mut Lexer<'source, StringContext<'source>>,
) -> Vec<Token<'source>> {
    let mut token_lexer = lex.clone().morph::<Token>();
    let mut res = vec![];

    while let Some(Ok(token)) = token_lexer.next() {
        match token {
            Token::StringLiteral(mut tokens) => res.append(&mut tokens),
            Token::RBrace => {
                res.push(Token::RBrace);
                break;
            }
            token => res.push(token),
        }
    }

    lex.bump(token_lexer.span().end - lex.span().end);

    res
}

fn string_literal_definition<'source>(
    lex: &mut Lexer<'source, Token<'source>>,
) -> Vec<Token<'source>> {
    let mut string_lexer = lex.clone().morph::<StringContext<'source>>();

    let mut res = vec![];
    while let Some(Ok(token)) = string_lexer.next() {
        match token {
            StringContext::Quote => break,
            StringContext::Content => {
                let slice = string_lexer.slice();
                res.push(Token::StringFragment(slice))
            }
            StringContext::InterpolationStart(mut tokens) => {
                res.push(Token::LBrace);
                res.append(&mut tokens)
            }
        }
    }

    lex.bump(string_lexer.span().end - lex.span().end);

    res
}
