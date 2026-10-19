//! Creating struct types and instantiating polymorphic structs.
use super::literals::const_operand;
use super::model::{PolyStruct, StructSource};
use crate::sema::lower::Operand;
use crate::sema::scope::{EntityKind, ScopeKind};
use crate::sema::value::PolyStructId;
use crate::sema::{
    Compiler, Diagnostic, HashMap, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err, scope,
    value,
};
use crate::types::{ArrayKind, LayoutState, StructInfo, TypeKind};

impl Compiler {
    pub fn new_struct_type(
        &mut self,
        name: Sym,
        lit: Rc<ast::StructLit>,
        parent: ScopeId,
        bindings: Vec<(Sym, Value, TypeId)>,
        poly: Option<(PolyStructId, Vec<Value>)>,
    ) -> TypeId {
        let ty = self.types.new_struct(StructInfo {
            name,
            ast: Some(lit.id),
            // A tagged union is a struct: the tag, then the members' union.
            is_union: lit.kind == ast::StructKind::Union && lit.tag.is_none(),
            fields: Rc::from([]),
            size: 0,
            align: 1,
            layout: LayoutState::Pending,
            poly_parent: None,
            poly_args: poly.as_ref().map(|(_, v)| v.clone()).unwrap_or_default(),
            span: lit.span,
        });
        let module = self.scope(parent).module;
        let scope = self.new_scope(ScopeKind::Struct(ty), Some(parent), module, None);
        for (n, v, t) in bindings {
            // A parameter is declared where it is written, so a use goes there.
            let at = lit
                .params
                .iter()
                .filter_map(|p| p.name)
                .find(|p| p.name == n)
                .map_or(lit.span, |p| p.span);
            self.add_const(scope, n, at, v, t);
        }
        let file_scope = scope;
        for stmt in &lit.body {
            match &stmt.kind {
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Var => {
                    for n in &decl.names {
                        self.field_declarations
                            .entry(scope)
                            .or_default()
                            .insert(n.name, decl.clone());
                    }
                }
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Const => {
                    for (index, n) in decl.names.iter().enumerate() {
                        self.add_entity(
                            scope,
                            n.name,
                            n.span,
                            EntityKind::Decl {
                                decl: decl.clone(),
                                index,
                            },
                            true,
                        );
                    }
                }
                ast::StmtKind::StaticIf(_) => {
                    self.push_pending(
                        scope,
                        scope::Pending {
                            stmt: stmt.clone(),
                            exported: true,
                            file_scope,
                            state: scope::PendingState::Waiting,
                        },
                    );
                }
                _ => {}
            }
        }
        let s = self.types.as_struct(ty).unwrap();
        self.struct_asts.insert(
            s,
            StructSource {
                lit,
                scope,
                extra: Vec::new(),
                inits: Vec::new(),
            },
        );
        ty
    }

    /// The `Type` value standing for a polymorphic struct that has no arguments yet.
    pub fn poly_struct_type(&mut self, ps: PolyStructId) -> TypeId {
        let name = self.poly_structs[ps.0 as usize].name;
        self.types.intern(TypeKind::PolyStruct {
            id: ps,
            name,
        })
    }

    pub fn new_poly_struct(
        &mut self,
        name: Sym,
        lit: Rc<ast::StructLit>,
        scope: ScopeId,
    ) -> PolyStructId {
        self.poly_structs.push(PolyStruct {
            name,
            lit,
            scope,
            instances: HashMap::default(),
            baked: Vec::new(),
            origin: None,
        });
        PolyStructId(self.poly_structs.len() as u32 - 1)
    }

    /// A bare generic name denotes its default instance when every parameter has a default.
    /// Keep the source type available separately for generic patterns and reflection.
    pub(in crate::sema) fn poly_struct_has_defaults(&self, ps: PolyStructId) -> bool {
        self.poly_structs[ps.0 as usize]
            .lit
            .params
            .iter()
            .all(|p| p.default.is_some())
    }

    /// `Table(int, string)`: instantiate a polymorphic struct.
    /// The constant value of a polymorphic struct argument. Every declared parameter
    /// type is enforced before binding, including procedure specialization and scalar
    /// narrowing. Untyped parameters retain their compile-time value.
    pub(in crate::sema) fn struct_arg_value(
        &mut self,
        scope: ScopeId,
        op: Operand,
        ty: TypeId,
        span: Span,
    ) -> Result<Value> {
        if ty == TypeId::TYPE {
            return self.operand_as_type(op, span).map(Value::Type);
        }
        if ty != TypeId::VOID {
            return self.const_value_of_operand(scope, op, ty, span);
        }
        match op {
            Operand::Type(t) => Ok(Value::Type(t)),
            Operand::Const {
                value, ..
            } => Ok(value),
            Operand::Procs(p) if p.len() == 1 => Ok(Value::Proc(p[0])),
            other => err(
                span,
                format!("expected a constant, found {}", self.describe(&other)),
            ),
        }
    }

    /// The declared type of a polymorphic struct's value parameter (by position or name), when
    /// it can be evaluated outside the parameter scope; used to type `.MEMBER` arguments.
    pub fn poly_struct_param_type(
        &mut self,
        ps: PolyStructId,
        index: usize,
        name: Option<Sym>,
    ) -> Option<TypeId> {
        let (lit, scope) = {
            let p = &self.poly_structs[ps.0 as usize];
            (p.lit.clone(), p.scope)
        };
        let param = match name {
            Some(n) => lit
                .params
                .iter()
                .find(|p| p.name.map(|i| i.name) == Some(n))?,
            None => lit.params.get(index)?,
        };
        let ty = param.ty.as_ref()?;
        self.eval_type(scope, ty).ok()
    }

    pub fn instantiate_struct(
        &mut self,
        ps: PolyStructId,
        args: Vec<(Option<Sym>, Operand)>,
        span: Span,
    ) -> Result<TypeId> {
        let (name, lit, def_scope) = {
            let p = &self.poly_structs[ps.0 as usize];
            (p.name, p.lit.clone(), p.scope)
        };
        if let Some(origin) = self.poly_structs[ps.0 as usize].origin {
            // Name the arguments for the origin and add the baked ones.
            let mut named = Vec::new();
            for (i, (n, v)) in args.into_iter().enumerate() {
                let n = n.or_else(|| {
                    lit.params
                        .get(i)
                        .filter(|p| !p.variadic)
                        .and_then(|p| p.name)
                        .map(|p| p.name)
                });
                named.push((n, v));
            }
            for (n, value, ty) in self.poly_structs[ps.0 as usize].baked.clone() {
                named.push((Some(n), const_operand(value, ty)));
            }
            return self.instantiate_struct(origin, named, span);
        }
        let module = self.scope(def_scope).module;
        let param_scope = self.new_scope(ScopeKind::StructParams, Some(def_scope), module, None);
        let mut values: Vec<Option<Operand>> = vec![None; lit.params.len()];
        // `struct(types: ..Type)`: the positional arguments from the variadic parameter on
        // become one constant `[] Type`.
        let variadic = lit.params.iter().position(|p| p.variadic);
        let mut pack: Vec<Operand> = Vec::new();
        let mut next = 0;
        for (n, v) in args {
            if n.is_none() && variadic.is_some_and(|vi| next >= vi) {
                pack.push(v);
                continue;
            }
            let index = if let Some(n) = n {
                lit.params
                    .iter()
                    .position(|p| p.name.map(|i| i.name) == Some(n))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::error(
                            span,
                            format!("`{name}` has no parameter `{n}`"),
                        ))
                    })?
            } else {
                let i = next;
                next += 1;
                i
            };
            if index >= values.len() {
                return err(span, format!("too many arguments for `{name}`"));
            }
            values[index] = Some(v);
        }
        let mut bindings = self.poly_structs[ps.0 as usize].baked.clone();
        let mut key = Vec::new();
        for (i, p) in lit.params.iter().enumerate() {
            let pname = p.name.map_or_else(|| Sym::intern("_"), |n| n.name);
            // `x: $T`: the argument's type binds `T` (also `[$N] $T` and other patterns).
            if let Some(t) = &p.ty
                && crate::sema::procs::has_poly(t)
                && let Some(op) = &values[i]
            {
                let arg_ty = match op {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(*ty, value),
                    Operand::Procs(procs) if procs.len() == 1 => self.proc_type(procs[0], span)?,
                    Operand::Type(_) => TypeId::TYPE,
                    other => other.ty(),
                };
                let mut found = Vec::new();
                let represented = match op.const_value() {
                    Some(Value::Type(t)) => Some(t),
                    _ => None,
                };
                self.match_pattern_value(t, arg_ty, &mut found, param_scope, represented)?;
                for (n, v, vty) in found {
                    self.add_const(param_scope, n, p.span, v.clone(), vty);
                    key.push(v.clone());
                    bindings.push((n, v, vty));
                }
            }
            let ty = match &p.ty {
                Some(t) => self.eval_type(param_scope, t)?,
                None => TypeId::VOID,
            };
            if Some(i) == variadic {
                let elem = if ty == TypeId::VOID {
                    TypeId::TYPE
                } else {
                    ty
                };
                let mut items = std::mem::take(&mut pack);
                if let Some(op) = values[i].take() {
                    items.insert(0, op);
                }
                let count = items.len() as u64;
                let esize = self.size_of(elem, p.span)?;
                let mut array =
                    value::Aggregate::zeroed(esize.saturating_mul(items.len() as u64), span)?;
                for (k, op) in items.into_iter().enumerate() {
                    let v = self.struct_arg_value(param_scope, op, elem, p.span)?;
                    let v = match (v, self.types.is_float(elem)) {
                        (Value::Int(i), true) => Value::float(i as f64),
                        (v, _) => v,
                    };
                    self.write_value(&mut array, k as u64 * esize, &v, elem, p.span)?;
                }
                let value = self.const_pack(elem, array, count, p.span)?;
                let ty = self.types.array(elem, ArrayKind::View);
                self.add_const(param_scope, pname, p.span, value.clone(), ty);
                key.push(crate::sema::instance_key_value(&self.types, &value, ty).value);
                bindings.push((pname, value, ty));
                continue;
            }
            let argument = match values[i].take() {
                Some(op) => op,
                None => match &p.default {
                    Some(d) => {
                        // The declared type lets `.FIRST` defaults find their enum.
                        let expected = (ty != TypeId::VOID).then_some(ty);
                        self.eval_const_or_run(param_scope, d, expected)?
                    }
                    None => {
                        return err(
                            span,
                            format!("missing argument for parameter `{pname}` of `{name}`"),
                        );
                    }
                },
            };
            let argument_ty = match &argument {
                Operand::Const {
                    ty,
                    value,
                    untyped: true,
                } => self.default_untyped(*ty, value),
                other => other.ty(),
            };
            let value = self.struct_arg_value(param_scope, argument, ty, p.span)?;
            let value = match (value, self.types.is_float(ty)) {
                (Value::Int(i), true) => Value::float(i as f64),
                (v, _) => v,
            };
            let ty = if ty == TypeId::VOID && argument_ty != TypeId::COMPILE_TIME {
                argument_ty
            } else if ty == TypeId::VOID {
                self.type_of_value(&value)
            } else {
                ty
            };
            self.add_const(param_scope, pname, p.span, value.clone(), ty);
            key.push(value.clone());
            bindings.push((pname, value, ty));
        }
        if let Some(block) = &lit.modify {
            let baked = self.poly_structs[ps.0 as usize].baked.len();
            let params = bindings.split_off(baked);
            let modified = self.run_struct_modify(def_scope, name, block, params, span)?;
            key = modified
                .iter()
                .map(|(_, v, t)| crate::sema::instance_key_value(&self.types, v, *t).value)
                .collect();
            bindings.extend(modified);
        }
        let cache_key = bindings
            .iter()
            .map(|(_, value, ty)| crate::sema::instance_key_value(&self.types, value, *ty))
            .collect::<Vec<_>>();
        if let Some(&t) = self.poly_structs[ps.0 as usize].instances.get(&cache_key) {
            return Ok(t);
        }
        if self.poly_structs[ps.0 as usize].instances.len() >= crate::sema::MAX_INSTANCES {
            return err(span, crate::sema::too_many_instances(name));
        }
        let t = self.new_struct_type(name, lit, def_scope, bindings, Some((ps, key.clone())));
        self.poly_structs[ps.0 as usize]
            .instances
            .insert(cache_key, t);
        Ok(t)
    }
}
