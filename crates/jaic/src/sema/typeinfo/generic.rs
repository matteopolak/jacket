//! Type information of generic structs and their parameters.
use super::tag;
use super::views::View;
use crate::diagnostics::Code;
use crate::sema::scope::{EntityKind, ScopeKind};
use crate::sema::value::Aggregate;
use crate::sema::{
    Compiler, Diagnostic, PreludeType, Result, ScopeId, Span, Sym, TypeId, Value, ast, ir, structs,
};

impl Compiler {
    /// The source descriptor of a generic struct: one per generic struct
    /// (keyed by its syntax), shared by every instance's `polymorph_source_struct` and by
    /// `type_info` of the bare generic struct.
    pub fn generic_struct_info_global(
        &mut self,
        key: ast::AstId,
        name: Sym,
        desc: TypeId,
        span: Span,
    ) -> Result<ir::GlobalId> {
        let cached = self.ty.generic_struct_infos.get(&key).copied();
        if let Some(g) = cached {
            return Ok(g);
        }
        let size = self.size_of(desc, span)?;
        let mut generic = Aggregate {
            bytes: vec![0; size as usize],
            relocs: Vec::new(),
        };
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
        let source = self
            .ty
            .poly_structs
            .iter()
            .find(|p| p.lit.id == key && p.origin.is_none())
            .ok_or_else(|| {
                Box::new(Diagnostic::new(
                    Code::TypeInfoGenericDeclarationMetadata,
                    span,
                    "generic declaration metadata is unavailable",
                ))
            })?;
        {
            let (lit, scope) = (source.lit.clone(), source.scope);
            let mut storage = Aggregate {
                bytes: Vec::new(),
                relocs: Vec::new(),
            };
            self.generic_parameters(&mut generic, desc, &lit, (scope, &[]), &mut storage, span)?;
            let len = storage.bytes.len();
            self.set_view(
                &mut generic,
                desc,
                "constant_storage",
                View::of(len, storage),
                span,
            )?;
        }
        let align = self.align_of(desc, span)?;
        let g = self.out.program.add_global(ir::Global {
            name: format!("type_info.generic.{}", name.as_str()),
            size,
            align,
            init: generic.bytes,
            relocs: generic.relocs,
            read_only: true,
            export: None,
        });
        self.ty.generic_struct_infos.insert(key, g);
        Ok(g)
    }

    /// Declared parameter defaults, evaluated with preceding actual bindings for
    /// an instance, or preceding declared defaults for the shared source record.
    /// A missing default has offset -1, distinct from a stored zero/null value.
    fn generic_parameters(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        lit: &ast::StructLit,
        (parent, actual): (ScopeId, &[(Sym, Value, TypeId)]),
        storage: &mut Aggregate,
        span: Span,
    ) -> Result<()> {
        let module = self.scope(parent).module;
        let scope = self.new_scope(ScopeKind::StructParams, Some(parent), module, None);
        // Inferred pattern bindings and baked arguments are not ordinary
        // declared parameters, but their types/default expressions can name them.
        for (name, value, ty) in actual.iter().filter(|(name, _, _)| {
            !lit.params
                .iter()
                .any(|p| p.name.is_some_and(|n| n.name == *name))
        }) {
            self.add_const(scope, *name, span, value.clone(), *ty);
        }
        let member_ty = self.preload_type(PreludeType::TypeInfoStructMember, span)?;
        let msize = self.size_of(member_ty, span)?;
        let malign = self.align_of(member_ty, span)?;
        let mut members = Aggregate {
            bytes: Vec::new(),
            relocs: Vec::new(),
        };
        for parameter in &lit.params {
            let name = parameter.name.map_or_else(|| Sym::intern("_"), |n| n.name);
            if actual.is_empty()
                && let Some(pattern) = &parameter.ty
            {
                let mut names = Vec::new();
                crate::sema::procs::for_poly_binders(pattern, &mut |name| names.push(name));
                for name in names {
                    if !self.scope(scope).names.contains_key(&name) {
                        self.add_entity(
                            scope,
                            name,
                            parameter.span,
                            EntityKind::UnboundGenericParameter,
                            false,
                        );
                    }
                }
            }
            let declared = match &parameter.ty {
                Some(ty) => match self.eval_type(scope, ty) {
                    Ok(ty) => Some(ty),
                    Err(e) if actual.is_empty() && e.code == Code::UnboundGenericParameter => None,
                    Err(e) => return Err(e),
                },
                None => None,
            };
            let default = match parameter
                .default
                .as_ref()
                .filter(|_| parameter.ty.is_none() || declared.is_some())
            {
                Some(expr) => match self.eval_const_or_run(scope, expr, declared) {
                    Ok(op) => {
                        let value = self.struct_arg_value(
                            scope,
                            op,
                            declared.unwrap_or(TypeId::VOID),
                            expr.span,
                        )?;
                        Some(
                            match (value, declared.is_some_and(|t| self.ty.types.is_float(t))) {
                                (Value::Int(i), true) => Value::float(i as f64),
                                (value, _) => value,
                            },
                        )
                    }
                    Err(e) if actual.is_empty() && e.code == Code::UnboundGenericParameter => None,
                    Err(e) => return Err(e),
                },
                None => None,
            };
            let ty = declared
                .or_else(|| default.as_ref().map(|v| self.type_of_value(v)))
                .or_else(|| {
                    actual
                        .iter()
                        .find(|(n, _, _)| *n == name)
                        .map(|(_, _, ty)| *ty)
                });
            let member = self.parameter_member(
                (member_ty, msize),
                name,
                ty,
                default.as_ref(),
                storage,
                parameter.span,
            )?;
            let at = members.bytes.len();
            members.bytes.resize(at + msize as usize, 0);
            structs::write_agg(&mut members, at as u64, &member);
            if let Some((_, value, ty)) = actual.iter().find(|(n, _, _)| *n == name) {
                self.add_const(scope, name, parameter.span, value.clone(), *ty);
            } else if let Some(value) = default {
                self.add_const(
                    scope,
                    name,
                    parameter.span,
                    value,
                    ty.expect("default type"),
                );
            } else {
                self.add_entity(
                    scope,
                    name,
                    parameter.span,
                    EntityKind::UnboundGenericParameter,
                    false,
                );
            }
        }
        self.set_view(
            agg,
            desc,
            "parameters",
            View::new(lit.params.len(), members, malign),
            span,
        )
    }

    fn parameter_member(
        &mut self,
        (member_ty, msize): (TypeId, u64),
        name: Sym,
        ty: Option<TypeId>,
        value: Option<&Value>,
        storage: &mut Aggregate,
        span: Span,
    ) -> Result<Aggregate> {
        let mut member = Aggregate {
            bytes: vec![0; msize as usize],
            relocs: Vec::new(),
        };
        let mut offset = -1;
        if let (Some(ty), Some(value)) = (ty, value) {
            let size = self.size_of(ty, span)?;
            let align = self.align_of(ty, span)?;
            let at = storage.bytes.len().next_multiple_of(align.max(1) as usize);
            storage.bytes.resize(at + size as usize, 0);
            if let Value::Type(t) = value {
                let global = self.type_info_global(*t, span)?;
                storage.relocs.push(ir::Reloc {
                    offset: at as u64,
                    target: ir::RelocTarget::Global(global),
                    addend: 0,
                });
            } else {
                self.write_value(storage, at as u64, value, ty, span)?;
            }
            offset = at as i128;
        }
        self.set_field(
            &mut member,
            member_ty,
            "name",
            Value::String(name.as_str().as_bytes().into()),
            span,
        )?;
        if let Some(ty) = ty {
            self.set_info_ptr(&mut member, member_ty, "type", ty, span)?;
        }
        self.set_field(&mut member, member_ty, "flags", Value::Int(1), span)?;
        self.set_field(
            &mut member,
            member_ty,
            "offset_in_bytes",
            Value::Int(-1),
            span,
        )?;
        self.set_field(
            &mut member,
            member_ty,
            "offset_into_constant_storage",
            Value::Int(offset),
            span,
        )?;
        Ok(member)
    }

    /// `specified_parameters` and `polymorph_source_struct` of a polymorphic struct instance's
    /// runtime descriptor. The arguments' values go to `storage` (`constant_storage`).
    pub(super) fn poly_struct_info(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        s: crate::types::StructId,
        storage: &mut Aggregate,
        span: Span,
    ) -> Result<()> {
        let member_ty = self.preload_type(PreludeType::TypeInfoStructMember, span)?;
        let msize = self.size_of(member_ty, span)?;
        let malign = self.align_of(member_ty, span)?;
        let mut members = Aggregate {
            bytes: Vec::new(),
            relocs: Vec::new(),
        };
        let actual = self.poly_struct_bindings(s);
        if let Some(source) = self.ty.struct_asts.get(&s) {
            let (lit, scope) = (source.lit.clone(), source.scope);
            let parent = self.scope(scope).parent.unwrap_or(scope);
            self.generic_parameters(agg, desc, &lit, (parent, &actual), storage, span)?;
        }
        for (name, value, mut ty) in actual.iter().cloned() {
            if self.size_of(ty, span).is_err() {
                ty = self.type_of_value(&value); // Untyped literal arguments.
            }
            let m = self.parameter_member(
                (member_ty, msize),
                name,
                Some(ty),
                Some(&value),
                storage,
                span,
            )?;
            let at = members.bytes.len() as u64;
            members.bytes.resize(at as usize + msize as usize, 0);
            structs::write_agg(&mut members, at, &m);
        }
        self.set_view(
            agg,
            desc,
            "specified_parameters",
            View::new(actual.len(), members, malign),
            span,
        )?;
        // The shared generic declaration descriptor preserves source identity.
        let generic_key = self
            .ty
            .struct_asts
            .get(&s)
            .map(|src| src.lit.id)
            .ok_or_else(|| {
                Box::new(Diagnostic::new(
                    Code::TypeInfoGenericInstanceDeclaration,
                    span,
                    "generic instance declaration metadata is unavailable",
                ))
            })?;
        let name = self.ty.types.struct_info(s).name;
        let g = self.generic_struct_info_global(generic_key, name, desc, span)?;
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
}
