use anyhow::bail;
use ast::Program;
use cranelift::prelude::{isa::TargetIsa, *};
use cranelift_codegen::{Context, ir::Type};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use std::{fs::File, io::Write, path::Path, process::Command, sync::Arc};
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
        dbg!(func_id);
        let func_decl = self.module.declarations().get_function_decl(func_id);
        dbg!(func_decl);

        let mut function_compiler =
            FunctionCompiler::new(&mut self.ctx, &mut self.fctx, self.isa.clone());
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
            let ty = self.type_from_ast(&param.ty)?.expect("Can't use void here");
            param_types.push(AbiParam::new(ty));
        }

        let ret_ty = if let Some(ty) = self.type_from_ast(&function.ret_ty)? {
            vec![AbiParam::new(ty)]
        } else {
            vec![]
        };

        let call_conv = self.isa.default_call_conv();
        let print_sig = Signature {
            call_conv,
            params: param_types,
            returns: ret_ty,
        };
        let function_id = self
            .module
            .declare_function(function.name.inner, Linkage::Export, &print_sig)
            .unwrap();

        Ok(function_id)
    }

    fn type_from_ast(&self, ast_ty: &ast::Type) -> anyhow::Result<Option<Type>> {
        match &ast_ty.kind {
            ast::TypeKind::Ident(_) => todo!(),
            ast::TypeKind::Int => Ok(Some(types::I64)),
            ast::TypeKind::Float => Ok(Some(types::F64)),
            ast::TypeKind::String => Ok(Some(self.module.target_config().pointer_type())),
            ast::TypeKind::Bool => Ok(Some(types::I8)),
            ast::TypeKind::Void => Ok(None),
            ast::TypeKind::Generic { .. } => todo!(),
        }
    }
}

struct FunctionCompiler<'a> {
    builder: FunctionBuilder<'a>,
    isa: Arc<dyn TargetIsa>,
}

impl<'a> FunctionCompiler<'a> {
    fn new(
        ctx: &'a mut Context,
        fctx: &'a mut FunctionBuilderContext,
        isa: Arc<dyn TargetIsa>,
    ) -> Self {
        let builder = FunctionBuilder::new(&mut ctx.func, fctx);
        Self { builder, isa }
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

    fn eval_expr(&mut self, expr: &ast::Expr) -> anyhow::Result<Value> {
        match expr {
            ast::Expr::IntLiteral(i) => {
                let v = self.builder.ins().iconst(types::I64, *i);
                Ok(v)
            }
            ast::Expr::FloatLiteral(_) => todo!(),
            ast::Expr::BoolLiteral(_) => todo!(),
            ast::Expr::StringLiteral(_) => todo!(),
            ast::Expr::StringInterpolation(_) => todo!(),
            ast::Expr::Ident(_) => todo!(),
            ast::Expr::Binary { .. } => todo!(),
            ast::Expr::Unary { .. } => todo!(),
            ast::Expr::Call { .. } => todo!(),
            ast::Expr::StructInstance { .. } => todo!(),
            ast::Expr::Member { .. } => todo!(),
        }
    }

    fn lower_statement(&mut self, statement: &ast::Statement) -> anyhow::Result<()> {
        match statement {
            ast::Statement::Let { .. } => todo!(),
            ast::Statement::Return { expr, .. } => match expr {
                None => {
                    self.builder.ins().return_(&[]);
                    Ok(())
                }
                Some(expr) => {
                    let ret = self.eval_expr(expr)?;
                    self.builder.ins().return_(&[ret]);
                    Ok(())
                }
            },
            ast::Statement::Function(_) => todo!(),
            ast::Statement::Struct(_) => todo!(),
            ast::Statement::Enum(_) => todo!(),
            ast::Statement::Expr(_) => todo!(),
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

        tracing::info!("wrote object file to {output:?}");
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

    let status = cc.status().unwrap();
    tracing::debug!(?status, "Object files linked");

    Ok(())
}

fn runtime_c_code() -> String {
    r#"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <ctype.h>
#ifndef _WIN32
#include <sys/time.h>
#include <sys/types.h>
#else
#define WIN32_LEAN_AND_MEAN
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <BaseTsd.h>
typedef SSIZE_T ssize_t;

struct timeval {
    long tv_sec;
    long tv_usec;
};

static int gettimeofday(struct timeval* tv, void* tz) {
    (void)tz;
    if (!tv) {
        return -1;
    }
    FILETIME ft;
    ULONGLONG timestamp;
    static const ULONGLONG EPOCH_OFFSET = 116444736000000000ULL;
    GetSystemTimeAsFileTime(&ft);
    timestamp = ((ULONGLONG)ft.dwHighDateTime << 32) | ft.dwLowDateTime;
    timestamp -= EPOCH_OFFSET;
    tv->tv_sec = (long)(timestamp / 10000000ULL);
    tv->tv_usec = (long)((timestamp % 10000000ULL) / 10ULL);
    return 0;
}

static ssize_t fowl_getline(char** lineptr, size_t* n, FILE* stream) {
    if (!lineptr || !n || !stream) {
        return -1;
    }
    if (*lineptr == NULL || *n == 0) {
        *n = 128;
        *lineptr = (char*)malloc(*n);
        if (!*lineptr) {
            return -1;
        }
    }

    size_t position = 0;
    for (;;) {
        int c = fgetc(stream);
        if (c == EOF) {
            if (position == 0) {
                return -1;
            }
            break;
        }
        if (position + 1 >= *n) {
            size_t new_size = *n * 2;
            char* new_ptr = (char*)realloc(*lineptr, new_size);
            if (!new_ptr) {
                return -1;
            }
            *lineptr = new_ptr;
            *n = new_size;
        }
        (*lineptr)[position++] = (char)c;
        if (c == '\n') {
            break;
        }
    }
    (*lineptr)[position] = '\0';
    return (ssize_t)position;
}

#define getline fowl_getline
#endif

int fowl_is_valid_utf8(const unsigned char* str, size_t len) {
    size_t i = 0;
    while (i < len) {
        if (str[i] == 0) break;
        int bytes_needed;
        if ((str[i] & 0x80) == 0) {
            bytes_needed = 1;
        } else if ((str[i] & 0xE0) == 0xC0) {
            bytes_needed = 2;
        } else if ((str[i] & 0xF0) == 0xE0) {
            bytes_needed = 3;
        } else if ((str[i] & 0xF8) == 0xF0) {
            bytes_needed = 4;
        } else {
            return 0;
        }
        if (i + bytes_needed > len) return 0;
        for (int j = 1; j < bytes_needed; j++) {
            if ((str[i + j] & 0xC0) != 0x80) return 0;
        }
        i += bytes_needed;
    }
    return 1;
}

char* fowl_normalize_text(const char* input) {
    if (!input) return NULL;
    size_t len = strlen(input);
    if (fowl_is_valid_utf8((const unsigned char*)input, len)) {
        char* result = (char*)malloc(len + 1);
        if (result) {
            memcpy(result, input, len + 1);
        }
        return result;
    }
    char* result = (char*)malloc(len * 3 + 1);
    if (!result) return NULL;
    size_t i = 0, out_pos = 0;
    while (i < len) {
        unsigned char c = (unsigned char)input[i];
        if (c == 0) break;
        int bytes_needed = 0, valid_sequence = 1;
        if ((c & 0x80) == 0) bytes_needed = 1;
        else if ((c & 0xE0) == 0xC0) bytes_needed = 2;
        else if ((c & 0xF0) == 0xE0) bytes_needed = 3;
        else if ((c & 0xF8) == 0xF0) bytes_needed = 4;
        else { valid_sequence = 0; bytes_needed = 1; }
        if (i + bytes_needed > len) valid_sequence = 0;
        else if (bytes_needed > 1) {
            for (int j = 1; j < bytes_needed && valid_sequence; j++) {
                if ((input[i + j] & 0xC0) != 0x80) valid_sequence = 0;
            }
        }
        if (valid_sequence) {
            for (int j = 0; j < bytes_needed; j++) result[out_pos++] = input[i + j];
            i += bytes_needed;
        } else {
            result[out_pos++] = (char)0xEF;
            result[out_pos++] = (char)0xBF;
            result[out_pos++] = (char)0xBD;
            i++;
        }
    }
    result[out_pos] = '\0';
    return result;
}

void fowl_std_io_print(const char* message) {
    if (!message) return;
    char* normalized = fowl_normalize_text(message);
    if (normalized) {
        printf("%s", normalized);
        fflush(stdout);
        free(normalized);
    }
}

void fowl_std_io_println(const char* message) {
    if (!message) {
        printf("\n");
        return;
    }
    char* normalized = fowl_normalize_text(message);
    if (normalized) {
        printf("%s\n", normalized);
        free(normalized);
    }
}

char* fowl_std_io_read_line() {
    char* line = NULL;
    size_t len = 0;
    ssize_t read = getline(&line, &len, stdin);
    if (read == -1) {
        free(line);
        return NULL;
    }
    if (read > 0 && line[read-1] == '\n') {
        line[read-1] = '\0';
    }
    return line;
}

void fowl_std_io_free_string(char* ptr) {
    if (ptr) free(ptr);
}

int64_t fowl_std_time_now_ms() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (int64_t)tv.tv_sec * 1000 + tv.tv_usec / 1000;
}

char* fowl_format_float(double value) {
    char* buffer = (char*)malloc(64);
    if (buffer) {
        int len = snprintf(buffer, 64, "%.9f", value);
        if (len > 0) {
            char* p = buffer + len - 1;
            while (p > buffer && *p == '0') {
                *p = '\0';
                p--;
            }
            if (p > buffer && *p == '.') *p = '\0';
        }
    }
    return buffer;
}

char* fowl_format_int(int64_t value) {
    char* buffer = (char*)malloc(32);
    if (buffer) snprintf(buffer, 32, "%lld", (long long)value);
    return buffer;
}

char* fowl_format_bool(bool value) {
    const char* str = value ? "true" : "false";
    size_t len = strlen(str);
    char* buffer = (char*)malloc(len + 1);
    if (buffer) {
        memcpy(buffer, str, len + 1);
    }
    return buffer;
}

char* fowl_concat_strings(const char* s1, const char* s2) {
    if (!s1 || !s2) return NULL;
    size_t len1 = strlen(s1), len2 = strlen(s2);
    char* result = (char*)malloc(len1 + len2 + 1);
    if (result) {
        memcpy(result, s1, len1);
        memcpy(result + len1, s2, len2 + 1);
    }
    return result;
}

void fowl_free_string(char* ptr) {
    if (ptr) free(ptr);
}

bool fowl_error_push_context() {
    // Simple stub - always succeeds
    return true;
}

bool fowl_error_pop_context() {
    // Simple stub - always succeeds
    return true;
}

bool fowl_error_raise(const char* message_ptr, size_t message_len) {
    if (message_ptr && message_len > 0) {
        // Print error message to stderr
        fprintf(stderr, "Exception: %.*sn", (int)message_len, message_ptr);
    } else {
        fprintf(stderr, "Exception raisedn");
    }
    // For now, just print and continue - full exception handling needs stack unwinding
    return true;
}

bool fowl_error_clear() {
    // Simple stub - always succeeds
    return true;
}

char* fowl_error_get_message() {
    // Simple stub - return empty string
    char* result = (char*)malloc(1);
    if (result) result[0] = '0';
    return result;
}

bool fowl_error_has_error() {
    // Simple stub - no error state tracking yet
    return false;
}

void fowl_error_rethrow() {
    // Simple stub - do nothing
}


char* fowl_builtin_stringify_int(int64_t value) {
    char* buffer = (char*)malloc(32);
    if (buffer) {
        snprintf(buffer, 32, "%lld", (long long)value);
    }
    return buffer;
}

char* fowl_builtin_stringify_float(double value) {
    char* buffer = (char*)malloc(64);
    if (buffer) {
        int len = snprintf(buffer, 64, "%.9f", value);
        if (len > 0) {
            char* p = buffer + len - 1;
            while (p > buffer && *p == '0') {
                *p = '0';
                p--;
            }
            if (p > buffer && *p == '.') *p = '0';
        }
    }
    return buffer;
}

char* fowl_builtin_stringify_bool(int value) {
    char* buffer = (char*)malloc(6);
    if (buffer) {
        const char* str = value ? "true" : "false";
        size_t len = value ? 4 : 5;
        memcpy(buffer, str, len + 1);
    }
    return buffer;
}


void fowl_std_fmt_println(const char* msg) {
    if (!msg) {
        printf("n");
        return;
    }
    char* normalized = fowl_normalize_text(msg);
    if (normalized) {
        printf("%sn", normalized);
        free(normalized);
    }
}

void fowl_std_fmt_print(const char* msg) {
    if (!msg) return;
    char* normalized = fowl_normalize_text(msg);
    if (normalized) {
        printf("%s", normalized);
        fflush(stdout);
        free(normalized);
    }
}

void fowl_std_fmt_eprintln(const char* msg) {
    if (!msg) {
        fprintf(stderr, "n");
        return;
    }
    char* normalized = fowl_normalize_text(msg);
    if (normalized) {
        fprintf(stderr, "%sn", normalized);
        free(normalized);
    }
}

char* fowl_std_fmt_stringify_float(double value) {
    char* buffer = (char*)malloc(64);
    if (buffer) {
        int len = snprintf(buffer, 64, "%.9f", value);
        if (len > 0) {
            char* p = buffer + len - 1;
            while (p > buffer && *p == '0') {
                *p = '0';
                p--;
            }
            if (p > buffer && *p == '.') *p = '0';
        }
    }
    return buffer;
}

char* fowl_std_fmt_stringify_int(int64_t value) {
    char* buffer = (char*)malloc(32);
    if (buffer) {
        snprintf(buffer, 32, "%lld", (long long)value);
    }
    return buffer;
}


int fowl_validate_utf8(const char* ptr) {
    if (!ptr) return 0;
    while (*ptr) {
        unsigned char c = (unsigned char)*ptr;
        if (c <= 0x7F) ptr++;
        else if (c <= 0xDF) {
            if (!ptr[1] || (ptr[1] & 0xC0) != 0x80) return 0;
            ptr += 2;
        } else if (c <= 0xEF) {
            if (!ptr[1] || !ptr[2] || (ptr[1] & 0xC0) != 0x80 || (ptr[2] & 0xC0) != 0x80) return 0;
            ptr += 3;
        } else if (c <= 0xF7) {
            if (!ptr[1] || !ptr[2] || !ptr[3] ||
                (ptr[1] & 0xC0) != 0x80 || (ptr[2] & 0xC0) != 0x80 || (ptr[3] & 0xC0) != 0x80) return 0;
            ptr += 4;
        } else return 0;
    }
    return 1;
}
"#.to_string()
}
