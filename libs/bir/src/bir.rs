use crate::ast as bir_ast;
use analyzer::ast as a_ast;
use std::collections::{HashMap, HashSet};

pub fn bir<'src>(program: a_ast::Program<'src>) -> bir_ast::Program<'src> {
    let mut converter = BirConverter::new();
    let mut declarations = Vec::new();
    for declaration in program.declarations {
        declarations.push(converter.visit_declaration(declaration));
    }
    bir_ast::Program { declarations }
}

struct BirConverter {
    closure_counter: usize,
    /// Variables declared with `let mut` in the current scope
    mutable_vars: HashSet<String>,
}

impl BirConverter {
    fn new() -> Self {
        Self {
            closure_counter: 0,
            mutable_vars: HashSet::new(),
        }
    }

    fn next_closure_name(&mut self) -> String {
        let id = self.closure_counter;
        self.closure_counter += 1;
        format!("__closure_{id}")
    }

    fn visit_declaration<'src>(
        &mut self,
        declaration: a_ast::Declaration<'src>,
    ) -> bir_ast::Declaration<'src> {
        match declaration {
            a_ast::Declaration::Struct(_) => todo!(),
            a_ast::Declaration::Enum(_) => todo!(),
            a_ast::Declaration::Function(function) => {
                bir_ast::Declaration::Function(self.visit_function(&function))
            }
        }
    }

    fn visit_function<'src>(
        &mut self,
        function: &a_ast::Function<'src>,
    ) -> bir_ast::Function<'src> {
        let params = function
            .params
            .iter()
            .map(|param| bir_ast::Param {
                name: param.name.into(),
                ty: visit_type(&param.ty.kind),
                default: param.default.as_ref().map(|def| self.visit_expr(def)),
            })
            .collect();
        bir_ast::Function {
            params,
            ret_ty: visit_type(&function.ret_ty),
            body: self.visit_block(&function.body),
            public: function.vis == a_ast::Vis::Public,
            name: function.name.clone(),
        }
    }

    fn visit_block<'src>(&mut self, block: &a_ast::Block<'src>) -> bir_ast::Block<'src> {
        let mut statements = Vec::new();
        for statement in &block.statements {
            statements.push(self.visit_statement(statement));
        }
        bir_ast::Block { statements }
    }

    fn visit_statement<'src>(
        &mut self,
        statement: &a_ast::Statement<'src>,
    ) -> bir_ast::Statement<'src> {
        match statement {
            a_ast::Statement::Let {
                name,
                expr,
                mutable,
                ..
            } => {
                if *mutable {
                    self.mutable_vars.insert(name.inner.to_string());
                }
                bir_ast::Statement::Let {
                    name: (*name).into(),
                    expr: self.visit_expr(expr),
                    mutable: *mutable,
                }
            }
            a_ast::Statement::Assign { name, expr, .. } => bir_ast::Statement::Assign {
                name: (*name).into(),
                expr: self.visit_expr(expr),
            },
            a_ast::Statement::Return { expr, .. } => bir_ast::Statement::Return {
                expr: expr.as_ref().map(|expr| self.visit_expr(expr)),
            },
            a_ast::Statement::Function(function) => {
                bir_ast::Statement::Function(self.visit_function(function))
            }
            a_ast::Statement::Struct(_) => todo!(),
            a_ast::Statement::Enum(_) => todo!(),
            a_ast::Statement::Expr(expr) => bir_ast::Statement::Expr(self.visit_expr(expr)),
            a_ast::Statement::ForLoop { cond, block, .. } => bir_ast::Statement::ForLoop {
                cond: cond.as_ref().map(|cond| self.visit_expr(cond)),
                block: self.visit_block(block),
            },
            a_ast::Statement::Break { .. } => bir_ast::Statement::Break,
            a_ast::Statement::Continue { .. } => bir_ast::Statement::Continue,
        }
    }

    fn visit_expr<'src>(&mut self, expr: &a_ast::Expr<'src>) -> bir_ast::Expr<'src> {
        match expr {
            a_ast::Expr::IntLiteral(i) => bir_ast::Expr::IntLiteral(*i),
            a_ast::Expr::FloatLiteral(f) => bir_ast::Expr::FloatLiteral(*f),
            a_ast::Expr::BoolLiteral(b) => bir_ast::Expr::BoolLiteral(*b),
            a_ast::Expr::StringLiteral(s) => bir_ast::Expr::StringLiteral(*s),
            a_ast::Expr::StringInterpolation(exprs) => bir_ast::Expr::StringInterpolation(
                exprs.iter().map(|expr| self.visit_expr(expr)).collect(),
            ),
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
                left: Box::new(self.visit_expr(left)),
                right: Box::new(self.visit_expr(right)),
            },
            a_ast::Expr::Unary { op, expr, ty } => todo!(),
            a_ast::Expr::Call { call, ty } => bir_ast::Expr::Call {
                call: bir_ast::Call {
                    callee: call.callee.to_string(),
                    args: call.args.iter().map(|arg| self.visit_expr(arg)).collect(),
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
                let params: Vec<_> = closure
                    .params
                    .iter()
                    .map(|param| bir_ast::Param {
                        name: param.name.into(),
                        ty: visit_type(&param.ty.kind),
                        default: param.default.as_ref().map(|def| self.visit_expr(def)),
                    })
                    .collect();

                let body = self.visit_block(&closure.body);

                // Collect captured variables: idents in the body that are not params or
                // locally-defined variables within the closure body.
                let mut local_vars: HashSet<String> =
                    params.iter().map(|p| p.name.inner.to_string()).collect();
                let mut captured_idents: HashMap<String, bir_ast::TypeKind<'src>> = HashMap::new();
                collect_idents_in_block(&body, &mut local_vars, &mut captured_idents);

                let captures: Vec<(String, bir_ast::TypeKind<'src>, bool)> = captured_idents
                    .into_iter()
                    .map(|(name, ty)| {
                        let is_mutable = self.mutable_vars.contains(&name);
                        (name, ty, is_mutable)
                    })
                    .collect();

                let mangled_name = self.next_closure_name();

                bir_ast::Expr::Closure {
                    mangled_name,
                    ret_ty: visit_type(&closure.ret_ty),
                    body,
                    params,
                    captures,
                }
            }
        }
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

/// Collect all free-variable Ident references in a block, excluding those in `local_vars`.
/// `local_vars` is grown as Let bindings are encountered.
fn collect_idents_in_block<'src>(
    block: &bir_ast::Block<'src>,
    local_vars: &mut HashSet<String>,
    out: &mut HashMap<String, bir_ast::TypeKind<'src>>,
) {
    for stmt in &block.statements {
        match stmt {
            bir_ast::Statement::Let { name, expr, .. } => {
                collect_idents_in_expr(expr, local_vars, out);
                local_vars.insert(name.inner.to_string());
            }
            bir_ast::Statement::Assign { expr, .. } => {
                collect_idents_in_expr(expr, local_vars, out);
            }
            bir_ast::Statement::Return { expr: Some(expr), .. } => {
                collect_idents_in_expr(expr, local_vars, out);
            }
            bir_ast::Statement::Expr(expr) => {
                collect_idents_in_expr(expr, local_vars, out);
            }
            bir_ast::Statement::ForLoop { cond, block } => {
                if let Some(cond) = cond {
                    collect_idents_in_expr(cond, local_vars, out);
                }
                collect_idents_in_block(block, local_vars, out);
            }
            _ => {}
        }
    }
}

fn collect_idents_in_expr<'src>(
    expr: &bir_ast::Expr<'src>,
    local_vars: &HashSet<String>,
    out: &mut HashMap<String, bir_ast::TypeKind<'src>>,
) {
    match expr {
        bir_ast::Expr::Ident { ident, ty } => {
            if !local_vars.contains(ident.inner) {
                out.insert(ident.inner.to_string(), ty.clone());
            }
        }
        bir_ast::Expr::Binary { left, right, .. } => {
            collect_idents_in_expr(left, local_vars, out);
            collect_idents_in_expr(right, local_vars, out);
        }
        bir_ast::Expr::Unary { expr, .. } => {
            collect_idents_in_expr(expr, local_vars, out);
        }
        bir_ast::Expr::Call { call, .. } => {
            for arg in &call.args {
                collect_idents_in_expr(arg, local_vars, out);
            }
        }
        bir_ast::Expr::StringInterpolation(parts) => {
            for part in parts {
                collect_idents_in_expr(part, local_vars, out);
            }
        }
        bir_ast::Expr::If {
            cond,
            then,
            else_block,
            ..
        } => {
            collect_idents_in_expr(cond, local_vars, out);
            collect_idents_in_block(then, &mut local_vars.clone(), out);
            if let Some(els) = else_block {
                collect_idents_in_block(els, &mut local_vars.clone(), out);
            }
        }
        // Closures within closures: their captures are handled when the inner closure is visited
        bir_ast::Expr::Closure { .. } => {}
        _ => {}
    }
}
