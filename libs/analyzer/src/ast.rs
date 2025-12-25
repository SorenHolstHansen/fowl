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

impl<'src> From<parser::ast::Ident<'src>> for Ident<'src> {
    fn from(value: parser::ast::Ident<'src>) -> Self {
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
pub struct Call<'src> {
    pub callee: String,
    pub args: Vec<Expr<'src>>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Expr<'src> {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    /// None if empty string
    StringLiteral(Option<&'src str>),
    StringInterpolation(Vec<Expr<'src>>),
    Ident {
        ident: Ident<'src>,
        ty: TypeKind<'src>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr<'src>>,
        right: Box<Expr<'src>>,
        ty: TypeKind<'src>,
    },
    // Unary operations
    Unary {
        op: UnaryOp,
        expr: Box<Expr<'src>>,
        ty: TypeKind<'src>,
    },

    Call {
        call: Call<'src>,
        ty: TypeKind<'src>,
    },
    StructInstance {
        name: Ident<'src>,
        fields: Vec<(Ident<'src>, Expr<'src>)>, // field name -> value
    },
    Member {
        object: Box<Expr<'src>>,
        field: Ident<'src>,
        ty: TypeKind<'src>,
    },
    If {
        ty: TypeKind<'src>,
        cond: Box<Expr<'src>>,
        then: Block<'src>,
        else_if_blocks: Vec<(Expr<'src>, Block<'src>)>,
        else_block: Option<Block<'src>>,
    },
}

impl<'src> Expr<'src> {
    pub fn ty(&self) -> &TypeKind<'src> {
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
            Expr::If { ty, .. } => ty,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub ty: TypeKind<'src>,
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
pub struct Function<'src> {
    pub span: Span<'src>,
    pub name: String,
    pub params: Vec<Param<'src>>,
    pub ret_ty: TypeKind<'src>,
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
        ty: TypeKind<'src>,
        expr: Expr<'src>,
        mutable: bool,
        span: Span<'src>,
    },
    Assign {
        name: Ident<'src>,
        expr: Expr<'src>,
        span: Span<'src>,
    },
    Return {
        span: Span<'src>,
        expr: Option<Expr<'src>>,
        ty: TypeKind<'src>,
    },
    Function(Function<'src>),
    Struct(Struct<'src>),
    Enum(Enum<'src>),
    Expr(Expr<'src>),
    ForLoop {
        span: Span<'src>,
        cond: Expr<'src>,
        block: Block<'src>,
    },
}

impl<'src> Statement<'src> {
    pub fn span(&'_ self) -> Span<'_> {
        match self {
            Statement::Let { span, .. } => *span,
            Statement::Assign { span, .. } => *span,
            Statement::Return { span, .. } => *span,
            Statement::Function(function) => function.span,
            Statement::Struct(s) => s.span,
            Statement::Enum(e) => e.span,
            Statement::Expr(_) => todo!(),
            Statement::ForLoop { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UseImport<'src> {
    pub module: Ident<'src>,
    pub alias: Option<Ident<'src>>,
}

#[derive(Debug, Clone)]
pub enum Declaration<'src> {
    Struct(Struct<'src>),
    Enum(Enum<'src>),
    Function(Function<'src>),
    Use { imports: Vec<UseImport<'src>> },
}
