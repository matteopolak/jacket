//! Canonical compiler-owned Type_Info schemas for one semantic session.
use super::*;
use jai_types::{RecordKind, ScalarType, TypeInfoTag, TypeKind, Variadic};
mod preload;

pub(crate) struct TypeInfoSchema {
    pub header: TypeId,
    pub integer: TypeId,
    pub float: TypeId,
    pub string: TypeId,
    pub pointer: TypeId,
    pub procedure: TypeId,
    pub record: TypeId,
    pub member: TypeId,
    pub array: TypeId,
    pub enumeration: TypeId,
    pub variant: TypeId,
    pub tag: TypeId,
    pub array_kind: TypeId,
    pub procedure_flags: TypeId,
    pub member_flags: TypeId,
    pub record_status: TypeId,
    pub record_nontextual: TypeId,
    pub record_textual: TypeId,
    pub enum_status: TypeId,
    pub enum_flags: TypeId,
    pub variant_flags: TypeId,
    pub header_pointer: TypeId,
    pub fields: HashMap<TypeId, Vec<(String, jai_types::FieldId, bool)>>,
    names: HashMap<String, TypeId>,
}
impl TypeInfoSchema {
    pub(crate) fn recognizes_name(path: &syntax::NamePath, symbols: &Symbols) -> bool {
        let name = std::iter::once(path.root)
            .chain(path.members.iter().copied())
            .map(|symbol| symbols.name(symbol))
            .collect::<Vec<_>>()
            .join(".");
        matches!(
            name.as_str(),
            "Type_Info"
                | "Type_Info_Integer"
                | "Type_Info_Float"
                | "Type_Info_String"
                | "Type_Info_Pointer"
                | "Type_Info_Procedure"
                | "Type_Info_Struct"
                | "Type_Info_Struct_Member"
                | "Type_Info_Array"
                | "Type_Info_Enum"
                | "Type_Info_Variant"
                | "Type_Info_Tag"
                | "Type_Info_Array.Array_Type"
                | "Type_Info_Procedure.Flags"
                | "Type_Info_Struct_Member.Flags"
                | "Struct_Status_Flags"
                | "Struct_Nontextual_Flags"
                | "Struct_Textual_Flags"
                | "Enum_Status_Flags"
                | "Enum_Type_Flags"
                | "Type_Info_Variant_Flags"
        )
    }
    pub fn new(types: &mut TypeRegistry) -> Result<Self, jai_types::TypeError> {
        let tag = enumeration(
            types,
            IntegerType::U32,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18],
        )?;
        let array_kind = enumeration(types, IntegerType::U16, &[0, 1, 2])?;
        let procedure_flags = enumeration(
            types,
            IntegerType::U32,
            &[1, 2, 4, 8, 32, 128, 256, 0x1000_0000, 0x2000_0000],
        )?;
        let member_flags = enumeration(types, IntegerType::U32, &[1, 2, 4, 8, 16])?;
        let record_status = enumeration(types, IntegerType::U32, &[1, 4])?;
        let record_nontextual = enumeration(types, IntegerType::U32, &[4, 64, 256])?;
        let record_textual = enumeration(types, IntegerType::U32, &[1, 2, 4, 8, 16, 32])?;
        let enum_status = enumeration(types, IntegerType::U16, &[1])?;
        let enum_flags = enumeration(types, IntegerType::U16, &[1, 2, 4])?;
        let variant_flags = enumeration(types, IntegerType::U32, &[1, 2])?;
        let header = types.reserve_record(RecordKind::Struct);
        let integer = types.reserve_record(RecordKind::Struct);
        let float = types.reserve_record(RecordKind::Struct);
        let string = types.reserve_record(RecordKind::Struct);
        let pointer = types.reserve_record(RecordKind::Struct);
        let procedure = types.reserve_record(RecordKind::Struct);
        let record = types.reserve_record(RecordKind::Struct);
        let member = types.reserve_record(RecordKind::Struct);
        let array = types.reserve_record(RecordKind::Struct);
        let enumeration_ty = types.reserve_record(RecordKind::Struct);
        let variant = types.reserve_record(RecordKind::Struct);
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let bool_ty = types.scalar(ScalarType::Bool);
        let string_ty = types.string();
        let header_pointer = types.pointer(header)?;
        let record_pointer = types.pointer(record)?;
        let integer_pointer = types.pointer(integer)?;
        let descriptor_view = types.slice(header_pointer)?;
        let member_view = types.slice(member)?;
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let bytes = types.slice(byte)?;
        let strings = types.slice(string_ty)?;
        let integers = types.slice(int)?;
        let void_pointer = types.pointer(types.void())?;
        let initializer = types.procedure(ProcedureType {
            parameters: Box::new([void_pointer]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })?;
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
            enumeration: enumeration_ty,
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
        schema.define(
            types,
            header,
            "Type_Info",
            &[("type", tag, false), ("runtime_size", int, false)],
        )?;
        schema.define(
            types,
            integer,
            "Type_Info_Integer",
            &[("info", header, true), ("signed", bool_ty, false)],
        )?;
        schema.define(types, float, "Type_Info_Float", &[("info", header, true)])?;
        schema.define(types, string, "Type_Info_String", &[("info", header, true)])?;
        schema.define(
            types,
            pointer,
            "Type_Info_Pointer",
            &[
                ("info", header, true),
                ("pointer_to", header_pointer, false),
            ],
        )?;
        schema.define(
            types,
            procedure,
            "Type_Info_Procedure",
            &[
                ("info", header, true),
                ("argument_types", descriptor_view, false),
                ("return_types", descriptor_view, false),
                ("procedure_flags", procedure_flags, false),
            ],
        )?;
        schema.define(
            types,
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
        schema.define(
            types,
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
        schema.define(
            types,
            array,
            "Type_Info_Array",
            &[
                ("info", header, true),
                ("element_type", header_pointer, false),
                ("array_type", array_kind, false),
                ("array_count", int, false),
            ],
        )?;
        schema.define(
            types,
            enumeration_ty,
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
        schema.define(
            types,
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
        schema.complete_reserved_any(types)?;
        types.bind_runtime_type_header(schema.header)?;
        Ok(schema)
    }
    pub(crate) fn complete_reserved_any(
        &self,
        types: &mut TypeRegistry,
    ) -> Result<(), jai_types::TypeError> {
        let Some(ty) = types.any_type() else {
            return Ok(());
        };
        match types.record_storage_definition(ty) {
            Err(jai_types::TypeError::Incomplete(owner)) if owner == ty => {
                types.define_any(ty, self.header)
            }
            Ok(definition)
                if definition.fields.len() == 2
                    && definition.fields[0] == self.header_pointer
                    && matches!(types.kind(definition.fields[1])?, TypeKind::Pointer(pointee) if *pointee == types.void()) =>
            {
                Ok(())
            }
            Ok(_) => Err(jai_types::TypeError::WrongKind(ty)),
            Err(error) => Err(error),
        }
    }
    fn define(
        &mut self,
        types: &mut TypeRegistry,
        ty: TypeId,
        name: &str,
        fields: &[(&str, TypeId, bool)],
    ) -> Result<(), jai_types::TypeError> {
        types.define_record(ty, fields.iter().map(|field| field.1).collect::<Vec<_>>())?;
        self.names.insert(name.into(), ty);
        self.fields.insert(
            ty,
            fields
                .iter()
                .enumerate()
                .map(|(index, field)| Ok((field.0.into(), types.field(ty, index)?.id, field.2)))
                .collect::<Result<Vec<_>, jai_types::TypeError>>()?,
        );
        Ok(())
    }
    pub fn descriptor_type(&self, tag: TypeInfoTag) -> TypeId {
        match tag {
            TypeInfoTag::Integer => self.integer,
            TypeInfoTag::Float => self.float,
            TypeInfoTag::Bool
            | TypeInfoTag::Void
            | TypeInfoTag::Type
            | TypeInfoTag::Code
            | TypeInfoTag::Any => self.header,
            TypeInfoTag::String => self.string,
            TypeInfoTag::Pointer => self.pointer,
            TypeInfoTag::Procedure => self.procedure,
            TypeInfoTag::Struct => self.record,
            TypeInfoTag::Array => self.array,
            TypeInfoTag::Enum => self.enumeration,
            TypeInfoTag::Variant => self.variant,
        }
    }
    pub fn type_name(&self, path: &syntax::NamePath, symbols: &Symbols) -> Option<TypeId> {
        let name = std::iter::once(path.root)
            .chain(path.members.iter().copied())
            .map(|symbol| symbols.name(symbol))
            .collect::<Vec<_>>()
            .join(".");
        self.names.get(&name).copied()
    }
    pub fn field_path(&self, ty: TypeId, name: &str) -> Option<Vec<jai_types::FieldId>> {
        let fields = self.fields.get(&ty)?;
        if let Some((_, field, _)) = fields.iter().find(|field| field.0 == name) {
            return Some(vec![*field]);
        }
        for (_, field, using) in fields {
            if *using && let Some(mut nested) = self.field_path(self.header, name) {
                nested.insert(0, *field);
                return Some(nested);
            }
        }
        None
    }
}

fn enumeration(
    types: &mut TypeRegistry,
    representation: IntegerType,
    values: &[i128],
) -> Result<TypeId, jai_types::TypeError> {
    let ty = types.reserve_enum(representation);
    types.define_enum(
        ty,
        values
            .iter()
            .map(|&value| IntegerValue::wrapping(representation, value))
            .collect::<Vec<_>>(),
    )?;
    Ok(ty)
}
