//! Materialize compiler-owned records using the caller's resolved type layouts.
use super::abi::{bytes, put_view};
use super::operations::{Call, temporary_string};
use super::{Kind, Schema};
use crate::{
    build::MetaOp,
    interp::{Interp, Rets, Trap},
    records::{FieldRef, Item},
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

pub(super) fn builder(cx: &Call, interp: &mut Interp) -> Result<u64, Trap> {
    let handle = if let Some(handle) = interp.public_api_state.builder {
        handle as u64
    } else {
        let handle = cx.invoke(interp, MetaOp::RecBuilder, &[])?[0];
        interp.public_api_state.builder = Some(handle as usize);
        handle
    };
    fn kind(schema: &Schema, ty: TypeId) -> u64 {
        match schema.get(ty).kind {
            Kind::Scalar => u64::from(schema.get(ty).size > 0),
            Kind::String => 2,
            Kind::Pointer(_) => 3,
            Kind::Record(_) => 4,
            Kind::Array {
                count: None, ..
            } => 5,
            Kind::Array {
                count: Some(_), ..
            } => 6,
        }
    }
    fn entries(
        cx: &Call,
        interp: &mut Interp,
        ty: TypeId,
        base: u64,
        output: &mut Vec<u8>,
    ) -> Result<(), Trap> {
        let Kind::Record(fields) = &cx.schema().get(ty).kind else {
            return Ok(());
        };
        for field in fields {
            if field.promoted && matches!(cx.schema().get(field.ty).kind, Kind::Record(_)) {
                entries(cx, interp, field.ty, base + field.offset, output)?;
                continue;
            }
            let layout = cx.schema().get(field.ty);
            let name = temporary_string(interp, field.name.as_bytes())?;
            let pair = bytes(interp, name, 16)?;
            let (element_kind, element_size, target, resizable) = match layout.kind {
                Kind::Pointer(inner) => (0, 0, descriptor(cx, interp, inner)?, false),
                Kind::Record(_) => (0, 0, descriptor(cx, interp, field.ty)?, false),
                Kind::Array {
                    element,
                    resizable,
                    ..
                } => {
                    let target = match cx.schema().get(element).kind {
                        Kind::Pointer(inner) => descriptor(cx, interp, inner)?,
                        Kind::Record(_) => descriptor(cx, interp, element)?,
                        _ => 0,
                    };
                    (
                        kind(cx.schema(), element),
                        cx.schema().get(element).size,
                        target,
                        resizable,
                    )
                }
                _ => (0, 0, 0, false),
            };
            for value in [
                base + field.offset,
                kind(cx.schema(), field.ty),
                layout.size,
            ] {
                output.extend_from_slice(&value.to_le_bytes());
            }
            output.extend_from_slice(&pair);
            output.extend_from_slice(&0u64.to_le_bytes());
            for value in [element_kind, element_size, target, u64::from(resizable)] {
                output.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(())
    }
    for (&ty, layout) in &cx.schema().types {
        let Some(_) = layout.descriptor else {
            continue;
        };
        let info = descriptor(cx, interp, ty)?;
        if !interp.public_api_state.planned.insert(info) {
            continue;
        }
        let name = temporary_string(interp, layout.name.as_bytes())?;
        if matches!(layout.kind, Kind::Record(_)) {
            cx.invoke(interp, MetaOp::RecType, &[handle, name, info])?;
            let mut plan = Vec::new();
            entries(cx, interp, ty, 0, &mut plan)?;
            let at = interp.register_data_global(&plan, 8, false, 1)?;
            cx.invoke(
                interp,
                MetaOp::RecPlan,
                &[
                    handle,
                    info,
                    layout.size,
                    at,
                    plan.len() as u64 / 80,
                    u64::from(layout.name == "Type_Info_Struct"),
                ],
            )?;
        } else {
            cx.invoke(interp, MetaOp::RecBuiltin, &[handle, name, info])?;
        }
    }
    Ok(handle)
}

pub(super) fn build(cx: &Call, interp: &mut Interp, id: i64, ty: TypeId) -> Result<u64, Trap> {
    let handle = builder(cx, interp)?;
    let expected = descriptor(cx, interp, ty)?;
    Ok(cx.invoke(interp, MetaOp::RecBuild, &[handle, id as u64, expected])?[0])
}

pub(super) fn record_id(cx: &Call, interp: &mut Interp, address: u64) -> Result<i64, Trap> {
    let handle = builder(cx, interp)?;
    let id = cx.invoke(interp, MetaOp::RecOf, &[handle, address])?[0] as i64;
    if id == 0 {
        return Err(Trap::new(
            "syntax/message pointer was not produced by this compiler",
        ));
    }
    Ok(id)
}

pub(super) fn add_string(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    let ws = cx.workspace(1)? as u64;
    if cx.arg(2) == 0 {
        return cx.invoke(interp, MetaOp::AddString, &[ws, cx.arg(0)]);
    }
    let ty = cx.schema().pointed(cx.binding.parameters[2])?;
    if cx.number(interp, ty, cx.arg(2), "workspace")? != ws {
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
    cx.invoke(
        interp,
        MetaOp::AddStringToModule,
        &[ws, cx.arg(0), record as u64],
    )
}

pub(super) fn provide_import(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    if cx.arg(1) == 0 {
        return Err(Trap::new("provide_import requires a failed import message"));
    }
    let ty = cx.schema().pointed(cx.binding.parameters[1])?;
    cx.invoke(
        interp,
        MetaOp::ProvideImport,
        &[
            cx.workspace(0)? as u64,
            cx.field(ty, cx.arg(1), "host_module_name")?.0,
            cx.field(ty, cx.arg(1), "target_module_name")?.0,
            cx.arg(2),
            cx.arg(3),
        ],
    )
}

pub(super) fn call(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    use super::Operation::{GetCode, GetNodes, MakeLive, ModifyProcedure, RootType};
    match cx.binding.operation {
        GetNodes => {
            let nodes = cx.invoke(interp, MetaOp::CodeNodes, &[cx.arg(0)])?[0] as i64;
            let (root, expressions) = {
                let registry = cx.shared.borrow();
                let root = match registry.records.field(nodes, "root") {
                    Some(FieldRef::Ref(id)) => id,
                    _ => 0,
                };
                let list = match registry.records.field(nodes, "expressions") {
                    Some(FieldRef::List(items)) => items
                        .iter()
                        .filter_map(|item| match item {
                            Item::Ref(id) => Some(id),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                };
                (root, list)
            };
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
            if cx.invoke(interp, MetaOp::CodeIsNull, &[cx.arg(0)])?[0] != 0 {
                return Ok(vec![2, 0].into());
            }
            let nodes = cx.invoke(interp, MetaOp::CodeNodes, &[cx.arg(0)])?[0] as i64;
            let (root, ty) = {
                let registry = cx.shared.borrow();
                let root = match registry.records.field(nodes, "root") {
                    Some(FieldRef::Ref(id)) => id,
                    _ => 0,
                };
                let ty = match registry.records.field(root, "type") {
                    Some(FieldRef::Ref(id)) => id,
                    _ => 0,
                };
                (root, ty)
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
            cx.invoke(
                interp,
                MetaOp::MakeLive,
                &[cx.workspace(0)? as u64, record as u64],
            )
        }
        GetCode => super::syntax::get_code(cx, interp),
        ModifyProcedure => super::syntax::modify(cx, interp),
        _ => unreachable!(),
    }
}
