#[derive(Debug, Clone)]
pub struct Program<'src> {
    pub declarations: Vec<Declaration<'src>>,
}

// TODO: Consider having a top-level Ident, rather than one per ast
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ident<'src> {
    pub inner: &'src str,
}

impl<'src> From<analyzer::ast::Ident<'src>> for Ident<'src> {
    fn from(value: analyzer::ast::Ident<'src>) -> Self {
        Ident { inner: value.inner }
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

impl From<analyzer::ast::BinaryOp> for BinaryOp {
    fn from(value: analyzer::ast::BinaryOp) -> Self {
        match value {
            analyzer::ast::BinaryOp::Add => BinaryOp::Add,
            analyzer::ast::BinaryOp::Sub => BinaryOp::Sub,
            analyzer::ast::BinaryOp::Mul => BinaryOp::Mul,
            analyzer::ast::BinaryOp::Div => BinaryOp::Div,
            analyzer::ast::BinaryOp::Mod => BinaryOp::Mod,
            analyzer::ast::BinaryOp::Exp => BinaryOp::Exp,
            analyzer::ast::BinaryOp::Eq => BinaryOp::Eq,
            analyzer::ast::BinaryOp::Ne => BinaryOp::Ne,
            analyzer::ast::BinaryOp::Lt => BinaryOp::Lt,
            analyzer::ast::BinaryOp::Gt => BinaryOp::Gt,
            analyzer::ast::BinaryOp::LtEq => BinaryOp::LtEq,
            analyzer::ast::BinaryOp::GtEq => BinaryOp::GtEq,
            analyzer::ast::BinaryOp::And => BinaryOp::And,
            analyzer::ast::BinaryOp::Or => BinaryOp::Or,
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
        ty: TypeKind<'src>,
        left: Box<Expr<'src>>,
        right: Box<Expr<'src>>,
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
    // TODO: Perhaps unify Closure and Function?
    Closure {
        mangled_name: String,
        params: Vec<Param<'src>>,
        ret_ty: TypeKind<'src>,
        body: Block<'src>,
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
            // TODO: Closure should have another type
            Expr::Closure { ret_ty, .. } => ret_ty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind<'src> {
    Ident(Ident<'src>),
    Int,
    Float,
    String,
    Bool,
    Void,
    Fn(Vec<TypeKind<'src>>, Box<TypeKind<'src>>),
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
            TypeKind::Fn(params, ret_ty) => write!(
                f,
                "Fn({}) {}",
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                ret_ty
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type<'src> {
    pub kind: TypeKind<'src>,
}

#[derive(Debug, Clone)]
pub struct Param<'src> {
    pub name: Ident<'src>,
    pub ty: TypeKind<'src>,
    pub default: Option<Expr<'src>>,
}

#[derive(Debug, Clone)]
pub struct Block<'src> {
    pub statements: Vec<Statement<'src>>,
}

#[derive(Debug, Clone)]
pub struct Function<'src> {
    pub name: String,
    pub params: Vec<Param<'src>>,
    pub ret_ty: TypeKind<'src>,
    pub body: Block<'src>,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub struct EnumVariant<'src> {
    pub name: Ident<'src>,
    pub fields: Vec<Type<'src>>,
}

#[derive(Debug, Clone)]
pub struct Struct<'src> {
    pub name: Ident<'src>,
    pub fields: Vec<(Ident<'src>, Type<'src>)>,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub struct Enum<'src> {
    pub name: Ident<'src>,
    pub variants: Vec<EnumVariant<'src>>,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub enum Statement<'src> {
    Let {
        name: Ident<'src>,
        expr: Expr<'src>,
        mutable: bool,
    },
    Assign {
        name: Ident<'src>,
        expr: Expr<'src>,
    },
    Return {
        expr: Option<Expr<'src>>,
    },
    Function(Function<'src>),
    Struct(Struct<'src>),
    Enum(Enum<'src>),
    Expr(Expr<'src>),
    ForLoop {
        cond: Option<Expr<'src>>,
        block: Block<'src>,
    },
    Break,
    Continue,
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
