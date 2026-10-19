//! Facts about the compiler and its command line.

use super::{Call, Interp, MetaOp, Rets, Trap, compiler_version};

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let shared = cx.shared;
    let arg = |i: usize| cx.arg(i);
    let return_string =
        |interp: &mut Interp, bytes: &[u8], i: usize| cx.return_string(interp, bytes, i);
    match op {
        MetaOp::BasePath => {
            // The install root: the directory above the stdlib, which is the last module
            // directory on the import path.
            let base = shared
                .borrow()
                .env
                .options
                .import_paths
                .last()
                .and_then(|stdlib| stdlib.parent())
                .map(|p| {
                    let mut text = crate::canonicalize(p)
                        .unwrap_or_else(|_| p.to_path_buf())
                        .display()
                        .to_string();
                    if !text.ends_with('/') && !text.ends_with('\\') {
                        text.push('/');
                    }
                    text
                })
                .unwrap_or_default();
            return_string(interp, base.as_bytes(), 0)?;
            Ok(Rets::default())
        }
        MetaOp::CommandLineCount => Ok(Rets::one(shared.borrow().env.command_line.len() as u64)),
        MetaOp::CommandLineArg => {
            let value = shared
                .borrow()
                .env
                .command_line
                .get(arg(0) as usize)
                .cloned()
                .unwrap_or_default();
            return_string(interp, value.as_bytes(), 1)?;
            Ok(Rets::default())
        }
        MetaOp::CompilerVersion => {
            return_string(interp, compiler_version().as_bytes(), 0)?;
            Ok(Rets::default())
        }
        _ => unreachable!("not a host primitive"),
    }
}
