use fowlc_error::{Diagnostic, IntoDiagnostic};
use fowlc_lexer::TokenKind;
use fowlc_span::Span;
use std::borrow::Cow;

pub(crate) struct SyntaxError<'src> {
    pub span: Span<'src>,
    pub expected: Cow<'static, str>,
}

impl<'src> IntoDiagnostic<'src> for SyntaxError<'src> {
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        Diagnostic::new("E0003", self.span, "syntax error").with_label(
            format!("Syntax error: expected {}", self.expected),
            self.span,
        )
    }
}

pub(crate) struct Unimplemented<'src> {
    pub span: Span<'src>,
    pub in_function: &'static str,
    pub token: TokenKind,
}

impl<'src> IntoDiagnostic<'src> for Unimplemented<'src> {
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        Diagnostic::new(
            "ENA",
            self.span,
            format!(
                "internal function '{}' not implemented yet for token '{:?}'",
                self.in_function, self.token
            ),
        )
        .with_label(
            format!(
                "internal function '{}' not implemented yet for token '{:?}'",
                self.in_function, self.token
            ),
            self.span,
        )
    }
}
