use crate::{
    Token,
    lexer_error::{LexerError, LexerErrorKind},
    lexing::YYC_INIT,
    token::TokenKind,
};
use span::Span;

#[derive(Clone)]
pub struct Lexer<'src> {
    pub(crate) input: &'src str,
    pub(crate) token: usize,
    pub(crate) cursor: usize,
    pub(crate) marker: usize,
    pub(crate) cond: usize,
    pub(crate) interpolation_depth: usize,
    pub(crate) eof: bool,
    pub(crate) peeked: Option<Result<Token<'src>, LexerError<'src>>>,
    pub(crate) force_next_token: Option<Result<Token<'src>, LexerError<'src>>>,
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
            force_next_token: None,
        }
    }
}

impl<'src> Lexer<'src> {
    pub fn peek(&mut self) -> Option<&Result<Token<'src>, LexerError<'src>>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }

        self.peeked = self.next();
        self.peeked.as_ref()
    }

    pub(crate) fn span(&self) -> Span {
        Span::new(self.token, self.cursor)
    }

    pub(crate) fn error(
        &mut self,
        kind: LexerErrorKind<'src>,
    ) -> Option<Result<Token<'src>, LexerError<'src>>> {
        Some(Err(LexerError {
            span: self.span(),
            kind,
        }))
    }

    pub(crate) fn token(
        &mut self,
        kind: TokenKind<'src>,
    ) -> Option<Result<Token<'src>, LexerError<'src>>> {
        let res = Some(Ok(Token {
            kind,
            span: self.span(),
        }));
        if kind == TokenKind::Eof {
            return res;
        }
        let rest = &self.input[self.cursor..];
        let mut next_is_newline = false;
        for char in rest.chars() {
            if char == '\n' {
                next_is_newline = true;
                break;
            } else if char.is_whitespace() {
                continue;
            } else {
                break;
            }
        }
        if next_is_newline {
            match kind {
                TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::None
                | TokenKind::Int
                | TokenKind::Float
                | TokenKind::String
                | TokenKind::Bool
                | TokenKind::Void
                | TokenKind::Ident(_)
                | TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::BoolLiteral(_)
                | TokenKind::StringInterpolationEnd
                | TokenKind::RParen
                | TokenKind::RBrace
                | TokenKind::RBracket => {
                    self.force_next_token = Some(Ok(Token {
                        kind: TokenKind::Semicolon,
                        span: Span::new(self.cursor, self.cursor + 1),
                    }))
                }
                _ => {}
            }
        }

        res
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
        let source = include_str!("../../../examples/simple/src/kitchen_sink.fo");

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
