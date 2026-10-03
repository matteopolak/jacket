//! Adopt the designated source module's nominal runtime descriptors in place.
use super::*;
use crate::modules::aggregates::Nominals;
use jai_modules::{Binding as ModuleBinding, ModuleGraph};
use jai_source::{LocatedDiagnostic, ModuleId};

impl crate::reflection::MetaContext {
    pub(crate) fn install_preload_schema(
        &mut self,
        graph: &ModuleGraph,
        nominals: &Nominals<'_>,
        types: &mut TypeRegistry,
        layout: Option<jai_types::LayoutPolicy>,
    ) -> Result<(), LocatedDiagnostic> {
        if let Some(schema) =
            TypeInfoSchema::from_preload(graph, nominals, &self.record_specializations, types)?
        {
            nominals
                .install_reflection_header(schema.header, types)
                .map_err(|error| {
                    let file = graph
                        .module(
                            graph
                                .prelude()
                                .expect("source schema has designated Preload"),
                        )
                        .unwrap()
                        .entry();
                    LocatedDiagnostic {
                        location: graph.locate(file, Span::new(0, 0)).unwrap(),
                        message: format!("source Preload Any descriptor header: {error}"),
                    }
                })?;
            if let Some(layout) = layout {
                schema.validate_preload_any_storage(
                    graph,
                    nominals,
                    &self.record_specializations,
                    types,
                    layout,
                )?;
            }
            self.schema = Some(std::sync::Arc::new(schema));
        }
        Ok(())
    }
}

impl TypeInfoSchema {
    fn validate_preload_any_storage(
        &self,
        graph: &ModuleGraph,
        nominals: &Nominals<'_>,
        records: &crate::modules::aggregates::parameterized::RecordSpecializations,
        types: &mut TypeRegistry,
        layout: jai_types::LayoutPolicy,
    ) -> Result<jai_types::AnyStorageBridge, LocatedDiagnostic> {
        let source = SourceTypes {
            graph,
            nominals,
            records,
            module: graph
                .prelude()
                .expect("source schema has designated Preload"),
        };
        let mirror = source.export("Any_Struct")?;
        let any = nominals
            .reserve_any_for_graph(graph, types)
            .map_err(|error| source.error(error.to_string()))?;
        let universal = jai_types::AnySchema::validate(types, any, self.header)
            .map_err(|error| source.error(error.to_string()))?;
        source.validate_record(
            types,
            mirror,
            "Any_Struct",
            &[
                (
                    "type",
                    types
                        .field(any, 0)
                        .map_err(|error| source.error(error.to_string()))?
                        .ty,
                    false,
                ),
                (
                    "value_pointer",
                    types
                        .field(any, 1)
                        .map_err(|error| source.error(error.to_string()))?
                        .ty,
                    false,
                ),
            ],
        )?;
        universal
            .storage_bridge(types, mirror, layout)
            .map_err(|error| source.error(format!("Any_Struct storage bridge: {error}")))
    }

    pub(crate) fn from_preload(
        graph: &ModuleGraph,
        nominals: &Nominals<'_>,
        records: &crate::modules::aggregates::parameterized::RecordSpecializations,
        types: &mut TypeRegistry,
    ) -> Result<Option<Self>, LocatedDiagnostic> {
        let Some(module) = graph.prelude() else {
            return Ok(None);
        };
        let source = SourceTypes {
            graph,
            nominals,
            records,
            module,
        };
        let header = source.export("Type_Info")?;
        let integer = source.export("Type_Info_Integer")?;
        let float = source.export("Type_Info_Float")?;
        let string = source.export("Type_Info_String")?;
        let pointer = source.export("Type_Info_Pointer")?;
        let procedure = source.export("Type_Info_Procedure")?;
        let record = source.export("Type_Info_Struct")?;
        let member = source.export("Type_Info_Struct_Member")?;
        let array = source.export("Type_Info_Array")?;
        let enumeration = source.export("Type_Info_Enum")?;
        let variant = source.export("Type_Info_Variant")?;
        let tag = source.export("Type_Info_Tag")?;
        let array_kind = source.field_type(array, "array_type")?;
        let procedure_flags = source.field_type(procedure, "procedure_flags")?;
        let member_flags = source.field_type(member, "flags")?;
        let record_status = source.export("Struct_Status_Flags")?;
        let record_nontextual = source.export("Struct_Nontextual_Flags")?;
        let record_textual = source.export("Struct_Textual_Flags")?;
        let enum_status = source.export("Enum_Status_Flags")?;
        let enum_flags = source.export("Enum_Type_Flags")?;
        let variant_flags = source.export("Type_Info_Variant_Flags")?;
        for (ty, representation, flags, values) in [
            (
                tag,
                IntegerType::U32,
                false,
                &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18][..],
            ),
            (array_kind, IntegerType::U16, false, &[0, 1, 2][..]),
            (
                procedure_flags,
                IntegerType::U32,
                true,
                &[1, 2, 4, 8, 32, 128, 256, 0x1000_0000, 0x2000_0000][..],
            ),
            (member_flags, IntegerType::U32, true, &[1, 2, 4, 8, 16][..]),
            (record_status, IntegerType::U32, true, &[1, 4][..]),
            (record_nontextual, IntegerType::U32, true, &[4, 64, 256][..]),
            (
                record_textual,
                IntegerType::U32,
                false,
                &[1, 2, 4, 8, 16, 32][..],
            ),
            (enum_status, IntegerType::U16, true, &[1][..]),
            (enum_flags, IntegerType::U16, true, &[1, 2, 4][..]),
            (variant_flags, IntegerType::U32, true, &[1, 2][..]),
        ] {
            source.validate_enum(types, ty, representation, flags, values)?;
        }
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let bool_ty = types.scalar(ScalarType::Bool);
        let string_ty = types.string();
        let header_pointer = types
            .pointer(header)
            .map_err(|error| source.error(error.to_string()))?;
        let record_pointer = types
            .pointer(record)
            .map_err(|error| source.error(error.to_string()))?;
        let integer_pointer = types
            .pointer(integer)
            .map_err(|error| source.error(error.to_string()))?;
        let descriptor_view = types
            .slice(header_pointer)
            .map_err(|error| source.error(error.to_string()))?;
        let member_view = types
            .slice(member)
            .map_err(|error| source.error(error.to_string()))?;
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let bytes = types
            .slice(byte)
            .map_err(|error| source.error(error.to_string()))?;
        let strings = types
            .slice(string_ty)
            .map_err(|error| source.error(error.to_string()))?;
        let integers = types
            .slice(int)
            .map_err(|error| source.error(error.to_string()))?;
        let void_pointer = types
            .pointer(types.void())
            .map_err(|error| source.error(error.to_string()))?;
        let initializer = types
            .procedure(ProcedureType {
                parameters: Box::new([void_pointer]),
                results: Box::new([]),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .map_err(|error| source.error(error.to_string()))?;
        let mut schema = Self {
            header,
            integer,
            float,
            string,
            pointer,
            procedure,
            record,
            member,
            array,
            enumeration,
            variant,
            tag,
            array_kind,
            procedure_flags,
            member_flags,
            record_status,
            record_nontextual,
            record_textual,
            enum_status,
            enum_flags,
            variant_flags,
            header_pointer,
            fields: HashMap::new(),
            names: HashMap::new(),
        };
        source.adopt_record(
            types,
            &mut schema,
            header,
            "Type_Info",
            &[("type", tag, false), ("runtime_size", int, false)],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            integer,
            "Type_Info_Integer",
            &[("info", header, true), ("signed", bool_ty, false)],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            float,
            "Type_Info_Float",
            &[("info", header, true)],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            string,
            "Type_Info_String",
            &[("info", header, true)],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            pointer,
            "Type_Info_Pointer",
            &[
                ("info", header, true),
                ("pointer_to", header_pointer, false),
            ],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            procedure,
            "Type_Info_Procedure",
            &[
                ("info", header, true),
                ("argument_types", descriptor_view, false),
                ("return_types", descriptor_view, false),
                ("procedure_flags", procedure_flags, false),
            ],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            member,
            "Type_Info_Struct_Member",
            &[
                ("name", string_ty, false),
                ("type", header_pointer, false),
                ("offset_in_bytes", int, false),
                ("flags", member_flags, false),
                ("notes", strings, false),
                ("offset_into_constant_storage", int, false),
            ],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            record,
            "Type_Info_Struct",
            &[
                ("info", header, true),
                ("name", string_ty, false),
                ("specified_parameters", member_view, false),
                ("members", member_view, false),
                ("status_flags", record_status, false),
                ("nontextual_flags", record_nontextual, false),
                ("textual_flags", record_textual, false),
                ("polymorph_source_struct", record_pointer, false),
                ("initializer", initializer, false),
                ("constant_storage", bytes, false),
                ("notes", strings, false),
            ],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            array,
            "Type_Info_Array",
            &[
                ("info", header, true),
                ("element_type", header_pointer, false),
                ("array_type", array_kind, false),
                ("array_count", int, false),
            ],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            enumeration,
            "Type_Info_Enum",
            &[
                ("info", header, true),
                ("name", string_ty, false),
                ("internal_type", integer_pointer, false),
                ("names", strings, false),
                ("values", integers, false),
                ("status_flags", enum_status, false),
                ("enum_type_flags", enum_flags, false),
            ],
        )?;
        source.adopt_record(
            types,
            &mut schema,
            variant,
            "Type_Info_Variant",
            &[
                ("info", header, true),
                ("name", string_ty, false),
                ("variant_of", header_pointer, false),
                ("variant_flags", variant_flags, false),
            ],
        )?;
        for (name, ty) in [
            ("Type_Info_Tag", tag),
            ("Type_Info_Array.Array_Type", array_kind),
            ("Type_Info_Procedure.Flags", procedure_flags),
            ("Type_Info_Struct_Member.Flags", member_flags),
            ("Struct_Status_Flags", record_status),
            ("Struct_Nontextual_Flags", record_nontextual),
            ("Struct_Textual_Flags", record_textual),
            ("Enum_Status_Flags", enum_status),
            ("Enum_Type_Flags", enum_flags),
            ("Type_Info_Variant_Flags", variant_flags),
        ] {
            schema.names.insert(name.into(), ty);
        }
        Ok(Some(schema))
    }
}

struct SourceTypes<'a, 's> {
    graph: &'a ModuleGraph,
    nominals: &'a Nominals<'s>,
    records: &'a crate::modules::aggregates::parameterized::RecordSpecializations,
    module: ModuleId,
}
impl SourceTypes<'_, '_> {
    fn error(&self, message: impl Into<String>) -> LocatedDiagnostic {
        let file = self.graph.module(self.module).unwrap().entry();
        LocatedDiagnostic {
            location: self.graph.locate(file, Span::new(0, 0)).unwrap(),
            message: format!("invalid source Preload schema: {}", message.into()),
        }
    }
    fn export(&self, name: &str) -> Result<TypeId, LocatedDiagnostic> {
        let binding = self.graph.symbols().find(name).and_then(|symbol| {
            self.graph
                .module(self.module)
                .unwrap()
                .exports()
                .get(&symbol)
        });
        let Some(ModuleBinding::Declaration(declaration)) = binding else {
            return Err(self.error(format!(
                "required type `{name}` is not exported by the designated Preload module"
            )));
        };
        self.nominals
            .declarations
            .get(declaration)
            .copied()
            .ok_or_else(|| {
                self.error(format!(
                    "Preload `{name}` does not denote a source nominal type"
                ))
            })
    }
    fn field_type(&self, owner: TypeId, name: &str) -> Result<TypeId, LocatedDiagnostic> {
        self.nominals
            .records
            .get(&owner)
            .and_then(|record| {
                record
                    .fields
                    .iter()
                    .find(|field| self.graph.symbols().name(field.name) == name)
            })
            .map(|field| field.ty)
            .ok_or_else(|| self.error(format!("source descriptor is missing field `{name}`")))
    }
    fn validate_enum(
        &self,
        types: &TypeRegistry,
        ty: TypeId,
        representation: IntegerType,
        flags: bool,
        values: &[i128],
    ) -> Result<(), LocatedDiagnostic> {
        let definition = types
            .enum_definition(ty)
            .map_err(|error| self.error(error.to_string()))?;
        let source_flags = self
            .nominals
            .enums
            .get(&ty)
            .map(|enumeration| enumeration.flags)
            .or_else(|| {
                self.records
                    .member_enum(ty)
                    .map(|enumeration| enumeration.flags)
            })
            .ok_or_else(|| self.error("schema enumeration lacks source metadata"))?;
        if definition.representation != representation
            || source_flags != flags
            || definition
                .values
                .iter()
                .map(|value| value.value())
                .ne(values.iter().copied())
        {
            return Err(self.error("source enumeration representation, flags or ordered values do not match the runtime descriptor contract"));
        }
        Ok(())
    }
    fn adopt_record(
        &self,
        types: &TypeRegistry,
        schema: &mut TypeInfoSchema,
        ty: TypeId,
        name: &str,
        expected: &[(&str, TypeId, bool)],
    ) -> Result<(), LocatedDiagnostic> {
        let fields = self.validate_record(types, ty, name, expected)?;
        schema.names.insert(name.into(), ty);
        schema.fields.insert(ty, fields);
        Ok(())
    }

    fn validate_record(
        &self,
        types: &TypeRegistry,
        ty: TypeId,
        name: &str,
        expected: &[(&str, TypeId, bool)],
    ) -> Result<Vec<(String, jai_types::FieldId, bool)>, LocatedDiagnostic> {
        let definition = types
            .record_definition(ty)
            .map_err(|error| self.error(error.to_string()))?;
        let metadata = self
            .nominals
            .records
            .get(&ty)
            .ok_or_else(|| self.error(format!("`{name}` lacks source field metadata")))?;
        if definition.kind != RecordKind::Struct || metadata.fields.len() != expected.len() {
            return Err(self.error(format!("`{name}` has the wrong record kind or field count")));
        }
        let mut fields = Vec::with_capacity(expected.len());
        for (actual, &(field_name, field_type, using)) in metadata.fields.iter().zip(expected) {
            if self.graph.symbols().name(actual.name) != field_name
                || actual.ty != field_type
                || actual.syntax.using != using
                || actual.syntax.conversion
                    != if using {
                        syntax::FieldConversion::Implicit
                    } else {
                        syntax::FieldConversion::None
                    }
                || types
                    .validate_field(ty, actual.id)
                    .map_err(|error| self.error(error.to_string()))?
                    != field_type
            {
                return Err(self.error(format!("`{name}.{field_name}` has the wrong source name, type, embedding or field identity")));
            }
            fields.push((field_name.into(), actual.id, using));
        }
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, PreludeSource, SourceOverlay};
    use std::path::Path;

    fn graph(preload: Option<&str>, application: &str) -> ModuleGraph {
        let mut provider = SourceOverlay::new();
        provider
            .insert(
                Path::new("/jai-schema/main.jai"),
                application.as_bytes().to_vec(),
            )
            .unwrap();
        if let Some(preload) = preload {
            provider
                .insert(
                    Path::new("/jai-schema/modules/module.jai"),
                    preload.as_bytes().to_vec(),
                )
                .unwrap();
        }
        ModuleGraph::load_with_bootstrap(
            Path::new("/jai-schema/main.jai"),
            GraphOptions {
                import_dirs: vec!["/jai-schema/modules".into()],
            },
            if preload.is_some() {
                PreludeSource::Search
            } else {
                PreludeSource::Disabled
            },
            &provider,
            None,
        )
        .unwrap()
    }

    #[test]
    fn source_schema_lookup_uses_designated_preload_identity_despite_local_shadow() {
        let graph = graph(
            Some("Type_Info :: struct { runtime_size: int; }"),
            "Type_Info :: struct { runtime_size: int; } main :: () {}",
        );
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let module = graph.prelude().unwrap();
        let records = crate::modules::aggregates::parameterized::RecordSpecializations::default();
        let source = SourceTypes {
            graph: &graph,
            nominals: &nominals,
            records: &records,
            module,
        };
        let canonical = source.export("Type_Info").unwrap();
        let name = graph.symbols().find("Type_Info").unwrap();
        let ModuleBinding::Declaration(local) =
            graph.module(graph.root()).unwrap().exports()[&name]
        else {
            panic!("local type expected");
        };
        assert_ne!(canonical, nominals.declarations[&local]);
        let ModuleBinding::Declaration(preload) = graph.module(module).unwrap().exports()[&name]
        else {
            panic!("preload type expected");
        };
        assert_eq!(canonical, nominals.declarations[&preload]);
    }

    #[test]
    fn required_source_schema_never_falls_back_to_application_declarations() {
        let graph = graph(
            Some("unrelated :: 42;"),
            "Type_Info :: struct { runtime_size: int; } main :: () {}",
        );
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let error =
            TypeInfoSchema::from_preload(&graph, &nominals, &Default::default(), &mut types)
                .err()
                .expect("missing canonical schema must fail");
        assert!(
            error
                .message
                .contains("required type `Type_Info` is not exported"),
            "{}",
            error.message
        );
        assert_eq!(
            graph.sources().get(error.location.source).unwrap().path(),
            Path::new("/jai-schema/modules/module.jai")
        );
    }

    #[test]
    fn explicit_no_preload_does_not_adopt_an_application_lookalike() {
        let graph = graph(
            None,
            "Type_Info :: struct { runtime_size: int; } main :: () {}",
        );
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        assert!(
            TypeInfoSchema::from_preload(&graph, &nominals, &Default::default(), &mut types)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn compiler_prelude_type_declarations_adopt_the_existing_source_nominals() {
        // Isolate the declarative protocol from compiler procedure bodies. The
        // complete authored prelude has a separate semantic integration test.
        let source = jai_modules::compiler_prelude_source();
        let prefix = &source[..source.find("get_current_workspace ::").unwrap()];
        let graph = graph(Some(prefix), "main :: () {}");
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records =
            crate::modules::aggregates::parameterized::RecordSpecializations::default();
        let mut evaluate = |file, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |_, span| {
                Err(Diagnostic::new(
                    span,
                    "unexpected named scalar dependency in the exact Preload type fixture",
                ))
            })
            .map_err(|error| LocatedDiagnostic {
                location: graph.locate(file, error.span).unwrap(),
                message: error.message,
            })
        };
        crate::modules::aggregates::parameterized::reserve_static_members(
            &graph,
            &nominals,
            &mut types,
            &mut records,
            &mut evaluate,
        )
        .unwrap();
        nominals
            .define_aliases_with_specializations(&graph, &mut types, &mut records, &mut evaluate)
            .unwrap();
        nominals
            .define_enums(&graph, &mut types, &mut |_, _, span| {
                Err(Diagnostic::new(
                    span,
                    "unexpected named enum dependency in Preload fixture",
                ))
            })
            .unwrap();
        nominals
            .define_records_with_specializations(&graph, &mut types, &mut records, &mut evaluate)
            .unwrap();
        let schema = TypeInfoSchema::from_preload(&graph, &nominals, &records, &mut types)
            .unwrap()
            .unwrap();
        let module = graph.module(graph.prelude().unwrap()).unwrap();
        let ModuleBinding::Declaration(header) =
            module.exports()[&graph.symbols().find("Type_Info").unwrap()]
        else {
            panic!("source type expected");
        };
        assert_eq!(schema.header, nominals.declarations[&header]);
        assert_eq!(schema.header_pointer, types.pointer(schema.header).unwrap());
        assert_eq!(
            schema.fields[&schema.pointer][1].1,
            types.field(schema.pointer, 1).unwrap().id
        );
        assert_eq!(
            records
                .member_bindings(schema.procedure)
                .unwrap()
                .ty(graph.symbols().find("Flags").unwrap()),
            Some(schema.procedure_flags)
        );
        assert_eq!(
            records
                .member_bindings(schema.array)
                .unwrap()
                .ty(graph.symbols().find("Array_Type").unwrap()),
            Some(schema.array_kind)
        );
        nominals
            .install_reflection_header(schema.header, &mut types)
            .unwrap();
        let bridge = schema
            .validate_preload_any_storage(
                &graph,
                &nominals,
                &records,
                &mut types,
                jai_types::LayoutPolicy::lp64(),
            )
            .unwrap();
        assert_ne!(bridge.universal_type(), bridge.record_type());
        let ModuleBinding::Declaration(mirror) =
            module.exports()[&graph.symbols().find("Any_Struct").unwrap()]
        else {
            panic!("source Any_Struct expected");
        };
        assert_eq!(bridge.record_type(), nominals.declarations[&mirror]);
        for (index, field) in [jai_types::AnyField::Type, jai_types::AnyField::ValuePointer]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                bridge.field(field),
                types.field(bridge.record_type(), index).unwrap()
            );
        }
        types.freeze().unwrap();
    }
}
