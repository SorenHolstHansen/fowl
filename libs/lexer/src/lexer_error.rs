use std::{
    num::{ParseFloatError, ParseIntError},
    path::Path,
};

use error::{Diagnostic, DiagnosticWithFile};
use span::Span;

use crate::Token;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexerError<'source> {
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    Generic(&'source str),
    #[default]
    Other,
}

impl<'source> From<ParseIntError> for LexerError<'source> {
    fn from(err: ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}

impl<'source> From<ParseFloatError> for LexerError<'source> {
    fn from(err: ParseFloatError) -> Self {
        Self::ParseFloatError(err)
    }
}

impl<'source> LexerError<'source> {
    pub fn from_lexer(lex: &mut logos::Lexer<'source, Token<'source>>) -> Self {
        LexerError::Generic(lex.slice())
    }
}

pub fn lexer_error_to_diagnostic<'source>(
    lexer_error: &'source LexerError,
    span: Span,
    file: &'source Path,
) -> DiagnosticWithFile<'source> {
    match lexer_error {
        LexerError::ParseIntError(e) => Diagnostic::error(span, e.to_string()).with_file(file),
        LexerError::ParseFloatError(e) => Diagnostic::error(span, e.to_string()).with_file(file),
        LexerError::Generic(t) => {
            Diagnostic::error(span, format!("Unknown token '{}'", t)).with_file(file)
        }
        LexerError::Other => Diagnostic::error(span, "Unknown lexer error").with_file(file),
    }
}
