use crate::parser::HuckAst::Num;
use crate::typecheck::CheckOutput;

use std::io::Write;

type CompileInput = CheckOutput;

// type CompileResult<T> = Result<T, String>;
type CompileResult<T> = T;

pub fn compile<T>(code: CompileInput, output: &mut T) -> CompileResult<()>
where T: Write
{
    match code {
        Num(n, _) => {
            write_header(output);
            output.write(
                format!("  movl ${}, %eax\n  ret\n", n).as_bytes()
            );
        },
    }
}

fn write_header<T>(output: &mut T) where T: Write {
    output.write(".global main\nmain:\n".as_bytes());
}
