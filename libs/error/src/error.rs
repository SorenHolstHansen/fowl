use ariadne::{Color, Label, Report, ReportKind, Source};
use span::Span;
use std::{borrow::Cow, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl DiagnosticSeverity {
    fn color(&self) -> Color {
        match self {
            DiagnosticSeverity::Error => Color::Red,
            DiagnosticSeverity::Warning => Color::Yellow,
            DiagnosticSeverity::Info => Color::Blue,
            DiagnosticSeverity::Hint => Color::Cyan,
        }
    }
}

#[derive(Clone)]
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    span: Span,
    message: Cow<'static, str>,
    suggestion: Option<Cow<'static, str>>,
    labels: Vec<(Cow<'static, str>, Span, DiagnosticSeverity)>,
    help: Option<Cow<'static, str>>,
}

impl Diagnostic {
    pub fn new<S: Into<Cow<'static, str>>>(
        severity: DiagnosticSeverity,
        span: Span,
        message: S,
    ) -> Self {
        Self {
            severity,
            span,
            message: message.into(),
            suggestion: None,
            labels: Vec::new(),
            help: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<Cow<'static, str>>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<Cow<'static, str>>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_error_label(mut self, span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        self.labels
            .push((message.into(), span, DiagnosticSeverity::Error));
        self
    }
    pub fn with_warning_label(mut self, span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        self.labels
            .push((message.into(), span, DiagnosticSeverity::Warning));
        self
    }
    pub fn with_info_label(mut self, span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        self.labels
            .push((message.into(), span, DiagnosticSeverity::Info));
        self
    }
    pub fn with_hint_label(mut self, span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        self.labels
            .push((message.into(), span, DiagnosticSeverity::Hint));
        self
    }

    pub fn report_kind(&self) -> ReportKind<'_> {
        match self.severity {
            DiagnosticSeverity::Error => ReportKind::Error,
            DiagnosticSeverity::Warning => ReportKind::Warning,
            DiagnosticSeverity::Info | DiagnosticSeverity::Hint => ReportKind::Advice,
        }
    }

    /// Create an info diagnostic
    pub fn info(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Info, span, message)
    }

    /// Create a hint diagnostic
    pub fn hint(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Hint, span, message)
    }

    /// Create an error diagnostic
    pub fn error(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Error, span, message)
    }

    /// Create a warning diagnostic
    pub fn warning(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Warning, span, message)
    }

    pub fn with_file<'source>(self, file: &'source Path) -> DiagnosticWithFile<'source> {
        DiagnosticWithFile {
            diagnostic: self,
            file,
        }
    }
}

#[derive(Clone)]
pub struct DiagnosticWithFile<'source> {
    diagnostic: Diagnostic,
    file: &'source Path,
}

pub fn emit_diagnostics<'source>(
    diagnostics: impl IntoIterator<Item = DiagnosticWithFile<'source>>,
    source: &'source str,
) {
    for DiagnosticWithFile { diagnostic, file } in diagnostics {
        let file = file.display().to_string();

        let span: std::ops::Range<usize> = diagnostic.span.into();
        let mut report = Report::build(diagnostic.report_kind(), (file.clone(), span.clone()))
            .with_message(&diagnostic.message);

        for (label, span, severity) in &diagnostic.labels {
            let span: std::ops::Range<usize> = (*span).into();
            report = report.with_label(
                Label::new((file.clone(), span))
                    .with_message(label)
                    .with_color(severity.color()),
            );
        }
        // Add suggestion if available
        if let Some(suggestion) = &diagnostic.suggestion {
            report = report.with_note(format!("💡 Suggestion: {}", suggestion));
        }

        // Add help text if available
        if let Some(help) = &diagnostic.help {
            report = report.with_note(format!("ℹ️  {}", help));
        } else {
            report = report
                .with_note("For more information, re-run with --debug to inspect tokens and AST.");
        }

        let _ = report.finish().eprint((file, Source::from(source)));
    }
}
