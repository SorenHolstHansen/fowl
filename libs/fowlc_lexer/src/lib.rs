mod token;
pub use token::{Token, TokenKind};
mod lexer;
pub mod lexer_error;
mod lexing;
pub use lexer::Lexer;
pub use token::INFIX_OPERATORS;

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;
    use crate::lexer_error::LexerError;

    #[track_caller]
    fn assert_lexer<'src>(
        source: &'src str,
        tokens: &[(TokenKind, &'src str, std::ops::Range<usize>)],
    ) {
        let mut lexer = Lexer::new(source, std::path::Path::new(""));

        for tuple in tokens {
            let mut next_token = lexer.next();
            while next_token
                .as_ref()
                .is_ok_and(|t| matches!(t.kind, TokenKind::Whitespace))
            {
                next_token = lexer.next();
            }

            match next_token {
                Ok(t) => {
                    let range = std::ops::Range::from(t.span.clone());
                    assert_eq!(&(t.kind, &source[range.clone()], range), tuple)
                }
                Err(e) => {
                    // assert_eq!(&(Err(e.kind), &source[e.span.clone()], e.span), tuple)
                }
            }
        }

        assert_eq!(lexer.next().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn test_keywords() {
        assert_lexer("fn", &[(TokenKind::Fn, "fn", 0..2)]);
        assert_lexer("let", &[(TokenKind::Let, "let", 0..3)]);
        assert_lexer("return", &[(TokenKind::Return, "return", 0..6)]);
        assert_lexer("if", &[(TokenKind::If, "if", 0..2)]);
        assert_lexer("for", &[(TokenKind::For, "for", 0..3)]);
        assert_lexer("break", &[(TokenKind::Break, "break", 0..5)]);
        assert_lexer("continue", &[(TokenKind::Continue, "continue", 0..8)]);
        assert_lexer("in", &[(TokenKind::In, "in", 0..2)]);
        assert_lexer("is", &[(TokenKind::Is, "is", 0..2)]);
        assert_lexer("use", &[(TokenKind::Use, "use", 0..3)]);
        assert_lexer("public", &[(TokenKind::Public, "public", 0..6)]);
        assert_lexer("internal", &[(TokenKind::Internal, "internal", 0..8)]);
        assert_lexer("private", &[(TokenKind::Private, "private", 0..7)]);
        assert_lexer("none", &[(TokenKind::None, "none", 0..4)]);
        assert_lexer("try", &[(TokenKind::Try, "try", 0..3)]);
        assert_lexer("catch", &[(TokenKind::Catch, "catch", 0..5)]);
        assert_lexer("throw", &[(TokenKind::Throw, "throw", 0..5)]);
        assert_lexer("struct", &[(TokenKind::Struct, "struct", 0..6)]);
        assert_lexer("enum", &[(TokenKind::Enum, "enum", 0..4)]);
        assert_lexer("and", &[(TokenKind::And, "and", 0..3)]);
        assert_lexer("or", &[(TokenKind::Or, "or", 0..2)]);
        assert_lexer("mut", &[(TokenKind::Mut, "mut", 0..3)]);
        assert_lexer("on", &[(TokenKind::On, "on", 0..2)]);
        assert_lexer("impl", &[(TokenKind::Impl, "impl", 0..4)]);
        assert_lexer("self", &[(TokenKind::Self_, "self", 0..4)]);
    }

    #[test]
    fn test_structural() {
        // Structural
        assert_lexer("_", &[(TokenKind::Underscore, "_", 0..1)]);
        assert_lexer(":", &[(TokenKind::Colon, ":", 0..1)]);
        assert_lexer(";", &[(TokenKind::Semicolon, ";", 0..1)]);
        assert_lexer("(", &[(TokenKind::LeftParenthesis, "(", 0..1)]);
        assert_lexer(")", &[(TokenKind::RightParenthesis, ")", 0..1)]);
        assert_lexer("{", &[(TokenKind::LeftBrace, "{", 0..1)]);
        assert_lexer("}", &[(TokenKind::RightBrace, "}", 0..1)]);
        assert_lexer("[", &[(TokenKind::LeftBracket, "[", 0..1)]);
        assert_lexer("]", &[(TokenKind::RightBracket, "]", 0..1)]);
        assert_lexer(",", &[(TokenKind::Comma, ",", 0..1)]);
        assert_lexer(".", &[(TokenKind::Dot, ".", 0..1)]);
    }

    #[test]
    fn test_operators() {
        // Operators
        assert_lexer("=", &[(TokenKind::Equal, "=", 0..1)]);
        assert_lexer("==", &[(TokenKind::EqualEqual, "==", 0..2)]);
        assert_lexer("!=", &[(TokenKind::NotEqual, "!=", 0..2)]);
        assert_lexer("<", &[(TokenKind::LessThan, "<", 0..1)]);
        assert_lexer(">", &[(TokenKind::GreaterThan, ">", 0..1)]);
        assert_lexer("<=", &[(TokenKind::LessThanOrEqual, "<=", 0..2)]);
        assert_lexer(">=", &[(TokenKind::GreaterThanOrEqual, ">=", 0..2)]);
        assert_lexer("+", &[(TokenKind::Plus, "+", 0..1)]);
        assert_lexer("-", &[(TokenKind::Minus, "-", 0..1)]);
        assert_lexer("*", &[(TokenKind::Star, "*", 0..1)]);
        assert_lexer("**", &[(TokenKind::StarStar, "**", 0..2)]);
        assert_lexer("/", &[(TokenKind::Slash, "/", 0..1)]);
        assert_lexer("%", &[(TokenKind::Percent, "%", 0..1)]);
        assert_lexer("!", &[(TokenKind::Bang, "!", 0..1)]);

        // Assignment operators
        assert_lexer("+=", &[(TokenKind::PlusEqual, "+=", 0..2)]);
        assert_lexer("-=", &[(TokenKind::MinusEqual, "-=", 0..2)]);
        assert_lexer("*=", &[(TokenKind::StarEqual, "*=", 0..2)]);
        assert_lexer("/=", &[(TokenKind::SlashEqual, "/=", 0..2)]);
    }

    #[test]
    fn test_parenthesis() {
        assert_lexer(
            "()",
            &[
                (TokenKind::LeftParenthesis, "(", 0..1),
                (TokenKind::RightParenthesis, ")", 1..2),
            ],
        );
    }

    #[test]
    fn test_brackets() {
        assert_lexer(
            "[]",
            &[
                (TokenKind::LeftBracket, "[", 0..1),
                (TokenKind::RightBracket, "]", 1..2),
            ],
        );
    }

    #[test]
    fn test_curly_braces() {
        assert_lexer(
            "{}",
            &[
                (TokenKind::LeftBrace, "{", 0..1),
                (TokenKind::RightBrace, "}", 1..2),
            ],
        );
    }

    #[test]
    fn test_comment() {
        assert_lexer("//", &[(TokenKind::Comment, "//", 0..2)]);

        assert_lexer("// testing", &[(TokenKind::Comment, "// testing", 0..10)]);

        assert_lexer(
            "// comment 1
            // comment 2",
            &[
                (TokenKind::Comment, "// comment 1", 0..12),
                (TokenKind::Comment, "// comment 2", 25..37),
            ],
        );

        assert_lexer(
            "// comment 1
            //
            // comment 2",
            &[
                (TokenKind::Comment, "// comment 1", 0..12),
                (TokenKind::Comment, "//", 25..27),
                (TokenKind::Comment, "// comment 2", 40..52),
            ],
        );

        assert_lexer(
            "// comment 1  ",
            &[(TokenKind::Comment, "// comment 1  ", 0..14)],
        );
    }

    #[test]
    fn test_identifier() {
        assert_lexer("foo", &[(TokenKind::Identifier, "foo", 0..3)]);
        assert_lexer("FOO", &[(TokenKind::Identifier, "FOO", 0..3)]);
        assert_lexer("_", &[(TokenKind::Underscore, "_", 0..1)]);
        assert_lexer("_1", &[(TokenKind::Identifier, "_1", 0..2)]);
        assert_lexer("_test1", &[(TokenKind::Identifier, "_test1", 0..6)]);
    }

    #[test]
    fn test_number() {
        assert_lexer(
            "0 0000",
            &[
                (TokenKind::IntegerLiteral, "0", 0..1),
                (TokenKind::IntegerLiteral, "0000", 2..6),
            ],
        );

        assert_lexer(
            "10.0 10.123",
            &[
                (TokenKind::FloatLiteral, "10.0", 0..4),
                (TokenKind::FloatLiteral, "10.123", 5..11),
            ],
        );

        assert_lexer(
            "0_0 00_00 1_000_000",
            &[
                (TokenKind::IntegerLiteral, "0_0", 0..3),
                (TokenKind::IntegerLiteral, "00_00", 4..9),
                (TokenKind::IntegerLiteral, "1_000_000", 10..19),
            ],
        );

        assert_lexer("-1", &[(TokenKind::IntegerLiteral, "-1", 0..2)]);
    }

    #[test]
    fn test_string() {
        assert_lexer(
            "\"\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::StringInterpolationEnd, "\"", 1..2),
            ],
        );
        assert_lexer(
            "\"    \"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::StringLiteral, "    ", 1..5),
                (TokenKind::StringInterpolationEnd, "\"", 5..6),
            ],
        );

        assert_lexer(
            "\"hello world\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::StringLiteral, "hello world", 1..12),
                (TokenKind::StringInterpolationEnd, "\"", 12..13),
            ],
        );

        assert_lexer(
            "\"hello\nworld\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::StringLiteral, "hello\nworld", 1..12),
                (TokenKind::StringInterpolationEnd, "\"", 12..13),
            ],
        );

        assert_lexer("\"", &[(TokenKind::StringInterpolationStart, "\"", 0..1)]);
    }

    #[test]
    fn test_string_interpolation() {
        assert_lexer(
            "\"{1}\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::LeftBrace, "{", 1..2),
                (TokenKind::IntegerLiteral, "1", 2..3),
                (TokenKind::RightBrace, "}", 3..4),
                (TokenKind::StringInterpolationEnd, "\"", 4..5),
            ],
        );
        assert_lexer(
            "\"hi {1} there {2} stop\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::StringLiteral, "hi ", 1..4),
                (TokenKind::LeftBrace, "{", 4..5),
                (TokenKind::IntegerLiteral, "1", 5..6),
                (TokenKind::RightBrace, "}", 6..7),
                (TokenKind::StringLiteral, " there ", 7..14),
                (TokenKind::LeftBrace, "{", 14..15),
                (TokenKind::IntegerLiteral, "2", 15..16),
                (TokenKind::RightBrace, "}", 16..17),
                (TokenKind::StringLiteral, " stop", 17..22),
                (TokenKind::StringInterpolationEnd, "\"", 22..23),
            ],
        );
        assert_lexer(
            "\"{\"{\"{\"hi\"}\"}\"}\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::LeftBrace, "{", 1..2),
                (TokenKind::StringInterpolationStart, "\"", 2..3),
                (TokenKind::LeftBrace, "{", 3..4),
                (TokenKind::StringInterpolationStart, "\"", 4..5),
                (TokenKind::LeftBrace, "{", 5..6),
                (TokenKind::StringInterpolationStart, "\"", 6..7),
                (TokenKind::StringLiteral, "hi", 7..9),
                (TokenKind::StringInterpolationEnd, "\"", 9..10),
                (TokenKind::RightBrace, "}", 10..11),
                (TokenKind::StringInterpolationEnd, "\"", 11..12),
                (TokenKind::RightBrace, "}", 12..13),
                (TokenKind::StringInterpolationEnd, "\"", 13..14),
                (TokenKind::RightBrace, "}", 14..15),
                (TokenKind::StringInterpolationEnd, "\"", 15..16),
            ],
        );
    }

    #[test]
    fn test_special_char_boundary() {
        assert_lexer("Ş", &[(TokenKind::Identifier, "Ş", 0..2)]);
        assert_lexer(
            "\"Ş\"",
            &[
                (TokenKind::StringInterpolationStart, "\"", 0..1),
                (TokenKind::StringLiteral, "Ş", 1..3),
                (TokenKind::StringInterpolationEnd, "\"", 3..4),
            ],
        );
        assert_lexer("identŞ", &[(TokenKind::Identifier, "identŞ", 0..7)])
    }

    #[test]
    fn unmatched_interpolation() {
        let mut lexer = Lexer::new("\"hi there}\"", std::path::Path::new(""));

        // Skipping the first couple things
        let _ = lexer.next();
        let _ = lexer.next();
        let next_token = lexer.next();

        assert_matches!(next_token, Err(LexerError::UnmatchedInterpolation(_)));

        // -------

        let mut lexer = Lexer::new("\"hi {1\"", std::path::Path::new(""));

        // Skipping the first couple things
        let _ = lexer.next();
        let _ = lexer.next();
        let _ = lexer.next();
        let _ = lexer.next();
        let next_token = lexer.next();

        assert_matches!(next_token, Err(LexerError::UnmatchedInterpolation(_)));
    }
}
