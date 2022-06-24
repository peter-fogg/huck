use crate::parser::{HuckAst, ParseOutput};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TypeInfo {
    Unit,
    Int64,
}

type CheckInput = ParseOutput;

pub type CheckOutput = HuckAst<TypeInfo>;

// I hate returning the tuple with redundant information
type CheckResult = Result<(CheckOutput, TypeInfo), String>;

pub fn check(ast: &CheckInput) -> CheckResult {
    match ast {
        HuckAst::Num(n, _) => Ok((HuckAst::Num(*n, TypeInfo::Int64), TypeInfo::Int64)),
        HuckAst::Plus(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Plus),
        HuckAst::Minus(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Minus),
        HuckAst::Times(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Times),
        HuckAst::Div(lhs, rhs, _) => check_binary(&lhs, &rhs, HuckAst::Div),
        HuckAst::Let(ident, init_expr, _) => {
            let (checked_expr, type_info) = check(&init_expr)?;
            Ok((HuckAst::Let(String::from(ident), Box::new(checked_expr), type_info), type_info))
        }
        HuckAst::Block(exprs, _) => {
            let mut last_expr_type = TypeInfo::Unit;
            let mut checked_exprs: Vec<CheckOutput> = vec![];
            checked_exprs.reserve_exact(exprs.len());

            for expr in exprs {
                let (checked_expr, type_info) = check(expr)?;
                checked_exprs.push(checked_expr);
                last_expr_type = type_info;
            }

            Ok((HuckAst::Block(checked_exprs, last_expr_type), last_expr_type))
        },
        _ => todo!("Need to implement symbol table, etc;")
    }
}

type BinaryExpr = fn (Box<CheckOutput>, Box<CheckOutput>, TypeInfo) -> CheckOutput;

fn check_binary(lhs: &CheckInput, rhs: &CheckInput, f: BinaryExpr) -> CheckResult {
    let (checked_lhs, l_type) = check(lhs)?;
    let (checked_rhs, r_type) = check(rhs)?;
    if l_type == r_type {
        Ok((f(Box::new(checked_lhs), Box::new(checked_rhs), l_type), l_type))
    } else {
        Err(format!("Cannot typecheck expressions {:?} and {:?}", lhs, rhs))
    }
}
