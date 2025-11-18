use std::collections::HashMap;

use super::ast as typecheck_ast;
use error::Diagnostic;
use parser::ast as parser_ast;

pub fn typecheck<'source>(
    program: parser_ast::Program<'source>,
) -> (typecheck_ast::Program<'source>, Vec<Diagnostic>) {
    let mut typechecker = Typechecker {
        errors: Vec::new(),
        variables: HashMap::new(),
    };

    typechecker.typecheck(program)
}

struct Typechecker<'source> {
    errors: Vec<Diagnostic>,
    variables: HashMap<&'source str, typecheck_ast::TypeKind<'source>>,
}

impl<'source> Typechecker<'source> {
    fn typecheck(
        &mut self,
        program: parser_ast::Program<'source>,
    ) -> (typecheck_ast::Program<'source>, Vec<Diagnostic>) {
        let mut declarations = Vec::with_capacity(program.declarations.len());

        for declaration in program.declarations {
            let decl = match self.visit_declaration(&declaration) {
                Ok(decl) => decl,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };
            declarations.push(decl);
        }

        (typecheck_ast::Program { declarations }, self.errors.clone())
    }

    fn visit_declaration(
        &mut self,
        declaration: &parser_ast::Declaration<'source>,
    ) -> Result<typecheck_ast::Declaration<'source>, Diagnostic> {
        match declaration {
            parser_ast::Declaration::Struct(_) => todo!(),
            parser_ast::Declaration::Enum(_) => todo!(),
            parser_ast::Declaration::Function(function) => {
                let f = self.visit_function(function)?;
                Ok(typecheck_ast::Declaration::Function(f))
            }
            parser_ast::Declaration::Use { .. } => todo!(),
            parser_ast::Declaration::PubUse { .. } => todo!(),
        }
    }

    fn visit_function(
        &mut self,
        function: &parser_ast::Function<'source>,
    ) -> Result<typecheck_ast::Function<'source>, Diagnostic> {
        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            params.push(typecheck_ast::Param {
                span: param.span,
                name: param.name.into(),
                ty: self.visit_type(&param.ty)?,
                default: None,
            });
        }

        let body = self.visit_block(&function.body)?;

        let ret_ty = body.ty.clone();
        match (&function.ret_ty.kind, &ret_ty) {
            (parser_ast::TypeKind::Int, typecheck_ast::TypeKind::Int) => {}
            (parser_ast::TypeKind::Float, typecheck_ast::TypeKind::Float) => {}
            (parser_ast::TypeKind::String, typecheck_ast::TypeKind::String) => {}
            (parser_ast::TypeKind::Bool, typecheck_ast::TypeKind::Bool) => {}
            (parser_ast::TypeKind::Void, typecheck_ast::TypeKind::Void) => {}
            (parser_ast::TypeKind::Ident(ident1), typecheck_ast::TypeKind::Ident(ident2))
                if ident1.inner == ident2.inner => {}
            _ => {
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
        }

        Ok(typecheck_ast::Function {
            span: function.span,
            name: function.name.into(),
            params,
            ret_ty,
            body,
            public: function.public,
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
                ty: _ty,
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
        match expr {
            parser_ast::Expr::IntLiteral(i) => Ok(typecheck_ast::Expr::IntLiteral(*i)),
            parser_ast::Expr::FloatLiteral(_) => todo!(),
            parser_ast::Expr::BoolLiteral(_) => todo!(),
            parser_ast::Expr::StringLiteral(_) => todo!(),
            parser_ast::Expr::StringInterpolation(_) => todo!(),
            parser_ast::Expr::Ident(i) => {
                let ident: typecheck_ast::Ident<'source> = (*i).into();
                let ty = self.variables.get(&ident.inner).unwrap().clone();
                Ok(typecheck_ast::Expr::Ident { ident, ty })
            }
            parser_ast::Expr::Binary { .. } => todo!(),
            parser_ast::Expr::Unary { .. } => todo!(),
            parser_ast::Expr::Call(_) => todo!(),
            parser_ast::Expr::StructInstance { .. } => todo!(),
            parser_ast::Expr::Member { .. } => todo!(),
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
