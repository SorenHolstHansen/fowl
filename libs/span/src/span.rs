use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<'src> {
    start: usize,
    end: usize,
    file: &'src Path,
    source: &'src str,
}

impl std::fmt::Debug for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}

impl<'src> Span<'src> {
    pub fn new(start: usize, end: usize, file: &'src Path, source: &'src str) -> Self {
        Self {
            start,
            end,
            file,
            source,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn merge(&self, other: Span) -> Span<'src> {
        Span::new(
            self.start.min(other.start),
            self.end.max(other.end),
            self.file,
            self.source,
        )
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn overlaps(&self, other: Span) -> bool {
        self.start <= other.start && self.end > other.start
            || self.start < other.end && self.end >= other.end
    }

    pub fn file(&self) -> &'src Path {
        self.file
    }

    pub fn source(&self) -> &'src str {
        self.source
    }
}

impl<'src> From<Span<'src>> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl<'src> std::fmt::Display for Span<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}
