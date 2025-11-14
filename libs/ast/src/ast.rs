use error::Diagnostic;
use span::Span;

#[derive(Debug, Clone)]
pub struct Program<'source> {
    pub declarations: Vec<Declaration<'source>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ident<'source> {
    pub inner: &'source str,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,

    Eq,
    Ne,
    Lt,
    Gt,
    LtEq,
    GtEq,

    And,
    Or,

    Bang,
    Call,
    StructInstance,
    // Indexing into a struct instance, i.e. my_struct.my_field
    FieldIndex,
}

impl Op {
    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        let res = match self {
            Op::And | Op::Or => (3, 4),
            Op::Ne | Op::Eq | Op::Lt | Op::LtEq | Op::Gt | Op::GtEq => (5, 6),
            Op::Add | Op::Sub => (7, 8),
            Op::Mul | Op::Div | Op::Mod => (9, 10),
            Op::Exp => (11, 12),
            Op::FieldIndex => (16, 15),
            _ => return None,
        };
        Some(res)
    }

    pub fn prefix_binding_power(&self) -> ((), u8) {
        match self {
            // Op::Return => ((), 1),
            Op::Bang | Op::Sub => ((), 11),
            _ => panic!("bad op: {:?}", self),
        }
    }

    pub fn postfix_binding_power(&self) -> Option<(u8, ())> {
        match self {
            Op::Call => Some((13, ())),
            Op::StructInstance => Some((15, ())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,

    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    And,
    Or,
}

impl TryFrom<Op> for BinaryOp {
    type Error = Diagnostic;

    fn try_from(op: Op) -> Result<Self, Self::Error> {
        match op {
            Op::Add => Ok(BinaryOp::Add),
            Op::Sub => Ok(BinaryOp::Sub),
            Op::Mul => Ok(BinaryOp::Mul),
            Op::Div => Ok(BinaryOp::Div),
            Op::Mod => Ok(BinaryOp::Mod),
            Op::Exp => todo!(),
            Op::And => Ok(BinaryOp::And),
            Op::Or => Ok(BinaryOp::Or),
            Op::Lt => Ok(BinaryOp::Lt),
            Op::LtEq => Ok(BinaryOp::LtEq),
            Op::Gt => Ok(BinaryOp::Gt),
            Op::GtEq => Ok(BinaryOp::GtEq),
            Op::Ne => Ok(BinaryOp::Ne),
            Op::Eq => Ok(BinaryOp::Eq),
            Op::Bang => todo!(),
            Op::Call => todo!(),
            Op::StructInstance => todo!(),
            Op::FieldIndex => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Expr<'source> {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    /// None if empty string
    StringLiteral(Option<&'source str>),
    StringInterpolation(Vec<Expr<'source>>),
    Ident(Ident<'source>),
    Binary {
        op: BinaryOp,
        left: Box<Expr<'source>>,
        right: Box<Expr<'source>>,
    },
    // Unary operations
    Unary {
        op: UnaryOp,
        expr: Box<Expr<'source>>,
    },

    Call {
        func: Box<Expr<'source>>,
        args: Vec<Expr<'source>>,
    },
    StructInstance {
        name: Ident<'source>,
        fields: Vec<(Ident<'source>, Expr<'source>)>, // field name -> value
    },
    Member {
        object: Box<Expr<'source>>,
        field: Ident<'source>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind<'source> {
    Ident(Ident<'source>),
    Int,
    Float,
    String,
    Bool,
    Void,
    Generic {
        base: Ident<'source>,
        args: Vec<Type<'source>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type<'source> {
    pub span: Span,
    pub kind: TypeKind<'source>,
}

#[derive(Debug, Clone)]
pub struct Param<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub ty: Type<'source>,
    pub default: Option<Expr<'source>>,
}

#[derive(Debug, Clone)]
pub struct Block<'source> {
    pub span: Span,
    pub statements: Vec<Statement<'source>>,
}

impl<'source> Block<'source> {
    pub fn set_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Function<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub params: Vec<Param<'source>>,
    pub ret_ty: Type<'source>,
    pub body: Block<'source>,
    pub public: bool,
}

impl<'source> Function<'source> {
    pub fn set_public(mut self, public: bool) -> Self {
        self.public = public;
        self
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariant<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub fields: Vec<Type<'source>>,
}

#[derive(Debug, Clone)]
pub struct Struct<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub fields: Vec<(Ident<'source>, Type<'source>)>,
    pub public: bool,
    // generics: Vec<String>, // Generic type parameters
}

#[derive(Debug, Clone)]
pub struct Enum<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub variants: Vec<EnumVariant<'source>>,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub enum Statement<'source> {
    Let {
        name: Ident<'source>,
        ty: Option<Type<'source>>,
        expr: Expr<'source>,
        mutable: bool,
        span: Span,
    },
    Return {
        span: Span,
        expr: Option<Expr<'source>>,
    },
    Function(Function<'source>),
    // Type definitions
    Struct(Struct<'source>),
    Enum(Enum<'source>),
    Expr(Expr<'source>),
}

#[derive(Debug, Clone)]
pub struct UseImport<'source> {
    pub module: Ident<'source>,
    pub alias: Option<Ident<'source>>,
}

#[derive(Debug, Clone)]
pub enum Declaration<'source> {
    Struct(Struct<'source>),
    Enum(Enum<'source>),
    Function(Function<'source>),
    Use {
        imports: Vec<UseImport<'source>>,
    },
    PubUse {
        module: &'source str,
        item: Option<&'source str>, // None means re-export all public items
        alias: Option<&'source str>, // Optional rename
    },
}
