//! Runtime type information: one read-only `Type_Info_*` descriptor global
//! per type, laid out with Preload's reflection structs. A `Type` value at
//! runtime is the address of its descriptor.
mod build;
mod constants;
mod generic;
mod tagged;
mod views;

use super::{Compiler, Diagnostic, PreludeType, Rc, Result, Span, Sym, TypeId, Value, err, ir};
use crate::diagnostics::Code;
use crate::sema::TypeState;
use crate::types::{LayoutState, TypeKind};

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

/// Which deferred descriptors `flush_deferred_infos` builds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema) enum InfoFlush {
    /// Those that can be built now, after a struct finished its layout.
    AfterLayout,
    /// All of them; a failure is an error.
    End,
}

impl Compiler {
    /// The Preload struct describing `ty` (`Type_Info_Struct`, `Type_Info_Integer`...).
    pub fn type_info_struct_type(&mut self, ty: TypeId, span: Span) -> Result<TypeId> {
        let name = descriptor_of(self.ty.types.kind(ty)).0;
        self.preload_type(name, span)
    }

    pub fn type_info_global(&mut self, ty: TypeId, span: Span) -> Result<ir::GlobalId> {
        if let Some(&g) = self.ty.infos.get(&ty) {
            if !self.ty.in_snapshot {
                self.ty.snapshot_infos.remove(&ty);
            }
            return Ok(g);
        }
        let desc = self.type_info_struct_type(ty, span)?;
        let size = self.size_of(desc, span)?;
        let align = self.align_of(desc, span)?;
        // Sizing the descriptor type may have described this type already.
        if let Some(&g) = self.ty.infos.get(&ty) {
            return Ok(g);
        }
        // A descriptor whose build failed before keeps its global, so code lowered since
        // (a self-reference) points at the one that is filled in now.
        let rebuilt = self.ty.failed_infos.remove(&ty);
        let g = match rebuilt {
            Some(g) => g,
            None => self.out.program.add_global(ir::Global {
                name: format!("type_info.{}", self.ty.types.full_name(ty)),
                size,
                align,
                init: Vec::new(),
                relocs: Vec::new(),
                read_only: true,
                export: None,
            }),
        };
        // Register first: descriptors may refer to themselves.
        self.ty.infos.insert(ty, g);
        if let Some(s) = self.ty.types.as_struct(ty) {
            let span = self.ty.types.struct_info(s).span;
            if (span.file.0 as usize) < self.src.sources.len() {
                let source = self.src.sources.get(span.file);
                let (line, col) = source.line_col(span.start);
                let path: Rc<str> = source.path.to_string().into();
                self.ct
                    .interp
                    .struct_locations
                    .insert(g, (path, i64::from(line), i64::from(col)));
            }
        }
        self.ty.building_infos.push((ty, g));
        let built = self.build_type_info(ty, desc, size, span);
        self.ty.building_infos.pop();
        let agg = match built {
            Ok(agg) => agg,
            Err(e) => {
                // Unregister it: a later request (a retried body) must build it again and
                // report the error, not get an empty descriptor from the cache.
                self.ty.infos.remove(&ty);
                self.ty.failed_infos.insert(ty, g);
                if self.ty.defer_infos {
                    self.ty.deferred_infos.push(ty);
                    return Ok(g);
                }
                return Err(e);
            }
        };
        let global = &mut self.out.program.globals[g.0 as usize];
        global.init = agg.bytes;
        global.relocs = agg.relocs;
        // Code that ran since the global was handed out saw it empty.
        if rebuilt.is_some() {
            let _ = self.ct.interp.refresh_global(&self.out.program, g);
        }
        Ok(g)
    }

    /// The descriptor a pointer refers to. A struct still being laid out gets its global now
    /// and its contents once the layout is done.
    pub(super) fn type_info_global_later(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<ir::GlobalId> {
        let unfinished = self
            .ty
            .types
            .as_struct(ty)
            .is_some_and(|s| self.ty.types.struct_info(s).layout != LayoutState::Done);
        if !unfinished || self.ty.infos.contains_key(&ty) {
            return self.type_info_global(ty, span);
        }
        if let Some(&g) = self.ty.failed_infos.get(&ty) {
            return Ok(g);
        }
        let desc = self.type_info_struct_type(ty, span)?;
        let (size, align) = (self.size_of(desc, span)?, self.align_of(desc, span)?);
        let g = self.out.program.add_global(ir::Global {
            name: format!("type_info.{}", self.ty.types.full_name(ty)),
            size,
            align,
            init: Vec::new(),
            relocs: Vec::new(),
            read_only: true,
            export: None,
        });
        self.ty.failed_infos.insert(ty, g);
        self.ty.deferred_infos.push(ty);
        Ok(g)
    }

    /// Build the descriptors `type_info_global` handed out before they could be built.
    pub(in crate::sema) fn flush_deferred_infos(
        &mut self,
        span: Span,
        mode: InfoFlush,
    ) -> Result<()> {
        let outer = (self.ty.defer_infos, self.ty.strict_sizes);
        (self.ty.defer_infos, self.ty.strict_sizes) = (false, true);
        let mut result = Ok(());
        while result.is_ok() && !self.ty.deferred_infos.is_empty() {
            let queue = std::mem::take(&mut self.ty.deferred_infos);
            let mut waiting = Vec::new();
            for (at, &entry) in queue.iter().enumerate() {
                if self.ty.infos.contains_key(&entry) {
                    continue;
                }
                match self.type_info_global(entry, span) {
                    Ok(_) => {}
                    Err(_) if mode == InfoFlush::AfterLayout => waiting.push(entry),
                    Err(e) => {
                        // Still queued, so the failure shows again until the type is fixed.
                        waiting.extend_from_slice(&queue[at..]);
                        result = Err(e);
                        break;
                    }
                }
            }
            let progressed = waiting.len() < queue.len();
            self.ty.deferred_infos.splice(0..0, waiting);
            if mode != InfoFlush::End || !progressed {
                break;
            }
        }
        (self.ty.defer_infos, self.ty.strict_sizes) = outer;
        result
    }

    /// Build the descriptors of the variadic type arguments of `owner`, which its `#insert`
    /// reads. A case that needs `owner` in return (or is being described while `owner` is
    /// laid out) is a cycle.
    pub(in crate::sema) fn build_variadic_cases(
        &mut self,
        owner: TypeId,
        span: Span,
    ) -> Result<()> {
        let Some(cases) = self.ty.variadic_cases.get(&owner).cloned() else {
            return Ok(());
        };
        let cycle = |this: &Self| {
            let name = this.ty.types.name(owner);
            err(
                Code::CircularDependency,
                span,
                format!("circular dependency while resolving `{name}`"),
            )
        };
        if self.ty.cases_building.contains(&owner) {
            return cycle(self);
        }
        self.ty.cases_building.push(owner);
        let outer = (self.ty.defer_infos, self.ty.strict_sizes);
        (self.ty.defer_infos, self.ty.strict_sizes) = (false, true);
        let mut result = Ok(());
        for case in cases {
            if self.ty.building_infos.iter().any(|&(ty, _)| ty == case) {
                result = cycle(self);
                break;
            }
            if let Err(e) = self.type_info_global(case, span) {
                result = Err(e);
                break;
            }
        }
        (self.ty.defer_infos, self.ty.strict_sizes) = outer;
        self.ty.cases_building.pop();
        result
    }

    /// Apply `compiler_set_type_info_flags` calls made by the `#run` that just returned:
    /// the struct's flags are or-ed in and an already built descriptor is rebuilt.
    pub fn apply_type_info_flags(&mut self, span: Span) -> Result<()> {
        for (g, flags) in std::mem::take(&mut self.ct.interp.pending_type_flags) {
            let Some(ty) = self.ty.type_from_info_global(g) else {
                continue;
            };
            if !matches!(self.ty.types.kind(ty), TypeKind::Struct(_)) {
                continue;
            }
            *self.ty.info_flags.entry(ty).or_insert(0) |= flags;
            let desc = self.type_info_struct_type(ty, span)?;
            let size = self.size_of(desc, span)?;
            let agg = self.build_type_info(ty, desc, size, span)?;
            let global = &mut self.out.program.globals[g.0 as usize];
            global.init = agg.bytes;
            global.relocs = agg.relocs;
            self.ct
                .interp
                .refresh_global(&self.out.program, g)
                .map_err(|t| {
                    Box::new(Diagnostic::new(
                        Code::CompileTimeTrap,
                        span,
                        t.into_message(),
                    ))
                })?;
        }
        Ok(())
    }
}

impl TypeState {
    /// Map a type-info descriptor address back to its type (compile-time reads).
    pub fn type_from_info_global(&self, g: ir::GlobalId) -> Option<TypeId> {
        self.infos.iter().find(|(_, v)| **v == g).map(|(&t, _)| t)
    }
}
