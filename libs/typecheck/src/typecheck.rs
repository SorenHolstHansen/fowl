use super::ast as typecheck_ast;
use error::Diagnostic;
use parser::ast::{self as parser_ast};
use std::collections::HashMap;
use utils::a_or_an;

pub fn typecheck<'source>(
    files: HashMap<String, parser_ast::Program<'source>>,
    package_name: &str,
) -> (typecheck_ast::Program<'source>, Vec<Diagnostic>) {
    let mut typechecker = Typechecker {
        parsed_files: files.clone(),
        typechecked_files: HashMap::new(),
        errors: Vec::new(),
        program: typecheck_ast::Program {
            declarations: Vec::new(),
        },
        variables: HashMap::new(),
    };

    typechecker.typecheck(files.get(&format!("{}.main", package_name)).unwrap());

    (typechecker.program, typechecker.errors)
}

struct Typechecker<'source> {
    parsed_files: HashMap<String, parser_ast::Program<'source>>,
    typechecked_files: HashMap<String, typecheck_ast::Program<'source>>,
    program: typecheck_ast::Program<'source>,
    errors: Vec<Diagnostic>,
    variables: HashMap<&'source str, typecheck_ast::TypeKind<'source>>,
}

impl<'source> Typechecker<'source> {
    fn typecheck(&mut self, program: &parser_ast::Program<'source>) {
        let mut declarations = Vec::with_capacity(program.declarations.len());

        for declaration in &program.declarations {
            let decl = match self.visit_declaration(&declaration) {
                Ok(Some(decl)) => decl,
                Ok(None) => continue,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };
            declarations.push(decl);
        }

        self.program = typecheck_ast::Program { declarations };
    }

    fn visit_declaration(
        &mut self,
        declaration: &parser_ast::Declaration<'source>,
    ) -> Result<Option<typecheck_ast::Declaration<'source>>, Diagnostic> {
        match declaration {
            parser_ast::Declaration::Struct(_) => todo!(),
            parser_ast::Declaration::Enum(_) => todo!(),
            parser_ast::Declaration::Function(function) => {
                let f = self.visit_function(function)?;
                Ok(Some(typecheck_ast::Declaration::Function(f)))
            }
            parser_ast::Declaration::Use { import } => {
                let last = import.last().unwrap();
                let module = import[..(import.len() - 1)]
                    .iter()
                    .map(|i| i.inner)
                    .collect::<Vec<_>>()
                    .join(".");
                let mut module_name = module;
                match self.parsed_files.get(&module_name) {
                    Some(_) => {}
                    None => {
                        module_name = format!("{}.{}", module_name, last.inner);
                    }
                };
                // TODO: import the artifacts into the module
                dbg!(module_name);
                Ok(None)
            }
        }
    }

    fn visit_function(
        &mut self,
        function: &parser_ast::Function<'source>,
    ) -> Result<typecheck_ast::Function<'source>, Diagnostic> {
        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            let ty = self.visit_type(&param.ty)?;
            self.variables.insert(param.name.inner, ty.kind);
            params.push(typecheck_ast::Param {
                span: param.span,
                name: param.name.into(),
                ty,
                default: None,
            });
        }

        let body = self.visit_block(&function.body)?;

        let ret_ty = body.ty.clone();
        let defined_ret_ty = self.visit_type(&function.ret_ty)?;
        if !self.validate_types_align(&defined_ret_ty.kind, &ret_ty) {
            return Err(Diagnostic::error(
                function.ret_ty.span,
                "Mismatched return type of function",
            )
            .with_info_label(
                function.ret_ty.span,
                format!("Return type defined as {}", function.ret_ty.kind),
            )
            .with_error_label(
                body.statements.last().unwrap().span(),
                format!(
                    "Mismatched types. Expected '{}' found '{}'",
                    function.ret_ty.kind, ret_ty
                ),
            ));
        }

        Ok(typecheck_ast::Function {
            span: function.span,
            name: function.name.into(),
            params,
            ret_ty,
            body,
            vis: function.vis.into(),
        })
    }

    fn visit_block(
        &mut self,
        block: &parser_ast::Block<'source>,
    ) -> Result<typecheck_ast::Block<'source>, Diagnostic> {
        let mut statements = Vec::with_capacity(block.statements.len());

        for statement in &block.statements {
            statements.push(self.visit_statement(statement)?);
        }

        let ty = statements
            .last()
            .map(|stmt| match stmt {
                typecheck_ast::Statement::Return { ty, .. } => ty.clone(),
                _ => typecheck_ast::TypeKind::Void,
            })
            .unwrap_or(typecheck_ast::TypeKind::Void);

        Ok(typecheck_ast::Block {
            span: block.span,
            statements,
            ty,
        })
    }

    fn visit_statement(
        &mut self,
        statement: &parser_ast::Statement<'source>,
    ) -> Result<typecheck_ast::Statement<'source>, Diagnostic> {
        match statement {
            parser_ast::Statement::Let {
                name,
                ty,
                expr,
                mutable,
                span,
            } => {
                let expr = self.visit_expr(expr)?;
                // TODO: check that ty and expr.ty match
                let name: typecheck_ast::Ident<'source> = (*name).into();
                let ty = expr.ty().clone();
                self.variables.insert(name.inner, ty.clone());
                Ok(typecheck_ast::Statement::Let {
                    name,
                    ty,
                    expr,
                    mutable: *mutable,
                    span: *span,
                })
            }
            parser_ast::Statement::Return { span, expr } => match expr {
                None => Ok(typecheck_ast::Statement::Return {
                    span: *span,
                    expr: None,
                    ty: typecheck_ast::TypeKind::Void,
                }),
                Some(expr) => {
                    let expr = self.visit_expr(expr)?;
                    Ok(typecheck_ast::Statement::Return {
                        span: *span,
                        ty: expr.ty().clone(),
                        expr: Some(expr),
                    })
                }
            },
            parser_ast::Statement::Function(_) => todo!(),
            parser_ast::Statement::Struct(_) => todo!(),
            parser_ast::Statement::Enum(_) => todo!(),
            parser_ast::Statement::Expr(_) => todo!(),
        }
    }

    fn visit_expr(
        &mut self,
        expr: &parser_ast::Expr<'source>,
    ) -> Result<typecheck_ast::Expr<'source>, Diagnostic> {
        match &expr.kind {
            parser_ast::ExprKind::IntLiteral(i) => Ok(typecheck_ast::Expr::IntLiteral(*i)),
            parser_ast::ExprKind::FloatLiteral(f) => Ok(typecheck_ast::Expr::FloatLiteral(*f)),
            parser_ast::ExprKind::BoolLiteral(_) => todo!(),
            parser_ast::ExprKind::StringLiteral(_) => todo!(),
            parser_ast::ExprKind::StringInterpolation(_) => todo!(),
            parser_ast::ExprKind::Ident(i) => {
                let ident: typecheck_ast::Ident<'source> = (*i).into();
                let ty = self
                    .variables
                    .get(&ident.inner)
                    .expect(&format!("Could not find variable '{}'", ident.inner))
                    .clone();
                Ok(typecheck_ast::Expr::Ident { ident, ty })
            }
            parser_ast::ExprKind::Binary { op, left, right } => {
                let left_expr = self.visit_expr(left)?;
                let right_expr = self.visit_expr(right)?;
                if matches!(op, parser_ast::BinaryOp::And | parser_ast::BinaryOp::Or)
                    && left_expr.ty() != &typecheck_ast::TypeKind::Bool
                    && right_expr.ty() != &typecheck_ast::TypeKind::Bool
                {
                    return Err(Diagnostic::error(
                        expr.span,
                        "Only boolean expression allowed on either side of 'and' or 'or'",
                    ));
                }
                if !self.validate_types_align(left_expr.ty(), right_expr.ty()) {
                    let left_ty = left_expr.ty().to_string();
                    let left_a_or_an = a_or_an(&left_ty);
                    let right_ty = right_expr.ty().to_string();
                    let right_a_or_an = a_or_an(&right_ty);
                    let message = match op {
                        parser_ast::BinaryOp::Add => format!(
                            "cannot add {} '{}' to {} '{}'",
                            left_a_or_an, left_ty, right_a_or_an, right_ty
                        ),
                        parser_ast::BinaryOp::Sub => format!(
                            "cannot subtract {} '{}' from {} '{}'",
                            right_a_or_an, right_ty, left_a_or_an, left_ty,
                        ),
                        parser_ast::BinaryOp::Mul => format!(
                            "cannot multiply {} '{}' with {} '{}'",
                            left_a_or_an, left_ty, right_a_or_an, right_ty
                        ),
                        parser_ast::BinaryOp::Div => format!(
                            "cannot divide {} '{}' with {} '{}'",
                            left_a_or_an, left_ty, right_a_or_an, right_ty
                        ),
                        parser_ast::BinaryOp::Mod => format!(
                            "cannot take the remainder of {} '{}' with {} '{}'",
                            left_a_or_an, left_ty, right_a_or_an, right_ty
                        ),
                        parser_ast::BinaryOp::Exp => format!(
                            "cannot take exponentiate {} '{}' with {} '{}'",
                            left_a_or_an, left_ty, right_a_or_an, right_ty
                        ),
                        parser_ast::BinaryOp::LtEq
                        | parser_ast::BinaryOp::GtEq
                        | parser_ast::BinaryOp::Ne
                        | parser_ast::BinaryOp::Lt
                        | parser_ast::BinaryOp::Gt
                        | parser_ast::BinaryOp::Eq => format!(
                            "cannot compare {} '{}' with {} '{}'",
                            left_a_or_an, left_ty, right_a_or_an, right_ty
                        ),
                        parser_ast::BinaryOp::And | parser_ast::BinaryOp::Or => {
                            unreachable!()
                        }
                    };
                    return Err(Diagnostic::error(expr.span, message)
                        .with_error_label(
                            left.span,
                            format!("This is {} '{}'", left_a_or_an, left_ty),
                        )
                        .with_error_label(
                            right.span,
                            format!("This is {} '{}'", right_a_or_an, right_ty),
                        ));
                }
                Ok(typecheck_ast::Expr::Binary {
                    op: (*op).into(),
                    ty: left_expr.ty().clone(),
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                })
            }
            parser_ast::ExprKind::Unary { .. } => todo!(),
            parser_ast::ExprKind::Call(call) => {
                dbg!(call);
                todo!()
            }
            parser_ast::ExprKind::StructInstance { .. } => todo!(),
            parser_ast::ExprKind::Member { .. } => todo!(),
        }
    }

    fn validate_types_align(
        &mut self,
        ty1: &typecheck_ast::TypeKind<'source>,
        ty2: &typecheck_ast::TypeKind<'source>,
    ) -> bool {
        match (ty1, ty2) {
            (typecheck_ast::TypeKind::Int, typecheck_ast::TypeKind::Int)
            | (typecheck_ast::TypeKind::Float, typecheck_ast::TypeKind::Float)
            | (typecheck_ast::TypeKind::String, typecheck_ast::TypeKind::String)
            | (typecheck_ast::TypeKind::Bool, typecheck_ast::TypeKind::Bool)
            | (typecheck_ast::TypeKind::Void, typecheck_ast::TypeKind::Void) => true,
            (typecheck_ast::TypeKind::Ident(ident1), typecheck_ast::TypeKind::Ident(ident2))
                if ident1.inner == ident2.inner =>
            {
                true
            }
            _ => false,
        }
    }

    fn visit_type(
        &mut self,
        ty: &parser_ast::Type<'source>,
    ) -> Result<typecheck_ast::Type<'source>, Diagnostic> {
        let kind = match &ty.kind {
            parser_ast::TypeKind::Ident(ident) => typecheck_ast::TypeKind::Ident((*ident).into()),
            parser_ast::TypeKind::Int => typecheck_ast::TypeKind::Int,
            parser_ast::TypeKind::Float => typecheck_ast::TypeKind::Float,
            parser_ast::TypeKind::String => typecheck_ast::TypeKind::String,
            parser_ast::TypeKind::Bool => typecheck_ast::TypeKind::Bool,
            parser_ast::TypeKind::Void => typecheck_ast::TypeKind::Void,
        };
        Ok(typecheck_ast::Type {
            span: ty.span,
            kind,
        })
    }
}
