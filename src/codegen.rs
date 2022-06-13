use crate::parser::HuckAst;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{FunctionValue, IntValue};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>
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
        };

        compiler.compile_main(expr)
    }

    pub fn compile_main(&mut self, expr: HuckAst) -> Result<FunctionValue<'ctx>, &'static str> {
        let fn_type = self.context.i64_type().fn_type(&[], false);
        let fn_val = self.module.add_function("main", fn_type, None);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let body = self.compile_expr(expr)?;
        self.builder.build_return(Some(&body));

        Ok(fn_val)
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
        }
    }
}
