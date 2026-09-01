pub use ariadne::Fmt;
use ariadne::{Label, Report, ReportKind, sources};
use span::Span;
use std::borrow::Cow;

pub mod colors {
    use ariadne::Color;

    pub static PRIMARY: Color = Color::BrightGreen;
}

#[derive(Clone, Debug)]
enum Element<'src> {
    Label {
        message: Cow<'static, str>,
        span: Span<'src>,
    },
    Note(Cow<'static, str>),
    Help(Cow<'static, str>),
}

pub struct Diagnostic<'src> {
    pub code: &'static str,
    pub span: Span<'src>,
    pub message: Cow<'static, str>,
    elements: Vec<Element<'src>>,
}

pub trait IntoDiagnostic<'src> {
    #[allow(clippy::wrong_self_convention)]
    fn into_diagnostic(&self) -> Diagnostic<'src>;
}

pub trait ResultExt<'src, T> {
    fn add_help<S: Into<Cow<'static, str>>>(self, help: S) -> Result<T, Diagnostic<'src>>;
    fn add_note<S: Into<Cow<'static, str>>>(self, note: S) -> Result<T, Diagnostic<'src>>;
    fn emit_ok(self) -> Option<T>;
}

impl<'src, T> ResultExt<'src, T> for Result<T, Diagnostic<'src>> {
    fn add_help<S: Into<Cow<'static, str>>>(self, help: S) -> Result<T, Diagnostic<'src>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.with_help(help)),
        }
    }

    fn add_note<S: Into<Cow<'static, str>>>(self, note: S) -> Result<T, Diagnostic<'src>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.with_note(note)),
        }
    }

    fn emit_ok(self) -> Option<T> {
        match self {
            Ok(ok) => Some(ok),
            Err(e) => {
                e.emit();
                None
            }
        }
    }
}

impl<'src> Diagnostic<'src> {
    pub fn new<M: Into<Cow<'static, str>>>(
        code: &'static str,
        span: Span<'src>,
        message: M,
    ) -> Self {
        Self {
            code,
            span,
            message: message.into(),
            elements: Vec::new(),
        }
    }

    pub fn with_help<S: Into<Cow<'static, str>>>(mut self, help: S) -> Diagnostic<'src> {
        self.elements.push(Element::Help(help.into()));
        self
    }

    pub fn with_note<S: Into<Cow<'static, str>>>(mut self, note: S) -> Diagnostic<'src> {
        self.elements.push(Element::Note(note.into()));
        self
    }

    pub fn with_label<M: Into<Cow<'static, str>>>(mut self, message: M, span: Span<'src>) -> Self {
        self.elements.push(Element::Label {
            message: message.into(),
            span,
        });

        self
    }

    pub fn emit(self) {
        let range: std::ops::Range<usize> = self.span.into();
        let file = self.span.file().display().to_string();
        let mut report = Report::build(ReportKind::Error, (file, range))
            .with_code(self.code)
            .with_message(self.message);

        let mut srcs = Vec::with_capacity(self.elements.len());
        for element in &self.elements {
            match element {
                Element::Note(note) => {
                    report.add_note(note);
                }
                Element::Help(help) => {
                    report.add_help(help);
                }
                Element::Label { message, span } => {
                    srcs.push((span.file().display().to_string(), span.source()));
                    let range: std::ops::Range<usize> = (*span).into();
                    report.add_label(
                        Label::new((span.file().display().to_string(), range))
                            .with_message(message),
                    );
                }
            }
        }

        report.finish().eprint(sources(srcs)).unwrap();
    }
}
