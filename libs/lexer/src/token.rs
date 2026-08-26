use std::ops::Range;

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
    Ident(&'src str),

    // Literals
    /// Int literal, the str kept as it might parse differently based on the desired int type
    IntLiteral(&'src str),
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

    /// Whitespace
    Whitespace(&'src str),

    /// Comments
    Comment(&'src str),

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Range<usize>,
}
