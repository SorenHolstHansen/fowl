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

impl<'source> From<parser::ast::Ident<'source>> for Ident<'source> {
    fn from(value: parser::ast::Ident<'source>) -> Self {
        Ident {
            inner: value.inner,
            span: value.span,
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

impl From<parser::ast::BinaryOp> for BinaryOp {
    fn from(value: parser::ast::BinaryOp) -> Self {
        match value {
            parser::ast::BinaryOp::Add => BinaryOp::Add,
            parser::ast::BinaryOp::Sub => BinaryOp::Sub,
            parser::ast::BinaryOp::Mul => BinaryOp::Mul,
            parser::ast::BinaryOp::Div => BinaryOp::Div,
            parser::ast::BinaryOp::Mod => BinaryOp::Mod,
            parser::ast::BinaryOp::Exp => BinaryOp::Exp,
            parser::ast::BinaryOp::Eq => BinaryOp::Eq,
            parser::ast::BinaryOp::Ne => BinaryOp::Ne,
            parser::ast::BinaryOp::Lt => BinaryOp::Lt,
            parser::ast::BinaryOp::Gt => BinaryOp::Gt,
            parser::ast::BinaryOp::LtEq => BinaryOp::LtEq,
            parser::ast::BinaryOp::GtEq => BinaryOp::GtEq,
            parser::ast::BinaryOp::And => BinaryOp::And,
            parser::ast::BinaryOp::Or => BinaryOp::Or,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub struct Call<'source> {
    pub callee: Box<Expr<'source>>,
    pub args: Vec<Expr<'source>>,
}

#[derive(Debug, Clone)]
pub enum Expr<'source> {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    /// None if empty string
    StringLiteral(Option<&'source str>),
    StringInterpolation(Vec<Expr<'source>>),
    Ident {
        ident: Ident<'source>,
        ty: TypeKind<'source>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr<'source>>,
        right: Box<Expr<'source>>,
        ty: TypeKind<'source>,
    },
    // Unary operations
    Unary {
        op: UnaryOp,
        expr: Box<Expr<'source>>,
        ty: TypeKind<'source>,
    },

    Call {
        call: Call<'source>,
        ty: TypeKind<'source>,
    },
    StructInstance {
        name: Ident<'source>,
        fields: Vec<(Ident<'source>, Expr<'source>)>, // field name -> value
    },
    Member {
        object: Box<Expr<'source>>,
        field: Ident<'source>,
        ty: TypeKind<'source>,
    },
}

impl<'source> Expr<'source> {
    pub fn ty(&self) -> &TypeKind<'source> {
        match self {
            Expr::IntLiteral(_) => &TypeKind::Int,
            Expr::FloatLiteral(_) => &TypeKind::Float,
            Expr::BoolLiteral(_) => &TypeKind::Bool,
            Expr::StringLiteral(_) => &TypeKind::String,
            Expr::StringInterpolation(_) => &TypeKind::String,
            Expr::Ident { ty, .. } => ty,
            Expr::Binary { ty, .. } => ty,
            Expr::Unary { ty, .. } => ty,
            Expr::Call { ty, .. } => ty,
            Expr::StructInstance { .. } => todo!(),
            Expr::Member { ty, .. } => ty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind<'source> {
    Ident(Ident<'source>),
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
    pub ty: TypeKind<'source>,
}

#[derive(Debug, Clone, Copy)]
pub enum Vis {
    Public,
    Internal,
    Private,
}

impl From<parser::ast::Vis> for Vis {
    fn from(value: parser::ast::Vis) -> Self {
        match value {
            parser::ast::Vis::Public => Vis::Public,
            parser::ast::Vis::Internal => Vis::Internal,
            parser::ast::Vis::Private => Vis::Private,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub params: Vec<Param<'source>>,
    pub ret_ty: TypeKind<'source>,
    pub body: Block<'source>,
    pub vis: Vis,
}

impl<'source> Function<'source> {
    pub fn set_vis(mut self, vis: Vis) -> Self {
        self.vis = vis;
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
    pub vis: Vis,
}

#[derive(Debug, Clone)]
pub struct Enum<'source> {
    pub span: Span,
    pub name: Ident<'source>,
    pub variants: Vec<EnumVariant<'source>>,
    pub vis: Vis,
}

#[derive(Debug, Clone)]
pub enum Statement<'source> {
    Let {
        name: Ident<'source>,
        ty: TypeKind<'source>,
        expr: Expr<'source>,
        mutable: bool,
        span: Span,
    },
    Return {
        span: Span,
        expr: Option<Expr<'source>>,
        ty: TypeKind<'source>,
    },
    Function(Function<'source>),
    // Type definitions
    Struct(Struct<'source>),
    Enum(Enum<'source>),
    Expr(Expr<'source>),
}

impl<'source> Statement<'source> {
    pub fn span(&self) -> Span {
        match self {
            Statement::Let { span, .. } => *span,
            Statement::Return { span, .. } => *span,
            Statement::Function(function) => function.span,
            Statement::Struct(s) => s.span,
            Statement::Enum(e) => e.span,
            Statement::Expr(_) => todo!(),
        }
    }
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
    Use { imports: Vec<UseImport<'source>> },
}
