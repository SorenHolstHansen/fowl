mod token;
pub use token::{Token, TokenKind};
mod lexer;
pub mod lexer_error;
mod lexing;
pub use lexer::Lexer;

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer_error::LexerErrorKind;

    #[track_caller]
    fn assert_lexer<'src>(
        source: &'src str,
        tokens: &[(
            Result<TokenKind<'src>, LexerErrorKind<'src>>,
            &'src str,
            std::ops::Range<usize>,
        )],
    ) {
        let mut lexer = Lexer::new(source);

        for tuple in tokens {
            let mut next_token = lexer.next();
            while next_token
                .as_ref()
                .is_ok_and(|t| matches!(t.kind, TokenKind::Whitespace(_)))
            {
                next_token = lexer.next();
            }

            match next_token {
                Ok(t) => assert_eq!(&(Ok(t.kind), &source[t.span.clone()], t.span), tuple),
                Err(e) => assert_eq!(&(Err(e.kind), &source[e.span.clone()], e.span), tuple),
            }
        }

        assert_eq!(lexer.next().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn test_keywords() {
        assert_lexer("fn", &[(Ok(TokenKind::Fn), "fn", 0..2)]);
        assert_lexer("let", &[(Ok(TokenKind::Let), "let", 0..3)]);
        assert_lexer("return", &[(Ok(TokenKind::Return), "return", 0..6)]);
        assert_lexer("if", &[(Ok(TokenKind::If), "if", 0..2)]);
        assert_lexer("for", &[(Ok(TokenKind::For), "for", 0..3)]);
        assert_lexer("break", &[(Ok(TokenKind::Break), "break", 0..5)]);
        assert_lexer("continue", &[(Ok(TokenKind::Continue), "continue", 0..8)]);
        assert_lexer("in", &[(Ok(TokenKind::In), "in", 0..2)]);
        assert_lexer("is", &[(Ok(TokenKind::Is), "is", 0..2)]);
        assert_lexer("use", &[(Ok(TokenKind::Use), "use", 0..3)]);
        assert_lexer("public", &[(Ok(TokenKind::Public), "public", 0..6)]);
        assert_lexer("internal", &[(Ok(TokenKind::Internal), "internal", 0..8)]);
        assert_lexer("private", &[(Ok(TokenKind::Private), "private", 0..7)]);
        assert_lexer("none", &[(Ok(TokenKind::None), "none", 0..4)]);
        assert_lexer("try", &[(Ok(TokenKind::Try), "try", 0..3)]);
        assert_lexer("catch", &[(Ok(TokenKind::Catch), "catch", 0..5)]);
        assert_lexer("throw", &[(Ok(TokenKind::Throw), "throw", 0..5)]);
        assert_lexer("struct", &[(Ok(TokenKind::Struct), "struct", 0..6)]);
        assert_lexer("enum", &[(Ok(TokenKind::Enum), "enum", 0..4)]);
        assert_lexer("and", &[(Ok(TokenKind::And), "and", 0..3)]);
        assert_lexer("or", &[(Ok(TokenKind::Or), "or", 0..2)]);
        assert_lexer("mut", &[(Ok(TokenKind::Mut), "mut", 0..3)]);
        assert_lexer("on", &[(Ok(TokenKind::On), "on", 0..2)]);
        assert_lexer("impl", &[(Ok(TokenKind::Impl), "impl", 0..4)]);
        assert_lexer("self", &[(Ok(TokenKind::Self_), "self", 0..4)]);
    }

    #[test]
    fn test_structural() {
        // Structural
        assert_lexer("_", &[(Ok(TokenKind::Underscore), "_", 0..1)]);
        assert_lexer(":", &[(Ok(TokenKind::Colon), ":", 0..1)]);
        assert_lexer(";", &[(Ok(TokenKind::Semicolon), ";", 0..1)]);
        assert_lexer("(", &[(Ok(TokenKind::LParen), "(", 0..1)]);
        assert_lexer(")", &[(Ok(TokenKind::RParen), ")", 0..1)]);
        assert_lexer("{", &[(Ok(TokenKind::LBrace), "{", 0..1)]);
        assert_lexer("}", &[(Ok(TokenKind::RBrace), "}", 0..1)]);
        assert_lexer("[", &[(Ok(TokenKind::LBracket), "[", 0..1)]);
        assert_lexer("]", &[(Ok(TokenKind::RBracket), "]", 0..1)]);
        assert_lexer(",", &[(Ok(TokenKind::Comma), ",", 0..1)]);
        assert_lexer(".", &[(Ok(TokenKind::Dot), ".", 0..1)]);
    }

    #[test]
    fn test_operators() {
        // Operators
        assert_lexer("=", &[(Ok(TokenKind::Eq), "=", 0..1)]);
        assert_lexer("==", &[(Ok(TokenKind::EqEq), "==", 0..2)]);
        assert_lexer("!=", &[(Ok(TokenKind::Neq), "!=", 0..2)]);
        assert_lexer("<", &[(Ok(TokenKind::Lt), "<", 0..1)]);
        assert_lexer(">", &[(Ok(TokenKind::Gt), ">", 0..1)]);
        assert_lexer("<=", &[(Ok(TokenKind::LtEq), "<=", 0..2)]);
        assert_lexer(">=", &[(Ok(TokenKind::GtEq), ">=", 0..2)]);
        assert_lexer("+", &[(Ok(TokenKind::Plus), "+", 0..1)]);
        assert_lexer("-", &[(Ok(TokenKind::Minus), "-", 0..1)]);
        assert_lexer("*", &[(Ok(TokenKind::Star), "*", 0..1)]);
        assert_lexer("**", &[(Ok(TokenKind::StarStar), "**", 0..2)]);
        assert_lexer("/", &[(Ok(TokenKind::Slash), "/", 0..1)]);
        assert_lexer("%", &[(Ok(TokenKind::Percent), "%", 0..1)]);
        assert_lexer("!", &[(Ok(TokenKind::Bang), "!", 0..1)]);

        // Assignment operators
        assert_lexer("+=", &[(Ok(TokenKind::PlusEq), "+=", 0..2)]);
        assert_lexer("-=", &[(Ok(TokenKind::MinusEq), "-=", 0..2)]);
        assert_lexer("*=", &[(Ok(TokenKind::StarEq), "*=", 0..2)]);
        assert_lexer("/=", &[(Ok(TokenKind::SlashEq), "/=", 0..2)]);
    }

    #[test]
    fn test_parenthesis() {
        assert_lexer(
            "()",
            &[
                (Ok(TokenKind::LParen), "(", 0..1),
                (Ok(TokenKind::RParen), ")", 1..2),
            ],
        );
    }

    #[test]
    fn test_brackets() {
        assert_lexer(
            "[]",
            &[
                (Ok(TokenKind::LBracket), "[", 0..1),
                (Ok(TokenKind::RBracket), "]", 1..2),
            ],
        );
    }

    #[test]
    fn test_curly_braces() {
        assert_lexer(
            "{}",
            &[
                (Ok(TokenKind::LBrace), "{", 0..1),
                (Ok(TokenKind::RBrace), "}", 1..2),
            ],
        );
    }

    #[test]
    fn test_comment() {
        assert_lexer("//", &[(Ok(TokenKind::Comment("//")), "//", 0..2)]);

        assert_lexer(
            "// testing",
            &[(Ok(TokenKind::Comment("// testing")), "// testing", 0..10)],
        );

        assert_lexer(
            "// comment 1
            // comment 2",
            &[
                (
                    Ok(TokenKind::Comment("// comment 1")),
                    "// comment 1",
                    0..12,
                ),
                (
                    Ok(TokenKind::Comment("// comment 2")),
                    "// comment 2",
                    25..37,
                ),
            ],
        );

        assert_lexer(
            "// comment 1
            //
            // comment 2",
            &[
                (
                    Ok(TokenKind::Comment("// comment 1")),
                    "// comment 1",
                    0..12,
                ),
                (Ok(TokenKind::Comment("//")), "//", 25..27),
                (
                    Ok(TokenKind::Comment("// comment 2")),
                    "// comment 2",
                    40..52,
                ),
            ],
        );

        assert_lexer(
            "// comment 1  ",
            &[(
                Ok(TokenKind::Comment("// comment 1  ")),
                "// comment 1  ",
                0..14,
            )],
        );
    }

    #[test]
    fn test_identifier() {
        assert_lexer("foo", &[(Ok(TokenKind::Ident("foo")), "foo", 0..3)]);
        assert_lexer("FOO", &[(Ok(TokenKind::Ident("FOO")), "FOO", 0..3)]);
        assert_lexer("_", &[(Ok(TokenKind::Underscore), "_", 0..1)]);
        assert_lexer("_1", &[(Ok(TokenKind::Ident("_1")), "_1", 0..2)]);
        assert_lexer(
            "_test1",
            &[(Ok(TokenKind::Ident("_test1")), "_test1", 0..6)],
        );
    }

    #[test]
    fn test_number() {
        assert_lexer(
            "0 0000",
            &[
                (Ok(TokenKind::IntLiteral(0)), "0", 0..1),
                (Ok(TokenKind::IntLiteral(0)), "0000", 2..6),
            ],
        );

        assert_lexer(
            "10.0 10.123",
            &[
                (Ok(TokenKind::FloatLiteral(10.0)), "10.0", 0..4),
                (Ok(TokenKind::FloatLiteral(10.123)), "10.123", 5..11),
            ],
        );

        assert_lexer(
            "0_0 00_00 1_000_000",
            &[
                (Ok(TokenKind::IntLiteral(0)), "0_0", 0..3),
                (Ok(TokenKind::IntLiteral(0)), "00_00", 4..9),
                (Ok(TokenKind::IntLiteral(1_000_000)), "1_000_000", 10..19),
            ],
        );

        assert_lexer("-1", &[(Ok(TokenKind::IntLiteral(-1)), "-1", 0..2)]);
    }

    #[test]
    fn test_string() {
        assert_lexer(
            "\"\"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 1..2),
            ],
        );
        assert_lexer(
            "\"    \"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (Ok(TokenKind::StringLiteral("    ")), "    ", 1..5),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 5..6),
            ],
        );

        assert_lexer(
            "\"hello world\"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (
                    Ok(TokenKind::StringLiteral("hello world")),
                    "hello world",
                    1..12,
                ),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 12..13),
            ],
        );

        assert_lexer(
            "\"hello\nworld\"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (
                    Ok(TokenKind::StringLiteral("hello\nworld")),
                    "hello\nworld",
                    1..12,
                ),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 12..13),
            ],
        );

        assert_lexer(
            "\"",
            &[(Ok(TokenKind::StringInterpolationStart), "\"", 0..1)],
        );
    }

    #[test]
    fn test_string_interpolation() {
        assert_lexer(
            "\"{1}\"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (Ok(TokenKind::LBrace), "{", 1..2),
                (Ok(TokenKind::IntLiteral(1)), "1", 2..3),
                (Ok(TokenKind::RBrace), "}", 3..4),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 4..5),
            ],
        );
        assert_lexer(
            "\"hi {1} there {2} stop\"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (Ok(TokenKind::StringLiteral("hi ")), "hi ", 1..4),
                (Ok(TokenKind::LBrace), "{", 4..5),
                (Ok(TokenKind::IntLiteral(1)), "1", 5..6),
                (Ok(TokenKind::RBrace), "}", 6..7),
                (Ok(TokenKind::StringLiteral(" there ")), " there ", 7..14),
                (Ok(TokenKind::LBrace), "{", 14..15),
                (Ok(TokenKind::IntLiteral(2)), "2", 15..16),
                (Ok(TokenKind::RBrace), "}", 16..17),
                (Ok(TokenKind::StringLiteral(" stop")), " stop", 17..22),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 22..23),
            ],
        );
        assert_lexer(
            "\"{\"{\"{\"hi\"}\"}\"}\"",
            &[
                (Ok(TokenKind::StringInterpolationStart), "\"", 0..1),
                (Ok(TokenKind::LBrace), "{", 1..2),
                (Ok(TokenKind::StringInterpolationStart), "\"", 2..3),
                (Ok(TokenKind::LBrace), "{", 3..4),
                (Ok(TokenKind::StringInterpolationStart), "\"", 4..5),
                (Ok(TokenKind::LBrace), "{", 5..6),
                (Ok(TokenKind::StringInterpolationStart), "\"", 6..7),
                (Ok(TokenKind::StringLiteral("hi")), "hi", 7..9),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 9..10),
                (Ok(TokenKind::RBrace), "}", 10..11),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 11..12),
                (Ok(TokenKind::RBrace), "}", 12..13),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 13..14),
                (Ok(TokenKind::RBrace), "}", 14..15),
                (Ok(TokenKind::StringInterpolationEnd), "\"", 15..16),
            ],
        );
    }
}
