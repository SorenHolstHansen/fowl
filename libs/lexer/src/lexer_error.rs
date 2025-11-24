use error::Diagnostic;
use span::Span;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone)]
pub enum LexerErrorKind<'src> {
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UnexpectedToken(&'src str),
    UnmatchedInterpolation(&'src str),
}

impl<'src> std::fmt::Display for LexerErrorKind<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorKind::ParseIntError(e) => write!(f, "ParseIntError({e:?})"),
            LexerErrorKind::ParseFloatError(e) => write!(f, "ParseFloatError({e:?})"),
            LexerErrorKind::UnexpectedToken(t) => write!(f, "UnexpectedToken({t})"),
            LexerErrorKind::UnmatchedInterpolation(t) => write!(f, "UnmatchedInterpolation({t})"),
        }
    }
}

#[derive(Clone)]
pub struct LexerError<'src> {
    pub(crate) span: Span<'src>,
    pub(crate) kind: LexerErrorKind<'src>,
}

impl<'src> From<&LexerError<'src>> for Diagnostic<'src> {
    fn from(value: &LexerError<'src>) -> Self {
        let span = value.span;
        match &value.kind {
            LexerErrorKind::ParseIntError(e) => Diagnostic::error(span, e.to_string()),
            LexerErrorKind::ParseFloatError(e) => Diagnostic::error(span, e.to_string()),
            LexerErrorKind::UnexpectedToken(t) => {
                Diagnostic::error(span, format!("Unknown token '{}'", t))
            }
            LexerErrorKind::UnmatchedInterpolation(t) => Diagnostic::error(
                span,
                format!(
                    "Unmatched '{}' found. If you want to print '{}', write '\\{}'",
                    t, t, t
                ),
            ),
        }
    }
}
