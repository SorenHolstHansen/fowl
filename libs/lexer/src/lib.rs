use std::fmt::Write;
mod token;
pub use token::{Token, TokenKind};
mod lexer;
pub mod lexer_error;
mod lexing;
pub use lexer::Lexer;

impl<'src> Lexer<'src> {
    pub fn pretty_string(self) -> String {
        let mut buf = String::new();
        pretty_print_tokens(self, &mut buf);
        buf
    }
}

fn pretty_print_tokens<'src>(lexer: Lexer<'src>, buf: &mut String) {
    let mut indent = 0;
    for token in lexer {
        match token {
            Ok(t) => {
                if let TokenKind::StringInterpolationStart = t.kind {
                    indent += 2;
                    writeln!(buf, "{:indent$}-> STRING INTERPOLATION", "").unwrap();
                } else if let TokenKind::StringInterpolationEnd = t.kind {
                    writeln!(buf, "{:indent$}<- STRING INTERPOLATION", "").unwrap();
                    indent -= 2;
                } else {
                    writeln!(buf, "{:indent$}{:8} {}", "", t.span, t.kind).unwrap()
                }
            }
            Err(e) => writeln!(buf, "{:indent$}{:8} {}", "", e.span, e.kind).unwrap(),
        }
    }
}

pub fn tokenize<'src>(source: &'src str) -> Lexer<'src> {
    Lexer::new(source)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = include_str!("../../../examples/kitchen_sink.fo");

        let mut lexer = tokenize(source);

        let mut previous = match lexer.next() {
            Some(Ok(t)) => t,
            Some(Err(e)) => panic!("Found lexer error {} {}", e.span, e.kind),
            None => panic!("Unexpected end of lexer"),
        };

        for t in lexer {
            match t {
                Ok(t) => {
                    assert!(
                        !t.span.overlaps(previous.span),
                        "Two spans overlapped. {} {}",
                        t.span,
                        previous.span
                    );
                    previous = t;
                }
                Err(e) => panic!("Found lexer error {} {}", e.span, e.kind),
            }
        }
    }
}
