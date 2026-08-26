use bir::ast::{self, Program, TypeKind};
use cranelift::prelude::{isa::TargetIsa, *};
use cranelift_codegen::{
    Context,
    ir::{BlockArg, MemFlags, StackSlot, StackSlotData, StackSlotKind, Type},
};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
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

    fn declare_c_functions(&mut self) {
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
    }

    fn lower_program(&mut self, program: &Program) {
        let functions: Vec<_> = program
            .declarations
            .iter()
            .filter_map(|decl| match decl {
                ast::Declaration::Function(function) => Some(function),
                _ => None,
            })
            .collect();

        let has_main = functions.iter().any(|f| f.name == "main");
        if !has_main {
            // TODO: This should have been checked earlier
            panic!("Please provide a 'main' function");
        }

        if functions.is_empty() {
            // TODO: This should have been checked earlier
            panic!("program contains no functions");
        }

        self.declare_c_functions();

        // Declare all top-level functions
        for function in &functions {
            self.declare_function(function);
        }

        // Collect all closures from the program and declare their trampolines
        let closures = collect_closures_in_program(program);
        for closure in &closures {
            self.declare_trampoline(closure);
        }

        // Compile all closure trampolines (must happen before outer function bodies
        // so the trampolines are defined when outer functions reference them)
        for closure in &closures {
            self.compile_trampoline(closure);
        }

        // Then, lower the bodies of all top-level functions
        for function in &functions {
            self.lower_function_body(function);
        }
    }

    fn lower_function_body(&mut self, function: &ast::Function) {
        let func_id = match self.module.declarations().get_name(&function.name).unwrap() {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };

        let mut function_compiler = FunctionCompiler::new(
            &mut self.ctx,
            &mut self.fctx,
            self.isa.clone(),
            &mut self.module,
        );
        function_compiler.compile(function);
        function_compiler.finalize();

        self.module.define_function(func_id, &mut self.ctx).unwrap();

        println!("{}:\n{}", function.name, self.ctx.func);

        self.ctx.clear();
    }

    fn declare_function(&mut self, function: &ast::Function) -> FuncId {
        let mut param_types = vec![];
        for param in &function.params {
            let ty = type_from_ast(&param.ty, &self.module)
                .expect("Can't use void here. Should already have been verified");
            param_types.push(AbiParam::new(ty));
        }

        let ret_ty = if let Some(ty) = type_from_ast(&function.ret_ty, &self.module) {
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

        self.module
            .declare_function(
                &function.name,
                if function.public || function.name == "main" {
                    Linkage::Export
                } else {
                    Linkage::Local
                },
                &sig,
            )
            .unwrap()
    }

    /// Declare a trampoline function for a closure.
    /// The trampoline signature is (env_ptr: *void, ...closure_params) -> ret_ty
    fn declare_trampoline(&mut self, closure: &ClosureIR) {
        let ptr_ty = self.module.target_config().pointer_type();
        let call_conv = self.isa.default_call_conv();

        let mut params = vec![AbiParam::new(ptr_ty)]; // env_ptr
        for p in &closure.params {
            if let Some(ty) = type_from_ast(&p.ty, &self.module) {
                params.push(AbiParam::new(ty));
            }
        }
        let returns = type_from_ast(&closure.ret_ty, &self.module)
            .map(|t| vec![AbiParam::new(t)])
            .unwrap_or_default();

        let sig = Signature {
            call_conv,
            params,
            returns,
        };
        let trampoline_name = trampoline_name(&closure.mangled_name);
        self.module
            .declare_function(&trampoline_name, Linkage::Local, &sig)
            .unwrap();
    }

    /// Compile the body of a closure trampoline.
    fn compile_trampoline(&mut self, closure: &ClosureIR) {
        let trampoline_name = trampoline_name(&closure.mangled_name);
        let func_id = match self
            .module
            .declarations()
            .get_name(&trampoline_name)
            .unwrap()
        {
            cranelift_module::FuncOrDataId::Func(id) => id,
            cranelift_module::FuncOrDataId::Data(_) => panic!("Trampoline is not a function"),
        };

        let mut fc = FunctionCompiler::new(
            &mut self.ctx,
            &mut self.fctx,
            self.isa.clone(),
            &mut self.module,
        );
        fc.compile_trampoline_body(closure);
        fc.finalize();

        self.module.define_function(func_id, &mut self.ctx).unwrap();

        println!("{}:\n{}", trampoline_name, self.ctx.func);

        self.ctx.clear();
    }
}

fn trampoline_name(mangled_name: &str) -> String {
    format!("{mangled_name}__trampoline")
}

fn type_from_ast(ast_ty: &ast::TypeKind, module: &ObjectModule) -> Option<Type> {
    match &ast_ty {
        ast::TypeKind::Ident(_) => todo!(),
        ast::TypeKind::Int => Some(types::I64),
        ast::TypeKind::Float => Some(types::F64),
        ast::TypeKind::String => Some(module.target_config().pointer_type()),
        ast::TypeKind::Bool => Some(types::I8),
        ast::TypeKind::Void => None,
        // Closures are represented as pointers to a { fn_ptr, env_ptr } struct
        ast::TypeKind::Fn(_, _) => Some(module.target_config().pointer_type()),
    }
}

/// Data needed to compile a closure trampoline.
struct ClosureIR<'src> {
    mangled_name: String,
    /// (name, type, is_mutable) — mutable captures are passed by pointer
    captures: Vec<(String, ast::TypeKind<'src>, bool)>,
    params: Vec<ast::Param<'src>>,
    ret_ty: ast::TypeKind<'src>,
    body: ast::Block<'src>,
}

fn collect_closures_in_program<'src>(program: &'src Program<'src>) -> Vec<ClosureIR<'src>> {
    let mut closures = Vec::new();
    for decl in &program.declarations {
        if let ast::Declaration::Function(func) = decl {
            collect_closures_in_block(&func.body, &mut closures);
        }
    }
    closures
}

fn collect_closures_in_block<'src>(block: &'src ast::Block<'src>, out: &mut Vec<ClosureIR<'src>>) {
    for stmt in &block.statements {
        collect_closures_in_statement(stmt, out);
    }
}

fn collect_closures_in_statement<'src>(
    stmt: &'src ast::Statement<'src>,
    out: &mut Vec<ClosureIR<'src>>,
) {
    match stmt {
        ast::Statement::Let { expr, .. } | ast::Statement::Assign { expr, .. } => {
            collect_closures_in_expr(expr, out)
        }
        ast::Statement::Return {
            expr: Some(expr), ..
        } => collect_closures_in_expr(expr, out),
        ast::Statement::Expr(expr) => collect_closures_in_expr(expr, out),
        ast::Statement::ForLoop { cond, block } => {
            if let Some(cond) = cond {
                collect_closures_in_expr(cond, out);
            }
            collect_closures_in_block(block, out);
        }
        _ => {}
    }
}

fn collect_closures_in_expr<'src>(expr: &'src ast::Expr<'src>, out: &mut Vec<ClosureIR<'src>>) {
    match expr {
        ast::Expr::Closure {
            mangled_name,
            captures,
            params,
            ret_ty,
            body,
        } => {
            out.push(ClosureIR {
                mangled_name: mangled_name.clone(),
                captures: captures.clone(),
                params: params.clone(),
                ret_ty: ret_ty.clone(),
                body: body.clone(),
            });
            collect_closures_in_block(body, out);
        }
        ast::Expr::Call { call, .. } => {
            for arg in &call.args {
                collect_closures_in_expr(arg, out);
            }
        }
        ast::Expr::Binary { left, right, .. } => {
            collect_closures_in_expr(left, out);
            collect_closures_in_expr(right, out);
        }
        ast::Expr::StringInterpolation(parts) => {
            for part in parts {
                collect_closures_in_expr(part, out);
            }
        }
        ast::Expr::If {
            cond,
            then,
            else_block,
            ..
        } => {
            collect_closures_in_expr(cond, out);
            collect_closures_in_block(then, out);
            if let Some(els) = else_block {
                collect_closures_in_block(els, out);
            }
        }
        _ => {}
    }
}

struct Loop {
    header_block: Block,
    exit_block: Block,
}

struct FunctionCompiler<'a> {
    builder: FunctionBuilder<'a>,
    isa: Arc<dyn TargetIsa>,
    module: &'a mut ObjectModule,
    variables: HashMap<String, Variable>,
    loop_stack: Vec<Loop>,
    /// Stack slots allocated for mutable captured variables in the outer function.
    /// When a closure captures a mutable var by reference, its current value is kept here.
    mutable_capture_slots: HashMap<String, (StackSlot, Type)>,
    /// Pointer variables for mutable captures inside a trampoline body.
    /// Maps the capture name to the Variable holding a pointer into the outer stack slot.
    mutable_capture_ptr_vars: HashMap<String, Variable>,
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
            loop_stack: Vec::new(),
            mutable_capture_slots: HashMap::new(),
            mutable_capture_ptr_vars: HashMap::new(),
        }
    }

    fn strcat(&mut self, message_buf: Value, message: Value) -> Value {
        let func_id = match self.module.declarations().get_name("rt_concat").unwrap() {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };
        let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);

        let args = vec![message_buf, message];

        let call = self.builder.ins().call(local_callee, &args);
        self.builder.inst_results(call)[0]
    }

    fn compile(&mut self, function: &ast::Function) {
        let call_conv = self.isa.default_call_conv();

        let params = function
            .params
            .iter()
            .map(|p| {
                let ty = type_from_ast(&p.ty, self.module).unwrap();
                AbiParam::new(ty)
            })
            .collect();

        let mut returns = Vec::new();
        if let Some(t) = type_from_ast(&function.ret_ty, self.module) {
            returns.push(AbiParam::new(t))
        }

        self.builder.func.signature = Signature {
            call_conv,
            params,
            returns,
        };

        // Create the functions entry block.
        let block0 = self.builder.create_block();
        self.builder.append_block_params_for_function_params(block0);
        self.builder.switch_to_block(block0);

        // When we know that there are no more other blocks which can jump to this block, we want to seal
        // it. This improves the quality of code generation.
        self.builder.seal_block(block0);
        for (i, param) in function.params.iter().enumerate() {
            let val = self.builder.block_params(block0)[i];
            let ty = type_from_ast(&param.ty, self.module).unwrap();
            let var = self
                .variables
                .entry(param.name.inner.to_string())
                .or_insert_with(|| self.builder.declare_var(ty));
            self.builder.def_var(*var, val);
        }

        if function.name == "std.io.print" {
            let print_func_id = match self.module.declarations().get_name("rt_print_str").unwrap() {
                cranelift_module::FuncOrDataId::Func(func_id) => func_id,
                cranelift_module::FuncOrDataId::Data(_) => todo!(),
            };
            let local_callee = self
                .module
                .declare_func_in_func(print_func_id, self.builder.func);
            let var = self
                .variables
                .get("message")
                .expect("Could not find variable 'message'");
            let v = self.builder.use_var(*var);
            let call = self.builder.ins().call(local_callee, &[v]);
            self.builder.inst_results(call);
            self.builder.ins().return_(&[]);
        } else {
            // Process all statements except the last one
            let statements = &function.body.statements;
            if statements.is_empty() {
                // Empty function body
                if self.builder.func.signature.returns.is_empty() {
                    self.builder.ins().return_(&[]);
                } else {
                    panic!("Non-void function must return a value");
                }
            } else {
                // Process all statements except possibly the last
                for statement in &statements[..statements.len() - 1] {
                    let _ = self.lower_statement(statement);
                }

                // Handle the last statement
                let last_stmt = statements.last().unwrap();
                match last_stmt {
                    ast::Statement::Return { .. } => {
                        // Explicit return, just lower it
                        let _ = self.lower_statement(last_stmt);
                    }
                    ast::Statement::Expr(expr)
                        if !self.builder.func.signature.returns.is_empty() =>
                    {
                        // Last expression in a non-void function becomes the return value
                        let ret_val = self
                            .eval_expr(expr)
                            .expect("Non-void function must return a value");
                        self.builder.ins().return_(&[ret_val]);
                    }
                    _ => {
                        // Any other statement
                        self.lower_statement(last_stmt);
                        // Add implicit return for void functions
                        if self.builder.func.signature.returns.is_empty() {
                            self.builder.ins().return_(&[]);
                        }
                    }
                }
            }
        }

        if let Err(err) = codegen::verify_function(self.builder.func, self.isa.as_ref()) {
            panic!(
                "Function '{}' failed verification with error\n{err}\n{err:#?}",
                function.name
            );
        };
    }

    /// Compile the body of a closure trampoline.
    /// The trampoline loads captures from env_ptr and then runs the closure body.
    fn compile_trampoline_body(&mut self, closure: &ClosureIR) {
        let ptr_ty = self.module.target_config().pointer_type();
        let ptr_size = ptr_ty.bytes() as i32;
        let call_conv = self.isa.default_call_conv();

        // Signature: (env_ptr: ptr, ...closure_params) -> ret_ty
        let mut params = vec![AbiParam::new(ptr_ty)];
        for p in &closure.params {
            if let Some(ty) = type_from_ast(&p.ty, self.module) {
                params.push(AbiParam::new(ty));
            }
        }
        let returns = type_from_ast(&closure.ret_ty, self.module)
            .map(|t| vec![AbiParam::new(t)])
            .unwrap_or_default();

        self.builder.func.signature = Signature {
            call_conv,
            params,
            returns,
        };

        let block0 = self.builder.create_block();
        self.builder.append_block_params_for_function_params(block0);
        self.builder.switch_to_block(block0);
        self.builder.seal_block(block0);

        // env_ptr is block_params[0]
        let env_ptr = self.builder.block_params(block0)[0];

        // Set up closure params as variables (block_params[1..])
        for (i, param) in closure.params.iter().enumerate() {
            if let Some(ty) = type_from_ast(&param.ty, self.module) {
                let val = self.builder.block_params(block0)[i + 1];
                let var = self.builder.declare_var(ty);
                self.builder.def_var(var, val);
                self.variables.insert(param.name.inner.to_string(), var);
            }
        }

        // Load captures from env_ptr at successive pointer-sized offsets.
        // Immutable captures: the slot holds the value directly.
        // Mutable captures: the slot holds a pointer to the outer stack slot;
        //   we keep that pointer in a Variable so reads/writes go through it.
        for (i, (name, ty, is_mutable)) in closure.captures.iter().enumerate() {
            if let Some(cl_ty) = type_from_ast(ty, self.module) {
                let offset = (i as i32) * ptr_size;
                if *is_mutable {
                    // Load the pointer to the outer variable's storage
                    let ptr_val =
                        self.builder
                            .ins()
                            .load(ptr_ty, MemFlags::new(), env_ptr, offset);
                    let ptr_var = self.builder.declare_var(ptr_ty);
                    self.builder.def_var(ptr_var, ptr_val);
                    self.mutable_capture_ptr_vars.insert(name.clone(), ptr_var);

                    // Load the current value through the pointer for SSA reads
                    let val = self
                        .builder
                        .ins()
                        .load(cl_ty, MemFlags::new(), ptr_val, 0);
                    let var = self.builder.declare_var(cl_ty);
                    self.builder.def_var(var, val);
                    self.variables.insert(name.clone(), var);
                } else {
                    // Immutable: value stored directly
                    let val =
                        self.builder
                            .ins()
                            .load(cl_ty, MemFlags::new(), env_ptr, offset);
                    let var = self.builder.declare_var(cl_ty);
                    self.builder.def_var(var, val);
                    self.variables.insert(name.clone(), var);
                }
            }
        }

        // Compile the closure body (same pattern as regular function body)
        let statements = &closure.body.statements;
        if statements.is_empty() {
            self.builder.ins().return_(&[]);
        } else {
            for stmt in &statements[..statements.len() - 1] {
                self.lower_statement(stmt);
            }
            let last_stmt = statements.last().unwrap();
            match last_stmt {
                ast::Statement::Return { .. } => {
                    self.lower_statement(last_stmt);
                }
                ast::Statement::Expr(expr) if !self.builder.func.signature.returns.is_empty() => {
                    let ret_val = self
                        .eval_expr(expr)
                        .expect("Non-void closure must return a value");
                    self.builder.ins().return_(&[ret_val]);
                }
                _ => {
                    self.lower_statement(last_stmt);
                    if self.builder.func.signature.returns.is_empty() {
                        self.builder.ins().return_(&[]);
                    }
                }
            }
        }

        if let Err(err) = codegen::verify_function(self.builder.func, self.isa.as_ref()) {
            panic!(
                "Trampoline '{}' failed verification:\n{err}",
                closure.mangled_name
            );
        }
    }

    fn finalize(self) {
        self.builder.finalize();
    }

    fn eval_call(&mut self, call: &ast::Call, ret_ty: &ast::TypeKind) -> Option<Value> {
        // If the callee name is a local variable, it's a closure call.
        if let Some(&closure_var) = self.variables.get(&call.callee) {
            let ptr_ty = self.module.target_config().pointer_type();
            let ptr_size = ptr_ty.bytes() as i32;

            // The variable holds a pointer to { fn_ptr, env_ptr }
            let closure_ptr = self.builder.use_var(closure_var);
            let fn_ptr = self
                .builder
                .ins()
                .load(ptr_ty, MemFlags::new(), closure_ptr, 0);
            let env_ptr = self
                .builder
                .ins()
                .load(ptr_ty, MemFlags::new(), closure_ptr, ptr_size);

            // Evaluate arguments
            let mut args = vec![env_ptr];
            for arg in &call.args {
                args.push(self.eval_expr(arg).expect("void arg to closure"));
            }

            // Build the indirect call signature
            let call_conv = self.isa.default_call_conv();
            let mut sig_params = vec![AbiParam::new(ptr_ty)]; // env_ptr
            for arg in &call.args {
                if let Some(ty) = type_from_ast(arg.ty(), self.module) {
                    sig_params.push(AbiParam::new(ty));
                }
            }
            let returns = type_from_ast(ret_ty, self.module)
                .map(|t| vec![AbiParam::new(t)])
                .unwrap_or_default();

            let sig = self.builder.import_signature(Signature {
                call_conv,
                params: sig_params,
                returns,
            });

            let inst = self.builder.ins().call_indirect(sig, fn_ptr, &args);
            let ret = self.builder.inst_results(inst).first().cloned();

            // Reload any mutable captured variables from their stack slots —
            // the closure may have written new values through the stored pointers.
            let slots: Vec<(String, StackSlot, Type)> = self
                .mutable_capture_slots
                .iter()
                .map(|(n, &(s, t))| (n.clone(), s, t))
                .collect();
            for (name, slot, ty) in slots {
                if let Some(&var) = self.variables.get(&name) {
                    let val = self.builder.ins().stack_load(ty, slot, 0);
                    self.builder.def_var(var, val);
                }
            }

            return ret;
        }

        // Regular (module-level) function call
        let func_id = match self.module.declarations().get_name(&call.callee).unwrap() {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };
        let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);
        let mut args = Vec::with_capacity(call.args.len());
        for arg in &call.args {
            args.push(
                self.eval_expr(arg)
                    .expect("Should have been caught in type checking, that this is void"),
            );
        }
        let call = self.builder.ins().call(local_callee, &args);
        let inst_results = self.builder.inst_results(call);
        inst_results.first().cloned()
    }

    fn eval_expr(&mut self, expr: &ast::Expr) -> Option<Value> {
        match expr {
            ast::Expr::IntLiteral(i) => {
                let v = self.builder.ins().iconst(types::I64, *i);
                Some(v)
            }
            ast::Expr::FloatLiteral(_) => todo!(),
            ast::Expr::BoolLiteral(_) => todo!(),
            ast::Expr::StringLiteral(s) => {
                let mut data_bytes = match s {
                    Some("\\n") => "\n".as_bytes().to_vec(),
                    Some("\\t") => "\t".as_bytes().to_vec(),
                    Some(s) => s.as_bytes().to_vec(),
                    None => "".as_bytes().to_vec(),
                };
                data_bytes.push(0);
                // Setup global string
                let id = self.module.declare_anonymous_data(false, false).unwrap();
                let mut data_description = DataDescription::new();
                data_description.define(data_bytes.into_boxed_slice());
                self.module.define_data(id, &data_description).unwrap();

                let local_id = self.module.declare_data_in_func(id, self.builder.func);
                let pointer = self.module.target_config().pointer_type();
                let s = self.builder.ins().symbol_value(pointer, local_id);
                Some(s)
            }
            ast::Expr::StringInterpolation(parts) => {
                let v = self.eval_string_interpolation(parts);
                Some(v)
            }
            ast::Expr::Ident { ident, .. } => {
                let var = self
                    .variables
                    .get(ident.inner)
                    .unwrap_or_else(|| panic!("Could not find variable '{}'", ident.inner));
                let v = self.builder.use_var(*var);
                Some(v)
            }
            ast::Expr::Binary {
                op, left, right, ..
            } => {
                let left_expr = self.eval_expr(left).expect("Should be there");
                let right_expr = self.eval_expr(right).expect("Should be there");
                match op {
                    ast::BinaryOp::Add => Some(self.builder.ins().iadd(left_expr, right_expr)),
                    ast::BinaryOp::Sub => Some(self.builder.ins().isub(left_expr, right_expr)),
                    ast::BinaryOp::Mul => Some(self.builder.ins().imul(left_expr, right_expr)),
                    ast::BinaryOp::Div => Some(self.builder.ins().sdiv(left_expr, right_expr)),
                    ast::BinaryOp::Mod => Some(self.builder.ins().srem(left_expr, right_expr)),
                    ast::BinaryOp::Exp => todo!(),
                    ast::BinaryOp::Eq => {
                        Some(self.builder.ins().icmp(IntCC::Equal, left_expr, right_expr))
                    }
                    ast::BinaryOp::Ne => Some(self.builder.ins().icmp(
                        IntCC::NotEqual,
                        left_expr,
                        right_expr,
                    )),
                    ast::BinaryOp::Lt => Some(self.builder.ins().icmp(
                        IntCC::SignedLessThan,
                        left_expr,
                        right_expr,
                    )),
                    ast::BinaryOp::Gt => Some(self.builder.ins().icmp(
                        IntCC::SignedGreaterThan,
                        left_expr,
                        right_expr,
                    )),
                    ast::BinaryOp::LtEq => Some(self.builder.ins().icmp(
                        IntCC::SignedLessThanOrEqual,
                        left_expr,
                        right_expr,
                    )),
                    ast::BinaryOp::GtEq => Some(self.builder.ins().icmp(
                        IntCC::SignedGreaterThanOrEqual,
                        left_expr,
                        right_expr,
                    )),
                    ast::BinaryOp::And => todo!(),
                    ast::BinaryOp::Or => Some(self.builder.ins().bor(left_expr, right_expr)),
                }
            }
            ast::Expr::Unary { .. } => todo!(),
            ast::Expr::Call { call, ty } => self.eval_call(call, ty),
            ast::Expr::StructInstance { .. } => todo!(),
            ast::Expr::Member { .. } => todo!(),
            ast::Expr::If {
                ty,
                cond,
                then,
                else_if_blocks: _,
                else_block: els,
            } => {
                let cond_value = self.eval_expr(cond).unwrap();
                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                let return_type = type_from_ast(ty, self.module);
                let has_return_value = return_type.is_some();

                if has_return_value {
                    self.builder
                        .append_block_param(merge_block, return_type.unwrap());
                }

                self.builder
                    .ins()
                    .brif(cond_value, then_block, &[], else_block, &[]);

                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                for statement in &then.statements[..then.statements.len() - 1] {
                    self.lower_statement(statement);
                }

                if let Some(last_in_then) = then.statements.last() {
                    if has_return_value {
                        let then_return = if let ast::Statement::Expr(expr) = last_in_then {
                            self.eval_expr(expr)
                        } else {
                            None
                        };
                        self.builder
                            .ins()
                            .jump(merge_block, &[BlockArg::Value(then_return.unwrap())]);
                    } else {
                        let terminates = self.lower_statement(last_in_then);
                        if !terminates {
                            self.builder.ins().jump(merge_block, &[]);
                        }
                    }
                } else {
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);

                if let Some(els) = els {
                    for statement in &els.statements[..els.statements.len() - 1] {
                        let _ = self.lower_statement(statement);
                    }

                    if let Some(last_in_else) = els.statements.last() {
                        if has_return_value {
                            let else_return = if let ast::Statement::Expr(expr) = last_in_else {
                                self.eval_expr(expr)
                            } else {
                                None
                            };
                            self.builder
                                .ins()
                                .jump(merge_block, &[BlockArg::Value(else_return.unwrap())]);
                        } else {
                            let terminates = self.lower_statement(last_in_else);
                            if !terminates {
                                self.builder.ins().jump(merge_block, &[]);
                            }
                        }
                    } else {
                        self.builder.ins().jump(merge_block, &[]);
                    }
                } else {
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);

                if has_return_value {
                    let phi = self.builder.block_params(merge_block)[0];
                    Some(phi)
                } else {
                    None
                }
            }
            ast::Expr::Closure {
                mangled_name,
                captures,
                ..
            } => {
                let ptr_ty = self.module.target_config().pointer_type();
                let ptr_size = ptr_ty.bytes();

                // Resolve the pre-compiled trampoline
                let t_name = trampoline_name(mangled_name);
                let trampoline_id = match self.module.declarations().get_name(&t_name).unwrap() {
                    cranelift_module::FuncOrDataId::Func(id) => id,
                    _ => panic!("Trampoline not found: {t_name}"),
                };
                let trampoline_ref = self
                    .module
                    .declare_func_in_func(trampoline_id, self.builder.func);
                let fn_ptr = self.builder.ins().func_addr(ptr_ty, trampoline_ref);

                // Allocate capture struct on the stack.
                // Immutable captures: slot holds the value.
                // Mutable captures: slot holds a pointer to a dedicated stack slot for that
                //   variable, so the trampoline can read and write back through the pointer.
                let env_ptr = if captures.is_empty() {
                    // No captures — pass a null pointer as env
                    self.builder.ins().iconst(ptr_ty, 0)
                } else {
                    let capture_slot = self.builder.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        captures.len() as u32 * ptr_size,
                        0,
                    ));
                    for (i, (name, _ty, is_mutable)) in captures.iter().enumerate() {
                        let offset = (i as u32 * ptr_size) as i32;
                        if *is_mutable {
                            // Allocate (or reuse) a dedicated stack slot for this variable
                            let var = *self
                                .variables
                                .get(name)
                                .unwrap_or_else(|| panic!("Captured variable '{name}' not found"));
                            let cur_val = self.builder.use_var(var);
                            let val_ty = self.builder.func.dfg.value_type(cur_val);

                            let var_slot =
                                if let Some(&(s, _)) = self.mutable_capture_slots.get(name) {
                                    s
                                } else {
                                    let s = self.builder.create_sized_stack_slot(
                                        StackSlotData::new(
                                            StackSlotKind::ExplicitSlot,
                                            val_ty.bytes(),
                                            0,
                                        ),
                                    );
                                    self.mutable_capture_slots.insert(name.clone(), (s, val_ty));
                                    s
                                };

                            // Sync current SSA value → stack slot
                            self.builder.ins().stack_store(cur_val, var_slot, 0);
                            // Store pointer to the slot in the capture struct
                            let slot_ptr =
                                self.builder.ins().stack_addr(ptr_ty, var_slot, 0);
                            self.builder.ins().stack_store(slot_ptr, capture_slot, offset);
                        } else {
                            // Immutable: store the value directly
                            let var = *self
                                .variables
                                .get(name)
                                .unwrap_or_else(|| panic!("Captured variable '{name}' not found"));
                            let val = self.builder.use_var(var);
                            self.builder.ins().stack_store(val, capture_slot, offset);
                        }
                    }
                    self.builder.ins().stack_addr(ptr_ty, capture_slot, 0)
                };

                // Allocate closure struct { fn_ptr, env_ptr } on the stack
                let closure_slot = self.builder.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    2 * ptr_size,
                    0,
                ));
                self.builder.ins().stack_store(fn_ptr, closure_slot, 0);
                self.builder
                    .ins()
                    .stack_store(env_ptr, closure_slot, ptr_size as i32);

                let closure_ptr = self.builder.ins().stack_addr(ptr_ty, closure_slot, 0);
                Some(closure_ptr)
            }
        }
    }

    fn format_int_fn(&mut self, value: Value) -> Value {
        let func_id = match self
            .module
            .declarations()
            .get_name("rt_int_to_str")
            .expect("rt_int_to_str should be here")
        {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };
        let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);

        let args = vec![value];

        let call = self.builder.ins().call(local_callee, &args);
        self.builder.inst_results(call)[0]
    }

    fn format_float_fn(&mut self, value: Value) -> Value {
        let func_id = match self
            .module
            .declarations()
            .get_name("rt_double_to_str")
            .expect("rt_double_to_str should be here")
        {
            cranelift_module::FuncOrDataId::Func(func_id) => func_id,
            cranelift_module::FuncOrDataId::Data(_) => todo!(),
        };
        let local_callee = self.module.declare_func_in_func(func_id, self.builder.func);

        let args = vec![value];

        let call = self.builder.ins().call(local_callee, &args);
        self.builder.inst_results(call)[0]
    }

    fn eval_string_interpolation(&mut self, parts: &[ast::Expr]) -> Value {
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
            let mut v = self.eval_expr(part).expect("Found void");
            match part.ty() {
                TypeKind::Int => {
                    v = self.format_int_fn(v);
                }
                TypeKind::Float => {
                    v = self.format_float_fn(v);
                }
                TypeKind::Ident(_) => {}
                _ => {}
            }
            let new_result = self.strcat(result_ptr, v);
            result_ptr = new_result;
        }

        result_ptr
    }

    fn lower_statement(&mut self, statement: &ast::Statement) -> bool {
        match statement {
            ast::Statement::Let { name, expr, .. } => {
                let value = self
                    .eval_expr(expr)
                    .expect("This should exist. Probably didn't analyze for void");
                // Use the actual type of the produced value (handles closures, ints, pointers, etc.)
                let value_type = self.builder.func.dfg.value_type(value);
                let var = self.builder.declare_var(value_type);
                self.builder.def_var(var, value);
                self.variables.insert(name.inner.to_string(), var);
                false
            }
            ast::Statement::Assign { name, expr } => {
                // Evaluate the expression first to get the new value
                let value = self
                    .eval_expr(expr)
                    .expect("Assignment expression must return a value");

                // If this variable is a mutable capture inside a trampoline,
                // write back through the stored pointer so the outer function sees the update.
                if let Some(&ptr_var) = self.mutable_capture_ptr_vars.get(name.inner) {
                    let ptr = self.builder.use_var(ptr_var);
                    self.builder.ins().store(MemFlags::new(), value, ptr, 0);
                }

                // Update the SSA variable
                let var = self
                    .variables
                    .get(name.inner)
                    .unwrap_or_else(|| panic!("Variable '{}' not found", name.inner));
                self.builder.def_var(*var, value);
                false
            }
            ast::Statement::Return { expr, .. } => match expr {
                None => {
                    self.builder.ins().return_(&[]);
                    true
                }
                Some(expr) => {
                    let ret = self
                        .eval_expr(expr)
                        .expect("Should have been caught in type checking, that this is void");
                    self.builder.ins().return_(&[ret]);
                    true
                }
            },
            ast::Statement::ForLoop { cond, block } => {
                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.loop_stack.push(Loop {
                    header_block,
                    exit_block,
                });

                self.builder.ins().jump(header_block, &[]);
                self.builder.switch_to_block(header_block);

                let condition_value = match cond {
                    Some(cond) => self.eval_expr(cond).unwrap(),
                    None => self.builder.ins().iconst(types::I64, 1),
                };
                self.builder
                    .ins()
                    .brif(condition_value, body_block, &[], exit_block, &[]);

                self.builder.switch_to_block(body_block);
                self.builder.seal_block(body_block);

                let mut last_terminates = false;
                for stmt in &block.statements {
                    last_terminates = self.lower_statement(stmt);
                }
                if !last_terminates {
                    self.builder.ins().jump(header_block, &[]);
                }

                self.builder.switch_to_block(exit_block);

                // We've reached the bottom of the loop, so there will be no
                // more backedges to the header to exits to the bottom.
                self.builder.seal_block(header_block);
                self.builder.seal_block(exit_block);

                // Pop the loop from the stack now that we're done processing it
                self.loop_stack.pop();

                // Just return 0 for now.
                // self.builder.ins().iconst(self.int, 0);
                false
            }
            ast::Statement::Function(_) => todo!(),
            ast::Statement::Struct(_) => todo!(),
            ast::Statement::Enum(_) => todo!(),
            ast::Statement::Expr(expr) => {
                self.eval_expr(expr);
                false
            }
            ast::Statement::Break { .. } => {
                let l = self
                    .loop_stack
                    .last()
                    .expect("Analyzer should have checked that the break is inside a loop");
                self.builder.ins().jump(l.exit_block, &[]);

                true
            }
            ast::Statement::Continue { .. } => {
                let l = self
                    .loop_stack
                    .last()
                    .expect("Analyzer should have checked that the continue is inside a loop");
                self.builder.ins().jump(l.header_block, &[]);

                true
            }
        }
    }
}

pub fn build_executable(program: &Program, output: &Path, options: &CodegenOptions) {
    let mut compiler = Compiler::new(options);
    compiler.lower_program(program);
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

    match cc.output() {
        Ok(output) => {
            if !output.status.success() {
                eprintln!(
                    "LINKING ERROR:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                panic!("linker failed");
            }
        }
        Err(e) => {
            println!("LINKING ERROR {e:?}");
            panic!()
        }
    }
}

fn runtime_c_code() -> &'static str {
    include_str!("./runtime.c")
}
