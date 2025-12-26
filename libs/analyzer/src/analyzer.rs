use super::ast as analyzer_ast;
use crate::ast::Vis;
use error::Diagnostic;
use parser::ast::{self as parser_ast};
use span::Span;
use std::collections::HashMap;
use utils::a_or_an;

pub fn analyzer<'src>(
    files: HashMap<String, parser_ast::Program<'src>>,
    package_name: &str,
) -> (analyzer_ast::Program<'src>, Vec<Diagnostic<'src>>) {
    let mut analyzer = Analyzer {
        parsed_files: files.clone(),
        analyzered_declarations: TypecheckedDeclarations::new(),
        errors: Vec::new(),
        program: analyzer_ast::Program {
            declarations: Vec::new(),
        },
        variables: HashMap::new(),
        current_module_name: format!("{}.main", package_name),
        context_name_mapping: ContextNameMapping(HashMap::new()),
    };

    analyzer.analyzer(&format!("{}.main", package_name));

    (analyzer.program, analyzer.errors)
}

struct TypecheckedDeclarations<'src>(
    HashMap<String, HashMap<String, analyzer_ast::Declaration<'src>>>,
);

impl<'src> TypecheckedDeclarations<'src> {
    fn insert(
        &mut self,
        module_name: &str,
        decl_name: &str,
        decl: &analyzer_ast::Declaration<'src>,
    ) {
        self.0
            .entry(module_name.to_string())
            .and_modify(|m| {
                m.insert(decl_name.to_string(), decl.clone());
            })
            .or_insert_with(|| {
                let mut hm = HashMap::new();
                hm.insert(decl_name.to_string(), decl.clone());
                hm
            });
    }

    fn get(&self, module_name: &str, decl_name: &str) -> Option<&analyzer_ast::Declaration<'src>> {
        self.0.get(module_name).and_then(|i| i.get(decl_name))
    }
}

impl<'src> TypecheckedDeclarations<'src> {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

struct ContextNameMapping(HashMap<String, HashMap<String, String>>);

impl ContextNameMapping {
    fn insert(&mut self, in_module_name: String, alias: String, reference: String) {
        self.0
            .entry(in_module_name.to_string())
            .and_modify(|m| {
                m.insert(alias.to_string(), reference.clone());
            })
            .or_insert_with(|| {
                let mut hm = HashMap::new();
                hm.insert(alias.to_string(), reference.clone());
                hm
            });
    }

    fn get(&self, in_module_name: &str, alias: &str) -> Option<&String> {
        self.0.get(in_module_name).and_then(|i| i.get(alias))
    }
}

#[derive(Clone, Copy, Debug)]
struct Variable<'src> {
    type_kind: analyzer_ast::TypeKind<'src>,
    mutable: bool,
    name_span: Span<'src>,
}

struct Analyzer<'src> {
    parsed_files: HashMap<String, parser_ast::Program<'src>>,
    analyzered_declarations: TypecheckedDeclarations<'src>,
    // Complete program after analyzering
    program: analyzer_ast::Program<'src>,
    errors: Vec<Diagnostic<'src>>,
    variables: HashMap<&'src str, Variable<'src>>,
    current_module_name: String,
    context_name_mapping: ContextNameMapping,
}

impl<'src> Analyzer<'src> {
    fn analyzer(&mut self, module: &str) {
        let previous_module_name = self.current_module_name.clone();
        self.current_module_name = module.to_string();
        let program = self.parsed_files.get(module).unwrap().clone();
        self.analyzer_inner(&program);
        self.current_module_name = previous_module_name;
    }

    fn analyzer_inner(&mut self, program: &parser_ast::Program<'src>) {
        for declaration in &program.declarations {
            let decl = match self.visit_declaration(declaration) {
                Ok(Some(decl)) => decl,
                Ok(None) => continue,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };
            self.program.declarations.push(decl);
        }
    }

    fn visit_declaration(
        &mut self,
        declaration: &parser_ast::Declaration<'src>,
    ) -> Result<Option<analyzer_ast::Declaration<'src>>, Diagnostic<'src>> {
        match declaration {
            parser_ast::Declaration::Struct(_) => todo!(),
            parser_ast::Declaration::Enum(_) => todo!(),
            parser_ast::Declaration::Function(function) => {
                // TODO: visit function declaration first, and later visit its body
                // to allow unordered definitions
                let f = self.visit_function(function)?;
                let function_decl = analyzer_ast::Declaration::Function(f);
                self.analyzered_declarations.insert(
                    &self.current_module_name,
                    function.name.inner,
                    &function_decl,
                );
                Ok(Some(function_decl))
            }
            parser_ast::Declaration::Use(use_) => {
                let import = &use_.import.iter().map(|i| i.inner).collect::<Vec<_>>();
                let last = import.last().unwrap();
                let module = import[..(import.len() - 1)].join(".");
                let mut module_name = module;
                if !self.parsed_files.contains_key(&module_name) {
                    module_name = format!("{}.{}", module_name, last);
                };
                if !self.parsed_files.contains_key(&module_name) {
                    return Err(Diagnostic::error(
                        use_.span,
                        format!("Unknown module {}", module_name),
                    ));
                }
                // TODO: import the artifacts into the module
                let decl = match self.analyzered_declarations.get(&module_name, last) {
                    Some(p) => p,
                    None => {
                        self.analyzer(&module_name);
                        self.analyzered_declarations
                            .get(&module_name, last)
                            .unwrap()
                    }
                };
                self.context_name_mapping.insert(
                    self.current_module_name.clone(),
                    last.to_string(),
                    import.join("."),
                );
                Ok(None)
            }
        }
    }

    fn visit_function(
        &mut self,
        function: &parser_ast::Function<'src>,
    ) -> Result<analyzer_ast::Function<'src>, Diagnostic<'src>> {
        // Clear up any variables from previous functions
        // TODO: if this is a function in a fuction, the variables should be wiped
        self.variables = HashMap::new();
        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            let ty = self.visit_type(&param.ty)?;
            self.variables.insert(
                param.name.inner,
                Variable {
                    type_kind: ty.kind,
                    mutable: false,
                    name_span: param.span,
                },
            );
            params.push(analyzer_ast::Param {
                span: param.span,
                name: param.name.into(),
                ty,
                default: None,
            });
        }

        if self.current_module_name.starts_with("std.") && function.name.inner == "print" {
            Ok(analyzer_ast::Function {
                span: function.span,
                name: format!("{}.{}", self.current_module_name, function.name.inner),
                params,
                ret_ty: analyzer_ast::TypeKind::Void,
                body: analyzer_ast::Block {
                    span: function.body.span,
                    statements: Vec::new(),
                    ty: analyzer_ast::TypeKind::Void,
                },
                vis: Vis::Public,
            })
        } else {
            let body = self.visit_block(&function.body)?;

            let ret_ty = body.ty;
            let defined_ret_ty = self.visit_type(&function.ret_ty)?;
            if !self.validate_types_align(&defined_ret_ty.kind, &ret_ty) {
                let span = function.body.statements.last().unwrap().span();
                return Err(Diagnostic::error(
                    function.ret_ty.span,
                    "Mismatched return type of function",
                )
                .with_info_label(
                    function.ret_ty.span,
                    format!("Return type defined as {}", function.ret_ty.kind),
                )
                .with_error_label(
                    *span,
                    format!(
                        "Mismatched types. Expected '{}' found '{}'",
                        function.ret_ty.kind, ret_ty
                    ),
                ));
            }

            let name = if function.name.inner == "main" {
                "main".to_string()
            } else {
                format!("{}.{}", self.current_module_name, function.name.inner)
            };
            Ok(analyzer_ast::Function {
                span: function.span,
                name,
                params,
                ret_ty,
                body,
                vis: function.vis.into(),
            })
        }
    }

    fn visit_block(
        &mut self,
        block: &parser_ast::Block<'src>,
    ) -> Result<analyzer_ast::Block<'src>, Diagnostic<'src>> {
        let mut statements = Vec::with_capacity(block.statements.len());

        for statement in &block.statements {
            statements.push(self.visit_statement(statement)?);
        }

        let ty = statements
            .last()
            .map(|stmt| match stmt {
                analyzer_ast::Statement::Return { ty, .. } => *ty,
                analyzer_ast::Statement::Expr(e) => *e.ty(),
                _ => analyzer_ast::TypeKind::Void,
            })
            .unwrap_or(analyzer_ast::TypeKind::Void);

        Ok(analyzer_ast::Block {
            span: block.span,
            statements,
            ty,
        })
    }

    fn visit_statement(
        &mut self,
        statement: &parser_ast::Statement<'src>,
    ) -> Result<analyzer_ast::Statement<'src>, Diagnostic<'src>> {
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
                let name: analyzer_ast::Ident<'src> = (*name).into();
                let ty = *expr.ty();
                self.variables.insert(
                    name.inner,
                    Variable {
                        type_kind: ty,
                        mutable: *mutable,
                        name_span: name.span,
                    },
                );
                Ok(analyzer_ast::Statement::Let {
                    name,
                    ty,
                    expr,
                    mutable: *mutable,
                    span: *span,
                })
            }
            parser_ast::Statement::Assign { name, expr, span } => {
                let expr = self.visit_expr(expr)?;
                // TODO: check that type of the name and expr.ty match
                let name: analyzer_ast::Ident<'src> = (*name).into();
                let ty = *expr.ty();
                let Some(expected_var) = self.variables.get(name.inner) else {
                    return Err(Diagnostic::error(
                        *span,
                        format!("A variable with the name '{}' does not exist", name.inner),
                    )
                    .with_error_label(*span, "here"));
                };
                if !expected_var.mutable {
                    return Err(Diagnostic::error(*span, "Variable is not mutable")
                        .with_error_label(
                            expected_var.name_span,
                            "Variable defined here. Consider making it mutable",
                        ));
                }
                if expected_var.type_kind != ty {
                    return Err(
                        Diagnostic::error(*span, "Mismatched types").with_error_label(
                            name.span,
                            format!(
                                "This variable has type '{}', but is being assigned a(n) '{}'",
                                expected_var.type_kind, ty
                            ),
                        ),
                    );
                }
                Ok(analyzer_ast::Statement::Assign {
                    name,
                    expr,
                    span: *span,
                })
            }
            parser_ast::Statement::Return { span, expr } => match expr {
                None => Ok(analyzer_ast::Statement::Return {
                    span: *span,
                    expr: None,
                    ty: analyzer_ast::TypeKind::Void,
                }),
                Some(expr) => {
                    let expr = self.visit_expr(expr)?;
                    Ok(analyzer_ast::Statement::Return {
                        span: *span,
                        ty: *expr.ty(),
                        expr: Some(expr),
                    })
                }
            },
            parser_ast::Statement::ForLoop { span, cond, block } => {
                let cond = match cond {
                    Some(cond) => {
                        let cond = self.visit_expr(cond)?;
                        if *cond.ty() != analyzer_ast::TypeKind::Bool {
                            return Err(Diagnostic::error(
                                *span,
                                "Expected the condition to be a boolean",
                            )
                            .with_error_label(*span, "here"));
                        };
                        Some(cond)
                    }
                    None => None,
                };
                Ok(analyzer_ast::Statement::ForLoop {
                    span: *span,
                    cond,
                    block: self.visit_block(block)?,
                })
            }
            parser_ast::Statement::Function(_) => todo!(),
            parser_ast::Statement::Struct(_) => todo!(),
            parser_ast::Statement::Enum(_) => todo!(),
            parser_ast::Statement::Expr(expr) => {
                let expr = self.visit_expr(expr)?;
                Ok(analyzer_ast::Statement::Expr(expr))
            }
        }
    }

    fn visit_expr(
        &mut self,
        expr: &parser_ast::Expr<'src>,
    ) -> Result<analyzer_ast::Expr<'src>, Diagnostic<'src>> {
        match &expr.kind {
            parser_ast::ExprKind::IntLiteral(i) => Ok(analyzer_ast::Expr::IntLiteral(*i)),
            parser_ast::ExprKind::FloatLiteral(f) => Ok(analyzer_ast::Expr::FloatLiteral(*f)),
            parser_ast::ExprKind::BoolLiteral(_) => todo!(),
            parser_ast::ExprKind::StringLiteral(str) => Ok(analyzer_ast::Expr::StringLiteral(*str)),
            parser_ast::ExprKind::StringInterpolation(si) => {
                let exprs = si
                    .iter()
                    .map(|e| self.visit_expr(e))
                    .collect::<Result<Vec<_>, Diagnostic<'_>>>();
                Ok(analyzer_ast::Expr::StringInterpolation(exprs?))
            }
            parser_ast::ExprKind::Ident(i) => {
                let ident: analyzer_ast::Ident<'src> = (*i).into();
                let var = *self
                    .variables
                    .get(&ident.inner)
                    .unwrap_or_else(|| panic!("Could not find variable '{}'", ident.inner));
                Ok(analyzer_ast::Expr::Ident {
                    ident,
                    ty: var.type_kind,
                })
            }
            parser_ast::ExprKind::Binary { op, left, right } => {
                let left_expr = self.visit_expr(left)?;
                let right_expr = self.visit_expr(right)?;
                if matches!(op, parser_ast::BinaryOp::And | parser_ast::BinaryOp::Or)
                    && left_expr.ty() != &analyzer_ast::TypeKind::Bool
                    && right_expr.ty() != &analyzer_ast::TypeKind::Bool
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
                let mut ty = *left_expr.ty();
                if matches!(
                    op,
                    parser_ast::BinaryOp::Eq
                        | parser_ast::BinaryOp::Ne
                        | parser_ast::BinaryOp::Lt
                        | parser_ast::BinaryOp::LtEq
                        | parser_ast::BinaryOp::Gt
                        | parser_ast::BinaryOp::GtEq
                ) {
                    ty = analyzer_ast::TypeKind::Bool;
                }
                Ok(analyzer_ast::Expr::Binary {
                    op: (*op).into(),
                    ty,
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                })
            }
            parser_ast::ExprKind::Unary { .. } => todo!(),
            parser_ast::ExprKind::Call(call) => match &call.callee.kind {
                parser_ast::ExprKind::Ident(ident) => {
                    let (decl, mapped) = if let Some(mapped) = self
                        .context_name_mapping
                        .get(&self.current_module_name, ident.inner)
                    {
                        let split = mapped.split(".").collect::<Vec<_>>();
                        let last = split.last().unwrap();
                        let module = &split[..(split.len() - 1)].join(".");
                        let decl = self.analyzered_declarations.get(module, last).unwrap();
                        (decl, mapped.to_string())
                    } else if let Some(decl) = self
                        .analyzered_declarations
                        .get(&self.current_module_name, ident.inner)
                    {
                        (
                            decl,
                            format!("{}.{}", self.current_module_name, ident.inner),
                        )
                    } else {
                        return Err(Diagnostic::error(
                            call.callee.span,
                            format!("Couldn't find '{}'", ident.inner),
                        ));
                    };
                    let func = match decl {
                        analyzer_ast::Declaration::Function(function) => function,
                        _ => todo!(),
                    };
                    if let Vis::Private = func.vis {
                        return Err(Diagnostic::error(call.callee.span, "Private function")
                            .with_error_label(call.callee.span, "This function is private")
                            .with_info_label(func.span, "Function defined here")
                            .with_help("make the function `internal` or `public` to use it"));
                    }
                    // TODO: verify call matches the signature of the function
                    Ok(analyzer_ast::Expr::Call {
                        ty: func.ret_ty,
                        call: analyzer_ast::Call {
                            callee: mapped.to_string(),
                            args: call
                                .args
                                .iter()
                                .map(|arg| match self.visit_expr(arg) {
                                    Ok(arg) => arg,
                                    Err(e) => todo!(),
                                })
                                .collect(),
                        },
                    })
                }
                _ => todo!(),
            },
            parser_ast::ExprKind::StructInstance { .. } => todo!(),
            parser_ast::ExprKind::Member { .. } => todo!(),
            parser_ast::ExprKind::If {
                cond,
                then,
                else_if_blocks,
                else_block,
            } => {
                let t_cond = self.visit_expr(cond)?;
                if *t_cond.ty() != analyzer_ast::TypeKind::Bool {
                    return Err(Diagnostic::error(
                        cond.span,
                        "Only boolean expressions allowed in if checks",
                    )
                    .with_error_label(cond.span, "here"));
                }
                let t_then = self.visit_block(then)?;
                let mut t_else_if_blocks = Vec::with_capacity(else_if_blocks.len());
                for (cond, block) in else_if_blocks {
                    t_else_if_blocks.push((self.visit_expr(cond)?, self.visit_block(block)?));
                }
                let t_else_block = if let Some(else_block) = else_block {
                    Some(self.visit_block(else_block)?)
                } else {
                    None
                };
                // TODO: check that all blocks either "yield" the right type or return the right type in the function
                Ok(analyzer_ast::Expr::If {
                    ty: t_then.ty,
                    cond: Box::new(t_cond),
                    then: t_then,
                    else_if_blocks: t_else_if_blocks,
                    else_block: t_else_block,
                })
            }
        }
    }

    fn validate_types_align(
        &mut self,
        ty1: &analyzer_ast::TypeKind<'src>,
        ty2: &analyzer_ast::TypeKind<'src>,
    ) -> bool {
        match (ty1, ty2) {
            (analyzer_ast::TypeKind::Int, analyzer_ast::TypeKind::Int)
            | (analyzer_ast::TypeKind::Float, analyzer_ast::TypeKind::Float)
            | (analyzer_ast::TypeKind::String, analyzer_ast::TypeKind::String)
            | (analyzer_ast::TypeKind::Bool, analyzer_ast::TypeKind::Bool)
            | (analyzer_ast::TypeKind::Void, analyzer_ast::TypeKind::Void) => true,
            (analyzer_ast::TypeKind::Ident(ident1), analyzer_ast::TypeKind::Ident(ident2))
                if ident1.inner == ident2.inner =>
            {
                true
            }
            _ => false,
        }
    }

    fn visit_type(
        &mut self,
        ty: &parser_ast::Type<'src>,
    ) -> Result<analyzer_ast::Type<'src>, Diagnostic<'src>> {
        let kind = match &ty.kind {
            parser_ast::TypeKind::Ident(ident) => analyzer_ast::TypeKind::Ident((*ident).into()),
            parser_ast::TypeKind::Int => analyzer_ast::TypeKind::Int,
            parser_ast::TypeKind::Float => analyzer_ast::TypeKind::Float,
            parser_ast::TypeKind::String => analyzer_ast::TypeKind::String,
            parser_ast::TypeKind::Bool => analyzer_ast::TypeKind::Bool,
            parser_ast::TypeKind::Void => analyzer_ast::TypeKind::Void,
        };
        Ok(analyzer_ast::Type {
            span: ty.span,
            kind,
        })
    }
}
