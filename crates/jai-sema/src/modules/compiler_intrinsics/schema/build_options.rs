//! Validate source options and retain exact nested nominal field identities.
use super::*;
use jai_vm::{BuildOptionsProjection, RecordFieldPath};
#[derive(Clone, Copy)]
struct Field {
    name: Option<Symbol>,
    id: jai_types::FieldId,
    ty: TypeId,
}
struct Schema<'a, 'g> {
    graph: &'a ModuleGraph,
    declarations: &'a ScopedDeclarations<'g>,
    types: &'a TypeRegistry,
    context: &'a CompilerBindingContext,
    meta: &'a crate::reflection::MetaContext,
}
impl Schema<'_, '_> {
    fn fields(&self, ty: TypeId) -> Result<Vec<Field>, String> {
        if let Some(record) = self.declarations.nominals.records.get(&ty) {
            return Ok(record
                .fields
                .iter()
                .map(|field| Field {
                    name: Some(field.name),
                    id: field.id,
                    ty: field.ty,
                })
                .collect());
        }
        self.meta
            .record_specializations
            .record(ty)
            .map(|record| {
                record
                    .shape
                    .fields
                    .iter()
                    .map(|field| Field {
                        name: field.name,
                        id: field.id,
                        ty: field.ty,
                    })
                    .collect()
            })
            .ok_or_else(|| "Build_Options nested field requires a checked source struct".into())
    }
    fn field_enum(&self, ty: TypeId, members: &[&str]) -> Result<(), String> {
        let (representation, flags, values) =
            if let Some(info) = self.declarations.nominals.enums.get(&ty) {
                (
                    info.representation,
                    info.flags,
                    info.members
                        .iter()
                        .map(|(&name, &value)| (name, value))
                        .collect::<Vec<_>>(),
                )
            } else if let Some(info) = self.meta.record_specializations.member_enum(ty) {
                (info.representation, info.flags, info.values.clone())
            } else {
                return Err("build option field requires its source enum".into());
            };
        if flags
            || representation != IntegerType::U8
            || values.len() != members.len()
            || self.types.enum_definition(ty).is_err()
        {
            return Err("build option field requires its exact source u8 enum".into());
        }
        for (value, name) in members.iter().enumerate() {
            if !self.graph.symbols().find(name).is_some_and(|symbol| {
                values
                    .iter()
                    .any(|&(name, integer)| name == symbol && integer.value() == value as i128)
            }) {
                return Err(format!("build option enum requires `{name}` = {value}"));
            }
        }
        Ok(())
    }
    fn visit(
        &self,
        ty: TypeId,
        prefix: &[jai_types::FieldId],
        llvm: bool,
        projection: &mut BuildOptionsProjection,
    ) -> Result<(), String> {
        for field in self.fields(ty)? {
            let mut fields = prefix.to_vec();
            fields.push(field.id);
            let Some(name) = field.name else {
                self.visit(field.ty, &fields, llvm, projection)?;
                continue;
            };
            let name = self.graph.symbols().name(name);
            if !llvm && matches!(name, "Commonly_Propagated" | "llvm_options") {
                if name == "llvm_options" {
                    record(
                        field.ty,
                        "Llvm_Options",
                        self.graph,
                        self.declarations,
                        self.context,
                    )?;
                }
                self.visit(field.ty, &fields, name == "llvm_options", projection)?;
                continue;
            }
            let path = path(&fields)?;
            let slot = match (llvm, name) {
                (false, "output_path")
                    if matches!(self.types.kind(field.ty), Ok(TypeKind::String)) =>
                {
                    &mut projection.output_path
                }
                (false, "output_type") => {
                    self.field_enum(
                        field.ty,
                        &[
                            "NO_OUTPUT",
                            "EXECUTABLE",
                            "DYNAMIC_LIBRARY",
                            "STATIC_LIBRARY",
                            "OBJECT_FILE",
                        ],
                    )?;
                    &mut projection.output_kind
                }
                (false, "runtime_support_definitions") => {
                    self.field_enum(
                        field.ty,
                        &["AUTO", "ENTRY_POINT_AND_INIT", "ONLY_INIT", "OMIT"],
                    )?;
                    &mut projection.runtime_support
                }
                (false, "backtrace_on_crash") => {
                    self.field_enum(field.ty, &["OFF", "ON"])?;
                    &mut projection.backtrace_on_crash
                }
                (true, "target_system_triple")
                    if matches!(self.types.kind(field.ty), Ok(TypeKind::String)) =>
                {
                    &mut projection.target
                }
                (true, "bitcode_optimization_setting") => {
                    validate_enum(
                        field.ty,
                        "Llvm_Bitcode_Optimization_Setting",
                        &["UNSET", "O0", "O1", "O2", "O3", "OS", "OZ"],
                        self.graph,
                        self.declarations,
                        self.types,
                        self.context,
                    )?;
                    &mut projection.bitcode
                }
                (true, "machine_code_optimization_setting") => {
                    validate_enum(
                        field.ty,
                        "Llvm_Machine_Code_Optimization_Setting",
                        &["UNSET", "NONE", "LESS", "DEFAULT", "AGGRESSIVE"],
                        self.graph,
                        self.declarations,
                        self.types,
                        self.context,
                    )?;
                    &mut projection.machine
                }
                _ => {
                    return Err(format!(
                        "unsupported {} field `{name}`; no options were changed",
                        if llvm {
                            "Llvm_Options"
                        } else {
                            "Build_Options"
                        }
                    ));
                }
            };
            if slot.replace(path).is_some() {
                return Err(format!(
                    "duplicate Build_Options field projection for `{name}`"
                ));
            }
        }
        Ok(())
    }
}
fn path(fields: &[jai_types::FieldId]) -> Result<RecordFieldPath, String> {
    match fields {
        [outer] => Ok(RecordFieldPath {
            outer: *outer,
            inner: None,
            leaf: None,
        }),
        [outer, inner] => Ok(RecordFieldPath {
            outer: *outer,
            inner: Some(*inner),
            leaf: None,
        }),
        [outer, inner, leaf] => Ok(RecordFieldPath {
            outer: *outer,
            inner: Some(*inner),
            leaf: Some(*leaf),
        }),
        _ => Err("Build_Options nesting exceeds the supported source schema".into()),
    }
}
pub(in crate::modules::compiler_intrinsics) fn build_options_projection(
    ty: TypeId,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
    meta: &crate::reflection::MetaContext,
) -> Result<BuildOptionsProjection, String> {
    record(ty, "Build_Options", graph, declarations, context)?;
    let mut projection = BuildOptionsProjection::default();
    Schema {
        graph,
        declarations,
        types,
        context,
        meta,
    }
    .visit(ty, &[], false, &mut projection)?;
    if projection == BuildOptionsProjection::default() {
        return Err("Build_Options must contain at least one supported field".into());
    }
    Ok(projection)
}
