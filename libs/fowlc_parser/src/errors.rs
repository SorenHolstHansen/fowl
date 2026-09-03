use fowlc_error::{Diagnostic, IntoDiagnostic};
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

pub(crate) struct SelfParamInUnassociatedFunction<'src> {
    pub span: Span<'src>,
}

impl<'src> IntoDiagnostic<'src> for SelfParamInUnassociatedFunction<'src> {
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        Diagnostic::new(
            "E0004",
            self.span,
            "`self` parameter is only allowed in associated functions.",
        )
        .with_label(
            "`self` parameter is only allowed in associated functions
              associated functions are those in `impl` or `trait` definitions",
            self.span,
        )
    }
}
