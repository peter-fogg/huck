use crate::parser::HuckAst;

use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{FunctionValue, IntValue, PointerValue};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    env: HashMap<String, PointerValue<'ctx>>
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {

    pub fn compile (
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
        fpm: &'a PassManager<FunctionValue<'ctx>>,
        expr: HuckAst,
    ) -> Result<FunctionValue<'ctx>, &'static str> {
        let mut compiler = Self {
            context,
            builder,
            module,
            fpm,
            env: HashMap::new()
        };

        compiler.compile_main(expr)
    }

    pub fn compile_main(&mut self, expr: HuckAst) -> Result<FunctionValue<'ctx>, &'static str> {
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

    fn compile_let(&mut self, ident: String, init_expr: HuckAst) -> Result<IntValue<'ctx>, &'static str> {
        let init_val = self.compile_expr(init_expr)?;
        let allocation = self.allocate_var(&ident);
        self.builder.build_store(allocation, init_val);
        self.env.insert(ident, allocation);
        Ok(init_val)
    }

    fn compile_var_ref(&mut self, ident: String) -> Result<IntValue<'ctx>, &'static str> {
        match self.env.get(&ident) {
            Some(allocation) => {
                Ok(self.builder.build_load(*allocation, "load").into_int_value())
            },
            None => Err("identifier not found")
        }

    }

    fn compile_expr(&mut self, expr: HuckAst) -> Result<IntValue<'ctx>, &'static str> {
        match expr {
            HuckAst::Num(n) => {
                Ok(self.context.i64_type().const_int(n, false))
            },
            HuckAst::Plus(lhs, rhs) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_add(lexpr, rexpr, "tmpadd"))
            },
            HuckAst::Minus(lhs, rhs) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_sub(lexpr, rexpr, "tmpsub"))
            },
            HuckAst::Times(lhs, rhs) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_mul(lexpr, rexpr, "tmpmul"))
            },
            HuckAst::Div(lhs, rhs) => {
                let lexpr = self.compile_expr(*lhs)?;
                let rexpr = self.compile_expr(*rhs)?;

                Ok(self.builder.build_int_signed_div(lexpr, rexpr, "tmpdiv"))
            },
            HuckAst::Block(exprs) => {
                let mut result = Err("Empty block");
                for expr in exprs {
                    result = self.compile_expr(expr);
                }
                result
            },
            HuckAst::Let(ident, init_expr) => self.compile_let(ident, *init_expr),
            HuckAst::VarRef(ident) => self.compile_var_ref(ident),
        }
    }

    fn allocate_var(&self, ident: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let fn_val = self.module.get_function("main").unwrap();

        let entry = fn_val.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.i64_type(), ident)
    }
}
