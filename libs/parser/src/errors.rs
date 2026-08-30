use error::{Diagnostic, IntoDiagnostic};
use span::Span;
use std::borrow::Cow;

pub(crate) struct SyntaxError<'src> {
    pub span: Span<'src>,
    pub expected: Vec<Cow<'static, str>>,
}

impl<'src> IntoDiagnostic<'src> for SyntaxError<'src> {
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        let mut label_message = "Syntax error: expected ".to_string();
        if self.expected.len() > 1 {
            label_message.push_str("one of ");
        }
        label_message.push_str(
            &self
                .expected
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", "),
        );
        Diagnostic {
            code: "E0003",
            span: self.span,
            message: "syntax error".into(),
            notes: vec![],
            help: vec![],
            labels: vec![(label_message.into(), self.span)],
        }
    }
}

pub(crate) struct SelfParamInUnassociatedFunction<'src> {
    pub span: Span<'src>,
}

impl<'src> IntoDiagnostic<'src> for SelfParamInUnassociatedFunction<'src> {
    fn into_diagnostic(&self) -> Diagnostic<'src> {
        Diagnostic {
            code: "E0004",
            span: self.span,
            message: "`self` parameter is only allowed in associated functions.".into(),
            notes: vec![],
            help: vec![
                "write the function as a method on a type: fn on MyType my_function(...".into(),
            ],
            labels: vec![(
                "`self` parameter is only allowed in associated functions
              associated functions are those in `impl` or `trait` definitions"
                    .into(),
                self.span,
            )],
        }
    }
}
