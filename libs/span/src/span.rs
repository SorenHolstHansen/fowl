#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    pub fn merge_range(&self, range: &std::ops::Range<usize>) -> Span {
        Span::new(self.start.min(range.start), self.end.max(range.end))
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(span: std::ops::Range<usize>) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}
