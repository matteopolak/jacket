//! Records for types, including polymorphic struct parameters.
use crate::records::{Item, Record, RecordTag, Records};
use crate::sema::typeinfo::descriptor_of;
use crate::sema::{Compiler, PreludeType, Span, TypeId, Value, value};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// `specified_parameters`, `constant_storage` and `polymorph_source_struct`
    /// of an instance of a polymorphic struct. Pointers in the constant storage
    /// (`Type` parameters) are listed in `__constant_pointers` as (offset,
    /// record) pairs for the metaprogram to patch.
    fn poly_parameters(&mut self, r: &mut Records, s: crate::types::StructId, rec: &mut Record) {
        let Some(span) = self.struct_asts.get(&s).map(|src| src.lit.span) else {
            return;
        };
        let mut storage = value::Aggregate {
            bytes: Vec::new(),
            relocs: Vec::new(),
        };
        let mut params = Vec::new();
        let mut pointers = Vec::new();
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
                let t = self.type_record(r, t);
                pointers.extend([Item::Int(offset as i64), Item::Int(t)]);
            } else if self
                .write_value(&mut storage, offset as u64, &value, ty, span)
                .is_err()
            {
                continue;
            }
            let ty = self.type_record(r, ty);
            let mut m = Record::new(RecordTag::TypeInfoStructMember);
            m.str("name", name.as_str().as_bytes())
                .ptr("type", ty)
                .int("flags", 0x1)
                .int("offset_into_constant_storage", offset as i64);
            params.push(r.add(m));
        }
        let mut generic = Record::new(RecordTag::TypeInfoStruct);
        generic
            .int("type", 7)
            .str("name", self.types.struct_info(s).name.as_str().as_bytes())
            .int("nontextual_flags", 0x100);
        rec.refs("specified_parameters", params)
            .list(
                "constant_storage",
                storage.bytes.iter().map(|&b| Item::Int(b as i64)).collect(),
            )
            .list("__constant_pointers", pointers)
            .ptr("polymorph_source_struct", r.add(generic));
    }

    /// The record of a type descriptor (`Type_Info_*`). Primitive types carry
    /// `__builtin`, their name, so the metaprogram can use its own descriptor.
    pub fn type_record(&mut self, r: &mut Records, ty: TypeId) -> i64 {
        if let Some(&id) = self.meta.export.types.get(&ty) {
            return id;
        }
        let kind = self.types.kind(ty).clone();
        let span = Span::NONE;
        let size = match &kind {
            TypeKind::Struct(s) => {
                let _ = self.layout_struct(*s, span);
                self.size_of(ty, span).map_or(-1, |s| s as i64)
            }
            _ => self.size_of(ty, span).map_or(0, |s| s as i64),
        };
        let builtin = matches!(
            kind,
            TypeKind::Void
                | TypeKind::Bool
                | TypeKind::Int { .. }
                | TypeKind::Float { .. }
                | TypeKind::WideFloat(_)
                | TypeKind::String
                | TypeKind::Type
                | TypeKind::Any
                | TypeKind::Code
        );
        let (prelude, info_tag) = descriptor_of(&kind);
        let tag = match prelude {
            PreludeType::TypeInfoInteger => RecordTag::TypeInfoInteger,
            PreludeType::TypeInfoFloat => RecordTag::TypeInfoFloat,
            PreludeType::TypeInfoString => RecordTag::TypeInfoString,
            PreludeType::TypeInfoPointer => RecordTag::TypeInfoPointer,
            PreludeType::TypeInfoProcedure => RecordTag::TypeInfoProcedure,
            PreludeType::TypeInfoStruct => RecordTag::TypeInfoStruct,
            PreludeType::TypeInfoArray => RecordTag::TypeInfoArray,
            PreludeType::TypeInfoEnum => RecordTag::TypeInfoEnum,
            PreludeType::TypeInfoVariant => RecordTag::TypeInfoVariant,
            _ => RecordTag::TypeInfo,
        };
        // Registered before the fields: types may refer to themselves.
        let id = r.reserve(tag);
        self.meta.export.types.insert(ty, id);
        let mut rec = Record::new(tag);
        rec.int("type", info_tag as i64).int("runtime_size", size);
        if builtin {
            rec.str("__builtin", self.types.name(ty).as_bytes());
        }
        match kind {
            TypeKind::Int {
                signed, ..
            } => {
                rec.int("signed", signed as i64);
            }
            TypeKind::Pointer(to) => {
                let to = self.type_record(r, to);
                rec.ptr("pointer_to", to);
            }
            TypeKind::Array {
                elem,
                kind,
            } => {
                let elem = self.type_record(r, elem);
                let (array_type, count) = match kind {
                    ArrayKind::Fixed(n) => (0, n as i64),
                    ArrayKind::View => (1, -1),
                    ArrayKind::Resizable => (2, -1),
                };
                rec.ptr("element_type", elem)
                    .int("array_type", array_type)
                    .int("array_count", count);
            }
            TypeKind::Proc(p) => {
                let args: Vec<i64> = p.params.iter().map(|&t| self.type_record(r, t)).collect();
                let rets: Vec<i64> = p.returns.iter().map(|&t| self.type_record(r, t)).collect();
                let flags = ((p.no_context as i64) * 0x8) | ((p.c_call as i64) * 0x20);
                rec.refs("argument_types", args)
                    .refs("return_types", rets)
                    .int("procedure_flags", flags);
            }
            TypeKind::Enum(e) => {
                let info = self.types.enum_info(e).clone();
                let base = self.type_record(r, info.base);
                rec.str("name", info.name.as_str().as_bytes())
                    .ptr("internal_type", base)
                    .list(
                        "names",
                        info.members
                            .iter()
                            .map(|(n, _)| Item::Str(n.as_str().as_bytes().into()))
                            .collect(),
                    )
                    .list(
                        "values",
                        info.members
                            .iter()
                            .map(|&(_, v)| Item::Int(v as i64))
                            .collect(),
                    )
                    .int("enum_type_flags", info.type_flags());
            }
            TypeKind::Distinct(d) => {
                let info = self.types.distincts[d.0 as usize].clone();
                let base = self.type_record(r, info.base);
                rec.str("name", info.name.as_str().as_bytes())
                    .ptr("variant_of", base)
                    .int(
                        "variant_flags",
                        if info.isa {
                            0x2
                        } else {
                            0x1
                        },
                    );
            }
            TypeKind::Struct(s) => {
                let info = self.types.struct_info(s).clone();
                let mut members = Vec::new();
                for field in info.fields.iter() {
                    let fty = self.type_record(r, field.ty);
                    let mut m = Record::new(RecordTag::TypeInfoStructMember);
                    m.str("name", field.name.map_or("", |n| n.as_str()).as_bytes())
                        .ptr("type", fty)
                        .int("offset_in_bytes", field.offset as i64)
                        .int(
                            "flags",
                            ((field.using as i64) * 0x4) | ((field.as_ as i64) * 0x10),
                        )
                        .list(
                            "notes",
                            field
                                .notes
                                .iter()
                                .map(|n| Item::Str(n.as_bytes().into()))
                                .collect(),
                        )
                        .int("offset_into_constant_storage", -1);
                    members.push(r.add(m));
                }
                rec.str("name", info.name.as_str().as_bytes())
                    .refs("members", members)
                    .int("textual_flags", (info.is_union as i64) * 0x2);
                if !info.poly_args.is_empty() {
                    self.poly_parameters(r, s, &mut rec);
                }
            }
            _ => {}
        }
        *r.get_mut(id).unwrap() = rec;
        id
    }
}
