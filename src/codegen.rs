use crate::typecheck::CheckOutput;

use std::io::Write;

type CompileInput = CheckOutput;

// type CompileResult<T> = Result<T, String>;
type CompileResult<T> = T;

pub fn compile<T>(_code: CompileInput, output: &mut T) -> CompileResult<()>
where T: Write
{
    write_header(output);
    output.write(
        "  movl $1, %eax\n  ret\n".as_bytes()
    );
}

fn write_header<T>(output: &mut T) where T: Write {
    output.write(".global main\nmain:\n".as_bytes());
}
