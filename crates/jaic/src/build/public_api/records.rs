//! Materialize compiler-owned records using the caller's resolved type layouts.
use super::abi::put_view;
use super::operations::Call;
use super::{Kind, Schema};
use crate::{
    build::{Workspaces, engine},
    interp::{Interp, Rets, Trap},
    record_build::{Builder, PlanEntry, Shape},
    records::{FieldRef, Name},
    sema::{ProvidedImport, ProvidedKind},
    types::TypeId,
};

fn descriptor(cx: &Call, interp: &mut Interp, ty: TypeId) -> Result<u64, Trap> {
    let global = cx
        .schema()
        .get(ty)
        .descriptor
        .ok_or_else(|| Trap::new("compiler record has no type descriptor"))?;
    interp.global_addr(cx.program, global)
}

/// How a value of `ty` is stored in a built record.
fn shape(schema: &Schema, ty: TypeId) -> Shape {
    let layout = schema.get(ty);
    match layout.kind {
        Kind::Scalar if layout.size > 0 => Shape::Int,
        Kind::Scalar => Shape::Other,
        Kind::String => Shape::String,
        Kind::Pointer(_) => Shape::Pointer,
        Kind::Record(_) => Shape::Struct,
        Kind::Array {
            count: None, ..
        } => Shape::Array,
        Kind::Array {
            count: Some(_), ..
        } => Shape::FixedArray,
    }
}

/// The members a record of type `ty` at `base` fills; promoted (`using`) struct members are
/// flattened into their container.
fn entries(
    cx: &Call,
    interp: &mut Interp,
    ty: TypeId,
    base: u64,
    output: &mut Vec<PlanEntry>,
) -> Result<(), Trap> {
    let schema = cx.schema();
    let Kind::Record(fields) = &schema.get(ty).kind else {
        return Ok(());
    };
    for field in fields {
        if field.promoted && matches!(schema.get(field.ty).kind, Kind::Record(_)) {
            entries(cx, interp, field.ty, base + field.offset, output)?;
            continue;
        }
        let layout = schema.get(field.ty);
        // A pointer's (or pointer element's) target, or the struct held in place.
        let mut target = |ty: TypeId| match schema.get(ty).kind {
            Kind::Pointer(inner) => descriptor(cx, interp, inner),
            Kind::Record(_) => descriptor(cx, interp, ty),
            _ => Ok(0),
        };
        let name = Name::intern(&field.name);
        let size = layout.size as usize;
        let entry = match layout.kind {
            Kind::Array {
                element,
                resizable,
                ..
            } => PlanEntry {
                offset: base + field.offset,
                shape: shape(schema, field.ty),
                size,
                name,
                element: shape(schema, element),
                element_size: schema.get(element).size,
                target: target(element)?,
                resizable,
            },
            _ => PlanEntry::single(
                base + field.offset,
                shape(schema, field.ty),
                size,
                name,
                target(field.ty)?,
            ),
        };
        output.push(entry);
    }
    Ok(())
}

/// This program's record builder, given the plan of every record type the binding's schema
/// knows that it has not been given yet.
fn builder(cx: &Call, interp: &mut Interp) -> Result<usize, Trap> {
    let handle = if let Some(handle) = interp.public_api_state.builder {
        handle
    } else {
        let mut reg = cx.registry();
        reg.builders.push(Builder::default());
        let handle = reg.builders.len() - 1;
        interp.public_api_state.builder = Some(handle);
        handle
    };
    for (&ty, layout) in &cx.schema().types {
        if layout.descriptor.is_none() {
            continue;
        }
        let info = descriptor(cx, interp, ty)?;
        if !interp.public_api_state.planned.insert(info) {
            continue;
        }
        if matches!(layout.kind, Kind::Record(_)) {
            let mut plan = Vec::new();
            entries(cx, interp, ty, 0, &mut plan)?;
            let builder = &mut cx.registry().builders[handle];
            builder.add_type(layout.name.clone(), info);
            builder.add_plan(info, layout.size, plan, layout.name == "Type_Info_Struct");
        } else {
            cx.registry().builders[handle].add_builtin(layout.name.clone(), info);
        }
    }
    Ok(handle)
}

/// The struct record `id` is built into, as a `ty`.
pub(super) fn build(cx: &Call, interp: &mut Interp, id: i64, ty: TypeId) -> Result<u64, Trap> {
    let handle = builder(cx, interp)?;
    let expected = descriptor(cx, interp, ty)?;
    let mut reg = cx.registry();
    let Workspaces {
        records,
        kept,
        builders,
        ..
    } = &mut *reg;
    builders[handle].build(records, kept, interp, id, expected)
}

/// The record a compiler-made pointer came from, or 0 for memory the program made.
pub(super) fn record_of(cx: &Call, interp: &mut Interp, address: u64) -> Result<i64, Trap> {
    let handle = builder(cx, interp)?;
    Ok(cx.registry().builders[handle].record_of(address))
}

pub(super) fn record_id(cx: &Call, interp: &mut Interp, address: u64) -> Result<i64, Trap> {
    let id = record_of(cx, interp, address)?;
    if id == 0 {
        return Err(Trap::new(
            "syntax/message pointer was not produced by this compiler",
        ));
    }
    Ok(id)
}

pub(super) fn add_string(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    let ws = cx.workspace(1)?;
    let text = cx.string(interp, 0)?;
    if cx.arg(2) == 0 {
        crate::build::engine::add_source(&cx.shared, interp, ws, text, false)?;
        return Ok(Rets::default());
    }
    let ty = cx.schema().pointed(cx.binding.parameters[2])?;
    if cx.number(interp, ty, cx.arg(2), "workspace")? as i64 != ws {
        return Err(Trap::new(
            "add_build_string message belongs to another workspace",
        ));
    }
    let kind = cx.number(interp, ty, cx.arg(2), "kind")?;
    if kind != 1 && kind != 2 {
        return Err(Trap::new(
            "add_build_string requires FILE or IMPORT message",
        ));
    }
    let record = record_id(cx, interp, cx.arg(2))?;
    cx.registry().add_string_to_module(ws, text, record)?;
    Ok(Rets::default())
}

pub(super) fn provide_import(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    if cx.arg(1) == 0 {
        return Err(Trap::new("provide_import requires a failed import message"));
    }
    let ty = cx.schema().pointed(cx.binding.parameters[1])?;
    let offer = ProvidedImport {
        host: cx.text(interp, ty, cx.arg(1), "host_module_name")?,
        target: cx.text(interp, ty, cx.arg(1), "target_module_name")?,
        kind: match cx.arg(2) {
            0 => ProvidedKind::ShortName,
            1 => ProvidedKind::PathToFile,
            2 => ProvidedKind::PathToDirectory,
            _ => ProvidedKind::FullText,
        },
        value: cx.string(interp, 3)?,
    };
    let ws = cx.workspace(0)?;
    cx.registry().provide_import(ws, offer)?;
    Ok(Rets::default())
}

pub(super) fn call(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    use super::Operation::{GetCode, GetNodes, MakeLive, ModifyProcedure, RootType};
    match cx.binding.operation {
        GetNodes => {
            let (root, expressions) = engine::code_nodes(&cx.shared, interp, cx.arg(0))?;
            let root_type = cx.schema().pointed(cx.binding.returns[0].0)?;
            let pointer = build(cx, interp, root, root_type)?;
            let mut values = Vec::new();
            for id in expressions {
                values.extend_from_slice(&build(cx, interp, id, root_type)?.to_le_bytes());
            }
            let data = interp.register_data_global(&values, 8, false, 1)?;
            put_view(interp, cx.result(1)?.0, values.len() as u64 / 8, data)?;
            Ok(Rets::one(pointer))
        }
        RootType => {
            if engine::code_is_null(interp, cx.arg(0)) {
                return Ok(vec![2, 0].into());
            }
            let (root, _) = engine::code_nodes(&cx.shared, interp, cx.arg(0))?;
            let ty = match cx.shared.borrow().records.field(root, "type") {
                Some(FieldRef::Ref(id)) => id,
                _ => 0,
            };
            if root == 0 {
                return Ok(vec![3, 0].into());
            }
            if ty == 0 {
                return Ok(vec![4, 0].into());
            }
            let info = cx.schema().named("Type_Info")?;
            Ok(vec![1, build(cx, interp, ty, info)?].into())
        }
        MakeLive => {
            let record = record_id(cx, interp, cx.arg(1))?;
            let ws = cx.workspace(0)?;
            cx.registry().make_live(ws, record)?;
            Ok(Rets::default())
        }
        GetCode => super::convert::get_code(cx, interp),
        ModifyProcedure => super::convert::modify(cx, interp),
        _ => unreachable!(),
    }
}
