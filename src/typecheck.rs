use crate::parser::{HuckAst, ParseOutput};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TypeInfo {
    Unit,
    Bool,
    Int64,
}

type CheckInput = ParseOutput;

pub type CheckOutput = HuckAst<TypeInfo>;

type CheckResult = Result<CheckOutput, String>;

pub fn check(ast: &CheckInput) -> CheckResult {
    match ast {
        HuckAst::Num(n, _) => Ok(HuckAst::Num(*n, TypeInfo::Int64)),
        HuckAst::BoolLit(b, _) => Ok(HuckAst::BoolLit(*b, TypeInfo::Bool)),
        HuckAst::Plus(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Plus),
        HuckAst::Minus(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Minus),
        HuckAst::Times(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Times),
        HuckAst::Div(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Div),
        HuckAst::Let(ident, init_expr, _) => {
            let checked_expr = check(&init_expr)?;
            let &type_info = checked_expr.get_metadata();
            Ok(HuckAst::Let(String::from(ident), Box::new(checked_expr), type_info))
        }
        HuckAst::Block(exprs, _) => {
            let mut last_expr_type = TypeInfo::Unit;
            let mut checked_exprs: Vec<CheckOutput> = vec![];
            checked_exprs.reserve_exact(exprs.len());

            for expr in exprs {
                let checked_expr = check(expr)?;
                let &type_info = checked_expr.get_metadata();
                checked_exprs.push(checked_expr);
                last_expr_type = type_info;
            }

            Ok(HuckAst::Block(checked_exprs, last_expr_type))
        },
        HuckAst::VarRef(_, _) => todo!("Need to implement symbol table, etc;")
    }
}

type BinaryExpr = fn (Box<CheckOutput>, Box<CheckOutput>, TypeInfo) -> CheckOutput;

fn check_binary(lhs: &CheckInput, rhs: &CheckInput, f: BinaryExpr) -> CheckResult {
    let checked_lhs = check(lhs)?;
    let checked_rhs = check(rhs)?;
    let &l_type = checked_lhs.get_metadata();
    let &r_type = checked_rhs.get_metadata();
    if l_type == r_type {
        Ok(f(Box::new(checked_lhs), Box::new(checked_rhs), l_type))
    } else {
        Err(format!("Cannot typecheck expressions {:?} and {:?}", lhs, rhs))
    }
}
