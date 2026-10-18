//! Building the type information record of a type.
use super::descriptor_of;
use crate::sema::scope::ScopeKind;
use crate::sema::value::Aggregate;
use crate::sema::*;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    pub(super) fn build_type_info(
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
        let tag = descriptor_of(&kind).1;
        self.set_field(&mut agg, desc, "type", Value::Int(tag), span)?;
        let runtime_size = match &kind {
            TypeKind::Struct(s)
                if self.types.struct_info(*s).layout != crate::types::LayoutState::Done =>
            {
                // A struct still being laid out (its own members ask for its type info): the
                // size is not known yet, and `-1` marks it so until the layout is done.
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
                self.set_field(
                    &mut agg,
                    desc,
                    "alignment",
                    Value::Int(info.align as i128),
                    span,
                )?;
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
                let member_ty = self.preload_type(PreludeType::TypeInfoStructMember, span)?;
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
                    let hidden_procedure =
                        tflags & 0x2 != 0 && matches!(self.types.kind(field.ty), TypeKind::Proc(_));
                    let reflected_type = if hidden_procedure {
                        TypeId::VOID_PTR
                    } else {
                        field.ty
                    };
                    self.set_info_ptr(&mut m, member_ty, "type", reflected_type, span)?;
                    self.set_field(
                        &mut m,
                        member_ty,
                        "offset_in_bytes",
                        Value::Int(field.offset as i128),
                        span,
                    )?;
                    let mut flags = if hidden_procedure {
                        0x8
                    } else {
                        0
                    };
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
                // A struct declared inside a procedure, a block or another struct is LOCAL.
                let local = self.struct_asts.get(&s).is_some_and(|src| {
                    let mut at = self.scope(src.scope).parent;
                    while let Some(p) = at
                        && self.scope(p).kind == ScopeKind::StructParams
                    {
                        at = self.scope(p).parent;
                    }
                    at.is_some_and(|p| {
                        !matches!(
                            self.scope(p).kind,
                            ScopeKind::Module | ScopeKind::File | ScopeKind::Root
                        )
                    })
                });
                if local {
                    self.set_field(&mut agg, desc, "status_flags", Value::Int(0x4), span)?;
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
}
