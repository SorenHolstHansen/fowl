use error::Diagnostic;
use span::Span;

// TODO: just use Diagnostics?
#[derive(Clone, Debug)]
pub enum LexerErrorKind<'src> {
    UnexpectedToken(&'src str),
    UnmatchedInterpolation(&'src str),
    EofReached,
}

impl<'src> std::fmt::Display for LexerErrorKind<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorKind::UnexpectedToken(t) => write!(f, "UnexpectedToken({t})"),
            LexerErrorKind::UnmatchedInterpolation(t) => write!(f, "UnmatchedInterpolation({t})"),
            LexerErrorKind::EofReached => write!(f, "EofReached"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LexerError<'src> {
    pub span: Span<'src>,
    pub(crate) kind: LexerErrorKind<'src>,
}

impl LexerError<'_> {
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, LexerErrorKind::EofReached)
    }
}

impl<'src> From<&LexerError<'src>> for Diagnostic<'src> {
    fn from(value: &LexerError<'src>) -> Self {
        let span = value.span;
        match &value.kind {
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
            LexerErrorKind::EofReached => Diagnostic::error(span, "End of file reached previously"),
        }
    }
}
