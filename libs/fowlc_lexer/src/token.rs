use fowlc_span::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    // Keywords
    /// `fn` keyword
    Fn,
    /// `let` keyword
    Let,
    /// `return` keyword
    Return,
    /// `if` keyword
    If,
    /// `for` keyword
    For,
    /// `break` keyword
    Break,
    /// `continue` keyword
    Continue,
    /// `in` keyword
    In,
    /// `is` keyword
    Is,
    /// `use` keyword
    Use,
    /// `public` keyword
    Public,
    /// `internal` keyword
    Internal,
    /// `private` keyword
    Private,
    /// `none` keyword
    None,
    /// `try` keyword
    Try,
    /// `catch` keyword
    Catch,
    /// `throw` keyword
    Throw,
    /// `struct` keyword
    Struct,
    /// `enum` keyword
    Enum,
    /// `and` keyword
    And,
    /// `or` keyword
    Or,
    /// `mut` keyword
    Mut,
    /// `on` keyword
    On,
    /// `impl` keyword
    Impl,
    /// `self` keyword
    Self_,

    /// Identifier
    Identifier,

    // Literals
    /// Int literal, the str kept as it might parse differently based on the desired int type
    IntegerLiteral,
    /// Float literal
    FloatLiteral,
    /// Bool literal
    BoolLiteral,

    /// Start of string interpolation. This is just a '"'
    StringInterpolationStart,
    /// End of string interpolation. This is just a '"'
    StringInterpolationEnd,
    /// A string literal
    StringLiteral,

    // Structural
    /// token `_`
    Underscore,
    /// token `:`
    Colon,
    /// token `;`
    Semicolon,
    /// token `(`
    LeftParenthesis,
    /// token `)`
    RightParenthesis,
    /// token `{`
    LeftBrace,
    /// token `}`
    RightBrace,
    /// token `[`
    LeftBracket,
    /// token `]`
    RightBracket,
    /// token `,`
    Comma,
    /// token `.`
    Dot,

    // Operators
    /// token `=`
    Equal,
    /// token `==`
    EqualEqual,
    /// token `!=`
    NotEqual,
    /// token `<`
    LessThan,
    /// token `>`
    GreaterThan,
    /// token `<=`
    LessThanOrEqual,
    /// token `>=`
    GreaterThanOrEqual,
    /// token `+`
    Plus,
    /// token `-`
    Minus,
    /// token `*`
    Star,
    /// token `**`
    StarStar,
    /// token `/`
    Slash,
    /// token `%`
    Percent,
    /// token `!`
    Bang,

    // Assignment operators
    /// token `+=`
    PlusEqual,
    /// token `-=`
    MinusEqual,
    /// token `*=`
    StarEqual,
    /// token `/=`
    SlashEqual,

    /// Whitespace
    Whitespace,

    /// Comments
    Comment,

    /// Non-tokens, but used for ast groups and tokens
    Declaration,
    Visibility,
    Type,
    FnParameters,
    FnParameter,
    ReturnType,
    Block,
    Statement,
    Expression,
    Operator,
    ParenExpr,
    BinaryOperator,
    CallExpression,

    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Public => write!(f, "public"),
            TokenKind::Internal => write!(f, "internal"),
            TokenKind::Private => write!(f, "private"),
            TokenKind::None => write!(f, "none"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Throw => write!(f, "throw"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::On => write!(f, "on"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::Self_ => write!(f, "self"),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::IntegerLiteral => write!(f, "int literal"),
            TokenKind::FloatLiteral => write!(f, "float literal"),
            TokenKind::BoolLiteral => write!(f, "bool literal"),
            TokenKind::StringInterpolationStart => write!(f, "\""),
            TokenKind::StringInterpolationEnd => write!(f, "\""),
            TokenKind::StringLiteral => write!(f, "string literal"),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::LeftParenthesis => write!(f, "("),
            TokenKind::RightParenthesis => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::LessThanOrEqual => write!(f, "<="),
            TokenKind::GreaterThanOrEqual => write!(f, ">="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::StarStar => write!(f, "**"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::PlusEqual => write!(f, "+="),
            TokenKind::MinusEqual => write!(f, "-="),
            TokenKind::StarEqual => write!(f, "*="),
            TokenKind::SlashEqual => write!(f, "/="),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Declaration => write!(f, "declaration"),
            TokenKind::Visibility => write!(f, "visibility"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::FnParameters => write!(f, "function parameters"),
            TokenKind::FnParameter => write!(f, "function parameter"),
            TokenKind::ReturnType => write!(f, "function return type"),
            TokenKind::Block => write!(f, "block"),
            TokenKind::Statement => write!(f, "statement"),
            TokenKind::Expression => write!(f, "expression"),
            TokenKind::Operator => write!(f, "operator"),
            TokenKind::ParenExpr => write!(f, "parenthesised expression"),
            TokenKind::BinaryOperator => write!(f, "binary oparation"),
            TokenKind::CallExpression => write!(f, "call expression"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

pub const INFIX_OPERATORS: &[TokenKind] = &[
    TokenKind::And,
    TokenKind::Or,
    TokenKind::Plus,
    TokenKind::Minus,
    TokenKind::Star,
    TokenKind::Slash,
    TokenKind::EqualEqual,
    TokenKind::LessThan,
    TokenKind::LessThanOrEqual,
    TokenKind::GreaterThan,
    TokenKind::GreaterThanOrEqual,
];

pub const OPERATOR_PRECEDENCE: &[(TokenKind, u8)] = &[
    (TokenKind::Equal, 1),
    (TokenKind::PlusEqual, 1),
    (TokenKind::MinusEqual, 1),
    (TokenKind::StarEqual, 1),
    (TokenKind::SlashEqual, 1),
    (TokenKind::EqualEqual, 4),
    (TokenKind::LessThan, 5),
    (TokenKind::GreaterThan, 5),
    (TokenKind::LessThanOrEqual, 6),
    (TokenKind::GreaterThanOrEqual, 6),
    (TokenKind::Plus, 7),
    (TokenKind::Minus, 7),
    (TokenKind::Star, 8),
    (TokenKind::Slash, 8),
    (TokenKind::Bang, 9),
    (TokenKind::Dot, 10),
];

impl TokenKind {
    pub fn precedence(&self) -> u8 {
        OPERATOR_PRECEDENCE
            .iter()
            .find(|(k, _)| k == self)
            .map_or(0, |(_, p)| *p)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub span: Span<'src>,
}
