use crate::ast as bir_ast;
use analyzer::ast as a_ast;

pub fn bir<'src>(program: a_ast::Program<'src>) -> bir_ast::Program<'src> {
    let mut declarations = Vec::new();
    for declaration in program.declarations {
        declarations.push(visit_declaration(declaration));
    }

    bir_ast::Program { declarations }
}

fn visit_declaration<'src>(declaration: a_ast::Declaration<'src>) -> bir_ast::Declaration<'src> {
    match declaration {
        a_ast::Declaration::Struct(_) => todo!(),
        a_ast::Declaration::Enum(_) => todo!(),
        a_ast::Declaration::Function(function) => {
            bir_ast::Declaration::Function(visit_function(&function))
        }
    }
}

fn visit_function<'src>(function: &a_ast::Function<'src>) -> bir_ast::Function<'src> {
    let params = function
        .params
        .iter()
        .map(|param| bir_ast::Param {
            name: param.name.into(),
            ty: visit_type(&param.ty.kind),
            default: param.default.as_ref().map(|def| visit_expr(def)),
        })
        .collect();
    bir_ast::Function {
        params,
        ret_ty: visit_type(&function.ret_ty),
        body: visit_block(&function.body),
        public: function.vis == a_ast::Vis::Public,
        name: function.name.clone(),
    }
}

fn visit_type<'src>(ty: &a_ast::TypeKind<'src>) -> bir_ast::TypeKind<'src> {
    match ty {
        a_ast::TypeKind::Ident(ident) => bir_ast::TypeKind::Ident((*ident).into()),
        a_ast::TypeKind::Int => bir_ast::TypeKind::Int,
        a_ast::TypeKind::Float => bir_ast::TypeKind::Float,
        a_ast::TypeKind::String => bir_ast::TypeKind::String,
        a_ast::TypeKind::Bool => bir_ast::TypeKind::Bool,
        a_ast::TypeKind::Void => bir_ast::TypeKind::Void,
        a_ast::TypeKind::Fn(params, ret_ty) => bir_ast::TypeKind::Fn(
            params.iter().map(|p| visit_type(p)).collect(),
            Box::new(visit_type(ret_ty)),
        ),
    }
}

fn visit_block<'src>(block: &a_ast::Block<'src>) -> bir_ast::Block<'src> {
    let mut statements = Vec::new();
    for statement in &block.statements {
        statements.push(visit_statement(statement));
    }

    bir_ast::Block { statements }
}

fn visit_statement<'src>(statement: &a_ast::Statement<'src>) -> bir_ast::Statement<'src> {
    match statement {
        a_ast::Statement::Let {
            name,
            expr,
            mutable,
            ..
        } => bir_ast::Statement::Let {
            name: (*name).into(),
            expr: visit_expr(expr),
            mutable: *mutable,
        },
        a_ast::Statement::Assign { name, expr, .. } => bir_ast::Statement::Assign {
            name: (*name).into(),
            expr: visit_expr(expr),
        },
        a_ast::Statement::Return { expr, .. } => bir_ast::Statement::Return {
            expr: expr.as_ref().map(|expr| visit_expr(expr)),
        },
        a_ast::Statement::Function(function) => {
            bir_ast::Statement::Function(visit_function(function))
        }
        a_ast::Statement::Struct(_) => todo!(),
        a_ast::Statement::Enum(_) => todo!(),
        a_ast::Statement::Expr(expr) => bir_ast::Statement::Expr(visit_expr(expr)),
        a_ast::Statement::ForLoop { cond, block, .. } => bir_ast::Statement::ForLoop {
            cond: cond.as_ref().map(|cond| visit_expr(cond)),
            block: visit_block(block),
        },
        a_ast::Statement::Break { .. } => bir_ast::Statement::Break,
        a_ast::Statement::Continue { .. } => bir_ast::Statement::Continue,
    }
}

fn visit_expr<'src>(expr: &a_ast::Expr<'src>) -> bir_ast::Expr<'src> {
    match expr {
        a_ast::Expr::IntLiteral(i) => bir_ast::Expr::IntLiteral(*i),
        a_ast::Expr::FloatLiteral(f) => bir_ast::Expr::FloatLiteral(*f),
        a_ast::Expr::BoolLiteral(b) => bir_ast::Expr::BoolLiteral(*b),
        a_ast::Expr::StringLiteral(s) => bir_ast::Expr::StringLiteral(*s),
        a_ast::Expr::StringInterpolation(exprs) => {
            bir_ast::Expr::StringInterpolation(exprs.iter().map(|expr| visit_expr(expr)).collect())
        }
        a_ast::Expr::Ident { ident, ty } => bir_ast::Expr::Ident {
            ident: (*ident).into(),
            ty: visit_type(ty),
        },
        a_ast::Expr::Binary {
            op,
            left,
            right,
            ty,
        } => bir_ast::Expr::Binary {
            op: (*op).into(),
            ty: visit_type(ty),
            left: Box::new(visit_expr(left)),
            right: Box::new(visit_expr(right)),
        },
        a_ast::Expr::Unary { op, expr, ty } => todo!(),
        a_ast::Expr::Call { call, ty } => bir_ast::Expr::Call {
            call: bir_ast::Call {
                callee: call.callee.to_string(),
                args: call.args.iter().map(|arg| visit_expr(arg)).collect(),
            },
            ty: visit_type(ty),
        },
        a_ast::Expr::StructInstance { name, fields } => todo!(),
        a_ast::Expr::Member { object, field, ty } => todo!(),
        a_ast::Expr::If {
            ty,
            cond,
            then,
            else_if_blocks,
            else_block,
        } => todo!(),
        a_ast::Expr::Closure(closure) => {
            let params = closure
                .params
                .iter()
                .map(|param| bir_ast::Param {
                    name: param.name.into(),
                    ty: visit_type(&param.ty.kind),
                    default: param.default.as_ref().map(|def| visit_expr(def)),
                })
                .collect();

            bir_ast::Expr::Closure {
                // TODO: change this
                mangled_name: "test".to_string(),
                ret_ty: visit_type(&closure.ret_ty),
                body: visit_block(&closure.body),
                params,
            }
        }
    }
}
