pub use ariadne::Fmt;
use ariadne::{Label, Report, ReportKind, sources};
use span::Span;
use std::borrow::Cow;

pub mod colors {
    use ariadne::Color;

    pub static PRIMARY: Color = Color::BrightGreen;
}

pub struct Diagnostic<'src> {
    pub code: &'static str,
    pub span: Span<'src>,
    pub message: Cow<'static, str>,
    pub notes: Vec<Cow<'static, str>>,
    pub help: Vec<Cow<'static, str>>,
    pub labels: Vec<(Cow<'static, str>, Span<'src>)>,
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
            Err(e) => Err(e.add_help(help)),
        }
    }

    fn add_note<S: Into<Cow<'static, str>>>(self, note: S) -> Result<T, Diagnostic<'src>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.add_note(note)),
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
    pub fn add_help<S: Into<Cow<'static, str>>>(mut self, help: S) -> Diagnostic<'src> {
        self.help.push(help.into());
        self
    }

    pub fn add_note<S: Into<Cow<'static, str>>>(mut self, note: S) -> Diagnostic<'src> {
        self.notes.push(note.into());
        self
    }

    pub fn emit(self) {
        let range: std::ops::Range<usize> = self.span.into();
        let file = self.span.file().display().to_string();
        let mut report = Report::build(ReportKind::Error, (file, range))
            .with_code(self.code)
            .with_message(self.message);

        for note in self.notes {
            report.add_note(note);
        }
        for help in self.help {
            report.add_help(help);
        }
        let mut srcs = Vec::with_capacity(self.labels.len());
        for label in self.labels {
            let message = label.0;
            let span = label.1;
            srcs.push((span.file().display().to_string(), span.source()));
            let range: std::ops::Range<usize> = span.into();
            report.add_label(
                Label::new((span.file().display().to_string(), range)).with_message(message),
            );
        }

        report.finish().eprint(sources(srcs)).unwrap();
    }
}
