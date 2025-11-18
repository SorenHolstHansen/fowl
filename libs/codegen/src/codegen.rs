use anyhow::bail;
use cranelift::prelude::{isa::TargetIsa, *};
use cranelift_codegen::{Context, ir::Type};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use parser::ast::{self, Program};
use std::{collections::HashMap, fs::File, io::Write, path::Path, process::Command, sync::Arc};
use target_lexicon::Triple;

pub struct CodegenOptions {
    /// Target triple for cross-compilation (defaults to native)
    pub target: Option<Triple>,
}

struct Compiler {
    isa: Arc<dyn TargetIsa>,
    module: ObjectModule,
    ctx: Context,
    fctx: FunctionBuilderContext,
}

impl Compiler {
    fn new(options: &CodegenOptions) -> Self {
        let isa = {
            let mut builder = settings::builder();

            // disable optimizations so dissassembly will more directly correlated to our Cranelift usage
            builder.set("opt_level", "none").unwrap();

            builder.enable("is_pic").unwrap();

            let flags = settings::Flags::new(builder);

            isa::lookup(options.target.clone().unwrap_or(target_lexicon::HOST))
                .unwrap()
                .finish(flags)
                .unwrap()
        };
        let module = {
            let translation_unit_name = b"output_a_binary";
            let libcall_names = cranelift_module::default_libcall_names();
            let builder =
                ObjectBuilder::new(isa.clone(), translation_unit_name, libcall_names).unwrap();
            ObjectModule::new(builder)
        };
        let ctx = codegen::Context::new();
        let fctx = FunctionBuilderContext::new();
        Self {
            isa,
            module,
            ctx,
            fctx,
        }
    }

    fn finish(self) -> ObjectProduct {
        // If we have additional information such as unwind information or DWARF debug information,
        // they can be added to `Product`. For this example we skip such optional additions.
        self.module.finish()
    }

    fn declare_c_functions(&mut self) -> anyhow::Result<()> {
        let pointer_type = self.module.target_config().pointer_type();
        let call_conv = self.isa.default_call_conv();
        // register "print" function
        let sig = Signature {
            call_conv,
            params: vec![AbiParam::new(pointer_type)],
            returns: vec![],
        };
        self.module
            .declare_function("rt_print_str", Linkage::Import, &sig)
            .unwrap();

        // register "concat" function
        let sig = Signature {
            call_conv,
            params: vec![AbiParam::new(pointer_type), AbiParam::new(pointer_type)],
            returns: vec![AbiParam::new(pointer_type)],
        };
        self.module
            .declare_function("rt_concat", Linkage::Import, &sig)
            .unwrap();

        // register "int_to_str" function
        let sig = Signature {
            call_conv,
            params: vec![AbiParam::new(pointer_type)],
            returns: vec![AbiParam::new(pointer_type)],
        };
        self.module
            .declare_function("rt_int_to_str", Linkage::Import, &sig)
            .unwrap();

        Ok(())
    }

    fn lower_program(&mut self, program: &Program) -> anyhow::Result<()> {
        let functions: Vec<_> = program
            .declarations
            .iter()
            .filter_map(|decl| match decl {
                ast::Declaration::Function(function) => Some(function),
                _ => None,
            })
            .collect();

        if functions.is_empty() {
            bail!("program contains no functions");
        }

        self.declare_c_functions()?;

        // First, declare all functions (without bodies)
        for function in &functions {
            self.declare_function(function)?;
        }

        // Then, lower the bodies of all functions
        for function in &functions {
            self.lower_function_body(function)?;
        }

        Ok(())
    }

    fn lower_function_body(&mut self, function: &ast::Function) -> anyhow::Result<()> {
        let func_id = match self
            .module
            .declarations()
            .get_name(function.name.inner)
            .unwrap()
        {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };

        let mut function_compiler = FunctionCompiler::new(
            &mut self.ctx,
            &mut self.fctx,
            self.isa.clone(),
            &mut self.module,
        );
        function_compiler.compile(function)?;
        function_compiler.finalize();

        println!("fn main:\n{}", &self.ctx.func);
        self.module.define_function(func_id, &mut self.ctx).unwrap();

        self.ctx.clear();
        Ok(())
    }

    fn declare_function(&mut self, function: &ast::Function) -> anyhow::Result<FuncId> {
        let mut param_types = vec![];
        for param in &function.params {
            let ty = type_from_ast(&param.ty, &self.module)?.expect("Can't use void here");
            param_types.push(AbiParam::new(ty));
        }

        let ret_ty = if let Some(ty) = type_from_ast(&function.ret_ty, &self.module)? {
            vec![AbiParam::new(ty)]
        } else {
            vec![]
        };

        let call_conv = self.isa.default_call_conv();
        let sig = Signature {
            call_conv,
            params: param_types,
            returns: ret_ty,
        };
        let function_id = self
            .module
            .declare_function(function.name.inner, Linkage::Export, &sig)
            .unwrap();

        Ok(function_id)
    }
}

fn type_from_ast(ast_ty: &ast::Type, module: &ObjectModule) -> anyhow::Result<Option<Type>> {
    match &ast_ty.kind {
        ast::TypeKind::Ident(_) => todo!(),
        ast::TypeKind::Int => Ok(Some(types::I64)),
        ast::TypeKind::Float => Ok(Some(types::F64)),
        ast::TypeKind::String => Ok(Some(module.target_config().pointer_type())),
        ast::TypeKind::Bool => Ok(Some(types::I8)),
        ast::TypeKind::Void => Ok(None),
        ast::TypeKind::Generic { .. } => todo!(),
    }
}

struct FunctionCompiler<'a> {
    builder: FunctionBuilder<'a>,
    isa: Arc<dyn TargetIsa>,
    module: &'a mut ObjectModule,
    variables: HashMap<String, Variable>,
}

impl<'a> FunctionCompiler<'a> {
    fn new(
        ctx: &'a mut Context,
        fctx: &'a mut FunctionBuilderContext,
        isa: Arc<dyn TargetIsa>,
        module: &'a mut ObjectModule,
    ) -> Self {
        let builder = FunctionBuilder::new(&mut ctx.func, fctx);
        Self {
            builder,
            isa,
            module,
            variables: HashMap::new(),
        }
    }

    fn strcat(&mut self, message_buf: Value, message: Value) -> anyhow::Result<Value> {
        let func_id = match self.module.declarations().get_name("rt_concat").unwrap() {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };
        let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);

        let args = vec![message_buf, message];

        let call = self.builder.ins().call(local_callee, &args);
        Ok(self.builder.inst_results(call)[0])
    }

    fn compile(&mut self, function: &ast::Function) -> anyhow::Result<()> {
        let call_conv = self.isa.default_call_conv();

        self.builder.func.signature = Signature {
            call_conv,
            params: vec![],
            // Since we're linking to libc, we can return the exit code from main.
            returns: vec![AbiParam::new(types::I64)],
        };

        // Create the functions entry block.
        let block0 = self.builder.create_block();
        self.builder.switch_to_block(block0);

        // When we know that there are no more other blocks which can jump to this block, we want to seal
        // it. This improves the quality of code generation.
        self.builder.seal_block(block0);

        for statement in &function.body.statements {
            self.lower_statement(statement)?;
        }

        if let Err(err) = codegen::verify_function(self.builder.func, self.isa.as_ref()) {
            panic!("verifier error: {err}");
        }

        Ok(())
    }

    fn finalize(self) {
        self.builder.finalize();
    }

    fn eval_call(&mut self, call: &ast::Call) -> anyhow::Result<Option<Value>> {
        match &*call.callee {
            ast::Expr::Ident(ident) => {
                let func_id = match self.module.declarations().get_name(ident.inner).unwrap() {
                    cranelift_module::FuncOrDataId::Func(func_id) => func_id,
                    cranelift_module::FuncOrDataId::Data(_) => todo!(),
                };
                let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);
                let mut args = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    args.push(
                        self.eval_expr(arg)?
                            .expect("Should have been caught in type checking, that this is void"),
                    );
                }
                let call = self.builder.ins().call(local_callee, &args);
                let inst_results = self.builder.inst_results(call);
                Ok(inst_results.first().cloned())
                // gn
            }
            _ => todo!(),
        }
    }

    fn eval_expr(&mut self, expr: &ast::Expr) -> anyhow::Result<Option<Value>> {
        match expr {
            ast::Expr::IntLiteral(i) => {
                let v = self.builder.ins().iconst(types::I64, *i);
                Ok(Some(v))
            }
            ast::Expr::FloatLiteral(_) => todo!(),
            ast::Expr::BoolLiteral(_) => todo!(),
            ast::Expr::StringLiteral(s) => {
                let s = s.unwrap_or_default();
                // Setup global string
                let id = self.module.declare_anonymous_data(false, false).unwrap();
                let mut data_description = DataDescription::new();
                data_description.define(format!("{s}\00").as_bytes().to_vec().into_boxed_slice());
                self.module.define_data(id, &data_description).unwrap();

                let local_id = self.module.declare_data_in_func(id, self.builder.func);
                let pointer = self.module.target_config().pointer_type();
                let s = self.builder.ins().symbol_value(pointer, local_id);
                Ok(Some(s))
            }
            ast::Expr::StringInterpolation(parts) => {
                let v = self.eval_string_interpolation(parts)?;
                Ok(Some(v))
            }
            ast::Expr::Ident(ident) => {
                let var = self.variables.get(ident.inner).unwrap();
                let v = self.builder.use_var(*var);
                Ok(Some(v))
            }
            ast::Expr::Binary { op, left, right } => {
                let left_expr = self.eval_expr(left)?.expect("Should be there");
                let right_expr = self.eval_expr(right)?.expect("Should be there");
                match op {
                    ast::BinaryOp::Add => Ok(Some(self.builder.ins().iadd(left_expr, right_expr))),
                    ast::BinaryOp::Sub => Ok(Some(self.builder.ins().isub(left_expr, right_expr))),
                    ast::BinaryOp::Mul => todo!(),
                    ast::BinaryOp::Div => todo!(),
                    ast::BinaryOp::Mod => todo!(),
                    ast::BinaryOp::Exp => todo!(),
                    ast::BinaryOp::Eq => todo!(),
                    ast::BinaryOp::Ne => todo!(),
                    ast::BinaryOp::Lt => todo!(),
                    ast::BinaryOp::Gt => todo!(),
                    ast::BinaryOp::LtEq => todo!(),
                    ast::BinaryOp::GtEq => todo!(),
                    ast::BinaryOp::And => todo!(),
                    ast::BinaryOp::Or => todo!(),
                }
            }
            ast::Expr::Unary { .. } => todo!(),
            ast::Expr::Call(call) => self.eval_call(call),
            ast::Expr::StructInstance { .. } => todo!(),
            ast::Expr::Member { .. } => todo!(),
        }
    }

    fn format_int_fn(&mut self, value: Value) -> anyhow::Result<Value> {
        let func_id = match self
            .module
            .declarations()
            .get_name("rt_int_to_str")
            .unwrap()
        {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };
        let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);

        let args = vec![value];

        let call = self.builder.ins().call(local_callee, &args);
        Ok(self.builder.inst_results(call)[0])
    }

    fn eval_string_interpolation(&mut self, parts: &[ast::Expr]) -> anyhow::Result<Value> {
        // Setup global string
        let id = self.module.declare_anonymous_data(true, false).unwrap();
        // TODO: pull the DataDescription top-level to reuse resources
        let mut data_description = DataDescription::new();
        data_description.define("\0".as_bytes().to_vec().into_boxed_slice());
        self.module.define_data(id, &data_description).unwrap();

        let local_id = self.module.declare_data_in_func(id, self.builder.func);
        let pointer = self.module.target_config().pointer_type();
        let mut result_ptr = self.builder.ins().symbol_value(pointer, local_id);

        for part in parts {
            let mut v = self.eval_expr(part)?.expect("Found void");
            dbg!(&part);
            match part {
                ast::Expr::IntLiteral(_) => {
                    v = self.format_int_fn(v)?;
                }
                ast::Expr::Ident(_) => {
                    // TODO: Check the type of ident
                    v = self.format_int_fn(v)?;
                }
                ast::Expr::Binary { .. } => {
                    // TODO: Check the type of ident
                    v = self.format_int_fn(v)?;
                }
                _ => {}
            }
            let new_result = self.strcat(result_ptr, v)?;
            result_ptr = new_result;
        }

        Ok(result_ptr)
    }

    fn lower_statement(&mut self, statement: &ast::Statement) -> anyhow::Result<()> {
        match statement {
            ast::Statement::Let { name, expr, .. } => {
                let value = self
                    .eval_expr(expr)?
                    .expect("This should exist. Probably didn't typecheck for void");
                // let ty = type_from_ast(&ty.clone().expect("This can't be void"), &self.module)?
                //     .expect("This can't be void");
                let var = self.builder.declare_var(
                    // TODO: get the type from type-checking, or look it up some place
                    types::I64,
                );
                self.builder.def_var(var, value);
                self.variables.insert(name.inner.to_string(), var);
                Ok(())
            }
            ast::Statement::Return { expr, .. } => match expr {
                None => {
                    self.builder.ins().return_(&[]);
                    Ok(())
                }
                Some(expr) => {
                    let ret = self
                        .eval_expr(expr)?
                        .expect("Should have been caught in type checking, that this is void");
                    self.builder.ins().return_(&[ret]);
                    Ok(())
                }
            },
            ast::Statement::Function(_) => todo!(),
            ast::Statement::Struct(_) => todo!(),
            ast::Statement::Enum(_) => todo!(),
            ast::Statement::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
        }
    }
}

pub fn build_executable(
    program: &Program,
    output: &Path,
    options: &CodegenOptions,
) -> anyhow::Result<()> {
    let mut compiler = Compiler::new(options);
    compiler.lower_program(program)?;
    let product = compiler.finish();

    // Generate the object file.
    let object_path = output.with_extension("o");
    {
        let bytes = product.emit().unwrap();

        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        let mut f = File::create(&object_path).unwrap();
        f.write_all(&bytes).unwrap();
    }

    let runtime_c = {
        let runtime_c = output.with_extension("runtime.c");
        let runtime_c_content = runtime_c_code();
        std::fs::write(&runtime_c, runtime_c_content).unwrap();
        runtime_c
    };

    let runtime_o = {
        let runtime_o = output.with_extension("runtime.o");
        let c_compiler = "cc"; // Or "clang"
        let mut cc = Command::new(c_compiler);

        cc.arg("-c").arg(runtime_c).arg("-o").arg(&runtime_o);

        let cc_status = cc.status().unwrap();

        if !cc_status.success() {
            panic!("failed to compile runtime C file");
        }
        runtime_o
    };

    let linker = "cc"; // or "clang", or "wasm-ld"
    let mut cc = Command::new(linker);

    cc.arg(&object_path).arg(runtime_o).arg("-o").arg(output);

    let _ = cc.status().unwrap();

    Ok(())
}

fn runtime_c_code() -> &'static str {
    include_str!("./runtime.c")
}
