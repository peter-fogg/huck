use crate::parser::{HuckAst, ParseOutput};

type TypeInfo = ();

type CheckInput = ParseOutput;

pub type CheckOutput = HuckAst<TypeInfo>;

type CheckResult = Result<CheckOutput, String>;

pub fn check(ast: CheckInput) -> CheckResult {
    Ok(ast)
}
