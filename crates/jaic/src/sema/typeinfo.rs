//! Runtime type information: one read-only `Type_Info_*` descriptor global
//! per type, laid out with Preload's reflection structs. A `Type` value at
//! runtime is the address of its descriptor.
use super::scope::{EntityKind, Resolved};
use super::value::Aggregate;
use super::*;
use crate::types::{ArrayKind, TypeKind};

/// A struct-scoped constant for type info: where it is declared, its name, type, value and notes.
type StructConstant = (Span, Sym, TypeId, Option<Value>, Vec<Rc<[u8]>>);

/// Tag values of Preload's `Type_Info_Tag`.
mod tag {
    pub const INTEGER: i128 = 0;
    pub const FLOAT: i128 = 1;
    pub const BOOL: i128 = 2;
    pub const STRING: i128 = 3;
    pub const POINTER: i128 = 4;
    pub const PROCEDURE: i128 = 5;
    pub const VOID: i128 = 6;
    pub const STRUCT: i128 = 7;
    pub const ARRAY: i128 = 8;
    pub const OVERLOAD_SET: i128 = 9;
    pub const ANY: i128 = 10;
    pub const ENUM: i128 = 11;
    pub const TYPE: i128 = 13;
    pub const POLYMORPHIC_VARIABLE: i128 = 12;
    pub const CODE: i128 = 14;
    pub const VARIANT: i128 = 18;
}

impl Compiler {
    /// The Preload struct describing `ty` (`Type_Info_Struct`, `Type_Info_Integer`...).
    pub fn type_info_struct_type(&mut self, ty: TypeId, span: Span) -> Result<TypeId> {
        let name = match self.types.kind(ty) {
            TypeKind::Int {
                ..
            } => "Type_Info_Integer",
            TypeKind::Float {
                ..
            }
            | TypeKind::WideFloat(_) => "Type_Info_Float",
            TypeKind::String => "Type_Info_String",
            TypeKind::Pointer(_) | TypeKind::Null => "Type_Info_Pointer",
            TypeKind::Proc(_) => "Type_Info_Procedure",
            TypeKind::Struct(_)
            | TypeKind::PolyStruct {
                ..
            } => "Type_Info_Struct",
            TypeKind::Array {
                ..
            } => "Type_Info_Array",
            TypeKind::Enum(_) => "Type_Info_Enum",
            TypeKind::Distinct(_) => "Type_Info_Variant",
            _ => "Type_Info",
        };
        self.preload_type(name, span)
    }

    pub fn type_info_global(&mut self, ty: TypeId, span: Span) -> Result<ir::GlobalId> {
        if let Some(&g) = self.type_infos.get(&ty) {
            return Ok(g);
        }
        let desc = self.type_info_struct_type(ty, span)?;
        let size = self.size_of(desc, span)?;
        let align = self.align_of(desc, span)?;
        // A descriptor whose build failed before keeps its global, so code lowered since
        // (a self-reference) points at the one that is filled in now.
        let g = match self.failed_type_infos.remove(&ty) {
            Some(g) => g,
            None => self.program.add_global(ir::Global {
                name: format!("type_info.{}", self.types.name(ty)),
                size,
                align,
                init: Vec::new(),
                relocs: Vec::new(),
                read_only: true,
                export: None,
            }),
        };
        // Register first: descriptors may refer to themselves.
        self.type_infos.insert(ty, g);
        if let Some(s) = self.types.as_struct(ty) {
            let span = self.types.struct_info(s).span;
            if (span.file.0 as usize) < self.sources.len() {
                let source = self.sources.get(span.file);
                let (line, col) = source.line_col(span.start);
                let path: Rc<str> = source.path.as_str().into();
                self.interp
                    .struct_locations
                    .insert(g, (path, i64::from(line), i64::from(col)));
            }
        }
        let agg = match self.build_type_info(ty, desc, size, span) {
            Ok(agg) => agg,
            Err(e) => {
                // Unregister it: a later request (a retried body) must build it again and
                // report the error, not get an empty descriptor from the cache.
                self.type_infos.remove(&ty);
                self.failed_type_infos.insert(ty, g);
                return Err(e);
            }
        };
        let global = &mut self.program.globals[g.0 as usize];
        global.init = agg.bytes;
        global.relocs = agg.relocs;
        Ok(g)
    }

    fn set_field(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        value: Value,
        span: Span,
    ) -> Result<()> {
        let Some((path, fty)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.types.name(desc)),
            );
        };
        let offset = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        self.write_value(agg, offset, &value, fty, span)
    }

    fn set_info_ptr(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        target: TypeId,
        span: Span,
    ) -> Result<()> {
        let Some((path, _)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.types.name(desc)),
            );
        };
        let offset = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        let g = self.type_info_global(target, span)?;
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }

    /// Write a `[] T` view field pointing at a new read-only global.
    #[allow(clippy::too_many_arguments)]
    fn set_view(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        count: usize,
        data: Aggregate,
        align: u64,
        span: Span,
    ) -> Result<()> {
        let Some((path, _)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.types.name(desc)),
            );
        };
        let offset: u64 = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        agg.bytes[offset as usize..offset as usize + 8]
            .copy_from_slice(&(count as u64).to_le_bytes());
        if count > 0 {
            let g = self.program.add_global(ir::Global {
                name: format!("type_info.{name}.{}", self.program.globals.len()),
                size: data.bytes.len() as u64,
                align,
                init: data.bytes,
                relocs: data.relocs,
                read_only: true,
                export: None,
            });
            agg.relocs.push(ir::Reloc {
                offset: offset + 8,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        Ok(())
    }

    fn info_view(&mut self, types: &[TypeId], span: Span) -> Result<Aggregate> {
        let mut data = Aggregate {
            bytes: vec![0; types.len() * 8],
            relocs: Vec::new(),
        };
        for (i, &t) in types.iter().enumerate() {
            let g = self.type_info_global(t, span)?;
            data.relocs.push(ir::Reloc {
                offset: i as u64 * 8,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        Ok(data)
    }

    fn string_view(&mut self, strings: &[Rc<[u8]>], span: Span) -> Result<Aggregate> {
        let mut data = Aggregate {
            bytes: vec![0; strings.len() * 16],
            relocs: Vec::new(),
        };
        for (i, s) in strings.iter().enumerate() {
            self.write_value(
                &mut data,
                i as u64 * 16,
                &Value::String(s.clone()),
                TypeId::STRING,
                span,
            )?;
        }
        Ok(data)
    }

    fn build_type_info(
        &mut self,
        ty: TypeId,
        desc: TypeId,
        size: u64,
        span: Span,
    ) -> Result<Aggregate> {
        let mut agg = Aggregate {
            bytes: vec![0; size as usize],
            relocs: Vec::new(),
        };
        let kind = self.types.kind(ty).clone();
        let tag = match &kind {
            TypeKind::Int {
                ..
            } => tag::INTEGER,
            TypeKind::Float {
                ..
            }
            | TypeKind::WideFloat(_) => tag::FLOAT,
            TypeKind::Bool => tag::BOOL,
            TypeKind::String => tag::STRING,
            TypeKind::Pointer(_) | TypeKind::Null => tag::POINTER,
            TypeKind::Proc(_) => tag::PROCEDURE,
            TypeKind::Void => tag::VOID,
            TypeKind::Struct(_) => tag::STRUCT,
            TypeKind::Array {
                ..
            } => tag::ARRAY,
            TypeKind::Any => tag::ANY,
            TypeKind::Enum(_) => tag::ENUM,
            TypeKind::Type => tag::TYPE,
            TypeKind::Code => tag::CODE,
            TypeKind::Distinct(_) => tag::VARIANT,
            TypeKind::CompileTimeOnly => tag::OVERLOAD_SET,
            TypeKind::PolyParam => tag::POLYMORPHIC_VARIABLE,
            TypeKind::PolyStruct {
                ..
            } => tag::STRUCT,
        };
        self.set_field(&mut agg, desc, "type", Value::Int(tag), span)?;
        let runtime_size = match &kind {
            TypeKind::Struct(s)
                if self.types.struct_info(*s).layout != crate::types::LayoutState::Done =>
            {
                self.size_of(ty, span).map(|s| s as i128).unwrap_or(-1)
            }
            _ => self.size_of(ty, span)? as i128,
        };
        self.set_field(
            &mut agg,
            desc,
            "runtime_size",
            Value::Int(runtime_size),
            span,
        )?;
        match kind {
            TypeKind::Int {
                signed, ..
            } => self.set_field(&mut agg, desc, "signed", Value::Bool(signed), span)?,
            TypeKind::Pointer(to) => self.set_info_ptr(&mut agg, desc, "pointer_to", to, span)?,
            TypeKind::Null => {
                self.set_info_ptr(&mut agg, desc, "pointer_to", TypeId::VOID, span)?
            }
            TypeKind::Array {
                elem,
                kind,
            } => {
                self.set_info_ptr(&mut agg, desc, "element_type", elem, span)?;
                let (at, count) = match kind {
                    ArrayKind::Fixed(n) => (0, n as i128),
                    ArrayKind::View => (1, -1),
                    ArrayKind::Resizable => (2, -1),
                };
                self.set_field(&mut agg, desc, "array_type", Value::Int(at), span)?;
                self.set_field(&mut agg, desc, "array_count", Value::Int(count), span)?;
            }
            TypeKind::Proc(p) => {
                let args = self.info_view(&p.params, span)?;
                self.set_view(
                    &mut agg,
                    desc,
                    "argument_types",
                    p.params.len(),
                    args,
                    8,
                    span,
                )?;
                let rets = self.info_view(&p.returns, span)?;
                self.set_view(
                    &mut agg,
                    desc,
                    "return_types",
                    p.returns.len(),
                    rets,
                    8,
                    span,
                )?;
                let mut flags = 0;
                if p.no_context {
                    flags |= 0x8;
                }
                if p.c_call {
                    flags |= 0x20;
                }
                self.set_field(&mut agg, desc, "procedure_flags", Value::Int(flags), span)?;
            }
            TypeKind::Enum(e) => {
                let info = self.types.enum_info(e).clone();
                // Like anonymous structs, an inline `enum {...}` has no name at runtime.
                let name = match info.name.as_str() {
                    "enum" => "",
                    name => name,
                };
                self.set_field(
                    &mut agg,
                    desc,
                    "name",
                    Value::String(name.as_bytes().into()),
                    span,
                )?;
                self.set_info_ptr(&mut agg, desc, "internal_type", info.base, span)?;
                let names: Vec<Rc<[u8]>> = info
                    .members
                    .iter()
                    .map(|(n, _)| Rc::from(n.as_str().as_bytes()))
                    .collect();
                let names_data = self.string_view(&names, span)?;
                self.set_view(&mut agg, desc, "names", names.len(), names_data, 8, span)?;
                let mut values = Aggregate {
                    bytes: Vec::new(),
                    relocs: Vec::new(),
                };
                for (_, v) in &info.members {
                    values.bytes.extend_from_slice(&(*v as i64).to_le_bytes());
                }
                self.set_view(
                    &mut agg,
                    desc,
                    "values",
                    info.members.len(),
                    values,
                    8,
                    span,
                )?;
                let flags = i128::from(info.type_flags());
                self.set_field(&mut agg, desc, "enum_type_flags", Value::Int(flags), span)?;
            }
            TypeKind::PolyStruct {
                name, ..
            } => {
                self.set_field(
                    &mut agg,
                    desc,
                    "name",
                    Value::String(name.as_str().as_bytes().into()),
                    span,
                )?;
                self.set_field(&mut agg, desc, "nontextual_flags", Value::Int(0x100), span)?;
            }
            TypeKind::Distinct(d) => {
                let info = self.types.distincts[d.0 as usize].clone();
                self.set_field(
                    &mut agg,
                    desc,
                    "name",
                    Value::String(info.name.as_str().as_bytes().into()),
                    span,
                )?;
                self.set_info_ptr(&mut agg, desc, "variant_of", info.base, span)?;
                self.set_field(
                    &mut agg,
                    desc,
                    "variant_flags",
                    Value::Int(if info.isa {
                        0x2
                    } else {
                        0x1
                    }),
                    span,
                )?;
            }
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let info = self.types.struct_info(s).clone();
                // Anonymous structs are named after the keyword internally; at runtime
                // they have no name.
                let name = match info.name.as_str() {
                    "struct" | "union" => "",
                    name => name,
                };
                self.set_field(
                    &mut agg,
                    desc,
                    "name",
                    Value::String(name.as_bytes().into()),
                    span,
                )?;
                let member_ty = self.preload_type("Type_Info_Struct_Member", span)?;
                let msize = self.size_of(member_ty, span)?;
                let malign = self.align_of(member_ty, span)?;
                // `Type_Info_Flags` from `compiler_set_type_info_flags` plus the directives.
                let mut tflags = self.type_info_flags.get(&ty).copied().unwrap_or(0);
                if let Some(src) = self.struct_asts.get(&s) {
                    if src.lit.flags.type_info_none {
                        tflags |= 0x1;
                    }
                    if src.lit.flags.type_info_procedures_are_void_pointers {
                        tflags |= 0x2;
                    }
                }
                let (mut fields, bindings) = self.tagged_union_members(s, &info.fields, span)?;
                if tflags & 0x1 != 0 {
                    fields.clear();
                } else if tflags & 0x2 != 0 {
                    for field in fields.iter_mut() {
                        if matches!(self.types.kind(field.ty), TypeKind::Proc(_)) {
                            field.ty = TypeId::VOID_PTR;
                        }
                    }
                }
                // `constant_storage` holds the polymorphic arguments, then the body's constants.
                let mut storage = Aggregate {
                    bytes: Vec::new(),
                    relocs: Vec::new(),
                };
                if !info.poly_args.is_empty() {
                    self.poly_struct_info(&mut agg, desc, s, &mut storage, span)?;
                }
                let constants = if tflags & 0x1 != 0 {
                    Vec::new()
                } else {
                    self.struct_constants(s)
                };
                // Members in declaration order: each constant goes before the first field
                // declared after it.
                let mut order = Vec::with_capacity(fields.len() + constants.len());
                let mut next_const = 0;
                for (i, field) in fields.iter().enumerate() {
                    while next_const < constants.len()
                        && constants[next_const].0.file == field.span.file
                        && constants[next_const].0.start < field.span.start
                    {
                        order.push(Err(next_const));
                        next_const += 1;
                    }
                    order.push(Ok(i));
                }
                order.extend((next_const..constants.len()).map(Err));
                let mut members = Aggregate {
                    bytes: vec![0; (msize as usize) * order.len()],
                    relocs: Vec::new(),
                };
                for (slot, entry) in order.iter().enumerate() {
                    let field = match *entry {
                        Ok(i) => &fields[i],
                        Err(c) => {
                            let (_, name, ty, ref value, ref cnotes) = constants[c];
                            let m = self.constant_member(
                                member_ty,
                                msize,
                                name,
                                ty,
                                value.clone(),
                                cnotes,
                                &mut storage,
                                span,
                            )?;
                            structs::write_agg(&mut members, slot as u64 * msize, &m);
                            continue;
                        }
                    };
                    let mut m = Aggregate {
                        bytes: vec![0; msize as usize],
                        relocs: Vec::new(),
                    };
                    if let Some(img) = self.default_initializer(member_ty, span)? {
                        structs::write_agg(&mut m, 0, &img);
                    }
                    let name = field
                        .name
                        .map(|n| n.as_str().to_string())
                        .unwrap_or_default();
                    self.set_field(
                        &mut m,
                        member_ty,
                        "name",
                        Value::String(name.as_bytes().into()),
                        span,
                    )?;
                    self.set_info_ptr(&mut m, member_ty, "type", field.ty, span)?;
                    self.set_field(
                        &mut m,
                        member_ty,
                        "offset_in_bytes",
                        Value::Int(field.offset as i128),
                        span,
                    )?;
                    let mut flags = 0;
                    if field.using {
                        flags |= 0x4;
                    }
                    if field.as_ {
                        flags |= 0x10;
                    }
                    if field.overlay {
                        flags |= 0x20;
                    }
                    self.set_field(&mut m, member_ty, "flags", Value::Int(flags), span)?;
                    let notes: Vec<Rc<[u8]>> =
                        field.notes.iter().map(|n| Rc::from(n.as_bytes())).collect();
                    let notes_data = self.string_view(&notes, span)?;
                    self.set_view(&mut m, member_ty, "notes", notes.len(), notes_data, 8, span)?;
                    structs::write_agg(&mut members, slot as u64 * msize, &m);
                }
                self.set_view(
                    &mut agg,
                    desc,
                    "members",
                    order.len(),
                    members,
                    malign,
                    span,
                )?;
                if !info.poly_args.is_empty() || !storage.bytes.is_empty() {
                    let storage_len = storage.bytes.len();
                    self.set_view(
                        &mut agg,
                        desc,
                        "constant_storage",
                        storage_len,
                        storage,
                        8,
                        span,
                    )?;
                }
                if !bindings.is_empty() {
                    self.set_tagged_union_bindings(&mut agg, desc, &bindings, span)?;
                }
                // A tagged union (`union(tag) {..}`) is laid out as a struct.
                let tagged = self
                    .struct_asts
                    .get(&s)
                    .is_some_and(|src| src.lit.tag.is_some());
                if info.is_union || tagged {
                    let flags = if tagged {
                        0x42
                    } else {
                        0x2
                    };
                    self.set_field(&mut agg, desc, "textual_flags", Value::Int(flags), span)?;
                }
                let no_padding = self
                    .struct_asts
                    .get(&s)
                    .is_some_and(|src| src.lit.flags.no_padding);
                if no_padding {
                    let cur = if info.is_union || tagged {
                        if tagged {
                            0x42
                        } else {
                            0x2
                        }
                    } else {
                        0
                    };
                    self.set_field(&mut agg, desc, "textual_flags", Value::Int(cur | 0x4), span)?;
                }
                // Every member is `= ---`: ALL_MEMBERS_UNINITIALIZED. The initializer stays
                // set (it leaves the members untouched).
                let all_uninit = self.struct_asts.get(&s).is_some_and(|src| {
                    !src.inits.is_empty()
                        && src.inits.iter().all(|(e, _)| {
                            e.as_ref()
                                .is_some_and(|e| matches!(e.kind, ast::ExprKind::Uninit))
                        })
                });
                if all_uninit {
                    self.set_field(&mut agg, desc, "nontextual_flags", Value::Int(0x40), span)?;
                }
                // Notes written on the struct itself: `S :: struct @thing { ... }`.
                let struct_notes: Vec<Rc<[u8]>> = self
                    .struct_asts
                    .get(&s)
                    .map(|src| {
                        src.lit
                            .notes
                            .iter()
                            .map(|n| Rc::from(n.text.as_bytes()))
                            .collect()
                    })
                    .unwrap_or_default();
                if !struct_notes.is_empty() {
                    let data = self.string_view(&struct_notes, span)?;
                    self.set_view(&mut agg, desc, "notes", struct_notes.len(), data, 8, span)?;
                }
                let (func, _) = self.initializer_proc(ty, span)?;
                let Some((path, _)) = self.find_member(desc, Sym::intern("initializer"), span)?
                else {
                    return err(span, "Type_Info_Struct has no initializer field");
                };
                let offset = path
                    .iter()
                    .map(|s| {
                        if let structs::PathStep::Offset(o) = s {
                            *o
                        } else {
                            0
                        }
                    })
                    .sum();
                agg.relocs.push(ir::Reloc {
                    offset,
                    target: ir::RelocTarget::Func(func),
                    addend: 0,
                });
            }
            _ => {}
        }
        Ok(agg)
    }

    /// The members a struct's type info lists. A tagged union lists its tag, then each
    /// variant member directly (not the anonymous union holding them), with the
    /// `(tag value, member index)` pairs selecting them.
    #[allow(clippy::type_complexity)]
    fn tagged_union_members(
        &mut self,
        s: crate::types::StructId,
        fields: &[crate::types::Field],
        span: Span,
    ) -> Result<(Vec<crate::types::Field>, Vec<(Value, usize)>)> {
        let Some(src) = self.struct_asts.get(&s).cloned() else {
            return Ok((fields.to_vec(), Vec::new()));
        };
        let (Some(_), [tag, variants]) = (&src.lit.tag, fields) else {
            return Ok((fields.to_vec(), Vec::new()));
        };
        let Some(vs) = self.types.as_struct(variants.ty) else {
            return Ok((fields.to_vec(), Vec::new()));
        };
        self.layout_struct(vs, span)?;
        let mut flat = vec![tag.clone()];
        for f in &self.types.struct_info(vs).fields {
            let mut f = f.clone();
            f.offset += variants.offset;
            flat.push(f);
        }
        let tag_members = match self.types.kind(tag.ty) {
            TypeKind::Enum(e) => self.types.enum_info(*e).members.clone(),
            _ => Vec::new(),
        };
        let mut bindings = Vec::new();
        for stmt in &src.lit.body {
            let ast::StmtKind::Decl(d) = &stmt.kind else {
                continue;
            };
            let Some(t) = &d.union_tag else {
                continue;
            };
            // `.A` names a member of the tag's enum; `4`, `-12` or `u16` is a constant of
            // the tag's type.
            let value = match &t.kind {
                ast::ExprKind::InferredMember(m) => {
                    match tag_members.iter().find(|(n, _)| *n == m.name) {
                        Some(&(_, v)) => Value::Int(v),
                        None => continue,
                    }
                }
                _ => match self.eval_const(src.scope, t, Some(tag.ty))? {
                    super::lower::Operand::Type(ty) => Value::Type(ty),
                    super::lower::Operand::Const {
                        value: v @ (Value::Int(_) | Value::Type(_)),
                        ..
                    } => v,
                    _ => continue,
                },
            };
            for n in &d.names {
                if let Some(index) = flat.iter().position(|f| f.name == Some(n.name)) {
                    bindings.push((value.clone(), index));
                }
            }
        }
        Ok((flat, bindings))
    }

    /// `tagged_union_bindings` of a tagged union's descriptor.
    fn set_tagged_union_bindings(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        bindings: &[(Value, usize)],
        span: Span,
    ) -> Result<()> {
        let binding_ty = self.preload_type("Type_Info_Tagged_Union_Binding", span)?;
        let size = self.size_of(binding_ty, span)?;
        let align = self.align_of(binding_ty, span)?;
        let mut items = Aggregate {
            bytes: vec![0; size as usize * bindings.len()],
            relocs: Vec::new(),
        };
        for (i, (value, index)) in bindings.iter().enumerate() {
            let index = *index;
            let mut b = Aggregate {
                bytes: vec![0; size as usize],
                relocs: Vec::new(),
            };
            match value {
                // A `Type` tag's constant is the type's descriptor address.
                Value::Type(t) => {
                    self.set_info_ptr(&mut b, binding_ty, "constant_value", *t, span)?;
                }
                v => self.set_field(&mut b, binding_ty, "constant_value", v.clone(), span)?,
            }
            self.set_field(
                &mut b,
                binding_ty,
                "member_index",
                Value::Int(index as i128),
                span,
            )?;
            structs::write_agg(&mut items, i as u64 * size, &b);
        }
        self.set_view(
            agg,
            desc,
            "tagged_union_bindings",
            bindings.len(),
            items,
            align,
            span,
        )
    }

    /// The parameter bindings of a polymorphic struct instance, in declaration
    /// order (baked ones from `#bake_arguments` first).
    pub(super) fn poly_struct_bindings(
        &self,
        s: crate::types::StructId,
    ) -> Vec<(Sym, Value, TypeId)> {
        let Some(scope) = self.struct_asts.get(&s).map(|src| src.scope) else {
            return Vec::new();
        };
        let mut bindings: Vec<(EntityId, Sym, Value, TypeId)> = self
            .scope(scope)
            .names
            .iter()
            .flat_map(|(&name, ids)| ids.iter().map(move |&e| (name, e)))
            .filter_map(|(name, e)| match &self.entity(e).kind {
                EntityKind::Const {
                    value,
                    ty,
                } => Some((e, name, value.clone(), *ty)),
                _ => None,
            })
            .collect();
        bindings.sort_by_key(|b| b.0);
        bindings.into_iter().map(|(_, n, v, t)| (n, v, t)).collect()
    }

    /// The constants a struct body declares (`Entry :: struct {..}`, `K :: 3;`, procedures),
    /// in declaration order, with their types and values. Overload sets and polymorphic
    /// procedures have no single value and are not listed. They are listed in
    /// `Type_Info_Struct.members` with the `CONSTANT` flag. A constant that fails to resolve
    /// is left out: type info never reports errors for code nothing uses.
    fn struct_constants(&mut self, s: crate::types::StructId) -> Vec<StructConstant> {
        let Some(scope) = self.struct_asts.get(&s).map(|src| src.scope) else {
            return Vec::new();
        };
        let mut ids: Vec<EntityId> = self
            .scope(scope)
            .names
            .values()
            .flatten()
            .copied()
            .filter(|&id| {
                matches!(
                    &self.entity(id).kind,
                    EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Const
                )
            })
            .collect();
        ids.sort_by_key(|&id| {
            let span = self.entity(id).span;
            (span.file, span.start)
        });
        // The Context's constants are hooks (`#add_context _hook :: proc;`) that code detects by
        // name, so their signatures are resolved even when nothing has used them yet.
        let is_context = self
            .context_type
            .is_some_and(|t| self.types.as_struct(t) == Some(s));
        let mut out = Vec::new();
        for id in ids {
            let (name, span) = (self.entity(id).name, self.entity(id).span);
            let notes: Vec<Rc<[u8]>> = match &self.entity(id).kind {
                EntityKind::Decl {
                    decl, ..
                } => decl
                    .notes
                    .iter()
                    .map(|n| Rc::from(n.text.as_bytes()))
                    .collect(),
                _ => Vec::new(),
            };
            let resolved = match self.resolve_entity(id) {
                Ok(Resolved::Const {
                    value,
                    ty,
                }) => Ok((ty, Some(value))),
                // A procedure's value is its address (Objective_C reads methods from there).
                // Only once its signature is known: resolving it here can lay out the types it
                // names early. Vk-Engine's entity methods take a `*World`, whose `#insert` reads
                // a list the metaprogram has not finished, so it failed.
                Ok(Resolved::Proc(p))
                    if !self.proc(p).is_poly && (is_context || self.proc(p).sig.is_some()) =>
                {
                    self.proc_type(p, span).map(|ty| (ty, Some(Value::Proc(p))))
                }
                Ok(_) => continue,
                Err(e) => Err(e),
            };
            if let Ok((ty, value)) = resolved {
                out.push((span, name, ty, value, notes));
            }
        }
        out
    }

    /// One `CONSTANT` entry of `Type_Info_Struct.members`; its value, if it has one, is
    /// appended to `storage` and `offset_into_constant_storage` points at it.
    #[allow(clippy::too_many_arguments)]
    fn constant_member(
        &mut self,
        member_ty: TypeId,
        msize: u64,
        name: Sym,
        mut ty: TypeId,
        value: Option<Value>,
        notes: &[Rc<[u8]>],
        storage: &mut Aggregate,
        span: Span,
    ) -> Result<Aggregate> {
        let mut m = Aggregate {
            bytes: vec![0; msize as usize],
            relocs: Vec::new(),
        };
        if let Some(img) = self.default_initializer(member_ty, span)? {
            structs::write_agg(&mut m, 0, &img);
        }
        if let Some(v) = &value
            && self.size_of(ty, span).is_err()
        {
            ty = self.type_of_value(v); // Untyped literals.
        }
        let mut offset: i128 = -1;
        if let Some(value) = value
            && let (Ok(size), Ok(align)) = (self.size_of(ty, span), self.align_of(ty, span))
        {
            let at = storage.bytes.len().next_multiple_of(align.max(1) as usize);
            storage.bytes.resize(at + size as usize, 0);
            if let Value::Type(t) = value {
                let g = self.type_info_global(t, span)?;
                storage.relocs.push(ir::Reloc {
                    offset: at as u64,
                    target: ir::RelocTarget::Global(g),
                    addend: 0,
                });
                offset = at as i128;
            } else if self
                .write_value(storage, at as u64, &value, ty, span)
                .is_ok()
            {
                offset = at as i128;
            } else {
                storage.bytes.truncate(at);
            }
        }
        self.set_field(
            &mut m,
            member_ty,
            "name",
            Value::String(name.as_str().as_bytes().into()),
            span,
        )?;
        self.set_info_ptr(&mut m, member_ty, "type", ty, span)?;
        self.set_field(&mut m, member_ty, "flags", Value::Int(0x1), span)?;
        let notes_data = self.string_view(notes, span)?;
        self.set_view(&mut m, member_ty, "notes", notes.len(), notes_data, 8, span)?;
        self.set_field(
            &mut m,
            member_ty,
            "offset_into_constant_storage",
            Value::Int(offset),
            span,
        )?;
        Ok(m)
    }

    /// `specified_parameters` and `polymorph_source_struct` of a polymorphic struct instance's
    /// runtime descriptor. The arguments' values go to `storage` (`constant_storage`).
    fn poly_struct_info(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        s: crate::types::StructId,
        storage: &mut Aggregate,
        span: Span,
    ) -> Result<()> {
        let member_ty = self.preload_type("Type_Info_Struct_Member", span)?;
        let msize = self.size_of(member_ty, span)?;
        let malign = self.align_of(member_ty, span)?;
        let mut members = Aggregate {
            bytes: Vec::new(),
            relocs: Vec::new(),
        };
        let mut count = 0;
        for (name, value, mut ty) in self.poly_struct_bindings(s) {
            if self.size_of(ty, span).is_err() {
                ty = self.type_of_value(&value); // Untyped literal arguments.
            }
            let (Ok(size), Ok(align)) = (self.size_of(ty, span), self.align_of(ty, span)) else {
                continue;
            };
            let offset = storage.bytes.len().next_multiple_of(align.max(1) as usize);
            storage.bytes.resize(offset + size as usize, 0);
            if let Value::Type(t) = value {
                let g = self.type_info_global(t, span)?;
                storage.relocs.push(ir::Reloc {
                    offset: offset as u64,
                    target: ir::RelocTarget::Global(g),
                    addend: 0,
                });
            } else if self
                .write_value(storage, offset as u64, &value, ty, span)
                .is_err()
            {
                storage.bytes.truncate(offset);
                continue;
            }
            let mut m = Aggregate {
                bytes: vec![0; msize as usize],
                relocs: Vec::new(),
            };
            self.set_field(
                &mut m,
                member_ty,
                "name",
                Value::String(name.as_str().as_bytes().into()),
                span,
            )?;
            self.set_info_ptr(&mut m, member_ty, "type", ty, span)?;
            self.set_field(&mut m, member_ty, "flags", Value::Int(0x1), span)?;
            self.set_field(
                &mut m,
                member_ty,
                "offset_into_constant_storage",
                Value::Int(offset as i128),
                span,
            )?;
            let at = members.bytes.len() as u64;
            members.bytes.resize(at as usize + msize as usize, 0);
            structs::write_agg(&mut members, at, &m);
            count += 1;
        }
        self.set_view(
            agg,
            desc,
            "specified_parameters",
            count,
            members,
            malign,
            span,
        )?;
        // A stand-in descriptor for the generic struct, carrying its name: one per generic
        // struct, so every instance's `polymorph_source_struct` is the same pointer.
        let generic_key = self.struct_asts.get(&s).map(|src| src.lit.id);
        let cached = generic_key.and_then(|k| self.generic_struct_infos.get(&k).copied());
        let size = self.size_of(desc, span)?;
        let mut generic = Aggregate {
            bytes: vec![0; size as usize],
            relocs: Vec::new(),
        };
        let name = self.types.struct_info(s).name;
        self.set_field(&mut generic, desc, "type", Value::Int(tag::STRUCT), span)?;
        self.set_field(
            &mut generic,
            desc,
            "name",
            Value::String(name.as_str().as_bytes().into()),
            span,
        )?;
        self.set_field(
            &mut generic,
            desc,
            "nontextual_flags",
            Value::Int(0x100),
            span,
        )?;
        let align = self.align_of(desc, span)?;
        let g = match cached {
            Some(g) => g,
            None => {
                let g = self.program.add_global(ir::Global {
                    name: format!("type_info.generic.{}", name.as_str()),
                    size,
                    align,
                    init: generic.bytes,
                    relocs: generic.relocs,
                    read_only: true,
                    export: None,
                });
                if let Some(k) = generic_key {
                    self.generic_struct_infos.insert(k, g);
                }
                g
            }
        };
        let Some((path, _)) =
            self.find_member(desc, Sym::intern("polymorph_source_struct"), span)?
        else {
            return Ok(());
        };
        let offset = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }

    /// Apply `compiler_set_type_info_flags` calls made by the `#run` that just returned:
    /// the struct's flags are or-ed in and an already built descriptor is rebuilt.
    pub fn apply_type_info_flags(&mut self, span: Span) -> Result<()> {
        for (g, flags) in std::mem::take(&mut self.interp.pending_type_flags) {
            let Some(ty) = self.type_from_info_global(g) else {
                continue;
            };
            if !matches!(self.types.kind(ty), TypeKind::Struct(_)) {
                continue;
            }
            *self.type_info_flags.entry(ty).or_insert(0) |= flags;
            let desc = self.type_info_struct_type(ty, span)?;
            let size = self.size_of(desc, span)?;
            let agg = self.build_type_info(ty, desc, size, span)?;
            let global = &mut self.program.globals[g.0 as usize];
            global.init = agg.bytes;
            global.relocs = agg.relocs;
            self.interp
                .refresh_global(&self.program, g)
                .map_err(|t| Box::new(Diagnostic::error(span, t.into_message())))?;
        }
        Ok(())
    }

    /// Map a type-info descriptor address back to its type (compile-time reads).
    pub fn type_from_info_global(&self, g: ir::GlobalId) -> Option<TypeId> {
        self.type_infos
            .iter()
            .find(|(_, v)| **v == g)
            .map(|(&t, _)| t)
    }
}
