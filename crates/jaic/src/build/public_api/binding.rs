//! Bind an operation to the actual declaration and its aggregate layouts.
use super::{Binding, Field, Kind, Layout, Operation, Schema};
use crate::{
    intern::Sym,
    ir,
    sema::{Compiler, ModuleId, ProcId, Result, Value, err, scope::Resolved},
    source::Span,
    types::{ArrayKind, TypeId, TypeKind},
};
use std::rc::Rc;

/// A schema shared by every declaration that reads the same modules' records. `entities`
/// counts the declarations the modules held when it was built, so a module that grew since
/// gets a fresh one.
pub(crate) struct Shared {
    modules: Option<Vec<ModuleId>>,
    entities: usize,
    schema: Rc<Schema>,
}

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
    let mut owners = Vec::new();
    for &ty in parameters.iter().chain(&signature.returns) {
        let mut ty = ty;
        while let TypeKind::Pointer(inner)
        | TypeKind::Array {
            elem: inner, ..
        } = compiler.ty.types.kind(ty)
        {
            ty = *inner;
        }
        if let Some(record) = compiler
            .ty
            .types
            .as_struct(compiler.ty.types.repr_struct(ty))
        {
            let scope = compiler.ty.struct_asts[&record].scope;
            let module = compiler.scope(scope).module;
            if !owners.contains(&module) {
                owners.push(module);
            }
        }
    }
    let options = match operation {
        Operation::GetOptions => Some(signature.returns[0]),
        Operation::SetOptionsDuringCompile => {
            Some(owned(compiler, &owners, "Build_Options", span)?)
        }
        _ => None,
    };
    let modules = operation.records().then(|| {
        owners.sort_unstable();
        owners.extend(
            compiler
                .ns
                .preload
                .filter(|prelude| !owners.contains(prelude)),
        );
        owners
    });
    let mut schema = shared(compiler, modules.as_deref(), span)?;
    let missing: Vec<_> = parameters
        .iter()
        .chain(&signature.returns)
        .chain(&options)
        .copied()
        .filter(|ty| !schema.types.contains_key(ty))
        .collect();
    if !missing.is_empty() {
        let extended = Rc::make_mut(&mut schema);
        for ty in missing {
            collect(compiler, extended, ty, modules.is_some(), span)?;
        }
        let entry = compiler
            .ct
            .interp
            .public_api_state
            .schemas
            .iter_mut()
            .find(|entry| entry.modules == modules)
            .unwrap();
        entry.schema = schema.clone();
    }
    let options = options
        .map(|ty| defaults(compiler, schema.get(ty), ty, span).map(|global| (ty, global)))
        .transpose()?;
    Ok(Binding {
        operation,
        parameters,
        returns,
        context: signature.has_context,
        schema,
        options,
    })
}

/// The schema for declarations reading `modules`' records, or for those that read none.
fn shared(compiler: &mut Compiler, modules: Option<&[ModuleId]>, span: Span) -> Result<Rc<Schema>> {
    let entities = modules.map_or(0, |modules| {
        modules
            .iter()
            .map(|module| {
                let scope = compiler.ns.modules[module.0 as usize].scope;
                compiler
                    .scope(scope)
                    .names
                    .values()
                    .map(Vec::len)
                    .sum::<usize>()
            })
            .sum()
    });
    let schemas = &compiler.ct.interp.public_api_state.schemas;
    if let Some(entry) = schemas
        .iter()
        .find(|entry| entry.modules.as_deref() == modules && entry.entities == entities)
    {
        return Ok(entry.schema.clone());
    }
    let mut schema = Schema::default();
    if let Some(modules) = modules {
        // Primitive descriptors preserve identity across workspace record graphs even
        // when no public record happens to contain a field of that primitive type.
        let mut roots = vec![
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
        ];
        for &module in modules {
            let scope = compiler.ns.modules[module.0 as usize].scope;
            let names: Vec<_> = compiler
                .scope(scope)
                .names
                .iter()
                .map(|(name, entities)| (*name, entities.clone()))
                .collect();
            for (name, entities) in names {
                for entity in entities {
                    let Ok(Resolved::Const {
                        value: Value::Type(ty),
                        ..
                    }) = compiler.resolve_entity(entity)
                    else {
                        continue;
                    };
                    match schema.names.insert(name.to_string(), ty) {
                        Some(other) if other != ty => return ambiguous(span, name.as_str()),
                        _ => roots.push(ty),
                    }
                }
            }
        }
        for ty in roots {
            collect(compiler, &mut schema, ty, true, span)?;
        }
    }
    let schema = Rc::new(schema);
    let schemas = &mut compiler.ct.interp.public_api_state.schemas;
    schemas.retain(|entry| entry.modules.as_deref() != modules);
    schemas.push(Shared {
        modules: modules.map(<[_]>::to_vec),
        entities,
        schema: schema.clone(),
    });
    Ok(schema)
}

/// The type `name` declares in the modules that own the declaration's records.
fn owned(compiler: &mut Compiler, owners: &[ModuleId], name: &str, span: Span) -> Result<TypeId> {
    let mut found = None;
    for &module in owners {
        for entity in compiler.module_declarations(module, Sym::intern(name))? {
            if let Resolved::Const {
                value: Value::Type(ty),
                ..
            } = compiler.resolve_entity(entity)?
            {
                if found.is_some_and(|other| other != ty) {
                    return ambiguous(span, name);
                }
                found = Some(ty);
            }
        }
    }
    found.map_or_else(
        || err(span, format!("compiler operation needs a `{name}` record")),
        Ok,
    )
}

fn ambiguous<T>(span: Span, name: &str) -> Result<T> {
    err(
        span,
        format!("compiler operation finds more than one `{name}` record"),
    )
}

/// A read-only copy of `ty`'s default value, where options start before a workspace's are read.
fn defaults(
    compiler: &mut Compiler,
    layout: &Layout,
    ty: TypeId,
    span: Span,
) -> Result<ir::GlobalId> {
    let (size, align) = (layout.size, layout.align);
    let image = compiler.default_initializer(ty, span)?;
    let (init, relocs) = image.map_or_else(
        || (Vec::new(), Vec::new()),
        |image| (image.bytes.clone(), image.relocs.clone()),
    );
    Ok(compiler.out.program.add_global(ir::Global {
        name: "compiler.options.defaults".into(),
        size,
        align,
        init,
        relocs,
        read_only: true,
        export: None,
    }))
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
