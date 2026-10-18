//! Adding sources to a workspace and the `Code` values handed to metaprograms.

use super::*;

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let trap = Trap::new;
    let shared = cx.shared;
    let arg = |i: usize| cx.arg(i);
    let load = |interp: &Interp, p: u64| cx.load(interp, p);
    let text = |interp: &Interp, i: usize| cx.text(interp, i);
    match op {
        MetaOp::AddFile | MetaOp::AddString => {
            let id = arg(0) as i64;
            let value = text(interp, 1)?;
            let mut reg = shared.borrow_mut();
            let ws = reg.ws(id).map_err(trap)?;
            if ws.stage == Stage::Done {
                return Err(trap(format!(
                    "workspace `{}` has already been compiled; add its sources before it completes",
                    ws.name
                )));
            }
            let fs = reg.env.fs.clone();
            // Resolved now: the metaprogram may change directory before it compiles.
            let cwd = interp
                .host
                .working_directory()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let path = fs.canonical(&cwd.join(&value));
            if op == MetaOp::AddFile && !fs.is_file(&path) {
                return Err(trap(if fs.is_dir(&path) {
                    format!("add_build_file: `{value}` is a directory, not a .jai file")
                } else {
                    format!(
                        "add_build_file: file `{value}` does not exist (relative paths start from the metaprogram's directory, {})",
                        cwd.display()
                    )
                }));
            }
            let source = if op == MetaOp::AddFile {
                ProgramSource::File(path)
            } else {
                ProgramSource::String(value)
            };
            let ws = reg.ws(id).map_err(trap)?;
            ws.queue_source(source.clone()).map_err(trap)?;
            let observer = reg.env.observer.take();
            drop(reg);
            if let Some(mut o) = observer {
                o.source_added(id, &source);
                shared.borrow_mut().env.observer = Some(o);
            }
            Ok(Rets::default())
        }
        MetaOp::AddStringToModule => {
            let (id, value, record) = (arg(0) as i64, text(interp, 1)?, arg(2) as i64);
            let mut reg = shared.borrow_mut();
            if let Some(Item::Int(file)) = reg.records.item(record, "__file", None) {
                reg.ws(id)
                    .map_err(trap)?
                    .queue_source(ProgramSource::FileString(
                        value,
                        crate::source::FileId(file as u32),
                    ))
                    .map_err(trap)?;
                return Ok(Rets::default());
            }
            // A FILE message stands for its module (the enclosing import).
            let records = &reg.records;
            let message = match records.item(record, "enclosing_import", None) {
                Some(Item::Ref(import)) => import,
                _ => record,
            };
            let module = match records.item(message, "__module", None) {
                Some(Item::Int(m)) => m as u32,
                _ => return Err(trap("add_build_string: the message names no module".into())),
            };
            let ws = reg.ws(id).map_err(trap)?;
            ws.queue_source(ProgramSource::ModuleString(value, module))
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::AddStringToCode => {
            let id = arg(0) as i64;
            let source = text(interp, 1)?;
            let code = arg(2);
            let mut reg = shared.borrow_mut();
            if id != reg.current_id() {
                return Err(trap(
                    "add_build_string: Code scope must belong to the running workspace".into(),
                ));
            }
            if code >= interp.codes.len() as u64 {
                return Err(trap("add_build_string: invalid Code scope".into()));
            }
            reg.ws(id)
                .map_err(trap)?
                .queue_source(ProgramSource::CodeString(
                    source,
                    crate::sema::value::CodeId(code as u32),
                ))
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::CodeIsNull => {
            let null = interp.codes.get(arg(0) as usize).is_some_and(|(body, _)| {
                matches!(&**body, crate::ast::CodeBody::Expr(e) if matches!(e.kind, crate::ast::ExprKind::Null))
            });
            Ok(Rets::one(null as u64))
        }
        MetaOp::CodeNodes => {
            let code = arg(0) as usize;
            let Some((body, text)) = interp.codes.get(code).cloned() else {
                return Err(trap("compiler_get_nodes: not a Code value".into()));
            };
            interp.nodes_codes.retain(|&c| c != code);
            interp.nodes_codes.push(code);
            let mut reg = shared.borrow_mut();
            let taken = interp.code_export_cursor.entry(code).or_default();
            if let Some((root, nodes)) = interp.code_exports.get(&code).and_then(|e| e.get(*taken))
            {
                *taken += 1;
                let mut result = crate::records::Record::new(crate::records::RecordTag::CodeNodes);
                result
                    .ptr("root", *root)
                    .refs("expressions", nodes.iter().copied());
                return Ok(Rets::one(reg.records.add(result) as u64));
            }
            // Names and types need the compiler: while nothing observable happened in this
            // compile-time run, ask it to export the code and run again (`call_thunk`).
            if interp.run_effects == Some(interp.effects) {
                interp.export_request = Some(code);
                return Err(trap("compiler_get_nodes: typed export requested".into()));
            }
            let (root, nodes) =
                crate::sema::code_export::export_code(&mut reg.records, &body, &text);
            let mut result = crate::records::Record::new(crate::records::RecordTag::CodeNodes);
            result.ptr("root", root).refs("expressions", nodes);
            Ok(Rets::one(reg.records.add(result) as u64))
        }
        MetaOp::ParseCode => {
            let (source, scope_from) = (text(interp, 0)?, arg(1) as usize);
            if scope_from >= interp.codes.len() {
                return Err(trap(
                    "compiler_get_code: no code to take the scope from".into(),
                ));
            }
            // A compile-time run that asks for typed nodes is run again from the start
            // (`export_request`), and does what the earlier attempt did: its n-th code is the
            // n-th code that attempt made, or it would ask for that new code's nodes again.
            let (made, replayed) = &mut interp.run_codes;
            if let Some(&id) = made.get(*replayed)
                && interp.made_codes.contains(&(id, scope_from))
                && interp.codes.get(id).is_some_and(|c| *c.1 == *source)
            {
                *replayed += 1;
                return Ok(Rets::one(id as u64));
            }
            let body = parse_code_text(crate::source::FileId(u32::MAX), &source)
                .map_err(|e| trap(format!("compiler_get_code: {e}")))?;
            let id = interp.codes.len();
            interp.codes.push((body, source.into()));
            interp.made_codes.push((id, scope_from));
            let (made, replayed) = &mut interp.run_codes;
            made.truncate(*replayed);
            made.push(id);
            *replayed = made.len();
            Ok(Rets::one(id as u64))
        }
        MetaOp::ModifyProcedure => {
            let (id, body, data, sources, count) =
                (arg(0) as i64, arg(1) as i64, arg(2), arg(3), arg(4));
            // A statement without a record (new, or edited in place) comes as source text.
            let stmts = (0..count)
                .map(|i| {
                    Ok(match load(interp, data + i * 8)? as i64 {
                        0 => {
                            let at = sources + i * 16;
                            let (len, ptr) = (load(interp, at)?, load(interp, at + 8)?);
                            let bytes = interp
                                .read(ptr, len as usize)
                                .ok_or_else(|| unreadable(ptr))?;
                            crate::sema::ModifiedStmt::Source(
                                String::from_utf8_lossy(&bytes).into_owned(),
                            )
                        }
                        r => crate::sema::ModifiedStmt::Record(r),
                    })
                })
                .collect::<Result<_, Trap>>()?;
            let mut reg = shared.borrow_mut();
            reg.ws(id).map_err(trap)?.modifications.push((body, stmts));
            Ok(Rets::default())
        }
        _ => unreachable!("not a sources primitive"),
    }
}

/// Parse the source text of a `Code` value (an expression, statement or block).
pub fn parse_code_text(
    file: crate::source::FileId,
    text: &str,
) -> Result<Rc<crate::ast::CodeBody>, String> {
    let source = format!("__jaic_code :: #code {text};\n");
    let parsed = crate::parser::parse_file(file, &source).map_err(|d| d.message.clone())?;
    parsed
        .stmts
        .into_iter()
        .find_map(|s| match s.kind {
            crate::ast::StmtKind::Decl(d) => match d.value.as_ref().map(|v| &v.kind) {
                Some(crate::ast::ExprKind::Code(body)) => Some(body.clone()),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| "could not parse the printed code".to_string())
}
