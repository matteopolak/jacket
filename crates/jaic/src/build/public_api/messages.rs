//! Public compiler messages and fair interception state belong to the compiler.
use super::abi::{bytes, put_string, put_view, view};
use super::operations::{Call, put_strings};
use super::{Kind, Operation};
use crate::{
    build::{Event, engine},
    interp::{Interp, Rets, Trap},
    types::TypeId,
};

pub(super) fn call(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    if cx.binding.operation == Operation::CustomLinkComplete {
        let ws = cx.workspace(0)?;
        let (at, ty, schema) = interp
            .public_api_state
            .link_phases
            .remove(&ws)
            .ok_or_else(|| Trap::new("workspace is not waiting for custom linking"))?;
        let exit_code = schema.read_number(interp, ty, at)? as i64;
        cx.registry().complete_custom_link(ws, exit_code)?;
        return Ok(Rets::default());
    }
    let count = interp.public_api_state.intercepted.len();
    for _ in 0..count {
        let index = interp.public_api_state.cursor % count;
        interp.public_api_state.cursor = (index + 1) % count;
        let ws = interp.public_api_state.intercepted[index];
        if interp.public_api_state.finished.contains(&ws) {
            continue;
        }
        // Stepping another workspace's compilation is observable.
        interp.effects += 1;
        let Some(event) = engine::next_event(&cx.shared, interp, ws)? else {
            interp.public_api_state.finished.push(ws);
            continue;
        };
        let pointer = materialize(cx, interp, ws, &event)?;
        if event.kind == crate::build::EVENT_COMPLETE {
            interp.public_api_state.finished.push(ws);
            interp.public_api_state.last_complete = pointer;
        }
        return Ok(Rets::one(pointer));
    }
    Ok(Rets::one(interp.public_api_state.last_complete))
}

fn allocate(
    cx: &Call,
    interp: &mut Interp,
    name: &str,
    kind: u64,
    ws: i64,
) -> Result<(u64, TypeId), Trap> {
    let ty = cx.schema().named(name)?;
    let layout = cx.schema().get(ty)?;
    let at = interp.register_data_global(&vec![0; layout.size as usize], layout.align, false, 1)?;
    cx.set_number(interp, ty, at, "kind", kind)?;
    cx.set_number(interp, ty, at, "workspace", ws as u64)?;
    Ok((at, ty))
}

fn materialize(cx: &Call, interp: &mut Interp, ws: i64, event: &Event) -> Result<u64, Trap> {
    if matches!(event.kind, 1 | 4 | 5 | 6 | 9) {
        let base = cx.schema().pointed(cx.binding.returns[0].0)?;
        let id = *event
            .ints
            .first()
            .ok_or_else(|| Trap::new("compiler event has no record"))?;
        let pointer = super::records::build(cx, interp, id, base)?;
        if event.kind == crate::build::EVENT_TYPECHECKED {
            let ty = cx.schema().named("Message_Typechecked")?;
            let flags = cx.registry().workspace(ws)?.intercept_flags;
            filter(cx, interp, pointer, ty, flags)?;
        }
        return Ok(pointer);
    }
    let number = |i: usize| {
        event
            .ints
            .get(i)
            .copied()
            .map(|value| value as u64)
            .ok_or_else(|| Trap::new("compiler message integer field missing"))
    };
    let text = |i: usize| {
        event
            .strings
            .get(i)
            .ok_or_else(|| Trap::new("compiler message string field missing"))
    };
    match event.kind {
        2 => {
            let (at, ty) = allocate(cx, interp, "Message_Phase", 4, ws)?;
            let phase = number(0)?;
            cx.set_number(interp, ty, at, "phase", phase)?;
            if phase == 3 || phase == 5 {
                let mut cursor = 0usize;
                for (i, name) in [
                    "compiler_generated_object_files",
                    "support_object_files",
                    "system_libraries",
                    "user_libraries",
                ]
                .iter()
                .enumerate()
                {
                    let count = usize::try_from(number(i + 1)?)
                        .map_err(|_| Trap::new("invalid compiler event string count"))?;
                    let end = cursor
                        .checked_add(count)
                        .filter(|&end| end <= event.strings.len())
                        .ok_or_else(|| Trap::new("compiler event string count exceeds message"))?;
                    let values = event.strings[cursor..end]
                        .iter()
                        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                        .collect::<Vec<_>>();
                    put_strings(interp, cx.field(ty, at, name)?.0, &values)?;
                    cursor = end;
                }
                put_string(
                    interp,
                    cx.field(ty, at, "executable_name")?.0,
                    text(cursor)?,
                )?;
            } else if phase == 4 {
                cx.set_number(interp, ty, at, "executable_write_failed", number(1)?)?;
                cx.set_number(interp, ty, at, "linker_exit_code", number(2)?)?;
                put_string(interp, cx.field(ty, at, "executable_name")?.0, text(0)?)?;
            }
            if phase == 5 {
                let (exit_at, exit_ty) = cx.field(ty, at, "linker_exit_code")?;
                interp.public_api_state.link_phases.insert(
                    ws,
                    (exit_at, exit_ty, std::rc::Rc::clone(&cx.binding.schema)),
                );
            }
            Ok(at)
        }
        3 => {
            let (at, ty) = allocate(cx, interp, "Message_Complete", 6, ws)?;
            cx.set_number(interp, ty, at, "error_code", number(0)?)?;
            Ok(at)
        }
        7 => Ok(allocate(cx, interp, "Message", 8, ws)?.0),
        8 => {
            let (at, ty) = allocate(cx, interp, "Message_Debug_Dump", 7, ws)?;
            put_string(interp, cx.field(ty, at, "dump_text")?.0, text(0)?)?;
            Ok(at)
        }
        _ => Err(Trap::new("unsupported compiler message event")),
    }
}

fn filter(cx: &Call, interp: &mut Interp, at: u64, ty: TypeId, flags: i64) -> Result<(), Trap> {
    if flags == 0 {
        return Ok(());
    }
    let all_address = cx.field(ty, at, "all")?;
    let Kind::Array {
        element: all_element,
        ..
    } = cx.schema().get(all_address.1)?.kind
    else {
        return Err(Trap::new("TYPECHECKED all must be an array"));
    };
    let all_size = cx.schema().get(all_element)?.size;
    let mut all = Vec::new();
    for (name, skip) in [
        ("declarations", 2),
        ("procedure_headers", 4),
        ("procedure_bodies", 8),
        ("structs", 16),
        ("others", 32),
    ] {
        let (address, array) = cx.field(ty, at, name)?;
        let Kind::Array {
            element, ..
        } = cx.schema().get(array)?.kind
        else {
            return Err(Trap::new("TYPECHECKED fields must be arrays"));
        };
        let size = cx.schema().get(element)?.size;
        let (count, data) = view(interp, address)?;
        let mut selected = Vec::new();
        for index in 0..count {
            if flags & skip != 0 {
                continue;
            }
            let item = data + index * size;
            let expression = cx.number(interp, element, item, "expression")?;
            if flags & 1 != 0 {
                if expression == 0 {
                    continue;
                }
                let node = cx.schema().named("Code_Node")?;
                let keep = if name == "others" {
                    cx.number(interp, node, expression, "kind")? == 40
                } else {
                    let (expression, node) = if name == "procedure_bodies" {
                        let body = cx.schema().named("Code_Procedure_Body")?;
                        (
                            cx.number(interp, body, expression, "header")?,
                            cx.schema().named("Code_Procedure_Header")?,
                        )
                    } else {
                        (
                            expression,
                            cx.schema().named(match name {
                                "declarations" => "Code_Declaration",
                                "procedure_headers" => "Code_Procedure_Header",
                                "structs" => "Code_Struct",
                                _ => "Code_Node",
                            })?,
                        )
                    };
                    expression != 0 && view(interp, cx.field(node, expression, "notes")?.0)?.0 > 0
                };
                if !keep {
                    continue;
                }
            }
            selected.extend_from_slice(&bytes(interp, item, size as usize)?);
            let views = cx.field(element, item, "subexpressions")?.0;
            let start = all.len();
            all.resize(start + all_size as usize, 0);
            let (expression_offset, expression_type) =
                cx.schema().field(all_element, "expression")?;
            let (views_offset, views_type) = cx.schema().field(all_element, "subexpressions")?;
            let expression_size = cx.schema().get(expression_type)?.size as usize;
            let views_size = cx.schema().get(views_type)?.size as usize;
            all[start + expression_offset as usize
                ..start + expression_offset as usize + expression_size]
                .copy_from_slice(&expression.to_le_bytes()[..expression_size]);
            all[start + views_offset as usize..start + views_offset as usize + views_size]
                .copy_from_slice(&bytes(interp, views, views_size)?);
        }
        let pointer = interp.register_data_global(&selected, 8, false, 1)?;
        put_view(interp, address, selected.len() as u64 / size, pointer)?;
    }
    let pointer = interp.register_data_global(&all, 8, false, 1)?;
    put_view(
        interp,
        cx.field(ty, at, "all")?.0,
        all.len() as u64 / all_size,
        pointer,
    )
}
