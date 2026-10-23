//! Results of a run in the host twin, brought into the target compiler: types matched by
//! declaration, values checked to have the same layout on both sides, data copied.
use super::lower::Operand;
use super::{
    Compiler, HashMap, HashSet, ProcId, Rc, Result, ScopeId, Span, Sym, TypeId, Value, err, ir,
    procs::ProcTarget, scope::Resolved,
};
use crate::source::SourcePath;
use crate::types::{EnumId, ProcType, StructId, TypeKind};

impl Compiler {
    /// Whether a result of the host type `ty` has a counterpart in this compiler: it is checked
    /// before the run, so that a run whose result cannot cross (Code, a distinct type, a type
    /// local to a procedure) stays on this compiler. A layout that differs is still an error.
    pub(super) fn can_import_host_type(
        &mut self,
        twin: &mut Compiler,
        ty: TypeId,
        span: Span,
    ) -> bool {
        if matches!(twin.ty.types.kind(ty), TypeKind::Code) {
            return false;
        }
        let mut importer = Importer {
            target: self,
            host: twin,
            span,
            types: HashMap::default(),
            globals: HashMap::default(),
            checked: HashSet::default(),
        };
        importer.ty(ty).is_ok()
    }

    /// Convert `operands`, the results of a run in `twin`, to this compiler's types and data.
    pub(super) fn import_host_results(
        &mut self,
        twin: &mut Compiler,
        operands: Vec<Operand>,
        span: Span,
    ) -> Result<Vec<Operand>> {
        let mut importer = Importer {
            target: self,
            host: twin,
            span,
            types: HashMap::default(),
            globals: HashMap::default(),
            checked: HashSet::default(),
        };
        operands
            .into_iter()
            .map(|operand| importer.operand(operand))
            .collect()
    }
}

struct Importer<'a> {
    target: &'a mut Compiler,
    host: &'a mut Compiler,
    span: Span,
    types: HashMap<TypeId, TypeId>,
    globals: HashMap<ir::GlobalId, ir::GlobalId>,
    /// Pairs of (host, target) types whose layouts were compared or are being compared.
    checked: HashSet<(TypeId, TypeId)>,
}

impl Importer<'_> {
    fn fail<T>(&self, message: impl Into<String>) -> Result<T> {
        err(self.span, message)
    }

    fn operand(&mut self, operand: Operand) -> Result<Operand> {
        match operand.without_callable() {
            Operand::Void => Ok(Operand::Void),
            Operand::Type(ty) => Ok(Operand::Type(self.ty(ty)?)),
            Operand::Const {
                ty,
                value,
                untyped,
            } => {
                let imported = self.ty(ty)?;
                self.same_layout(ty, imported)?;
                let value = self.value(value)?;
                Ok(Operand::Const {
                    ty: imported,
                    value,
                    untyped,
                })
            }
            _ => self.fail(
                "this #run result is a procedure or another compile-time entity of the host \
                 program and cannot be used by the target program",
            ),
        }
    }

    fn value(&mut self, value: Value) -> Result<Value> {
        match value {
            Value::Int(_)
            | Value::Float(..)
            | Value::Bool(_)
            | Value::String(_)
            | Value::Null
            | Value::Void => Ok(value),
            Value::Type(ty) => Ok(Value::Type(self.ty(ty)?)),
            Value::Bytes(image) => {
                let mut image = (*image).clone();
                for relocation in &mut image.relocs {
                    relocation.target = self.data(relocation.target)?;
                }
                Ok(Value::Bytes(Rc::new(image)))
            }
            Value::Proc(procedure) => Ok(Value::Proc(self.target_proc(procedure)?)),
            Value::Code(_) => self.fail(
                "a #run result that is Code cannot be used by the target program: it names \
                 declarations of the host program",
            ),
        }
    }

    /// Copy the global `target` points at, and what it points at in turn.
    fn data(&mut self, target: ir::RelocTarget) -> Result<ir::RelocTarget> {
        let old = match target {
            ir::RelocTarget::Global(old) => old,
            ir::RelocTarget::Func(_) | ir::RelocTarget::Foreign(_) => {
                return self.procedure_address(target);
            }
        };
        if let Some(&copied) = self.globals.get(&old) {
            return Ok(ir::RelocTarget::Global(copied));
        }
        // A variable of the program is the same variable in the target program.
        if let Some(&(name, span)) = self.host.out.declared_globals.get(&old) {
            let own = self.target_variable(name, span)?;
            self.globals.insert(old, own);
            return Ok(ir::RelocTarget::Global(own));
        }
        let mut global = self.host.out.program.globals[old.0 as usize].clone();
        global.name = "host.run.data".into();
        global.export = None;
        let copied = self.target.out.program.add_global(global.clone());
        self.globals.insert(old, copied);
        for relocation in &mut global.relocs {
            relocation.target = self.data(relocation.target)?;
        }
        self.target.out.program.globals[copied.0 as usize] = global;
        Ok(ir::RelocTarget::Global(copied))
    }

    /// The target's type for the host type `host`.
    fn ty(&mut self, host: TypeId) -> Result<TypeId> {
        if let Some(&known) = self.types.get(&host) {
            return Ok(known);
        }
        let kind = self.host.ty.types.kind(host).clone();
        let imported = match kind {
            TypeKind::Pointer(inner) => {
                let inner = self.ty(inner)?;
                self.target.ty.types.pointer(inner)
            }
            TypeKind::Array {
                elem,
                kind,
            } => {
                let elem = self.ty(elem)?;
                self.target.ty.types.array(elem, kind)
            }
            TypeKind::Proc(procedure) => {
                let params = procedure
                    .params
                    .iter()
                    .map(|&param| self.ty(param))
                    .collect::<Result<_>>()?;
                let returns = procedure
                    .returns
                    .iter()
                    .map(|&ret| self.ty(ret))
                    .collect::<Result<_>>()?;
                self.target
                    .ty
                    .types
                    .intern(TypeKind::Proc(Rc::new(ProcType {
                        params,
                        returns,
                        ..(*procedure).clone()
                    })))
            }
            TypeKind::Struct(id) => self.named_struct(host, id)?,
            TypeKind::Enum(id) => self.named_enum(host, id)?,
            TypeKind::Distinct(_) => {
                return self.fail(
                    "a #run result of a distinct type cannot be used by the target program yet",
                );
            }
            TypeKind::PolyParam
            | TypeKind::PolyStruct {
                ..
            } => {
                return self.fail("a #run result cannot have a polymorphic type");
            }
            other => self.target.ty.types.intern(other),
        };
        self.types.insert(host, imported);
        Ok(imported)
    }

    fn named_struct(&mut self, host_type: TypeId, id: StructId) -> Result<TypeId> {
        let info = self.host.ty.types.structs[id.0 as usize].clone();
        if info.poly_parent.is_some() {
            return self.fail(format!(
                "a #run result of the polymorphic instance `{}` cannot be used by the target \
                 program yet; return its contents instead",
                info.name
            ));
        }
        self.find_declaration(host_type, info.name, info.span, "struct")
    }

    fn named_enum(&mut self, host_type: TypeId, id: EnumId) -> Result<TypeId> {
        let info = self.host.ty.types.enum_info(id).clone();
        self.find_declaration(host_type, info.name, info.span, "enum")
    }

    /// The target's type for the host type `host`, declared as `name` at `span` of the host
    /// program's file: the same declaration, compared by path and position. A declaration the
    /// target keeps in another file of the same directory (an OS-specific copy of the type,
    /// `types.jai` and `windows-types.jai`) counts when it has the same name and layout.
    fn find_declaration(
        &mut self,
        host: TypeId,
        name: Sym,
        span: Span,
        what: &str,
    ) -> Result<TypeId> {
        let Some(file) = self.host.src.sources.try_get(span.file) else {
            return self.fail(format!("the {what} `{name}` has no source location"));
        };
        let path: SourcePath = file.path.clone();
        let directory = path
            .as_file()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf);
        let mut exact = Vec::new();
        let mut siblings = Vec::new();
        for info in &self.target.src.files {
            if info.path == path {
                exact.push(info.scope);
            } else if directory.is_some()
                && info.path.as_file().and_then(std::path::Path::parent) == directory.as_deref()
            {
                siblings.push(info.scope);
            }
        }
        for (scopes, is_exact) in [(exact, true), (siblings, false)] {
            for scope in scopes {
                let Ok(ids) = self.target.lookup(scope, name) else {
                    continue;
                };
                for id in ids {
                    let Ok(Resolved::Const {
                        value: Value::Type(ty),
                        ..
                    }) = self.target.resolve_entity(id)
                    else {
                        continue;
                    };
                    if if is_exact {
                        self.declared_at(ty, &path, span)
                    } else {
                        self.is_counterpart(host, ty)
                    } {
                        return Ok(ty);
                    }
                }
            }
        }
        self.fail(format!(
            "the {what} `{name}` returned by this #run is not declared the same way in the \
             target program (it may be local to a procedure or to an OS-specific branch)"
        ))
    }

    /// The address of the target's own copy of a procedure a result points at.
    fn procedure_address(&mut self, target: ir::RelocTarget) -> Result<ir::RelocTarget> {
        let host = self.host.pr.procs.iter().position(|info| {
            matches!(
                (info.target, target),
                (Some(ProcTarget::Func(a)), ir::RelocTarget::Func(b)) if a == b
            ) || matches!(
                (info.target, target),
                (Some(ProcTarget::Foreign(a)), ir::RelocTarget::Foreign(b)) if a == b
            )
        });
        let Some(host) = host else {
            return self
                .fail("a #run result points at code that is not a procedure of the program");
        };
        let own = self.target_proc(ProcId(host as u32))?;
        Ok(match self.target.proc_func(own, self.span)? {
            ProcTarget::Func(function) => ir::RelocTarget::Func(function),
            ProcTarget::Foreign(foreign) => ir::RelocTarget::Foreign(foreign),
        })
    }

    /// The target's procedure for a procedure of the host program: the same declaration, found
    /// by name and position in the file scope. Procedures made inside procedures and instances
    /// of polymorphic ones have no such declaration.
    fn target_proc(&mut self, host: ProcId) -> Result<ProcId> {
        let info = self.host.proc(host);
        let (name, span, scope) = (info.name, info.span, info.scope);
        let local = info.is_poly || info.bindings.is_some();
        let path = self
            .host
            .src
            .files
            .iter()
            .find(|file| file.scope == scope)
            .map(|file| file.path.clone());
        let (Some(path), false) = (path, local) else {
            return self.fail(format!(
                "a #run result holds the procedure `{name}`, which is local or polymorphic and \
                 has no counterpart in the target program"
            ));
        };
        let scopes: Vec<ScopeId> = self
            .target
            .src
            .files
            .iter()
            .filter(|info| info.path == path)
            .map(|info| info.scope)
            .collect();
        for scope in scopes {
            let Ok(ids) = self.target.lookup(scope, name) else {
                continue;
            };
            for id in ids {
                let found = match self.target.resolve_entity(id) {
                    Ok(Resolved::Proc(procedure)) => vec![procedure],
                    Ok(Resolved::ProcSet(set)) => set,
                    _ => continue,
                };
                for procedure in found {
                    let declared = self.target.proc(procedure).span;
                    if declared.start == span.start && declared.end == span.end {
                        return Ok(procedure);
                    }
                }
            }
        }
        self.fail(format!(
            "a #run result holds the procedure `{name}`, which the target program does not declare \
             the same way"
        ))
    }

    /// The target's global variable declared as `name` at `span` of the host program's file.
    fn target_variable(&mut self, name: Sym, span: Span) -> Result<ir::GlobalId> {
        let path = self
            .host
            .src
            .sources
            .try_get(span.file)
            .map(|file| file.path.clone());
        let scopes: Vec<ScopeId> = self
            .target
            .src
            .files
            .iter()
            .filter(|info| Some(&info.path) == path.as_ref())
            .map(|info| info.scope)
            .collect();
        for scope in scopes {
            let Ok(ids) = self.target.lookup(scope, name) else {
                continue;
            };
            for id in ids {
                if let Ok(Resolved::Global {
                    storage: ir::Storage::Data(global),
                    ..
                }) = self.target.resolve_entity(id)
                {
                    return Ok(global);
                }
            }
        }
        self.fail(format!(
            "a #run result points at the variable `{name}`, which the target program does not              declare the same way"
        ))
    }

    /// Whether `target` is the same kind of type as `host`, with the same name and layout.
    fn is_counterpart(&mut self, host: TypeId, target: TypeId) -> bool {
        let describe = |compiler: &Compiler, ty: TypeId| match compiler.ty.types.kind(ty) {
            TypeKind::Struct(id) => Some((true, compiler.ty.types.structs[id.0 as usize].name)),
            TypeKind::Enum(id) => Some((false, compiler.ty.types.enum_info(*id).name)),
            _ => None,
        };
        if describe(self.host, host) != describe(self.target, target) {
            return false;
        }
        let before = self.checked.clone();
        let same = self.same_layout(host, target).is_ok();
        if !same {
            self.checked = before;
        }
        same
    }

    fn declared_at(&self, ty: TypeId, path: &SourcePath, host_span: Span) -> bool {
        let span = match self.target.ty.types.kind(ty) {
            TypeKind::Struct(id) => self.target.ty.types.structs[id.0 as usize].span,
            TypeKind::Enum(id) => self.target.ty.types.enum_info(*id).span,
            _ => return false,
        };
        span.start == host_span.start
            && span.end == host_span.end
            && self
                .target
                .src
                .sources
                .try_get(span.file)
                .is_some_and(|file| &file.path == path)
    }

    /// Whether a host value of type `host` has the bytes the target expects of `target`.
    fn same_layout(&mut self, host: TypeId, target: TypeId) -> Result<()> {
        if !self.checked.insert((host, target)) {
            return Ok(());
        }
        let span = self.span;
        match (
            self.host.ty.types.kind(host).clone(),
            self.target.ty.types.kind(target).clone(),
        ) {
            (TypeKind::Struct(h), TypeKind::Struct(t)) => {
                let (host_size, target_size) = (
                    self.host.size_of(host, span)?,
                    self.target.size_of(target, span)?,
                );
                let (host_info, target_info) = (
                    self.host.ty.types.structs[h.0 as usize].clone(),
                    self.target.ty.types.structs[t.0 as usize].clone(),
                );
                let same = host_size == target_size
                    && host_info.align == target_info.align
                    && host_info.fields.len() == target_info.fields.len()
                    && host_info
                        .fields
                        .iter()
                        .zip(target_info.fields.iter())
                        .all(|(a, b)| a.name == b.name && a.offset == b.offset);
                if !same {
                    return self.fail(format!(
                        "`{}` has a different layout on the host (size {host_size}) than in the \
                         target program (size {target_size}); return its fields individually",
                        host_info.name
                    ));
                }
                for (a, b) in host_info.fields.iter().zip(target_info.fields.iter()) {
                    self.same_layout(a.ty, b.ty)?;
                }
                Ok(())
            }
            (TypeKind::Enum(h), TypeKind::Enum(t)) => {
                let host_info = self.host.ty.types.enum_info(h).clone();
                let target_info = self.target.ty.types.enum_info(t).clone();
                if host_info.members != target_info.members {
                    return self.fail(format!(
                        "the members of enum `{}` differ between the host and the target program",
                        host_info.name
                    ));
                }
                let base = self.ty(host_info.base)?;
                self.same_layout(host_info.base, base)
            }
            (
                TypeKind::Array {
                    elem: h, ..
                },
                TypeKind::Array {
                    elem: t, ..
                },
            )
            | (TypeKind::Pointer(h), TypeKind::Pointer(t)) => self.same_layout(h, t),
            _ => Ok(()),
        }
    }
}
