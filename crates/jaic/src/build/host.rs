//! Facts about the compiler and its command line, and the clang bridge.

use super::*;

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let trap = Trap::new;
    let shared = cx.shared;
    let arg = |i: usize| cx.arg(i);
    let string = |interp: &Interp, i: usize| cx.string(interp, i);
    let text = |interp: &Interp, i: usize| cx.text(interp, i);
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
        MetaOp::MainFile => {
            let file = shared.borrow().main_file.clone();
            return_string(interp, file.as_bytes(), 0)?;
            Ok(Rets::default())
        }
        MetaOp::MainOverride => {
            // 0: executable name, 1: output path, 2: `OFF` when the crash handler is not wanted.
            let value = {
                let reg = shared.borrow();
                let overrides = &reg.main_overrides;
                match arg(0) {
                    0 => overrides.exe.clone(),
                    1 => overrides.output_path.clone(),
                    _ if overrides.no_backtrace_on_crash => "OFF".to_string(),
                    _ => String::new(),
                }
            };
            return_string(interp, value.as_bytes(), 1)?;
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
        MetaOp::Clang => {
            let (name, bytes) = (text(interp, 0)?, string(interp, 3)?);
            crate::clang::call(&name, arg(1) as i64, arg(2) as i64, &bytes)
                .map(|v| Rets::one(v as u64))
                .map_err(trap)
        }
        MetaOp::ClangText => {
            return_string(interp, &crate::clang::last_text(), 0)?;
            Ok(Rets::default())
        }
        _ => unreachable!("not a host primitive"),
    }
}
