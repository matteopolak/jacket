//! Runtime type information: one read-only `Type_Info_*` descriptor global
//! per type, laid out with Preload's reflection structs. A `Type` value at
//! runtime is the address of its descriptor.
mod build;
mod constants;
mod generic;
mod tagged;
mod views;

use super::{Compiler, Diagnostic, PreludeType, Rc, Result, Span, Sym, TypeId, Value, ir};
use crate::types::TypeKind;

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

/// The descriptor of a kind of type: the Preload struct that describes it and the
/// `Type_Info_Tag` stored in its `type` field. The runtime descriptors and the metaprogram's
/// records are both built from this one table.
pub(in crate::sema) fn descriptor_of(kind: &TypeKind) -> (PreludeType, i128) {
    match kind {
        TypeKind::Int {
            ..
        } => (PreludeType::TypeInfoInteger, tag::INTEGER),
        TypeKind::Float {
            ..
        }
        | TypeKind::WideFloat(_) => (PreludeType::TypeInfoFloat, tag::FLOAT),
        TypeKind::Bool => (PreludeType::TypeInfo, tag::BOOL),
        TypeKind::String => (PreludeType::TypeInfoString, tag::STRING),
        TypeKind::Pointer(_) | TypeKind::Null => (PreludeType::TypeInfoPointer, tag::POINTER),
        TypeKind::Proc(_) => (PreludeType::TypeInfoProcedure, tag::PROCEDURE),
        TypeKind::Void => (PreludeType::TypeInfo, tag::VOID),
        TypeKind::Struct(_)
        | TypeKind::PolyStruct {
            ..
        } => (PreludeType::TypeInfoStruct, tag::STRUCT),
        TypeKind::Array {
            ..
        } => (PreludeType::TypeInfoArray, tag::ARRAY),
        TypeKind::Any => (PreludeType::TypeInfo, tag::ANY),
        TypeKind::Enum(_) => (PreludeType::TypeInfoEnum, tag::ENUM),
        TypeKind::Type => (PreludeType::TypeInfo, tag::TYPE),
        TypeKind::Code => (PreludeType::TypeInfo, tag::CODE),
        TypeKind::Distinct(_) => (PreludeType::TypeInfoVariant, tag::VARIANT),
        TypeKind::CompileTimeOnly => (PreludeType::TypeInfo, tag::OVERLOAD_SET),
        TypeKind::PolyParam => (PreludeType::TypeInfo, tag::POLYMORPHIC_VARIABLE),
    }
}

impl Compiler {
    /// The Preload struct describing `ty` (`Type_Info_Struct`, `Type_Info_Integer`...).
    pub fn type_info_struct_type(&mut self, ty: TypeId, span: Span) -> Result<TypeId> {
        let name = descriptor_of(self.types.kind(ty)).0;
        self.preload_type(name, span)
    }

    pub fn type_info_global(&mut self, ty: TypeId, span: Span) -> Result<ir::GlobalId> {
        if let Some(&g) = self.type_infos.get(&ty) {
            if !self.in_snapshot {
                self.snapshot_infos.remove(&ty);
            }
            return Ok(g);
        }
        let desc = self.type_info_struct_type(ty, span)?;
        let size = self.size_of(desc, span)?;
        let align = self.align_of(desc, span)?;
        // A descriptor whose build failed before keeps its global, so code lowered since
        // (a self-reference) points at the one that is filled in now.
        let g = match self.failed_type_infos.remove(&ty) {
            Some(g) => g,
            None => self.out.program.add_global(ir::Global {
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
                let path: Rc<str> = source.path.to_string().into();
                self.ct
                    .interp
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
        let global = &mut self.out.program.globals[g.0 as usize];
        global.init = agg.bytes;
        global.relocs = agg.relocs;
        Ok(g)
    }

    /// Apply `compiler_set_type_info_flags` calls made by the `#run` that just returned:
    /// the struct's flags are or-ed in and an already built descriptor is rebuilt.
    pub fn apply_type_info_flags(&mut self, span: Span) -> Result<()> {
        for (g, flags) in std::mem::take(&mut self.ct.interp.pending_type_flags) {
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
            let global = &mut self.out.program.globals[g.0 as usize];
            global.init = agg.bytes;
            global.relocs = agg.relocs;
            self.ct
                .interp
                .refresh_global(&self.out.program, g)
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
