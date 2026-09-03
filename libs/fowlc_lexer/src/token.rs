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
    Ident,

    // Literals
    /// Int literal, the str kept as it might parse differently based on the desired int type
    IntLiteral,
    /// Float literal
    FloatLiteral,
    /// Bool literal
    BoolLiteral,

    /// Start of string interpolation
    StringInterpolationStart,
    /// End of string interpolation
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
    LParen,
    /// token `)`
    RParen,
    /// token `{`
    LBrace,
    /// token `}`
    RBrace,
    /// token `[`
    LBracket,
    /// token `]`
    RBracket,
    /// token `,`
    Comma,
    /// token `.`
    Dot,

    // Operators
    /// token `=`
    Eq,
    /// token `==`
    EqEq,
    /// token `!=`
    Neq,
    /// token `<`
    Lt,
    /// token `>`
    Gt,
    /// token `<=`
    LtEq,
    /// token `>=`
    GtEq,
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
    PlusEq,
    /// token `-=`
    MinusEq,
    /// token `*=`
    StarEq,
    /// token `/=`
    SlashEq,

    /// Whitespace
    Whitespace,

    /// Comments
    Comment,

    /// Non-tokens, but used for ast groups and tokens
    Declaration,
    Vis,
    Type,
    FnParameters,
    FnParameter,
    ReturnType,
    Block,
    Statement,
    Expression,
    Operator,
    ParenExpr,
    BinaryOp,

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
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::IntLiteral => write!(f, "int literal"),
            TokenKind::FloatLiteral => write!(f, "float literal"),
            TokenKind::BoolLiteral => write!(f, "bool literal"),
            TokenKind::StringInterpolationStart => write!(f, "\""),
            TokenKind::StringInterpolationEnd => write!(f, "\""),
            TokenKind::StringLiteral => write!(f, "string literal"),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Eq => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::Neq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::StarStar => write!(f, "**"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Declaration => write!(f, "declaration"),
            TokenKind::Vis => write!(f, "visibility"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::FnParameters => write!(f, "function parameters"),
            TokenKind::FnParameter => write!(f, "function parameter"),
            TokenKind::ReturnType => write!(f, "function return type"),
            TokenKind::Block => write!(f, "block"),
            TokenKind::Statement => write!(f, "statement"),
            TokenKind::Expression => write!(f, "expression"),
            TokenKind::Operator => write!(f, "operator"),
            TokenKind::ParenExpr => write!(f, "parenthesised expression"),
            TokenKind::BinaryOp => write!(f, "binary oparation"),
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
    TokenKind::EqEq,
    TokenKind::Lt,
    TokenKind::LtEq,
    TokenKind::Gt,
    TokenKind::GtEq,
];

pub const OPERATOR_PRECEDENCE: &[(TokenKind, u8)] = &[
    (TokenKind::Eq, 1),
    (TokenKind::PlusEq, 1),
    (TokenKind::MinusEq, 1),
    (TokenKind::StarEq, 1),
    (TokenKind::SlashEq, 1),
    (TokenKind::EqEq, 4),
    (TokenKind::Lt, 5),
    (TokenKind::Gt, 5),
    (TokenKind::LtEq, 6),
    (TokenKind::GtEq, 6),
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
