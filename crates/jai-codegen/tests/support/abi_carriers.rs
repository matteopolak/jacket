//! Normalize LLVM carrier grouping while preserving scalar widths and register classes.
use inkwell::types::{BasicTypeEnum, FunctionType};
fn shape(ty: BasicTypeEnum<'_>) -> String {
    match ty {
        BasicTypeEnum::IntType(t) => format!("i{}", t.get_bit_width()),
        BasicTypeEnum::FloatType(t) => format!("f{}", t.get_bit_width()),
        BasicTypeEnum::PointerType(_) => "ptr".into(),
        BasicTypeEnum::ArrayType(t) => format!("[{}x{}]", t.len(), shape(t.get_element_type())),
        BasicTypeEnum::VectorType(t) => {
            format!("<{}x{}>", t.get_size(), shape(t.get_element_type()))
        }
        BasicTypeEnum::StructType(t) => format!(
            "{{{}}}",
            t.get_field_types()
                .into_iter()
                .map(shape)
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => panic!("unsupported oracle carrier"),
    }
}
fn float_members(ty: BasicTypeEnum<'_>, members: &mut Vec<u32>) -> Option<()> {
    if members.len() > 4 {
        return None;
    }
    match ty {
        BasicTypeEnum::FloatType(t) => members.push(t.get_bit_width()),
        BasicTypeEnum::StructType(t) => {
            for field in t.get_field_types() {
                float_members(field, members)?;
            }
        }
        BasicTypeEnum::ArrayType(t) if t.len() <= 4 => {
            for _ in 0..t.len() {
                float_members(t.get_element_type(), members)?;
            }
        }
        _ => return None,
    }
    Some(())
}
fn return_shape(ty: BasicTypeEnum<'_>) -> String {
    // AArch64 returns HFAs as their floating register members. Clang retains
    // nested source record/array grouping; our marshal carrier flattens it.
    // Native nested-HFA fixtures independently verify this equivalence.
    if ty.is_struct_type() {
        let mut members = vec![];
        if float_members(ty, &mut members).is_some()
            && !members.is_empty()
            && members.len() <= 4
            && members.iter().all(|&width| width == members[0])
        {
            return format!(
                "{{{}}}",
                members
                    .into_iter()
                    .map(|width| format!("f{width}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
    }
    shape(ty)
}
pub fn function_shape(ty: FunctionType<'_>) -> (String, Vec<String>) {
    (
        ty.get_return_type()
            .map(return_shape)
            .unwrap_or("void".into()),
        ty.get_param_types()
            .into_iter()
            .map(|ty| shape(ty.try_into().unwrap()))
            .collect(),
    )
}
