use std::num::{ParseFloatError, ParseIntError};

use error::Diagnostic;
use span::Span;

use crate::lexing::YYC_INIT;

pub enum LexerErrorKind<'src> {
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UnexpectedToken(&'src str),
    Other,
}

impl<'src> std::fmt::Display for LexerErrorKind<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorKind::ParseIntError(e) => write!(f, "ParseIntError({e:?})"),
            LexerErrorKind::ParseFloatError(e) => write!(f, "ParseFloatError({e:?})"),
            LexerErrorKind::UnexpectedToken(t) => write!(f, "UnexpectedToken({t})"),
            LexerErrorKind::Other => write!(f, "Other "),
        }
    }
}

pub struct LexerError<'src> {
    pub(crate) span: Span,
    pub(crate) kind: LexerErrorKind<'src>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind<'src> {
    // Keywords
    Fn,
    Let,
    Return,
    If,
    Else,
    For,
    Break,
    Continue,
    In,
    Use,
    Pub,
    Match,
    None,
    Try,
    Catch,
    Throw,
    Struct,
    Enum,
    And,
    Or,
    Mut,

    // Types
    Int,
    Float,
    String,
    Bool,
    Void,

    // Identifiers
    Ident(&'src str),

    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),

    StringInterpolationStart,
    StringInterpolationEnd,
    StringLiteral(&'src str),

    // Structural
    Colon,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    QuestionMark,

    // Operators
    Eq,
    EqEq,
    Neq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Plus,
    Minus,
    Star,
    StarStar,
    Slash,
    Percent,
    Bang,

    // Assignment operators
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,

    Eof,
}

pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span,
}

impl std::fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::None => write!(f, "none"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Throw => write!(f, "throw"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Void => write!(f, "void"),
            TokenKind::Ident(i) => write!(f, "Ident({})", i),
            TokenKind::IntLiteral(i) => write!(f, "IntLiteral({})", i),
            TokenKind::FloatLiteral(fl) => write!(f, "FloatLiteral({})", fl),
            TokenKind::BoolLiteral(b) => write!(f, "BoolLiteral({})", b),
            TokenKind::StringInterpolationStart => write!(f, "-> StringInterpolationStart"),
            TokenKind::StringInterpolationEnd => write!(f, "<- StringInterpolationEnd"),
            TokenKind::StringLiteral(s) => write!(f, "StringLiteral(\"{}\")", s),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::QuestionMark => write!(f, "?"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::Neq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::StarStar => write!(f, "**"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer<'src> {
    pub(crate) input: &'src str,
    pub(crate) token: usize,
    pub(crate) cursor: usize,
    pub(crate) marker: usize,
    pub(crate) cond: usize,
    pub(crate) interpolation_depth: usize,
    pub(crate) eof: bool,
    pub(crate) peeked: Option<Result<Token<'src>, Diagnostic>>,
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
            peeked: None,
        }
    }
}

// impl<'src> Iterator for Lexer<'src> {
//     type Item = Result<Token<'src>, Diagnostic>;

// fn next(&mut self) -> Option<Self::Item> {
//     if let Some(next) = self.peeked.take() {
//         return Some(next);
//     }

//     loop {
//         let cursor_onwards = &self.source[self.cursor..];
//         let mut chars = cursor_onwards.chars();
//         let Some(ch) = chars.next() else {
//             return Some(Ok(Token {
//                 kind: TokenKind::Eof,
//                 span: Span::new(self.cursor, self.cursor),
//             }));
//         };
//         let cursor = self.cursor;
//         self.cursor += 1;
//         let remainder = &self.source[self.cursor..];

//         let just = move |kind: TokenKind<'src>| {
//             Some(Ok(Token {
//                 kind,
//                 span: Span::new(cursor, cursor + 1),
//             }))
//         };

//         // Fn,
//         // Let,
//         // Return,
//         // If,
//         // Else,
//         // For,
//         // Break,
//         // Continue,
//         // In,
//         // Use,
//         // Pub,
//         // Match,
//         // None,
//         // Try,
//         // Catch,
//         // Throw,
//         // Struct,
//         // Enum,
//         // And,
//         // Or,
//         // Mut,

//         // // Types
//         // Int,
//         // Float,
//         // String,
//         // Bool,
//         // Void,

//         // // Identifiers
//         // Ident(&'src str),

//         // // Literals
//         // IntLiteral(i64),
//         // FloatLiteral(f64),
//         // BoolLiteral(bool),

//         // StringLiteral(&'src str),
//         // StringFragment(&'src str),

//         enum Started<'src> {
//             Slash,
//             Star,
//             String,
//             Plus,
//             Minus,
//             Number,
//             Ident,
//             IfEqualElse(TokenKind<'src>, TokenKind<'src>),
//         }
//         let started = match ch {
//             '(' => return just(TokenKind::LParen),
//             ')' => return just(TokenKind::LParen),
//             '{' => return just(TokenKind::LBrace),
//             '}' => return just(TokenKind::RBrace),
//             '[' => return just(TokenKind::LBracket),
//             ']' => return just(TokenKind::RBracket),
//             ',' => return just(TokenKind::Comma),
//             '.' => return just(TokenKind::Dot),
//             ';' => return just(TokenKind::Semicolon),
//             ':' => return just(TokenKind::Colon),
//             '?' => return just(TokenKind::QuestionMark),
//             '%' => return just(TokenKind::Percent),
//             '+' => Started::Plus,
//             '-' => Started::Minus,
//             '*' => Started::Star,
//             '/' => Started::Slash,
//             '=' => Started::IfEqualElse(TokenKind::EqEq, TokenKind::Eq),
//             '!' => Started::IfEqualElse(TokenKind::Neq, TokenKind::Bang),
//             '<' => Started::IfEqualElse(TokenKind::LtEq, TokenKind::Lt),
//             '>' => Started::IfEqualElse(TokenKind::GtEq, TokenKind::Gt),
//             '"' => Started::String,
//             '0'..='9' => Started::Number,
//             'a'..='z' | 'A'..='Z' | '_' => Started::Ident,
//             c if c.is_whitespace() => return self.next(),
//             c => {
//                 return Some(Err(Diagnostic::error(
//                     Span::new(cursor, cursor + 1),
//                     format!("Unexpected token '{}'", c),
//                 )));
//             }
//         };

//         match started {
//             Started::Star => {
//                 if remainder.starts_with('*') {
//                     self.cursor += 1;
//                     return Some(Ok(Token {
//                         kind: TokenKind::StarStar,
//                         span: Span::new(cursor, cursor + 2),
//                     }));
//                 } else if remainder.starts_with("=") {
//                     self.cursor += 1;
//                     return Some(Ok(Token {
//                         kind: TokenKind::StarEq,
//                         span: Span::new(cursor, cursor + 2),
//                     }));
//                 } else {
//                     return Some(Ok(Token {
//                         kind: TokenKind::Star,
//                         span: Span::new(cursor, cursor + 1),
//                     }));
//                 }
//             }
//             Started::Slash => {
//                 if remainder.starts_with('/') {
//                     let line_end = remainder.find('\n').unwrap_or_else(|| remainder.len());
//                     self.cursor += line_end;
//                     continue;
//                 } else if remainder.starts_with('=') {
//                     self.cursor += 1;
//                     return Some(Ok(Token {
//                         kind: TokenKind::SlashEq,
//                         span: Span::new(cursor, cursor + 2),
//                     }));
//                 } else {
//                     return Some(Ok(Token {
//                         kind: TokenKind::Slash,
//                         span: Span::new(cursor, cursor + 1),
//                     }));
//                 }
//             }
//             Started::String => todo!(),
//             Started::Plus => {}
//             Started::Minus => {}
//             Started::Number => {
//                 let first_non_digit = cursor_onwards
//                     .find(|c| !matches!(c, '.' | '0'..='9'))
//                     .unwrap_or_else(|| cursor_onwards.len());

//                 let literal = &cursor_onwards[..first_non_digit];
//                 let span = Span::new(cursor, first_non_digit);
//                 if literal.contains('.') {
//                     match literal.parse::<f64>() {
//                         Ok(f) => {
//                             return Some(Ok(Token {
//                                 kind: TokenKind::FloatLiteral(f),
//                                 span,
//                             }));
//                         }
//                         Err(_) => {
//                             return Some(Err(Diagnostic::error(
//                                 span,
//                                 "Failed to parse this as a float",
//                             )));
//                         }
//                     }
//                 } else {
//                     match literal.parse::<i64>() {
//                         Ok(f) => {
//                             return Some(Ok(Token {
//                                 kind: TokenKind::IntLiteral(f),
//                                 span,
//                             }));
//                         }
//                         Err(_) => {
//                             return Some(Err(Diagnostic::error(
//                                 span,
//                                 "Failed to parse this as an integer",
//                             )));
//                         }
//                     }
//                 }
//             }
//             Started::Ident => todo!(),
//             Started::IfEqualElse(if_equal, els) => {
//                 if remainder.starts_with("=") {
//                     self.cursor += 1;
//                     return Some(Ok(Token {
//                         kind: if_equal,
//                         span: Span::new(cursor, cursor + 2),
//                     }));
//                 } else {
//                     return Some(Ok(Token {
//                         kind: els,
//                         span: Span::new(cursor, cursor + 2),
//                     }));
//                 }
//             }
//         }
//     }
// }
// }

impl<'src> Lexer<'src> {
    pub(crate) fn span(&self) -> Span {
        Span::new(self.token, self.cursor)
    }

    pub(crate) fn token(
        &mut self,
        kind: TokenKind<'src>,
    ) -> Option<Result<Token<'src>, LexerError<'src>>> {
        Some(Ok(Token {
            kind,
            span: self.span(),
        }))
    }

    pub(crate) fn token_text(&self) -> &'src str {
        &self.input[self.token..self.cursor]
    }

    pub(crate) fn int(&mut self) -> Option<Result<Token<'src>, LexerError<'src>>> {
        match self.token_text().parse::<i64>() {
            Ok(i) => self.token(TokenKind::IntLiteral(i)),
            Err(e) => Some(Err(LexerError {
                span: self.span(),
                kind: LexerErrorKind::ParseIntError(e),
            })),
        }
    }

    pub(crate) fn float(&mut self) -> Option<Result<Token<'src>, LexerError<'src>>> {
        match self.token_text().parse::<f64>() {
            Ok(f) => self.token(TokenKind::FloatLiteral(f)),
            Err(e) => Some(Err(LexerError {
                span: self.span(),
                kind: LexerErrorKind::ParseFloatError(e),
            })),
        }
    }

    pub(crate) fn ident(&mut self) -> Option<Result<Token<'src>, LexerError<'src>>> {
        self.token(TokenKind::Ident(self.token_text()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lex2() {
        let source = include_str!("../../../examples/basic/kitchen_sink.fo");

        let lexer = Lexer::new(source);
        for token in lexer {
            match token {
                Ok(t) => {
                    println!("TOKEN {} {}", t.span, t.kind);
                }
                Err(e) => {
                    println!("ERROR {}", e.kind);
                }
            }
        }
    }
}
