use crate::parser::{HuckAst, ParseOutput};

use std::collections::HashMap;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TypeInfo {
    Unit,
    Bool,
    Int64,
}

type CheckInput = ParseOutput;

pub type CheckOutput = HuckAst<TypeInfo>;

type CheckResult = Result<CheckOutput, String>;

pub struct Checker {
    env: Vec<HashMap<String, TypeInfo>>
}

impl Checker {
    pub fn new() -> Self {
        Self {
            env: vec![HashMap::new()]
        }
    }

    fn begin_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.env.pop();
    }

    fn add_var(&mut self, ident: String, info: TypeInfo) {
        let map = self.env.last_mut().unwrap();
        map.insert(ident, info);
    }

    fn get_var(&mut self, ident: String) -> Option<TypeInfo> {
        for map in self.env.iter().rev() {
            if let Some(&info) = map.get(&ident) {
                return Some(info)
            } else {
                continue;
            }
        }
        None
    }

    pub fn check(&mut self, ast: &CheckInput) -> CheckResult {
        match ast {
            HuckAst::Num(n, _) => Ok(HuckAst::Num(*n, TypeInfo::Int64)),
            HuckAst::BoolLit(b, _) => Ok(HuckAst::BoolLit(*b, TypeInfo::Bool)),
            HuckAst::Plus(lhs, rhs, _) => self.check_binary(lhs, rhs, HuckAst::Plus),
            HuckAst::Minus(lhs, rhs, _) => self.check_binary(lhs, rhs, HuckAst::Minus),
            HuckAst::Times(lhs, rhs, _) => self.check_binary(lhs, rhs, HuckAst::Times),
            HuckAst::Div(lhs, rhs, _) => self.check_binary(lhs, rhs, HuckAst::Div),
            HuckAst::Let(ident, init_expr, _) => {
                let checked_expr = self.check(init_expr)?;
                let &type_info = checked_expr.get_metadata();
                self.add_var(ident.to_string(), type_info);
                Ok(HuckAst::Let(String::from(ident), Box::new(checked_expr), type_info))
            }
            HuckAst::Block(exprs, _) => {
                self.begin_scope();
                let mut last_expr_type = TypeInfo::Unit;
                let mut checked_exprs: Vec<CheckOutput> = vec![];
                checked_exprs.reserve_exact(exprs.len());

                for expr in exprs {
                    let checked_expr = self.check(expr)?;
                    let &type_info = checked_expr.get_metadata();
                    checked_exprs.push(checked_expr);
                    last_expr_type = type_info;
                }

                self.end_scope();
                Ok(HuckAst::Block(checked_exprs, last_expr_type))
            },
            HuckAst::If(test_expr, then_expr, else_expr, _) => {
                let checked_test = self.check(test_expr)?;
                if *checked_test.get_metadata() != TypeInfo::Bool {
                    Err(String::from("Require boolean condition for if expression"))
                } else {
                    let checked_then = self.check(then_expr)?;
                    let checked_else = self.check(else_expr)?;
                    let &then_type = checked_then.get_metadata();
                    let &else_type = checked_else.get_metadata();
                    if then_type == else_type {
                        Ok(
                            HuckAst::If(
                                Box::new(checked_test),
                                Box::new(checked_then),
                                Box::new(checked_else),
                                then_type
                            )
                        )
                    } else {
                        Err(format!(
                            "Conditional branch types {:?} and {:?} do not match",
                            then_type,
                            else_type
                        ))
                    }
                }
            },
            HuckAst::VarRef(ident, _) => {
                if let Some(type_info) = self.get_var(ident.to_string()) {
                    Ok(HuckAst::VarRef(String::from(ident), type_info))
                } else {
                    Err(format!("Unbound variable {:?}", ident))
                }
            },
        }
    }
    
    fn check_binary(&mut self, lhs: &CheckInput, rhs: &CheckInput, f: BinaryExpr) -> CheckResult {
        let checked_lhs = self.check(lhs)?;
        let checked_rhs = self.check(rhs)?;
        let &l_type = checked_lhs.get_metadata();
        let &r_type = checked_rhs.get_metadata();
        if l_type == r_type {
            Ok(f(Box::new(checked_lhs), Box::new(checked_rhs), l_type))
        } else {
            Err(format!("Cannot typecheck expressions {:?} and {:?}", lhs, rhs))
        }
    }
}

type BinaryExpr = fn (Box<CheckOutput>, Box<CheckOutput>, TypeInfo) -> CheckOutput;
