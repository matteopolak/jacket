//! Materialize reflection into actual typed, immutable descriptor objects.
use super::*;
use jai_types::{DescriptorId, DescriptorKind, ReflectionGraph, TypeDescriptor, TypeKind};
use std::sync::Arc;

type MaterializedDescriptors = (Arc<StaticData>, Vec<(TypeId, TypeId, StaticAddress)>);

impl Resolver<'_> {
    fn ensure_reflection_schema(&mut self, span: Span) -> Result<(), Diagnostic> {
        if self.meta.schema.is_none() {
            let schema = match self.graph_scope {
                Some(scope) => scope
                    .generated_reflection_schema(self.types, span)?
                    .ok_or_else(|| {
                        Diagnostic::new(span, "source Preload reflection schema is not ready")
                    })?,
                None => Arc::new(
                    schema::TypeInfoSchema::new(self.types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                ),
            };
            self.meta.schema = Some(schema);
        }
        self.meta
            .schema
            .as_ref()
            .expect("schema ready")
            .complete_reserved_any(self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
    pub(crate) fn schema_header_type(&mut self, span: Span) -> Result<TypeId, Diagnostic> {
        self.reflection_pointer_types(span)?;
        Ok(self
            .meta
            .schema
            .as_ref()
            .expect("schema initialized")
            .header)
    }
    pub(crate) fn reflection_pointer_types(
        &mut self,
        span: Span,
    ) -> Result<(TypeId, TypeId), Diagnostic> {
        self.ensure_reflection_schema(span)?;
        let header = self
            .meta
            .schema
            .as_ref()
            .expect("schema initialized")
            .header_pointer;
        let void = self
            .types
            .pointer(self.types.void())
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok((header, void))
    }

    pub(crate) fn type_info_header_expression(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Pointer {
            value:
                ValueExpr::StaticAddress {
                    data,
                    mut address,
                    ty: pointer,
                },
            ..
        } = self.type_info_expression(ty, span)?
        else {
            unreachable!("reflection produces a static pointer");
        };
        let schema = self
            .meta
            .schema
            .as_ref()
            .expect("materialization creates schema");
        let TypeKind::Pointer(pointee) = *self
            .types
            .kind(pointer)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        else {
            unreachable!();
        };
        if pointee != schema.header {
            address = address.project(StaticProjection::Field(
                self.types
                    .field(pointee, 0)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .id,
            ));
        }
        Ok(Expr::Pointer {
            ty: schema.header_pointer,
            value: ValueExpr::StaticAddress {
                data,
                address,
                ty: schema.header_pointer,
            },
        })
    }
    /// Determine a query's concrete pointer type without emitting descriptor storage.
    pub(crate) fn type_info_pointer_type(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        self.ensure_reflection_schema(span)?;
        let tag = match self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Void => jai_types::TypeInfoTag::Void,
            TypeKind::Type => jai_types::TypeInfoTag::Type,
            TypeKind::Code => jai_types::TypeInfoTag::Code,
            TypeKind::Any(_) => jai_types::TypeInfoTag::Any,
            TypeKind::Bool => jai_types::TypeInfoTag::Bool,
            TypeKind::Integer(_) => jai_types::TypeInfoTag::Integer,
            TypeKind::Float(_) => jai_types::TypeInfoTag::Float,
            TypeKind::String => jai_types::TypeInfoTag::String,
            TypeKind::Pointer(_) => jai_types::TypeInfoTag::Pointer,
            TypeKind::FixedArray { .. } | TypeKind::Slice(_) | TypeKind::DynamicArray(_) => {
                jai_types::TypeInfoTag::Array
            }
            TypeKind::Procedure(_) => jai_types::TypeInfoTag::Procedure,
            TypeKind::Record(_) => jai_types::TypeInfoTag::Struct,
            TypeKind::Enum(_) => jai_types::TypeInfoTag::Enum,
            TypeKind::Distinct(_) => jai_types::TypeInfoTag::Variant,
        };
        let descriptor = self
            .meta
            .schema
            .as_ref()
            .expect("schema initialized")
            .descriptor_type(tag);
        self.types
            .pointer(descriptor)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }

    pub(crate) fn type_info_expression(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        // The revision controller is activated with the shared reflection
        // provider fields. Until that atomic registration, reject stale rows.
        for (&record, &published) in &self.meta.storage_policies {
            if self
                .types
                .record_reflection_policy(record)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                != published
            {
                return Err(Diagnostic::new(
                    span,
                    "type_info is waiting for descriptor revision after record reflection flags changed",
                ));
            }
        }
        if let Some((pointer, data, address)) = self.meta.storage.get(&ty) {
            return Ok(Expr::Pointer {
                ty: *pointer,
                value: ValueExpr::StaticAddress {
                    data: Arc::clone(data),
                    address: address.clone(),
                    ty: *pointer,
                },
            });
        }
        self.ensure_reflection_schema(span)?;
        let mut metadata = match self.graph_scope {
            Some(scope) => scope.reflection_metadata(self.types),
            None => jai_types::ReflectionMetadata::default(),
        };
        self.meta
            .record_specializations
            .append_reflection_metadata(&mut metadata, self.symbols);
        self.meta
            .local_declarations
            .append_reflection_metadata(&mut metadata, self.symbols);
        if let Some(context) = self.context {
            context.append_reflection_metadata(&mut metadata, self.symbols);
        }
        let graph = match ReflectionGraph::build(self.types, ty, self.target_layout, &metadata)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            ReflectionReadiness::Ready(graph) => graph,
            ReflectionReadiness::Pending(dependencies) => {
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "type_info is waiting for target or nominal dependencies: {dependencies:?}"
                    ),
                ));
            }
        };
        let mut policies = Vec::new();
        for descriptor in graph.descriptors() {
            let represented = descriptor.id.represented_type();
            if matches!(descriptor.kind, DescriptorKind::Record { .. }) {
                let policy = self
                    .types
                    .record_reflection_policy(represented)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                policies.push((represented, policy));
            }
        }
        let schema = self.meta.schema.as_ref().expect("schema was created");
        let materialized = materialize(
            self.types,
            &graph,
            schema,
            &self.meta.storage,
            &mut self.meta.storage_builder,
        );
        let (data, additions) = match materialized {
            Ok(result) => result,
            Err(error) => {
                self.meta.storage_builder.discard_unpublished();
                return Err(Diagnostic::new(span, error.to_string()));
            }
        };
        for (represented, pointer, address) in additions {
            self.meta
                .storage
                .insert(represented, (pointer, Arc::clone(&data), address));
        }
        self.meta.storage_policies.extend(policies);
        let (pointer, _, address) = self.meta.storage.get(&ty).expect("root was published");
        let pointer = *pointer;
        let address = address.clone();
        Ok(Expr::Pointer {
            ty: pointer,
            value: ValueExpr::StaticAddress {
                data,
                address,
                ty: pointer,
            },
        })
    }
    pub(crate) fn reflection_field_path(
        &self,
        ty: TypeId,
        member: Symbol,
        span: Span,
    ) -> Result<Option<Vec<jai_types::FieldId>>, Diagnostic> {
        let Some(schema) = self.meta.schema.as_ref() else {
            return Ok(None);
        };
        let spelling = self.symbols.name(member);
        if ty == schema.record
            && matches!(
                spelling,
                "specified_parameters" | "polymorph_source_struct" | "initializer"
            )
        {
            return Err(Diagnostic::new(
                span,
                format!("reflection field '{spelling}' is not materialized by this compiler"),
            ));
        }
        Ok(schema.field_path(ty, spelling))
    }

    pub(crate) fn reflection_pointer_coercion(
        &self,
        value: ValueExpr,
        actual: TypeId,
        target: TypeId,
        span: Span,
    ) -> Result<Option<ValueExpr>, Diagnostic> {
        let Some(field) = self.reflection_pointer_conversion_field(actual, target, span)? else {
            return Ok(None);
        };
        let value = match value {
            ValueExpr::StaticAddress { data, address, .. } => ValueExpr::StaticAddress {
                data,
                address: address.project(StaticProjection::Field(field)),
                ty: target,
            },
            value => ValueExpr::PointerCast {
                value: Box::new(value),
                ty: target,
                mode: CastMode::Unchecked,
            },
        };
        Ok(Some(value))
    }

    pub(crate) fn reflection_pointer_convertible(
        &self,
        actual: TypeId,
        target: TypeId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        Ok(self
            .reflection_pointer_conversion_field(actual, target, span)?
            .is_some())
    }

    fn reflection_pointer_conversion_field(
        &self,
        actual: TypeId,
        target: TypeId,
        span: Span,
    ) -> Result<Option<jai_types::FieldId>, Diagnostic> {
        let Some(schema) = self.meta.schema.as_ref() else {
            return Ok(None);
        };
        if target != schema.header_pointer {
            return Ok(None);
        }
        let TypeKind::Pointer(pointee) = *self
            .types
            .kind(actual)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        else {
            return Ok(None);
        };
        let Some(fields) = schema.fields.get(&pointee) else {
            return Ok(None);
        };
        let Some((_, field, true)) = fields.first() else {
            return Ok(None);
        };
        if self
            .types
            .field_type(*field)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            != schema.header
        {
            return Ok(None);
        }
        Ok(Some(*field))
    }
    pub(crate) fn reflection_type_name(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        if !schema::TypeInfoSchema::recognizes_name(path, self.symbols) {
            return Ok(None);
        }
        if let Some(scope) = self.graph_scope {
            let ty = scope.generated_reflection_type(path, self.types, span)?;
            if ty.is_some() {
                self.ensure_reflection_schema(span)?;
            }
            return Ok(ty);
        }
        self.ensure_reflection_schema(span)?;
        Ok(self
            .meta
            .schema
            .as_ref()
            .and_then(|schema| schema.type_name(path, self.symbols)))
    }
}

fn materialize(
    types: &mut TypeRegistry,
    graph: &ReflectionGraph,
    schema: &schema::TypeInfoSchema,
    published: &HashMap<TypeId, (TypeId, Arc<StaticData>, StaticAddress)>,
    builder: &mut StaticDataBuilder,
) -> Result<MaterializedDescriptors, Box<dyn std::error::Error>> {
    let mut objects = HashMap::new();
    let mut additions = Vec::new();
    for descriptor in graph.descriptors() {
        let represented = descriptor.id.represented_type();
        let object = match published.get(&represented) {
            Some((_, _, address)) => address.object(),
            None => {
                let pointee = schema.descriptor_type(descriptor.tag());
                let object = builder.reserve(pointee, types)?;
                let pointer = types.pointer(pointee)?;
                additions.push((represented, pointer, StaticAddress::new(object)));
                object
            }
        };
        objects.insert(descriptor.id, object);
    }
    for descriptor in graph.descriptors() {
        if published.contains_key(&descriptor.id.represented_type()) {
            continue;
        }
        let value = descriptor_value(types, graph, schema, &objects, builder, descriptor)?;
        builder.define_type_descriptor(
            objects[&descriptor.id],
            value,
            graph,
            descriptor.id,
            types,
        )?;
    }
    let data = builder.publish(types, StaticDataLimits::default())?;
    Ok((Arc::new(data), additions))
}

fn descriptor_value(
    types: &mut TypeRegistry,
    graph: &ReflectionGraph,
    schema: &schema::TypeInfoSchema,
    objects: &HashMap<DescriptorId, StaticObjectId>,
    builder: &mut StaticDataBuilder,
    descriptor: &TypeDescriptor,
) -> Result<StaticValue, Box<dyn std::error::Error>> {
    let ty = schema.descriptor_type(descriptor.tag());
    let header = header(types, schema, descriptor)?;
    let fields = match &descriptor.kind {
        DescriptorKind::Void
        | DescriptorKind::Type
        | DescriptorKind::Code
        | DescriptorKind::Any
        | DescriptorKind::Bool => return Ok(header),
        DescriptorKind::Integer { representation } => vec![
            header,
            StaticValue::constant(jai_ir::ConstantValue {
                ty: types.scalar(ScalarType::Bool),
                kind: ConstantKind::Bool(representation.signed()),
            }),
        ],
        DescriptorKind::Float { .. } | DescriptorKind::String => vec![header],
        DescriptorKind::Pointer { pointee } => vec![
            header,
            reference(
                graph,
                schema,
                objects,
                *pointee,
                schema.header_pointer,
                true,
                types,
            )?,
        ],
        DescriptorKind::Procedure {
            parameters,
            results,
            convention,
            context,
            ..
        } => {
            let argument_view = types.field(ty, 1)?.ty;
            let return_view = types.field(ty, 2)?.ty;
            let arguments = parameters
                .iter()
                .map(|&id| {
                    reference(
                        graph,
                        schema,
                        objects,
                        id,
                        schema.header_pointer,
                        true,
                        types,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let results = results
                .iter()
                .map(|&id| {
                    reference(
                        graph,
                        schema,
                        objects,
                        id,
                        schema.header_pointer,
                        true,
                        types,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let flags = if *context == ContextMode::None { 8 } else { 0 }
                | if *convention == CallingConvention::C {
                    32
                } else {
                    0
                }
                | if *convention == CallingConvention::CppMethod {
                    0x1000_0000
                } else {
                    0
                };
            vec![
                header,
                view(types, builder, argument_view, arguments)?,
                view(types, builder, return_view, results)?,
                enum_value(types, schema.procedure_flags, flags)?,
            ]
        }
        DescriptorKind::Record {
            fields,
            constants,
            metadata,
            ..
        } => {
            let member_view = types.field(ty, 3)?.ty;
            let note_view = types.field(schema.member, 4)?.ty;
            let mut members = Vec::with_capacity(fields.len());
            for field in fields {
                let flags = if field.using { 4 } else { 0 }
                    | if field.procedure_as_void_pointer(types, descriptor.id.represented_type())? {
                        8
                    } else {
                        0
                    };
                members.push(record(
                    schema.member,
                    vec![
                        string(types, field.name.as_deref().unwrap_or_default()),
                        reference(
                            graph,
                            schema,
                            objects,
                            field.ty,
                            schema.header_pointer,
                            true,
                            types,
                        )?,
                        int(types, i128::from(field.offset_in_bytes))?,
                        enum_value(types, schema.member_flags, flags)?,
                        view(
                            types,
                            builder,
                            note_view,
                            field.notes.iter().map(|note| string(types, note)).collect(),
                        )?,
                        int(types, -1)?,
                    ],
                ));
            }
            let storage = types.field(ty, 9)?.ty;
            let (constant_storage, offsets) =
                type_constant_storage(types, graph, schema, objects, builder, constants, storage)?;
            let meta_descriptor = graph.descriptor(types.meta_type()).ok();
            for (constant, offset) in constants.iter().zip(offsets) {
                let meta_descriptor = meta_descriptor
                    .ok_or("record Type constants have no canonical Type descriptor")?;
                members.push(record(
                    schema.member,
                    vec![
                        string(types, &constant.name),
                        reference(
                            graph,
                            schema,
                            objects,
                            meta_descriptor.id,
                            schema.header_pointer,
                            true,
                            types,
                        )?,
                        int(types, 0)?,
                        enum_value(types, schema.member_flags, 1)?,
                        empty_view(note_view),
                        int(types, i128::from(offset))?,
                    ],
                ));
            }
            let record_pointer = types.field(ty, 7)?.ty;
            let initializer = types.field(ty, 8)?.ty;
            let notes = types.field(ty, 10)?.ty;
            vec![
                header,
                string(types, descriptor.name.as_deref().unwrap_or_default()),
                empty_view(types.field(ty, 2)?.ty),
                view(types, builder, member_view, members)?,
                enum_value(
                    types,
                    schema.record_status,
                    i128::from(metadata.status_flags),
                )?,
                enum_value(
                    types,
                    schema.record_nontextual,
                    i128::from(metadata.nontextual_flags),
                )?,
                enum_value(
                    types,
                    schema.record_textual,
                    i128::from(metadata.textual_flags),
                )?,
                zero(record_pointer),
                zero(initializer),
                constant_storage,
                view(
                    types,
                    builder,
                    notes,
                    metadata
                        .notes
                        .iter()
                        .map(|note| string(types, note))
                        .collect(),
                )?,
            ]
        }
        DescriptorKind::FixedArray { element, .. }
        | DescriptorKind::Slice { element }
        | DescriptorKind::DynamicArray { element } => {
            let (kind, count) = match descriptor.kind {
                DescriptorKind::FixedArray { count, .. } => (0, i128::from(count)),
                DescriptorKind::Slice { .. } => (1, -1),
                _ => (2, -1),
            };
            vec![
                header,
                reference(
                    graph,
                    schema,
                    objects,
                    *element,
                    schema.header_pointer,
                    true,
                    types,
                )?,
                enum_value(types, schema.array_kind, kind)?,
                int(types, count)?,
            ]
        }
        DescriptorKind::Enum {
            representation,
            members,
            flags,
        } => {
            let integer_ty = types.scalar(ScalarType::Int(*representation));
            let descriptor_id = graph.descriptor(integer_ty)?.id;
            let internal_pointer = types.field(ty, 2)?.ty;
            let names_view = types.field(ty, 3)?.ty;
            let values_view = types.field(ty, 4)?.ty;
            let names = members
                .iter()
                .map(|member| string(types, member.name.as_deref().unwrap_or_default()))
                .collect();
            // Preload represents enum reflection values as s64 bit patterns.
            let values = members
                .iter()
                .map(|member| {
                    StaticValue::constant(jai_ir::ConstantValue {
                        ty: types.scalar(ScalarType::Int(IntegerType::S64)),
                        kind: ConstantKind::Int(IntegerValue::wrapping(
                            IntegerType::S64,
                            member.value.value(),
                        )),
                    })
                })
                .collect();
            vec![
                header,
                string(types, descriptor.name.as_deref().unwrap_or_default()),
                reference(
                    graph,
                    schema,
                    objects,
                    descriptor_id,
                    internal_pointer,
                    false,
                    types,
                )?,
                view(types, builder, names_view, names)?,
                view(types, builder, values_view, values)?,
                enum_value(types, schema.enum_status, 0)?,
                enum_value(types, schema.enum_flags, if *flags { 1 } else { 0 })?,
            ]
        }
        DescriptorKind::Distinct {
            kind,
            representation,
        } => {
            let flags = match kind {
                jai_types::DistinctKind::Distinct => 1,
                jai_types::DistinctKind::IsA => 2,
            };
            vec![
                header,
                string(types, descriptor.name.as_deref().unwrap_or_default()),
                reference(
                    graph,
                    schema,
                    objects,
                    *representation,
                    schema.header_pointer,
                    true,
                    types,
                )?,
                enum_value(types, schema.variant_flags, flags)?,
            ]
        }
    };
    Ok(record(ty, fields))
}

fn type_constant_storage(
    types: &mut TypeRegistry,
    graph: &ReflectionGraph,
    schema: &schema::TypeInfoSchema,
    objects: &HashMap<DescriptorId, StaticObjectId>,
    builder: &mut StaticDataBuilder,
    constants: &[jai_types::ReflectedTypeConstant],
    bytes: TypeId,
) -> Result<(StaticValue, Vec<u64>), Box<dyn std::error::Error>> {
    if constants.is_empty() {
        return Ok((empty_view(bytes), vec![]));
    }
    let meta = types.meta_type();
    let count = u64::try_from(constants.len())?;
    let backing = types.fixed_array(meta, count)?;
    let layout = jai_types::LayoutEngine::new(types, graph.policy())
        .layout(backing)?
        .clone();
    let stride = layout
        .array_stride
        .ok_or("Type constant storage has no target stride")?;
    let object = builder.reserve(backing, types)?;
    let values = constants
        .iter()
        .map(|constant| {
            reference(
                graph,
                schema,
                objects,
                graph.descriptor(constant.represented_type)?.id,
                meta,
                true,
                types,
            )
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    builder.define(
        object,
        StaticValue {
            ty: backing,
            kind: StaticValueKind::Array(values),
        },
    )?;
    let view = builder.byte_view(object, graph.policy(), 0, layout.size, types)?;
    let offsets = (0..count)
        .map(|index| {
            index
                .checked_mul(stride)
                .ok_or("Type constant storage offset overflow")
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        StaticValue {
            ty: bytes,
            kind: StaticValueKind::Slice {
                data: Some(view.address()),
                count: layout.size,
            },
        },
        offsets,
    ))
}
fn header(
    types: &TypeRegistry,
    schema: &schema::TypeInfoSchema,
    descriptor: &TypeDescriptor,
) -> Result<StaticValue, Box<dyn std::error::Error>> {
    Ok(record(
        schema.header,
        vec![
            StaticValue::constant(jai_ir::ConstantValue {
                ty: schema.tag,
                kind: ConstantKind::Enum(IntegerValue::wrapping(
                    IntegerType::U32,
                    descriptor.tag() as i128,
                )),
            }),
            int(types, descriptor.runtime_size().map_or(-1, i128::from))?,
        ],
    ))
}
fn reference(
    graph: &ReflectionGraph,
    schema: &schema::TypeInfoSchema,
    objects: &HashMap<DescriptorId, StaticObjectId>,
    id: DescriptorId,
    pointer: TypeId,
    base: bool,
    types: &TypeRegistry,
) -> Result<StaticValue, Box<dyn std::error::Error>> {
    let mut address = StaticAddress::new(objects[&id]);
    let target = schema.descriptor_type(graph.get(id)?.tag());
    if base && target != schema.header {
        address = address.project(StaticProjection::Field(types.field(target, 0)?.id));
    }
    Ok(StaticValue {
        ty: pointer,
        kind: StaticValueKind::Address(address),
    })
}
fn view(
    types: &mut TypeRegistry,
    builder: &mut StaticDataBuilder,
    view: TypeId,
    elements: Vec<StaticValue>,
) -> Result<StaticValue, Box<dyn std::error::Error>> {
    if elements.is_empty() {
        return Ok(empty_view(view));
    }
    let TypeKind::Slice(element) = *types.kind(view)? else {
        return Err("reflection field is not a slice".into());
    };
    let count = u64::try_from(elements.len())?;
    let array = types.fixed_array(element, count)?;
    let object = builder.reserve(array, types)?;
    builder.define(
        object,
        StaticValue {
            ty: array,
            kind: StaticValueKind::Array(elements),
        },
    )?;
    Ok(StaticValue {
        ty: view,
        kind: StaticValueKind::Slice {
            data: Some(StaticAddress::new(object).project(StaticProjection::Index(0))),
            count,
        },
    })
}
fn record(ty: TypeId, fields: Vec<StaticValue>) -> StaticValue {
    StaticValue {
        ty,
        kind: StaticValueKind::Record(fields),
    }
}
fn empty_view(ty: TypeId) -> StaticValue {
    StaticValue {
        ty,
        kind: StaticValueKind::Slice {
            data: None,
            count: 0,
        },
    }
}
fn zero(ty: TypeId) -> StaticValue {
    StaticValue::constant(jai_ir::ConstantValue {
        ty,
        kind: ConstantKind::Zero,
    })
}
fn string(types: &TypeRegistry, bytes: &[u8]) -> StaticValue {
    StaticValue::constant(jai_ir::ConstantValue {
        ty: types.string(),
        kind: ConstantKind::StringBytes(bytes.to_vec()),
    })
}
fn int(types: &TypeRegistry, value: i128) -> Result<StaticValue, Box<dyn std::error::Error>> {
    Ok(StaticValue::constant(jai_ir::ConstantValue {
        ty: types.scalar(ScalarType::Int(IntegerType::S64)),
        kind: ConstantKind::Int(
            IntegerValue::checked(IntegerType::S64, value)
                .ok_or("reflection integer exceeds s64")?,
        ),
    }))
}
fn enum_value(
    types: &TypeRegistry,
    ty: TypeId,
    value: i128,
) -> Result<StaticValue, Box<dyn std::error::Error>> {
    Ok(StaticValue::constant(jai_ir::ConstantValue {
        ty,
        kind: ConstantKind::Enum(IntegerValue::wrapping(
            types.enum_definition(ty)?.representation,
            value,
        )),
    }))
}
