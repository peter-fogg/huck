use crate::parser::HuckAst;
use crate::typecheck::{CheckOutput, TypeInfo};

use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::{AnyType, AnyTypeEnum, BasicTypeEnum};
use inkwell::values::{FunctionValue, IntValue, PointerValue};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    env: HashMap<String, PointerValue<'ctx>>
}

type CompileInput = CheckOutput;

type CompileResult<T> = Result<T, String>;

impl<'a, 'ctx> Compiler<'a, 'ctx> {

    pub fn compile (
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
        fpm: &'a PassManager<FunctionValue<'ctx>>,
        expr: CompileInput,
    ) -> CompileResult<FunctionValue<'ctx>> {
        let mut compiler = Self {
            context,
            builder,
            module,
            fpm,
            env: HashMap::new()
        };

        compiler.compile_main(expr)
    }

    pub fn compile_main(&mut self, expr: CompileInput) -> CompileResult<FunctionValue<'ctx>> {
        let fn_type = self.context.i64_type().fn_type(&[], false);

        // try calling print_int
        let print_fn_type = self.context.void_type().fn_type(&[self.context.i64_type().into()], false);
        let print_int = self.module.add_function("print_int", print_fn_type, None);
        
        // end call

        let fn_val = self.module.add_function("main", fn_type, None);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let body = self.compile_expr(expr)?;
        self.builder.build_call(print_int, &[body.into()], "tmp").try_as_basic_value();

        self.builder.build_return(Some(&body));

        Ok(fn_val)
    }

    fn compile_let(&mut self, ident: String, init_expr: CompileInput) -> CompileResult<IntValue<'ctx>> {
        let llvm_type = self.get_llvm_basic_type(&init_expr);
        let init_val = self.compile_expr(init_expr)?;
        let allocation = self.allocate_var(&ident, llvm_type);
        self.builder.build_store(allocation, init_val);
        self.env.insert(ident, allocation);
        Ok(init_val)
    }

    fn compile_var_ref(&mut self, ident: String) -> CompileResult<IntValue<'ctx>> {
        match self.env.get(&ident) {
            Some(allocation) => {
                Ok(self.builder.build_load(*allocation, "load").into_int_value())
            },
            None => Err(format!("identifier \"{}\" not found", ident))
        }

    }

    fn compile_expr(&mut self, expr: CompileInput) -> CompileResult<IntValue<'ctx>> {
        match expr {
            HuckAst::Num(n, _) => {
                Ok(self.context.i64_type().const_int(n, false))
            },
            HuckAst::Plus(lhs, rhs, _) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_add(lexpr, rexpr, "tmpadd"))
            },
            HuckAst::Minus(lhs, rhs, _) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_sub(lexpr, rexpr, "tmpsub"))
            },
            HuckAst::Times(lhs, rhs, _) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_mul(lexpr, rexpr, "tmpmul"))
            },
            HuckAst::Div(lhs, rhs, _) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_signed_div(lexpr, rexpr, "tmpdiv"))
            },
            HuckAst::Block(exprs, _) => {
                let mut result = Err(String::from("Empty block"));
                for expr in exprs {
                    result = self.compile_expr(expr);
                }
                result
            },
            HuckAst::Let(ident, init_expr, _) => self.compile_let(ident, *init_expr),
            HuckAst::VarRef(ident, _) => self.compile_var_ref(ident),
            HuckAst::BoolLit(b, _) => Ok(self.context.bool_type().const_int(
                if b { 1 } else { 0 },
                false
            )),
            HuckAst::If(_, _, _, _) => todo!("Haven't gotten to if statements yet"),
        }
    }

    fn allocate_var(&self, ident: &str, llvm_type: BasicTypeEnum<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let fn_val = self.module.get_function("main").unwrap();

        let entry = fn_val.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(llvm_type, ident)
    }

    fn get_llvm_basic_type(&self, expr: &CompileInput) -> BasicTypeEnum<'ctx> {
        // this is a hack
        BasicTypeEnum::try_from(self.get_llvm_type(expr)).unwrap()
    }

    fn get_llvm_type(&self, expr: &CompileInput) -> AnyTypeEnum<'ctx> {
        let type_info = expr.get_metadata();
        match type_info {
            TypeInfo::Unit => self.context.void_type().as_any_type_enum(),
            TypeInfo::Bool => self.context.bool_type().as_any_type_enum(),
            TypeInfo::Int64 => self.context.i64_type().as_any_type_enum(),
        }
    }
}
