//! Bind an operation to the actual declaration and its aggregate layouts.
use super::{Binding, Field, Kind, Layout, Operation, Schema};
use crate::{
    ir,
    sema::{Compiler, ProcId, Result, Value, err, scope::Resolved},
    source::Span,
    types::{ArrayKind, TypeId, TypeKind},
};
use std::rc::Rc;

pub(crate) fn bind(
    compiler: &mut Compiler,
    procedure: ProcId,
    operation: Operation,
    span: Span,
) -> Result<Binding> {
    let signature = compiler.signature(procedure, span)?;
    let parameters: Vec<_> = signature
        .params
        .iter()
        .map(|parameter| parameter.ty)
        .collect();
    validate(compiler, operation, &parameters, &signature.returns, span)?;
    let returns: Vec<_> = signature
        .returns
        .iter()
        .map(|&ty| (ty, compiler.is_memory_type(ty)))
        .collect();
    let mut roots: Vec<_> = parameters
        .iter()
        .chain(signature.returns.iter())
        .copied()
        .collect();
    let mut modules = Vec::new();
    for &ty in &roots {
        let ty = compiler.ty.types.pointee(ty).unwrap_or(ty);
        if let Some(record) = compiler
            .ty
            .types
            .as_struct(compiler.ty.types.repr_struct(ty))
        {
            let scope = compiler.ty.struct_asts[&record].scope;
            let module = compiler.scope(scope).module;
            if !modules.contains(&module) {
                modules.push(module);
            }
        }
    }
    if operation.records() {
        // Primitive descriptors preserve identity across workspace record graphs even
        // when no public record happens to contain a field of that primitive type.
        roots.extend([
            TypeId::VOID,
            TypeId::BOOL,
            TypeId::S8,
            TypeId::S16,
            TypeId::S32,
            TypeId::S64,
            TypeId::U8,
            TypeId::U16,
            TypeId::U32,
            TypeId::U64,
            TypeId::F32,
            TypeId::F64,
            TypeId::STRING,
            TypeId::TYPE,
            TypeId::ANY,
            TypeId::CODE,
        ]);
        if let Some(prelude) = compiler.ns.preload {
            modules.push(prelude);
        }
        for module in modules.clone() {
            let scope = compiler.ns.modules[module.0 as usize].scope;
            let entities: Vec<_> = compiler
                .scope(scope)
                .names
                .values()
                .flatten()
                .copied()
                .collect();
            for entity in entities {
                if let Ok(Resolved::Const {
                    value: Value::Type(ty),
                    ..
                }) = compiler.resolve_entity(entity)
                {
                    roots.push(ty);
                }
            }
        }
    }
    if operation == Operation::SetOptionsDuringCompile {
        for module in modules {
            if let Ok(ty) = compiler.module_type(module, "Build_Options", span) {
                roots.push(ty);
                break;
            }
        }
    }
    let mut schema = Schema::default();
    for ty in roots {
        collect(compiler, &mut schema, ty, operation.records(), span)?;
    }
    if matches!(
        operation,
        Operation::GetOptions | Operation::SetOptionsDuringCompile
    ) && let Some((&ty, _)) = schema
        .types
        .iter()
        .find(|(_, layout)| layout.name == "Build_Options")
    {
        let image = compiler.default_initializer(ty, span)?;
        let layout = schema.types.get_mut(&ty).unwrap();
        let (init, relocs) = image.map_or_else(
            || (Vec::new(), Vec::new()),
            |image| (image.bytes.clone(), image.relocs.clone()),
        );
        let global = compiler.out.program.add_global(ir::Global {
            name: "compiler.options.defaults".into(),
            size: layout.size,
            align: layout.align,
            init,
            relocs,
            read_only: true,
            export: None,
        });
        layout.default = Some(global);
    }
    Ok(Binding {
        operation,
        parameters,
        returns,
        context: signature.has_context,
        schema: Rc::new(schema),
    })
}

fn collect(
    compiler: &mut Compiler,
    schema: &mut Schema,
    ty: TypeId,
    descriptors: bool,
    span: Span,
) -> Result<()> {
    if schema.types.contains_key(&ty) {
        return Ok(());
    }
    let repr = compiler.ty.types.repr_struct(ty);
    let kind = compiler.ty.types.kind(repr).clone();
    if matches!(
        kind,
        TypeKind::CompileTimeOnly | TypeKind::PolyParam | TypeKind::PolyStruct { .. }
    ) {
        return Ok(());
    }
    let (size, align) = if kind == TypeKind::Void {
        (0, 1)
    } else {
        (compiler.size_of(ty, span)?, compiler.align_of(ty, span)?)
    };
    let name = match &kind {
        TypeKind::Struct(record) => compiler.ty.types.struct_info(*record).name.to_string(),
        _ => compiler.ty.types.name(ty),
    };
    schema.types.insert(
        ty,
        Layout {
            size,
            align,
            name,
            kind: Kind::Scalar,
            descriptor: None,
            default: None,
        },
    );
    if let TypeKind::Enum(id) = kind {
        schema.enums.insert(
            ty,
            compiler
                .ty
                .types
                .enum_info(id)
                .members
                .iter()
                .map(|(name, value)| (name.to_string(), *value as i64))
                .collect(),
        );
    }
    let shape = match kind {
        TypeKind::String => Kind::String,
        TypeKind::Pointer(inner) => {
            collect(compiler, schema, inner, descriptors, span)?;
            Kind::Pointer(inner)
        }
        TypeKind::Array {
            elem,
            kind,
        } => {
            collect(compiler, schema, elem, descriptors, span)?;
            Kind::Array {
                element: elem,
                count: match kind {
                    ArrayKind::Fixed(n) => Some(n),
                    _ => None,
                },
                resizable: kind == ArrayKind::Resizable,
            }
        }
        TypeKind::Struct(record) => {
            let fields = compiler.ty.types.struct_info(record).fields.clone();
            let mut members = Vec::new();
            for field in fields.iter() {
                collect(compiler, schema, field.ty, descriptors, span)?;
                members.push(Field {
                    name: field.name.map_or_else(String::new, |name| name.to_string()),
                    offset: field.offset,
                    ty: field.ty,
                    promoted: field.using,
                });
            }
            Kind::Record(members)
        }
        _ => Kind::Scalar,
    };
    schema.types.get_mut(&ty).unwrap().kind = shape;
    if descriptors {
        schema.types.get_mut(&ty).unwrap().descriptor = Some(compiler.type_info_global(ty, span)?);
    }
    Ok(())
}

/// Validate the calling convention before a host handler can inspect its slots.
/// Aggregate fields are resolved separately; their offsets are never part of this signature.
fn validate(
    compiler: &Compiler,
    operation: Operation,
    params: &[TypeId],
    returns: &[TypeId],
    span: Span,
) -> Result<()> {
    #[derive(Clone, Copy)]
    enum Shape {
        Number,
        Boolean,
        Text,
        Code,
        Type,
        Pointer,
        Record,
        Bytes,
        Texts,
        Pointers,
    }
    use Operation::{
        AddDataSegment, AddFile, AddGlobalData, AddLibraryDirectory, AddString, AddStringToModule,
        BasePath, BeginIntercept, CommandLine, CreateWorkspace, CurrentWorkspace,
        CustomLinkComplete, DestroyWorkspace, DeveloperDebug, EndIntercept, GetCode, GetNodes,
        GetOptions, GetType, MakeLive, MemoryBreakpoint, ModifyProcedure, Name, ProvideImport,
        RemapImport, Report, RootType, RuntimeInfo, SetOptions, SetOptionsDuringCompile,
        SetTypeInfoFlags, SetWorkspaceStatus, StructLocation, TypeTable, Version, WaitForMessage,
    };
    use Shape::{Boolean, Bytes, Code, Number, Pointer, Pointers, Record, Text, Texts, Type};
    let (inputs, outputs): (&[Shape], &[Shape]) = match operation {
        CreateWorkspace => (&[Text], &[Number]),
        DestroyWorkspace | EndIntercept | CustomLinkComplete => (&[Number], &[]),
        CurrentWorkspace => (&[], &[Number]),
        Name => (&[Number], &[Text]),
        GetOptions => (&[Number], &[Record]),
        SetOptions => (&[Record, Number, Record], &[]),
        SetOptionsDuringCompile => (&[Record, Number], &[]),
        AddFile => (&[Text, Number, Record], &[]),
        AddString => (&[Text, Number, Code, Record], &[]),
        AddStringToModule => (&[Text, Number, Pointer, Record], &[]),
        Report if params.len() == 3 => (&[Text, Record, Number], &[]),
        Report => (&[Text, Number, Number, Text, Number], &[]),
        Version => (&[Pointer], &[Text]),
        CommandLine => (&[], &[Texts]),
        BasePath => (&[], &[Text]),
        AddLibraryDirectory => (&[Text], &[]),
        SetTypeInfoFlags => (&[Type, Number], &[]),
        StructLocation => (&[Number, Pointer], &[Record]),
        RemapImport => (&[Number, Text, Text, Text], &[]),
        ProvideImport => (&[Number, Pointer, Number, Text], &[]),
        GetNodes => (&[Code], &[Pointer, Pointers]),
        GetCode => (&[Pointer, Code], &[Code]),
        RootType => (&[Code], &[Number, Type]),
        GetType => (&[Pointer], &[Type]),
        TypeTable => (&[Number], &[Pointers]),
        MakeLive | ModifyProcedure => (&[Number, Pointer], &[]),
        BeginIntercept => (&[Number, Number], &[]),
        WaitForMessage => (&[], &[Pointer]),
        SetWorkspaceStatus => (&[Number, Number], &[]),
        AddDataSegment => (&[Text, Number, Number, Number], &[Pointer, Boolean]),
        AddGlobalData => (&[Bytes, Number, Pointer, Number], &[Bytes]),
        RuntimeInfo => (&[Number], &[Record]),
        MemoryBreakpoint | DeveloperDebug => (&[Pointer], &[]),
    };
    fn accepts(compiler: &Compiler, ty: TypeId, shape: Shape) -> bool {
        let kind = compiler.ty.types.kind(compiler.ty.types.repr_struct(ty));
        match (shape, kind) {
            (
                Number,
                TypeKind::Int {
                    ..
                }
                | TypeKind::Enum(_),
            )
            | (Boolean, TypeKind::Bool)
            | (Text, TypeKind::String)
            | (Code, TypeKind::Code)
            | (Type, TypeKind::Type)
            | (Pointer, TypeKind::Pointer(_))
            | (Record, TypeKind::Struct(_)) => true,
            (
                Bytes | Texts | Pointers,
                TypeKind::Array {
                    elem,
                    kind: ArrayKind::View,
                },
            ) => match shape {
                Bytes => matches!(
                    compiler.ty.types.kind(*elem),
                    TypeKind::Int {
                        bits: 8,
                        signed: false
                    }
                ),
                Texts => matches!(compiler.ty.types.kind(*elem), TypeKind::String),
                Pointers => matches!(compiler.ty.types.kind(*elem), TypeKind::Pointer(_)),
                _ => false,
            },
            _ => false,
        }
    }
    if params.len() != inputs.len()
        || returns.len() != outputs.len()
        || params
            .iter()
            .zip(inputs)
            .chain(returns.iter().zip(outputs))
            .any(|(&ty, &shape)| !accepts(compiler, ty, shape))
    {
        return err(
            span,
            format!(
                "incompatible signature for compiler operation {}",
                operation.name()
            ),
        );
    }
    Ok(())
}
