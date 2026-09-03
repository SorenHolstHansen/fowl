use fowlc_error::{Diagnostic, Fmt, IntoDiagnostic};
use fowlc_span::Span;

#[derive(Clone, Debug)]
pub enum LexerError<'src> {
    /// Unexpected or unknown character
    UnexpectedCharacter(UnexpectedCharacter<'src>),

    /// Interpolation was never closed, as in, an unmatched "{"
    UnmatchedInterpolation(UnmatchedInterpolation<'src>),

    /// Eof has already been reached, nothing more to see
    EofAlreadyReached,
}

impl<'src> LexerError<'src> {
    pub fn into_diagnostic(&self) -> Diagnostic<'src> {
        match self {
            LexerError::UnexpectedCharacter(e) => e.into_diagnostic(),
            LexerError::UnmatchedInterpolation(e) => e.into_diagnostic(),
            LexerError::EofAlreadyReached => panic!("This should not be called in this case"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnexpectedCharacter<'src> {
    pub(crate) span: Span<'src>,
    pub(crate) char: &'src str,
}

impl<'src> IntoDiagnostic<'src> for UnexpectedCharacter<'src> {
    #[track_caller]
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        Diagnostic::new("E0001", self.span, "unexpected character").with_label(
            format!(
                "unexpected character \"{}\"",
                self.char.fg(fowlc_error::colors::PRIMARY)
            ),
            self.span,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) enum MissingBrace {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct UnmatchedInterpolation<'src> {
    pub(crate) span: Span<'src>,
    pub(crate) missing: MissingBrace,
}

impl<'src> IntoDiagnostic<'src> for UnmatchedInterpolation<'src> {
    #[track_caller]
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        Diagnostic::new("E0002", self.span, "unmatched interpolation")
            .with_help(match self.missing {
                MissingBrace::Left => "add an opening '{'",
                MissingBrace::Right => "add a closing '}'",
            })
            .with_label(
                match self.missing {
                    MissingBrace::Left => "the '}' was never started",
                    MissingBrace::Right => "the '}' was never closed",
                },
                self.span,
            )
    }
}
