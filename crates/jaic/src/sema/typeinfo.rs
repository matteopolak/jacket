//! Runtime type information: one read-only `Type_Info_*` descriptor global
//! per type, laid out with Preload's reflection structs. A `Type` value at
//! runtime is the address of its descriptor.
use super::value::Aggregate;
use super::*;
use crate::types::{ArrayKind, TypeKind};

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
            } => "Type_Info_Float",
            TypeKind::String => "Type_Info_String",
            TypeKind::Pointer(_) => "Type_Info_Pointer",
            TypeKind::Proc(_) => "Type_Info_Procedure",
            TypeKind::Struct(_) => "Type_Info_Struct",
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
        let g = self.program.add_global(ir::Global {
            name: format!("type_info.{}", self.types.name(ty)),
            size,
            align,
            init: Vec::new(),
            relocs: Vec::new(),
            read_only: true,
            export: None,
        });
        // Register first: descriptors may refer to themselves.
        self.type_infos.insert(ty, g);
        let agg = self.build_type_info(ty, desc, size, span)?;
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
                format!("{} has no field '{name}'", self.types.name(desc)),
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
                format!("{} has no field '{name}'", self.types.name(desc)),
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
                format!("{} has no field '{name}'", self.types.name(desc)),
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
            } => tag::FLOAT,
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
                self.set_field(
                    &mut agg,
                    desc,
                    "name",
                    Value::String(info.name.as_str().as_bytes().into()),
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
                let flags = if info.is_flags {
                    0x1
                } else {
                    0
                };
                self.set_field(&mut agg, desc, "enum_type_flags", Value::Int(flags), span)?;
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
                    "name",
                    Value::String(info.name.as_str().as_bytes().into()),
                    span,
                )?;
                let member_ty = self.preload_type("Type_Info_Struct_Member", span)?;
                let msize = self.size_of(member_ty, span)?;
                let malign = self.align_of(member_ty, span)?;
                let mut members = Aggregate {
                    bytes: vec![0; (msize as usize) * info.fields.len()],
                    relocs: Vec::new(),
                };
                for (i, field) in info.fields.iter().enumerate() {
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
                    self.set_field(&mut m, member_ty, "flags", Value::Int(flags), span)?;
                    let notes: Vec<Rc<[u8]>> =
                        field.notes.iter().map(|n| Rc::from(n.as_bytes())).collect();
                    let notes_data = self.string_view(&notes, span)?;
                    self.set_view(&mut m, member_ty, "notes", notes.len(), notes_data, 8, span)?;
                    structs::write_agg(&mut members, i as u64 * msize, &m);
                }
                self.set_view(
                    &mut agg,
                    desc,
                    "members",
                    info.fields.len(),
                    members,
                    malign,
                    span,
                )?;
                if info.is_union {
                    self.set_field(&mut agg, desc, "textual_flags", Value::Int(0x2), span)?;
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

    /// Map a type-info descriptor address back to its type (compile-time reads).
    pub fn type_from_info_global(&self, g: ir::GlobalId) -> Option<TypeId> {
        self.type_infos
            .iter()
            .find(|(_, v)| **v == g)
            .map(|(&t, _)| t)
    }
}
