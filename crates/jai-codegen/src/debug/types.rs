//! Source-named descriptors use nominal identities and the selected target layout.
use super::{Error, LineTables, bridge_error, source_descriptor};
use jai_ir::DebugSources;
use jai_llvm::{
    DebugCallingConvention, DebugMember, DebugRecordKind, DebugResultMember, DebugType,
    DebugVariadic,
};
use jai_types::{Layout, LayoutEngine, RecordKind, TypeId, TypeKind, TypeView};
use std::collections::{HashMap, HashSet};
#[cfg(test)]
mod tests;

impl LineTables<'_, '_> {
    pub(super) fn runtime_type(
        &self,
        root: TypeId,
        types: &dyn TypeView,
        sources: Option<&DebugSources>,
    ) -> Result<Option<DebugType>, Error> {
        if let Some(&ty) = self.runtime_types.borrow().get(&root) {
            return Ok(Some(ty));
        }
        let mut layouts = LayoutEngine::new(types, self.policy);
        let mut pending = vec![root];
        let mut seen = HashSet::new();
        let mut graph: HashMap<TypeId, (TypeKind, Layout)> = HashMap::new();
        // Preflight the entire reachable graph before emitting any partial record.
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) || self.runtime_types.borrow().contains_key(&ty) {
                continue;
            }
            let kind = types.kind(ty).map_err(Error::Type)?.clone();
            match &kind {
                TypeKind::Bool | TypeKind::Integer(_) | TypeKind::Float(_) => {}
                TypeKind::Pointer(pointee) => {
                    if !matches!(types.kind(*pointee).map_err(Error::Type)?, TypeKind::Void) {
                        pending.push(*pointee);
                    }
                }
                TypeKind::FixedArray { element, count } => {
                    if i64::try_from(*count).is_err() {
                        return Ok(None);
                    }
                    pending.push(*element);
                }
                TypeKind::Procedure(id) => {
                    let signature = types.procedure_type(*id).map_err(Error::Type)?;
                    pending.extend(
                        signature
                            .parameters
                            .iter()
                            .chain(signature.results.iter())
                            .copied(),
                    );
                }
                TypeKind::Record(_) => {
                    let Some(sources) = sources else {
                        return Ok(None);
                    };
                    if sources.type_source(ty).is_none() {
                        return Ok(None);
                    }
                    let record = types.record_definition(ty).map_err(Error::Type)?;
                    for (index, _) in record.fields.iter().enumerate() {
                        let field = types.field(ty, index).map_err(Error::Type)?;
                        if sources.field_source(field.id).is_none() {
                            return Ok(None);
                        }
                        pending.push(field.ty);
                    }
                }
                _ => return Ok(None),
            }
            let layout = layouts
                .layout(ty)
                .map_err(|error| Error::Metadata(error.to_string()))?
                .clone();
            graph.insert(ty, (kind, layout));
        }
        let mut cache = self.runtime_types.borrow_mut();
        for (&ty, (kind, layout)) in &graph {
            if let TypeKind::Record(_) = kind {
                let source = sources
                    .and_then(|sources| sources.type_source(ty))
                    .expect("preflight source record");
                let definition = types.record_definition(ty).map_err(Error::Type)?;
                let record = self
                    .session
                    .begin_record(
                        source_descriptor(&source.location)?,
                        source.name.as_deref(),
                        match definition.kind {
                            RecordKind::Struct => DebugRecordKind::Struct,
                            RecordKind::Union => DebugRecordKind::Union,
                        },
                        bits(layout.size)?,
                        alignment(layout.alignment)?,
                    )
                    .map_err(bridge_error)?;
                cache.insert(ty, record);
            } else if let Some(primitive) = self.primitive_type(ty, types)? {
                cache.insert(
                    ty,
                    self.session
                        .primitive_type(primitive)
                        .map_err(bridge_error)?,
                );
            }
        }
        let mut stack: Vec<_> = graph.keys().map(|&ty| (ty, false)).collect();
        let mut visiting = HashSet::new();
        while let Some((ty, finish)) = stack.pop() {
            if cache.contains_key(&ty) {
                continue;
            }
            let (kind, layout) = &graph[&ty];
            if !finish {
                if !visiting.insert(ty) {
                    return Err(Error::Metadata(
                        "recursive structural debug type without a nominal record".into(),
                    ));
                }
                stack.push((ty, true));
                match kind {
                    TypeKind::Pointer(child)
                        if !matches!(types.kind(*child).map_err(Error::Type)?, TypeKind::Void) =>
                    {
                        stack.push((*child, false))
                    }
                    TypeKind::FixedArray { element, .. } => stack.push((*element, false)),
                    TypeKind::Procedure(id) => {
                        let signature = types.procedure_type(*id).map_err(Error::Type)?;
                        stack.extend(
                            signature
                                .parameters
                                .iter()
                                .chain(signature.results.iter())
                                .map(|&ty| (ty, false)),
                        );
                    }
                    _ => {}
                }
                continue;
            }
            let metadata = match kind {
                TypeKind::Pointer(child) => self.session.pointer_type(
                    cache.get(child).copied(),
                    bits(layout.size)?,
                    alignment(layout.alignment)?,
                ),
                TypeKind::FixedArray { element, count } => self.session.array_type(
                    cache[element],
                    *count,
                    bits(layout.size)?,
                    alignment(layout.alignment)?,
                ),
                TypeKind::Procedure(id) => {
                    let signature = types.procedure_type(*id).map_err(Error::Type)?;
                    let result = match signature.results.as_ref() {
                        [] => None,
                        [ty] => Some(cache[ty]),
                        results => {
                            let tuple = layouts
                                .tuple_layout(results)
                                .map_err(|error| Error::Metadata(error.to_string()))?;
                            let members = results
                                .iter()
                                .enumerate()
                                .map(|(index, ty)| {
                                    let child = layouts
                                        .layout(*ty)
                                        .map_err(|error| Error::Metadata(error.to_string()))?;
                                    Ok(DebugResultMember {
                                        ty: cache[ty],
                                        offset_bits: bits(tuple.field_offsets[index])?,
                                        alignment_bits: alignment(child.alignment)?,
                                    })
                                })
                                .collect::<Result<Vec<_>, Error>>()?;
                            Some(
                                self.session
                                    .result_tuple_type(
                                        bits(tuple.size)?,
                                        alignment(tuple.alignment)?,
                                        &members,
                                    )
                                    .map_err(bridge_error)?,
                            )
                        }
                    };
                    let parameters: Vec<_> =
                        signature.parameters.iter().map(|ty| cache[ty]).collect();
                    let convention = match signature.convention {
                        jai_types::CallingConvention::Jai
                        | jai_types::CallingConvention::C
                        | jai_types::CallingConvention::CppMethod => DebugCallingConvention::Normal,
                        jai_types::CallingConvention::Stdcall => DebugCallingConvention::X86Stdcall,
                    };
                    let subroutine = self
                        .session
                        .subroutine_type_with_convention(
                            result,
                            &parameters,
                            if matches!(signature.variadic, jai_types::Variadic::C { .. }) {
                                DebugVariadic::C
                            } else {
                                DebugVariadic::None
                            },
                            convention,
                        )
                        .map_err(bridge_error)?;
                    self.runtime_signatures.borrow_mut().insert(ty, subroutine);
                    self.session.pointer_type(
                        Some(subroutine),
                        bits(layout.size)?,
                        alignment(layout.alignment)?,
                    )
                }
                _ => unreachable!("primitive/record slots are reserved first"),
            }
            .map_err(bridge_error)?;
            cache.insert(ty, metadata);
            visiting.remove(&ty);
        }
        for (&ty, (kind, layout)) in &graph {
            if !matches!(kind, TypeKind::Record(_)) {
                continue;
            }
            let sources = sources.expect("preflight source inventory");
            let definition = types.record_definition(ty).map_err(Error::Type)?;
            let mut members = Vec::with_capacity(definition.fields.len());
            for (index, _) in definition.fields.iter().enumerate() {
                let field = types.field(ty, index).map_err(Error::Type)?;
                let source = sources
                    .field_source(field.id)
                    .expect("preflight source field");
                let field_layout = layouts
                    .layout(field.ty)
                    .map_err(|error| Error::Metadata(error.to_string()))?;
                let field_alignment = definition
                    .layout
                    .field_alignments
                    .get(index)
                    .copied()
                    .flatten()
                    .unwrap_or(if definition.layout.packed {
                        1
                    } else {
                        field_layout.alignment
                    });
                members.push(DebugMember {
                    name: &source.name,
                    source: source_descriptor(&source.location)?,
                    ty: cache[&field.ty],
                    offset_bits: bits(layout.field_offsets[index])?,
                    alignment_bits: alignment(field_alignment)?,
                });
            }
            self.session
                .finish_record(cache[&ty], &members)
                .map_err(bridge_error)?;
        }
        Ok(cache.get(&root).copied())
    }
}
fn bits(bytes: u64) -> Result<u64, Error> {
    bytes
        .checked_mul(8)
        .ok_or_else(|| Error::Metadata("debug type byte size overflows bits".into()))
}
fn alignment(bytes: u32) -> Result<u32, Error> {
    bytes
        .checked_mul(8)
        .ok_or_else(|| Error::Metadata("debug type alignment overflows bits".into()))
}
