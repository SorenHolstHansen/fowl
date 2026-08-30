use error::{Diagnostic, Fmt, IntoDiagnostic};
use span::Span;

#[derive(Clone, Debug)]
pub enum LexerError<'src> {
    /// Unexpected or unknown character
    UnexpectedCharacter(UnexpectedCharacter<'src>),

    /// Interpolation was never closed, as in, an unmatched "{"
    UnmatchedInterpolation(UnmatchedInterpolation<'src>),
}

impl<'src> LexerError<'src> {
    pub fn into_diagnostic(&self) -> Diagnostic<'src> {
        match self {
            LexerError::UnexpectedCharacter(e) => e.into_diagnostic(),
            LexerError::UnmatchedInterpolation(e) => e.into_diagnostic(),
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
        Diagnostic {
            span: self.span,
            message: "unexpected character".into(),
            code: "E0001",
            help: vec![],
            notes: vec![],
            labels: vec![(
                format!(
                    "unexpected character \"{}\"",
                    self.char.fg(error::colors::PRIMARY)
                )
                .into(),
                self.span,
            )],
        }
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
        Diagnostic {
            span: self.span,
            message: "unmatched interpolation".into(),
            code: "E0002",
            help: vec![match self.missing {
                MissingBrace::Left => "add an opening '{'".into(),
                MissingBrace::Right => "add a closing '}'".into(),
            }],
            notes: vec![],
            labels: vec![(
                match self.missing {
                    MissingBrace::Left => "the '}' was never started".into(),
                    MissingBrace::Right => "the '}' was never closed".into(),
                },
                self.span,
            )],
        }
    }
}
