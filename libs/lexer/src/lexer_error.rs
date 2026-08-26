use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum LexerErrorKind<'src> {
    /// Unexpected or unknown character
    UnexpectedCharacter(&'src str),

    /// Interpolation was never closed, as in an unmatched {
    UnmatchedInterpolation(&'src str),

    /// EOF already reached
    EofReached,
}

#[derive(Clone, Debug)]
pub struct LexerError<'src> {
    pub span: Range<usize>,
    pub(crate) kind: LexerErrorKind<'src>,
}

impl LexerError<'_> {
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, LexerErrorKind::EofReached)
    }
}
