use ariadne::{Color, Label, Report, ReportKind, sources};
use span::Span;
use std::borrow::Cow;

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
            DiagnosticSeverity::Info => Color::Green,
            DiagnosticSeverity::Hint => Color::Cyan,
        }
    }
}

#[derive(Clone)]
enum Element<'src> {
    Label {
        message: Cow<'static, str>,
        span: Span<'src>,
        severity: DiagnosticSeverity,
    },
    Note(Cow<'static, str>),
    Help(Cow<'static, str>),
}

#[derive(Clone)]
pub struct Diagnostic<'src> {
    severity: DiagnosticSeverity,
    span: Span<'src>,
    message: Cow<'static, str>,
    elements: Vec<Element<'src>>,
}

impl<'src> Diagnostic<'src> {
    pub fn new<S: Into<Cow<'static, str>>>(
        severity: DiagnosticSeverity,
        span: Span<'src>,
        message: S,
    ) -> Self {
        Self {
            severity,
            span,
            message: message.into(),
            elements: Vec::new(),
        }
    }

    pub fn with_note(mut self, suggestion: impl Into<Cow<'static, str>>) -> Self {
        self.elements.push(Element::Note(suggestion.into()));
        self
    }

    pub fn with_help(mut self, help: impl Into<Cow<'static, str>>) -> Self {
        self.elements.push(Element::Help(help.into()));
        self
    }

    pub fn with_error_label(
        mut self,
        span: Span<'src>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.elements.push(Element::Label {
            message: message.into(),
            span,
            severity: DiagnosticSeverity::Error,
        });
        self
    }
    pub fn with_warning_label(
        mut self,
        span: Span<'src>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.elements.push(Element::Label {
            message: message.into(),
            span,
            severity: DiagnosticSeverity::Warning,
        });
        self
    }
    pub fn with_info_label(
        mut self,
        span: Span<'src>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.elements.push(Element::Label {
            message: message.into(),
            span,
            severity: DiagnosticSeverity::Info,
        });
        self
    }
    pub fn with_hint_label(
        mut self,
        span: Span<'src>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.elements.push(Element::Label {
            message: message.into(),
            span,
            severity: DiagnosticSeverity::Hint,
        });
        self
    }

    fn report_kind(&self) -> ReportKind<'_> {
        match self.severity {
            DiagnosticSeverity::Error => ReportKind::Error,
            DiagnosticSeverity::Warning => ReportKind::Warning,
            DiagnosticSeverity::Info | DiagnosticSeverity::Hint => ReportKind::Advice,
        }
    }

    /// Create an info diagnostic
    pub fn info(span: Span<'src>, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Info, span, message)
    }

    /// Create a hint diagnostic
    pub fn hint(span: Span<'src>, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Hint, span, message)
    }

    /// Create an error diagnostic
    pub fn error(span: Span<'src>, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Error, span, message)
    }

    /// Create a warning diagnostic
    pub fn warning(span: Span<'src>, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(DiagnosticSeverity::Warning, span, message)
    }
}

pub fn emit_diagnostics<'src>(diagnostics: impl IntoIterator<Item = Diagnostic<'src>>) {
    for diagnostic in diagnostics {
        let range: std::ops::Range<usize> = diagnostic.span.into();
        let file = diagnostic.span.file().display().to_string();
        let mut report = Report::build(diagnostic.report_kind(), (file, range))
            .with_message(&diagnostic.message);

        let mut srcs = Vec::with_capacity(diagnostic.elements.len());
        for element in &diagnostic.elements {
            match element {
                Element::Note(note) => {
                    report.add_note(note);
                }
                Element::Help(help) => {
                    report.add_help(help);
                }
                Element::Label {
                    message,
                    span,
                    severity,
                } => {
                    srcs.push((span.file().display().to_string(), span.source()));
                    let range: std::ops::Range<usize> = (*span).into();
                    report.add_label(
                        Label::new((span.file().display().to_string(), range))
                            .with_message(message)
                            .with_color(severity.color()),
                    );
                }
            }
        }

        let _ = report.finish().eprint(sources(srcs));
    }
}
