#[macro_use]
extern crate afl;
use lexer::*;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let mut lexer = Lexer::new(s);

            loop {
                let next_token = lexer.next();
                if let Ok(Token { kind, .. }) = next_token
                    && kind == TokenKind::Eof
                {
                    break;
                }
            }
        }
    });
}
