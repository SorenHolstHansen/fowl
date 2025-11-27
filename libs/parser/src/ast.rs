use span::Span;

#[derive(Debug, Clone)]
pub struct Program<'src> {
    pub declarations: Vec<Declaration<'src>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ident<'src> {
    pub inner: &'src str,
    pub span: Span<'src>,
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

impl BinaryOp {
    pub fn from_op(op: Op) -> Option<BinaryOp> {
        match op {
            Op::Add => Some(BinaryOp::Add),
            Op::Sub => Some(BinaryOp::Sub),
            Op::Mul => Some(BinaryOp::Mul),
            Op::Div => Some(BinaryOp::Div),
            Op::Mod => Some(BinaryOp::Mod),
            Op::Exp => todo!(),
            Op::And => Some(BinaryOp::And),
            Op::Or => Some(BinaryOp::Or),
            Op::Lt => Some(BinaryOp::Lt),
            Op::LtEq => Some(BinaryOp::LtEq),
            Op::Gt => Some(BinaryOp::Gt),
            Op::GtEq => Some(BinaryOp::GtEq),
            Op::Ne => Some(BinaryOp::Ne),
            Op::Eq => Some(BinaryOp::Eq),
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
pub struct Call<'src> {
    pub callee: Box<Expr<'src>>,
    pub args: Vec<Expr<'src>>,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'src> {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    /// None if empty string
    StringLiteral(Option<&'src str>),
    StringInterpolation(Vec<Expr<'src>>),
    Ident(Ident<'src>),
    Binary {
        op: BinaryOp,
        left: Box<Expr<'src>>,
        right: Box<Expr<'src>>,
    },
    // Unary operations
    Unary {
        op: UnaryOp,
        expr: Box<Expr<'src>>,
    },

    Call(Call<'src>),
    StructInstance {
        name: Ident<'src>,
        fields: Vec<(Ident<'src>, Expr<'src>)>, // field name -> value
    },
    Member {
        object: Box<Expr<'src>>,
        field: Ident<'src>,
    },
}

#[derive(Debug, Clone)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub span: Span<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind<'src> {
    Ident(Ident<'src>),
    Int,
    Float,
    String,
    Bool,
    Void,
}

impl std::fmt::Display for TypeKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Ident(ident) => write!(f, "{}", ident.inner),
            TypeKind::Int => write!(f, "int"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::String => write!(f, "string"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Void => write!(f, "void"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Type<'src> {
    pub span: Span<'src>,
    pub kind: TypeKind<'src>,
}

#[derive(Debug, Clone)]
pub struct Param<'src> {
    pub span: Span<'src>,
    pub name: Ident<'src>,
    pub ty: Type<'src>,
    pub default: Option<Expr<'src>>,
}

#[derive(Debug, Clone)]
pub struct Block<'src> {
    pub span: Span<'src>,
    pub statements: Vec<Statement<'src>>,
}

impl<'src> Block<'src> {
    pub fn set_span(mut self, span: Span<'src>) -> Self {
        self.span = span;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Vis {
    Public,
    Internal,
    Private,
}

#[derive(Debug, Clone)]
pub struct Function<'src> {
    pub span: Span<'src>,
    pub name: Ident<'src>,
    pub params: Vec<Param<'src>>,
    pub ret_ty: Type<'src>,
    pub body: Block<'src>,
    pub vis: Vis,
}

impl<'src> Function<'src> {
    pub fn set_vis(mut self, vis: Vis) -> Self {
        self.vis = vis;
        self
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariant<'src> {
    pub span: Span<'src>,
    pub name: Ident<'src>,
    pub fields: Vec<Type<'src>>,
}

#[derive(Debug, Clone)]
pub struct Struct<'src> {
    pub span: Span<'src>,
    pub name: Ident<'src>,
    pub fields: Vec<(Ident<'src>, Type<'src>)>,
    pub vis: Vis,
}

#[derive(Debug, Clone)]
pub struct Enum<'src> {
    pub span: Span<'src>,
    pub name: Ident<'src>,
    pub variants: Vec<EnumVariant<'src>>,
    pub vis: Vis,
}

#[derive(Debug, Clone)]
pub enum Statement<'src> {
    Let {
        name: Ident<'src>,
        ty: Option<Type<'src>>,
        expr: Expr<'src>,
        mutable: bool,
        span: Span<'src>,
    },
    Return {
        span: Span<'src>,
        expr: Option<Expr<'src>>,
    },
    Function(Function<'src>),
    // Type definitions
    Struct(Struct<'src>),
    Enum(Enum<'src>),
    Expr(Expr<'src>),
}

impl<'src> Statement<'src> {
    pub fn span(&self) -> &Span<'src> {
        match self {
            Statement::Let { span, .. } => span,
            Statement::Return { span, .. } => span,
            Statement::Function(function) => &function.span,
            Statement::Struct(strct) => &strct.span,
            Statement::Enum(e) => &e.span,
            Statement::Expr(expr) => &expr.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Use<'src> {
    pub import: Vec<Ident<'src>>,
    pub span: Span<'src>,
}

#[derive(Debug, Clone)]
pub enum Declaration<'src> {
    Struct(Struct<'src>),
    Enum(Enum<'src>),
    Function(Function<'src>),
    Use(Use<'src>),
}
