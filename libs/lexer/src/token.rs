use span::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind<'src> {
    // Keywords
    /// `fn` keyword
    Fn,
    /// `let` keyword
    Let,
    /// `return` keyword
    Return,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `for` keyword
    For,
    /// `break` keyword
    Break,
    /// `continue` keyword
    Continue,
    /// `in` keyword
    In,
    /// `use` keyword
    Use,
    /// `public` keyword
    Public,
    /// `internal` keyword
    Internal,
    /// `private` keyword
    Private,
    /// `match` keyword
    Match,
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
    /// `self` keyword
    Self_,

    // Types
    /// `int` type
    Int,
    /// `float` type
    Float,
    /// `string` type
    String,
    /// `bool` type
    Bool,
    /// `void` type
    Void,

    /// Identifier
    Ident(&'src str),

    // Literals
    /// Int literal
    IntLiteral(i64),
    /// Float literal
    FloatLiteral(f64),
    /// Bool literal
    BoolLiteral(bool),

    /// Start of string interpolation
    StringInterpolationStart,
    /// End of string interpolation
    StringInterpolationEnd,
    /// A string literal
    StringLiteral(&'src str),

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

    Eof,
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span<'src>,
}

impl std::fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Public => write!(f, "public"),
            TokenKind::Internal => write!(f, "internal"),
            TokenKind::Private => write!(f, "private"),
            TokenKind::Match => write!(f, "match"),
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
            TokenKind::Self_ => write!(f, "self"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Void => write!(f, "void"),
            TokenKind::Ident(i) => write!(f, "Ident({})", i),
            TokenKind::IntLiteral(i) => write!(f, "IntLiteral({})", i),
            TokenKind::FloatLiteral(fl) => write!(f, "FloatLiteral({})", fl),
            TokenKind::BoolLiteral(b) => write!(f, "BoolLiteral({})", b),
            TokenKind::StringInterpolationStart => write!(f, "StringInterpolationStart"),
            TokenKind::StringInterpolationEnd => write!(f, "StringInterpolationEnd"),
            TokenKind::StringLiteral(s) => write!(f, "StringLiteral(\"{}\")", s),
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
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}
