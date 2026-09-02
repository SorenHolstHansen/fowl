#[macro_use]
extern crate afl;
use std::path::Path;

use lexer::*;
use parser::Parser;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let lexer = Lexer::new(s, Path::new(""));
            let parser = Parser::new(lexer);

            parser.parse();
        }
    });
}
