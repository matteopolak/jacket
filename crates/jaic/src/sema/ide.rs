//! Facts for editors. With `Compiler::ide` set, checking records what each identifier of the
//! user's files names and its type, and the extent of block and procedure scopes. The queries
//! below answer hover (`ide_hover`), completion (`ide_visible`, `ide_members`) and member
//! receivers (`ide_receiver`) from those facts and the compiler's scopes.
use super::lower::Operand;
use super::scope::{EntityId, EntityKind, EntityState, Found, Resolved, ScopeId, ScopeKind};
use super::value::ProcId;
use super::*;
use crate::types::{ArrayKind, StructId, TypeKind};

/// Lengths of `IdeFacts::refs`, `Compiler::entities` and `Compiler::procs` at a point.
#[derive(Clone, Copy, Debug)]
pub struct SketchMark {
    pub refs: usize,
    pub entities: usize,
    pub procs: usize,
}

#[derive(Default)]
struct RefIndex {
    /// How many of `refs` are in the index. References are only pushed, or truncated away by
    /// a sketch, so the index grows with them and starts over when they shrink.
    built: usize,
    /// Per file: indices into `refs` of spans up to `SHORT_REF` long, sorted by start, and of
    /// the longer ones, which are few and checked one by one.
    files: HashMap<FileId, (Vec<u32>, Vec<u32>)>,
}

/// `member_uses` by the file of the use, sorted by start, with the longest use's length.
#[derive(Default)]
struct MemberIndex {
    /// How many of `member_uses` are in the index: uses are only added, so a different count
    /// means it is rebuilt.
    built: usize,
    files: HashMap<FileId, (Vec<(Span, Span)>, u32)>,
}

/// Spans up to this many bytes are found by binary search; a lookup walks back at most this far.
const SHORT_REF: u32 = 64;

#[derive(Default)]
pub struct IdeFacts {
    /// Files whose path starts with one of these are recorded.
    roots: Vec<String>,
    wanted: HashMap<FileId, bool>,
    /// The exact body associated with an IDE-final lenient failure. Replacing a
    /// procedure's AST invalidates that failure even without another source load.
    pub(super) failed_body_lits: HashMap<ProcId, Rc<ast::ProcLit>>,
    pub refs: Vec<IdeRef>,
    /// Static filter selectors may denote an entire exported overload set.
    filter_targets: HashMap<Span, Vec<EntityId>>,
    pub scopes: Vec<(Span, ScopeId)>,
    /// What `#insert`, `#run`, `#if` and macro calls produced (see `ide_meta`).
    pub expansions: Vec<super::ide_meta::IdeExpansion>,
    /// Calls and the procedure and parameters they resolved to.
    pub calls: Vec<super::ide_meta::IdeCall>,
    pub(super) call_spans: HashSet<Span>,
    /// Actual custom-iteration selections per keyword, across real instantiations.
    /// `None` means an ambiguous selection; sketches never populate this map.
    pub(super) for_expansions: HashMap<Span, Option<Vec<ProcId>>>,
    /// The compile-time output channel, to tell what each `#run` printed.
    pub output: Option<Rc<std::cell::RefCell<crate::interp::SandboxHost>>>,
    /// Entity the named expression being checked resolved to (set by identifier
    /// lookup or explicit module-member lookup, after resolving its operand).
    pub(super) last_entity: Option<(Span, EntityId)>,
    /// Field the identifier being checked reached through a `using` (set by `check_ident`).
    pub(super) last_using_member: Option<Sym>,
    /// Names of a type that is neither a struct nor an enum (a polymorph's `T`, an alias of a
    /// builtin): the constant each resolved to, for go to definition.
    pub(super) type_names: HashMap<Span, EntityId>,
    /// Procedure type parameter names already recorded.
    pub(super) param_names: HashSet<Span>,
    /// Where `ide_sketch_polymorphs` began: the refs, entities and procedures from there on
    /// come from sketches with stand-in types, which lints skip.
    pub sketch_from: Option<SketchMark>,
    /// The stand-in types of sketches: anything may be done with a value of one (see
    /// `ide_sketch`).
    pub(super) stand_ins: HashSet<TypeId>,
    /// The stand-in for member `name` of stand-in type `t`.
    pub(super) stand_in_members: HashMap<(TypeId, Sym), TypeId>,
    /// Macros (and instances of polymorphic ones) that were expanded somewhere.
    pub(super) expanded_macros: HashSet<ProcId>,
    /// The sketch's own instance is being made, which binds stand-ins.
    pub(super) sketch_root: bool,
    /// A sketch is being checked: an expression that fails on a stand-in becomes one.
    pub(super) sketching: bool,
    /// The type of each expression the current sketch checked.
    pub(super) sketch_types: HashMap<Span, TypeId>,
    /// Each expression the current sketch began to check, failed or not.
    pub(super) sketch_tried: HashSet<Span>,
    /// The code an `#if` on `OS` or `CPU` did not take: another target's compile has its facts.
    pub skipped: Vec<Span>,
    /// `refs` by file, sorted by start, for position lookups (`refs_at`). Built on first use
    /// and again whenever `refs` has changed since (its length and buffer are the key).
    by_position: std::cell::RefCell<RefIndex>,
    /// Lint mode (`jailint`): also record the facts below.
    pub lint: bool,
    /// Type of each checked expression of the recorded files, by span.
    pub exprs: HashMap<Span, IdeExprFact>,
    /// Each `cast(T) x` / `xx x` of the recorded files whose target was known, by span.
    pub casts: HashMap<Span, IdeCast>,
    /// Every entity a name resolved to, in any file (a macro body elsewhere can use a caller's
    /// local through a backtick name), and the variables a `using` member reached.
    pub used: HashSet<EntityId>,
    /// Imports (scope, index into its `imports`) a name lookup found something through.
    pub used_imports: HashSet<(ScopeId, usize)>,
    /// The value type left of each `a.b` of the recorded files, by the span of `b` (`None`
    /// when checking it several times gave different types).
    pub receivers: HashMap<Span, Option<TypeId>>,
    /// The struct each `Poly(args)` of the recorded files instantiated, by the span of `Poly`.
    pub instances: HashMap<Span, Option<TypeId>>,
    /// The struct each field name of the recorded files was laid out in, by the name's span
    /// (`None` when several structs share it: the instances of a polymorphic struct).
    pub fields: HashMap<Span, Option<IdeFieldSite>>,
    /// Every use of a struct field or enum member of the recorded files, as (the member's
    /// name, the name where it is declared). Declarations map to themselves.
    pub member_uses: HashSet<(Span, Span)>,
    /// `member_uses` by position (`member_use_at`), built on first use after they change.
    members_by_position: std::cell::RefCell<MemberIndex>,
    /// Errors of procedure bodies that failed while the program compiled, beyond the one the
    /// compile stopped at.
    pub errors: Vec<Diagnostic>,
    /// Original lexical/syntax errors. Bodies covering them may supply editor facts,
    /// but must not be executed by compile-time code after malformed statements vanish.
    pub parse_errors: Vec<Diagnostic>,
    /// Where each enum member's name is declared.
    pub(super) enum_decls: HashMap<(crate::types::EnumId, Sym), Span>,
    /// The types an inferred member (`.NAME`) of the recorded files was expected to have, by
    /// the name's span. Recorded whether or not the name exists, for completion.
    pub(super) inferred_expected: HashMap<Span, Vec<TypeId>>,
}

/// Where a declared field lives: `field` of `declaring`, at `offset` in `owner` (a struct
/// that includes `declaring` as an anonymous `struct { ... }` member, or `declaring` itself).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdeFieldSite {
    pub owner: StructId,
    pub declaring: StructId,
    pub field: usize,
    pub offset: u64,
}

/// Memory layout for a hover, from the compiler's own layout: a type's size and alignment
/// (with `padding` the bytes no field covers), or a field's offset, size and alignment (with
/// `padding` the bytes skipped before it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdeLayout {
    pub offset: Option<u64>,
    pub size: u64,
    pub align: u64,
    pub padding: u64,
}

/// A hover: the span it covers, a declaration-like description and the layout, if any.
#[derive(Clone, Debug)]
pub struct IdeHover {
    pub span: Span,
    pub text: String,
    pub layout: Option<IdeLayout>,
}

/// What checking one expression gave. A span checked several times (polymorphic instances,
/// overload trials) with different types is `conflicting`.
#[derive(Clone, Copy, Debug)]
pub struct IdeExprFact {
    pub ty: TypeId,
    /// A compile-time constant (a literal, a named constant, a constant expression).
    pub constant: bool,
    pub conflicting: bool,
}

/// A cast and the types on both sides.
#[derive(Clone, Copy, Debug)]
pub struct IdeCast {
    pub target: TypeId,
    pub from: TypeId,
    /// The operand was a compile-time constant.
    pub from_constant: bool,
    pub conflicting: bool,
}

impl IdeFacts {
    /// The references whose span holds `offset` in `file`, in the order they were recorded.
    pub fn refs_at(&self, file: FileId, offset: u32) -> Vec<&IdeRef> {
        let mut index = self.by_position.borrow_mut();
        if self.refs.len() < index.built {
            *index = RefIndex::default();
        }
        if self.refs.len() > index.built {
            let from = index.built;
            let mut unsorted: Vec<FileId> = Vec::new();
            for (i, r) in self.refs.iter().enumerate().skip(from) {
                let (short, long) = index.files.entry(r.span.file).or_default();
                if r.span.end.saturating_sub(r.span.start) > SHORT_REF {
                    long.push(i as u32);
                    continue;
                }
                if short
                    .last()
                    .is_some_and(|&j| self.refs[j as usize].span.start > r.span.start)
                    && !unsorted.contains(&r.span.file)
                {
                    unsorted.push(r.span.file);
                }
                short.push(i as u32);
            }
            // Stable, so equal starts keep the order recorded; the list is two sorted runs.
            for file in unsorted {
                if let Some((short, _)) = index.files.get_mut(&file) {
                    short.sort_by_key(|&i| self.refs[i as usize].span.start);
                }
            }
            index.built = self.refs.len();
        }
        let Some((list, long)) = index.files.get(&file) else {
            return Vec::new();
        };
        let holds = |i: u32| {
            let span = self.refs[i as usize].span;
            span.start <= offset && offset <= span.end
        };
        let past = list.partition_point(|&i| self.refs[i as usize].span.start <= offset);
        let earliest = offset.saturating_sub(SHORT_REF);
        let mut found: Vec<u32> = list[..past]
            .iter()
            .rev()
            .take_while(|&&i| self.refs[i as usize].span.start >= earliest)
            .copied()
            .filter(|&i| holds(i))
            .chain(long.iter().copied().filter(|&i| holds(i)))
            .collect();
        // In the order recorded, which breaks ties as a scan of `refs` would.
        found.sort_unstable();
        found.into_iter().map(|i| &self.refs[i as usize]).collect()
    }

    /// The narrowest use of a struct field or enum member holding `offset` in `file`, with the
    /// declaration it names.
    pub fn member_use_at(&self, file: FileId, offset: u32) -> Option<(Span, Span)> {
        let mut index = self.members_by_position.borrow_mut();
        if index.built != self.member_uses.len() {
            let mut files: HashMap<FileId, (Vec<(Span, Span)>, u32)> = HashMap::default();
            for &(u, d) in &self.member_uses {
                let (uses, longest) = files.entry(u.file).or_default();
                uses.push((u, d));
                *longest = (*longest).max(u.end.saturating_sub(u.start));
            }
            for (uses, _) in files.values_mut() {
                uses.sort_unstable_by_key(|(u, d)| (u.start, u.end, d.file.0, d.start, d.end));
            }
            *index = MemberIndex {
                built: self.member_uses.len(),
                files,
            };
        }
        let (uses, longest) = index.files.get(&file)?;
        let past = uses.partition_point(|(u, _)| u.start <= offset);
        let earliest = offset.saturating_sub(*longest);
        uses[..past]
            .iter()
            .rev()
            .take_while(|(u, _)| u.start >= earliest)
            .filter(|(u, _)| offset <= u.end)
            .min_by_key(|(u, d)| (u.end - u.start, u.start, d.file.0, d.start))
            .copied()
    }

    /// Drop the references recorded after the first `len`; use this, not `refs.truncate`,
    /// so `refs_at`'s index does not keep the dropped ones.
    pub fn truncate_refs(&mut self, len: usize) {
        self.refs.truncate(len);
        *self.by_position.get_mut() = RefIndex::default();
    }

    pub fn new(roots: Vec<String>) -> Self {
        Self {
            roots,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct IdeRef {
    pub span: Span,
    pub what: IdeWhat,
    pub ty: TypeId,
    /// The name of a declaration (not a use).
    pub decl: bool,
}

#[derive(Clone, Debug)]
pub enum IdeWhat {
    Entity(EntityId),
    Member(Sym),
    Type(TypeId),
    Procs(Vec<ProcId>),
    Module(ModuleId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdeKind {
    Variable,
    Constant,
    Function,
    Type,
    Module,
    Field,
    EnumMember,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdeName {
    pub name: String,
    pub kind: IdeKind,
    pub detail: String,
    /// Where the name is declared (`Span::NONE` for builtins): doc comments are read from there.
    pub decl: Span,
}

/// What `a.b.` refers to, for member completion.
#[derive(Clone, Copy, Debug)]
pub enum IdeReceiver {
    Value(TypeId),
    Type(TypeId),
    Module(ModuleId),
}

impl Compiler {
    pub(super) fn ide_cached_body_failure(
        &self,
        id: ProcId,
        epoch: u64,
    ) -> Option<Box<Diagnostic>> {
        let ide = self.ide.as_ref()?;
        if !self.placeholders_final || ide.sketching {
            return None;
        }
        let (at, error) = self.lenient_failures.get(&id)?;
        (*at == epoch && Rc::ptr_eq(ide.failed_body_lits.get(&id)?, &self.proc(id).lit))
            .then(|| error.clone())
    }

    pub(super) fn ide_record_body_result(
        &mut self,
        id: ProcId,
        epoch: u64,
        misses: (u64, u64),
        result: &Result<()>,
    ) {
        if !self.placeholders_final || self.ide.as_ref().is_none_or(|ide| ide.sketching) {
            return;
        }
        match result {
            Ok(()) => {
                self.lenient_failures.remove(&id);
                self.ide.as_mut().unwrap().failed_body_lits.remove(&id);
            }
            Err(error) if (self.placeholder_misses, self.in_progress_misses) == misses => {
                let lit = self.proc(id).lit.clone();
                self.lenient_failures.insert(id, (epoch, error.clone()));
                self.ide.as_mut().unwrap().failed_body_lits.insert(id, lit);
            }
            Err(_) => {}
        }
    }

    pub(super) fn ide_wants_pub(&mut self, file: FileId) -> bool {
        self.ide_wants(file)
    }

    fn ide_wants(&mut self, file: FileId) -> bool {
        let Some(ide) = self.ide.as_mut() else {
            return false;
        };
        if let Some(&w) = ide.wanted.get(&file) {
            return w;
        }
        let path = &self.sources.get(file).path;
        let w = ide.roots.iter().any(|r| path.starts_with(r.as_str()));
        ide.wanted.insert(file, w);
        w
    }

    pub(super) fn ide_note_expr(&mut self, expr: &ast::Expr, op: &Operand) {
        if self.ide.as_ref().is_some_and(|i| i.lint) && self.ide_wants(expr.span.file) {
            let ty = op.ty();
            let constant = matches!(op, Operand::Const { .. });
            if let Some(ide) = self.ide.as_mut() {
                ide.exprs
                    .entry(expr.span)
                    .and_modify(|f| f.conflicting |= f.ty != ty || f.constant != constant)
                    .or_insert(IdeExprFact {
                        ty,
                        constant,
                        conflicting: false,
                    });
            }
        }
        if let ast::ExprKind::Call {
            callee, ..
        } = &expr.kind
            && let Operand::Type(t) = op
            && matches!(self.types.kind(*t), TypeKind::Struct(_))
        {
            let name = match &callee.kind {
                ast::ExprKind::Ident(_) => Some(callee.span),
                ast::ExprKind::Member(_, name) => Some(name.span),
                _ => None,
            };
            if let Some(span) = name
                && self.ide_wants(span.file)
            {
                self.ide_note_instance(span, *t);
            }
            return;
        }
        let (span, what) = match &expr.kind {
            ast::ExprKind::Ident(_) | ast::ExprKind::Member(_, _) => {
                let (span, qualified_member) = match &expr.kind {
                    ast::ExprKind::Member(_, name) => (name.span, Some(name.name)),
                    _ => (expr.span, None),
                };
                // Failed expressions can leave scratch state behind. Only consume
                // the association made for this exact source expression.
                let entity = self
                    .ide
                    .as_mut()
                    .and_then(|i| i.last_entity.take())
                    .and_then(|(at, id)| (at == span).then_some(id));
                let using_member = self.ide.as_mut().and_then(|i| i.last_using_member.take());
                let member = qualified_member.or(using_member);
                let what = match op {
                    Operand::Type(t) => {
                        if let Some(e) = entity
                            && !matches!(
                                self.types.kind(*t),
                                TypeKind::Struct(_) | TypeKind::Enum(_)
                            )
                            && let Some(ide) = self.ide.as_mut()
                        {
                            ide.type_names.insert(span, e);
                        }
                        IdeWhat::Type(*t)
                    }
                    Operand::Procs(p) => IdeWhat::Procs(p.clone()),
                    Operand::Module(m) => IdeWhat::Module(*m),
                    _ => match (entity, member) {
                        (Some(e), _) => IdeWhat::Entity(e),
                        (None, Some(m)) => IdeWhat::Member(m),
                        (None, None) => return,
                    },
                };
                (span, what)
            }
            ast::ExprKind::InferredMember(name) => (name.span, IdeWhat::Member(name.name)),
            _ => return,
        };
        if !self.ide_wants(span.file) {
            return;
        }
        let ty = op.ty();
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span,
                what,
                ty,
                decl: false,
            });
        }
    }

    /// Lint mode: `cast` at `span` converted a `from` operand to `target`.
    pub(super) fn ide_note_cast(&mut self, span: Span, target: TypeId, op: &Operand) {
        if !self.ide.as_ref().is_some_and(|i| i.lint) || !self.ide_wants(span.file) {
            return;
        }
        let from = op.ty();
        let from_constant = matches!(op, Operand::Const { .. });
        if let Some(ide) = self.ide.as_mut() {
            ide.casts
                .entry(span)
                .and_modify(|c| {
                    c.conflicting |=
                        c.target != target || c.from != from || c.from_constant != from_constant
                })
                .or_insert(IdeCast {
                    target,
                    from,
                    from_constant,
                    conflicting: false,
                });
        }
    }

    /// Lint mode: a lookup found a name through import `index` of `scope`.
    pub(super) fn ide_note_import_use(&mut self, scope: ScopeId, index: usize) {
        if let Some(ide) = self.ide.as_mut()
            && ide.lint
        {
            ide.used_imports.insert((scope, index));
        }
    }

    /// Lint mode: a name resolved to `id`.
    pub(super) fn ide_note_use(&mut self, id: EntityId) {
        if let Some(ide) = self.ide.as_mut()
            && ide.lint
        {
            ide.used.insert(id);
        }
    }

    /// A declaration: hovering its name shows it.
    pub(super) fn ide_note_entity(&mut self, id: EntityId) {
        let e = self.entity(id);
        let (span, name) = (e.span, e.name);
        if span.end <= span.start || !self.ide_wants(span.file) {
            return;
        }
        // Entity spans may cover the whole declaration: narrow to the name.
        let text = self.sources.snippet(span);
        let Some(at) = text.find(name.as_str()) else {
            return;
        };
        let start = span.start + at as u32;
        let span = Span {
            file: span.file,
            start,
            end: start + name.as_str().len() as u32,
        };
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            }
            | EntityKind::Const {
                ty, ..
            } => *ty,
            _ => TypeId::COMPILE_TIME,
        };
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Entity(id),
                ty,
                decl: true,
            });
        }
    }

    /// A semantic identifier resolved through a binding path that does not lower
    /// an ordinary expression (for example a using target or module argument label).
    pub(super) fn ide_note_entity_use(&mut self, span: Span, id: EntityId, ty: TypeId) {
        if !self.ide_wants(span.file) {
            return;
        }
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Entity(id),
                ty,
                decl: false,
            });
        }
    }

    pub(super) fn ide_note_module_arguments(&mut self, module: ModuleId, import: &ast::Import) {
        if self.ide.is_none() {
            return;
        }
        let parameters = self.modules[module.0 as usize].param_entities.clone();
        for arg in &import.params {
            let Some(name) = arg.name else {
                continue;
            };
            let Some(&id) = parameters
                .iter()
                .find(|&&id| self.entity(id).name == name.name)
            else {
                continue;
            };
            let ty = match self.entity(id).kind {
                EntityKind::Const {
                    ty, ..
                }
                | EntityKind::Local {
                    ty, ..
                } => ty,
                _ => TypeId::COMPILE_TIME,
            };
            self.ide_note_entity_use(name.span, id, ty);
        }
    }

    /// `scope` covers `span` of the source (a block, a procedure body).
    pub(super) fn ide_scope_span(&mut self, scope: ScopeId, span: Span) {
        if span.end > span.start
            && self.ide_wants(span.file)
            && let Some(ide) = self.ide.as_mut()
        {
            ide.scopes.push((span, scope));
        }
    }

    /// Recovered bodies supply facts but cannot be executed as valid procedures or code.
    pub(super) fn ide_require_parsed_body(&self, span: Span) -> Result<()> {
        if let Some(error) = self.ide.as_ref().and_then(|ide| {
            ide.parse_errors.iter().find(|e| {
                e.span.file == span.file && span.start <= e.span.start && e.span.start < span.end
            })
        }) {
            return Err(Box::new(error.clone()));
        }
        Ok(())
    }

    /// After compiling for an editor: check every procedure body and top-level declaration of
    /// the recorded files, not just what the program reaches. Returns errors from declarations
    /// and independent statements (a body that fails does not stop the others, and
    /// a declaration another one depends on reports its own error once, at its own span).
    pub fn ide_check_all(&mut self) -> Vec<Diagnostic> {
        let mut errors: Vec<Diagnostic> = Vec::new();
        if let Some(ide) = self.ide.as_mut() {
            errors.append(&mut ide.errors);
        } else {
            return errors;
        }
        // Normal compilation has ended (possibly before finish_program). The IDE
        // audit has a complete source snapshot: no metaprogram is still supplying
        // placeholders. Use the compiler's existing finalization rule so a failed
        // generator cannot stay speculative and recreate nested procedures forever.
        self.placeholders_final = true;
        for i in 0..self.entities.len() {
            let id = EntityId(i as u32);
            let e = self.entity(id);
            let (scope, span) = (e.scope, e.span);
            if matches!(e.kind, EntityKind::Decl { .. })
                && matches!(e.state, EntityState::Unresolved)
                && self.ide_wants(span.file)
            {
                // A record constant may depend on type_of of an earlier field.
                // Match ordinary unreferenced checking: establish the containing
                // record's fields before resolving its member constants.
                let result = if let ScopeKind::Struct(t) = self.scope(scope).kind
                    && let TypeKind::Struct(s) = self.types.kind(t).clone()
                {
                    self.layout_struct(s, span)
                        .and_then(|()| self.resolve_entity(id))
                } else {
                    self.resolve_entity(id)
                };
                if let Err(e) = result {
                    errors.push(*e);
                }
            }
        }
        // A struct with a member of an unknown type fails when it is laid out.
        let mut i = 0;
        while i < self.entities.len() {
            let id = EntityId(i as u32);
            i += 1;
            let e = self.entity(id);
            let span = e.span;
            if let EntityState::Done(Resolved::Const {
                value: Value::Type(t),
                ..
            }) = &e.state
                && let TypeKind::Struct(s) = *self.types.kind(*t)
                && self.types.struct_info(s).poly_args.is_empty()
                && self.ide_wants(span.file)
                && let Err(e) = self.layout_struct(s, span)
            {
                errors.push(*e);
            }
        }
        let mut i = 0;
        while i < self.procs.len() {
            let id = ProcId(i as u32);
            i += 1;
            // Header classification resolves names and can run compile-time code. Apply
            // the recording policy before starting independent audit work; actual calls
            // still resolve their selected dependencies through the normal checker.
            if !self.ide_wants(self.proc(id).span.file) {
                continue;
            }
            // `t: *Table` makes a procedure polymorphic, which is only known once names
            // resolve (a compile that stopped early may not have asked yet).
            let _ = self.refresh_implicit_poly(id);
            let p = self.proc(id);
            let span = p.span;
            if p.is_poly
                // A concrete instance is not a source declaration. Matching may
                // cache it before rejecting a candidate, or prefer another overload.
                // Actual uses queue their bodies through proc_func; auditing a
                // NotNeeded instance here would impose a body obligation on a call
                // that never selected it. Keep the cache for later selected uses.
                || p.bindings.is_some()
                || p.is_macro
                || p.lit.body.is_none()
                || p.body_state != super::procs::BodyState::NotNeeded
            {
                continue;
            }
            if let Err(e) = self.proc_func(id, span) {
                errors.push(*e);
            }
        }
        errors.extend(self.drain_bodies_lenient().map(|e| *e));
        self.ide_note_bodiless();
        let mut failed: Vec<_> = self
            .lenient_failures_list()
            .into_iter()
            .filter(|e| self.ide_wants(e.span.file))
            .collect();
        failed.sort_by_key(|e| (e.span.file.0, e.span.start));
        errors.extend(failed);
        if let Some(ide) = self.ide.as_mut() {
            errors.append(&mut ide.errors);
        }
        errors
    }

    /// Procedures without a body (`#foreign`, `#elsewhere`, `#compiler`): no body declares
    /// their parameters, so their names are recorded from the signature, as is the library
    /// `#foreign` names.
    fn ide_note_bodiless(&mut self) {
        for i in 0..self.procs.len() {
            let id = ProcId(i as u32);
            if !self.ide_wants(self.proc(id).span.file) {
                continue;
            }
            let _ = self.refresh_implicit_poly(id);
            let p = self.proc(id);
            if p.is_poly || p.lit.body.is_some() {
                continue;
            }
            let (span, scope, header) = (p.span, p.scope, p.lit.header.clone());
            if !self.ide_wants(span.file) {
                continue;
            }
            if let Ok(sig) = self.signature(id, span) {
                for (param, sp) in header.params.iter().zip(&sig.params) {
                    if let Some(name) = param.name.filter(|n| n.span.end > n.span.start) {
                        self.ide_note_param_name(name.span, name.name, sp.ty);
                    }
                }
            }
            if let Some(lib) = header.foreign.as_ref().and_then(|f| f.library)
                && let Ok(ids) = self.lookup(scope, lib.name)
                && let [lib_id] = ids[..]
                && let Some(ide) = self.ide.as_mut()
            {
                ide.refs.push(IdeRef {
                    span: lib.span,
                    what: IdeWhat::Entity(lib_id),
                    ty: TypeId::VOID,
                    decl: false,
                });
            }
        }
    }

    /// `#if cond` skipped `branch`: when `cond` asks about the target, record where, so the
    /// language server can ask a compile for another target about that code.
    pub(super) fn ide_note_skipped(&mut self, cond: &ast::Expr, branch: &[ast::Stmt]) {
        let (Some(first), Some(last)) = (branch.first(), branch.last()) else {
            return;
        };
        if self.ide.is_none() || !self.ide_wants(first.span.file) || !names_target(cond) {
            return;
        }
        let span = first.span.to(last.span);
        if let Some(ide) = self.ide.as_mut() {
            ide.skipped.push(span);
        }
    }

    /// The innermost recorded scope at `offset` of `file` (else the file's own scope).
    pub fn ide_scope_at(&self, file: FileId, offset: u32) -> Option<ScopeId> {
        let inner = self.ide.as_ref().and_then(|ide| {
            ide.scopes
                .iter()
                .filter(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
                .min_by_key(|(s, _)| s.end - s.start)
                .map(|(_, id)| *id)
        });
        inner.or_else(|| {
            (0..self.scopes.len())
                .map(|i| ScopeId(i as u32))
                .find(|&s| {
                    self.scope(s).kind == ScopeKind::File && self.scope(s).file == Some(file)
                })
        })
    }

    /// The innermost recorded name at `offset` of `file`, and whether every record of that
    /// exact span agrees (a polymorphic body is checked once per instance).
    fn ide_hover_ref(&self, file: FileId, offset: u32) -> Option<(IdeRef, bool)> {
        let here = self.ide.as_ref()?.refs_at(file, offset);
        let r = here
            .iter()
            .copied()
            .min_by_key(|r| r.span.end - r.span.start)?;
        let same = here
            .iter()
            .filter(|o| o.span == r.span)
            .all(|o| o.ty == r.ty && same_what(&o.what, &r.what));
        Some((r.clone(), same))
    }

    pub fn ide_hover(&mut self, file: FileId, offset: u32) -> Option<IdeHover> {
        if let Some((span, ids)) = self.ide_filter_targets(file, offset) {
            let text = ids
                .iter()
                .map(|&id| self.ide_entity_hover(id, None))
                .collect::<Vec<_>>()
                .join("\n");
            return Some(IdeHover {
                span,
                text,
                layout: None,
            });
        }
        let Some((r, same)) = self.ide_hover_ref(file, offset) else {
            return self.ide_field_hover(file, offset);
        };
        let name = self.sources.snippet(r.span).to_string();
        // `Poly(args)`: the instance, not the polymorphic struct.
        if let IdeWhat::Entity(e) = r.what
            && matches!(
                self.entity(e).state,
                EntityState::Done(Resolved::PolyStruct(_))
            )
            && let Some(&Some(t)) = self.ide.as_ref()?.instances.get(&r.span)
        {
            let text = self.ide_type_hover(&self.types.name(t), t);
            let layout = if same {
                self.ide_type_layout(t)
            } else {
                None
            };
            return Some(IdeHover {
                span: r.span,
                text,
                layout,
            });
        }
        let text = match &r.what {
            IdeWhat::Entity(e) => self.ide_entity_hover(*e, Some(r.ty)),
            IdeWhat::Member(_) => format!("{name}: {}", self.types.name(r.ty)),
            IdeWhat::Type(t) => self.ide_type_hover(&name, *t),
            // An overload set may include aliases (`print :: print_to_builder;`): show each
            // under the name used.
            IdeWhat::Procs(ps) => ps
                .iter()
                .map(|&p| format!("{name} :: {}", clip(&self.ide_header_text(p))))
                .collect::<Vec<_>>()
                .join("\n"),
            IdeWhat::Module(m) => {
                format!("{name} :: #import \"{}\"", self.modules[m.0 as usize].name)
            }
        };
        let layout = if same {
            match &r.what {
                IdeWhat::Entity(e) => self.ide_entity_layout(*e),
                IdeWhat::Member(m) => self.ide_member_layout(r.span, *m),
                IdeWhat::Type(t) => self.ide_type_layout(*t),
                IdeWhat::Procs(_) | IdeWhat::Module(_) => None,
            }
        } else {
            None
        };
        Some(IdeHover {
            span: r.span,
            text,
            layout,
        })
    }

    /// The declarations `ide_hover` describes, for reading their doc comments: one per
    /// overload, in the order of the hover's lines.
    pub fn ide_hover_decls(&mut self, file: FileId, offset: u32) -> Vec<Span> {
        if let Some((_, ids)) = self.ide_filter_targets(file, offset) {
            return self.ide_narrow_decls(
                ids.iter()
                    .map(|&id| (self.entity(id).span, self.entity(id).name.to_string()))
                    .collect(),
            );
        }
        let Some((r, _)) = self.ide_hover_ref(file, offset) else {
            let Some(site) = self.ide.as_ref().and_then(|ide| {
                ide.fields
                    .iter()
                    .find(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
                    .and_then(|(_, site)| *site)
            }) else {
                return Vec::new();
            };
            let f = &self.types.struct_info(site.declaring).fields[site.field];
            let named = f.name.map(|n| (f.span, n.to_string()));
            return self.ide_narrow_decls(named.into_iter().collect());
        };
        let procs: Vec<ProcId> = match &r.what {
            IdeWhat::Procs(ps) => ps.clone(),
            IdeWhat::Entity(e) => match self.entity(*e).kind {
                EntityKind::Decl {
                    ..
                }
                | EntityKind::Import(_) => match self.resolve_entity(*e).ok() {
                    Some(Resolved::Proc(p))
                    | Some(Resolved::Const {
                        value: value::Value::Proc(p),
                        ..
                    }) => vec![p],
                    Some(Resolved::ProcSet(ps)) => ps,
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        if procs.is_empty() {
            return self.ide_definition(file, offset);
        }
        let spans = procs
            .iter()
            .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
            .collect();
        self.ide_narrow_decls(spans)
    }

    /// The name of a field where its struct declares it. A field laid out in several structs
    /// (the instances of a polymorphic struct) has no one type and layout to show.
    fn ide_field_hover(&mut self, file: FileId, offset: u32) -> Option<IdeHover> {
        let (span, site) = self
            .ide
            .as_ref()?
            .fields
            .iter()
            .find(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
            .map(|(s, site)| (*s, *site))?;
        // A polymorphic struct's field: its declaration as written (`items: [N] T`).
        let Some(site) = site else {
            let file = &self.sources.try_get(span.file)?.text;
            let rest = file.get(span.start as usize..)?;
            let text = rest[..rest.find([';', '\n', '}']).unwrap_or(rest.len())].trim_end();
            return Some(IdeHover {
                span,
                text: text.to_string(),
                layout: None,
            });
        };
        let f = &self.types.struct_info(site.declaring).fields[site.field];
        let text = format!("{}: {}", f.name?, self.types.name(f.ty));
        Some(IdeHover {
            span,
            text,
            layout: Some(self.ide_field_layout(site)),
        })
    }

    /// Lay out what a hover describes (the program may not have needed its layout).
    fn ide_laid_out(&mut self, t: TypeId) -> bool {
        let span = Span {
            file: FileId(0),
            start: 0,
            end: 0,
        };
        self.ensure_complete(t, span).is_ok()
    }

    /// Size and alignment of a type, and for a struct the bytes its fields leave unused.
    fn ide_type_layout(&mut self, t: TypeId) -> Option<IdeLayout> {
        if matches!(
            self.types.kind(t),
            TypeKind::Void
                | TypeKind::CompileTimeOnly
                | TypeKind::PolyParam
                | TypeKind::PolyStruct { .. }
        ) || !self.ide_laid_out(t)
        {
            return None;
        }
        let size = self.types.size_of(t);
        let padding = match self.types.kind(t) {
            TypeKind::Struct(s) => {
                let mut spans: Vec<(u64, u64)> = self
                    .types
                    .struct_info(*s)
                    .fields
                    .iter()
                    .map(|f| (f.offset, f.offset + self.types.size_of(f.ty)))
                    .collect();
                spans.sort_unstable();
                let mut covered = 0;
                let mut reached = 0;
                for (start, end) in spans {
                    let start = start.max(reached);
                    if end > start {
                        covered += end - start;
                        reached = end;
                    }
                }
                size.saturating_sub(covered)
            }
            _ => 0,
        };
        Some(IdeLayout {
            offset: None,
            size,
            align: self.types.align_of(t),
            padding,
        })
    }

    /// A named type's layout, or a variable's or constant's when its type is an aggregate.
    fn ide_entity_layout(&mut self, id: EntityId) -> Option<IdeLayout> {
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            } => *ty,
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => return self.ide_type_layout(*t),
            EntityKind::Const {
                ty, ..
            } => *ty,
            EntityKind::Decl {
                ..
            } => match &self.entity(id).state {
                EntityState::Done(Resolved::Const {
                    value: value::Value::Type(t),
                    ..
                }) => return self.ide_type_layout(*t),
                EntityState::Done(
                    Resolved::Const {
                        ty, ..
                    }
                    | Resolved::Global {
                        ty, ..
                    },
                ) => *ty,
                _ => return None,
            },
            _ => return None,
        };
        if !self.ide_aggregate(ty) {
            return None;
        }
        self.ide_type_layout(ty)
    }

    /// Structs, unions, arrays, strings and `Any`: types whose size is worth showing on a value.
    fn ide_aggregate(&self, ty: TypeId) -> bool {
        match self.types.kind(ty) {
            TypeKind::Struct(_)
            | TypeKind::Array {
                ..
            }
            | TypeKind::String
            | TypeKind::Any => true,
            TypeKind::Distinct(d) => self.ide_aggregate(self.types.distincts[d.0 as usize].base),
            _ => false,
        }
    }

    /// The field `name` of the value left of the `.` at `span`.
    fn ide_member_layout(&mut self, span: Span, name: Sym) -> Option<IdeLayout> {
        let mut ty = (*self.ide.as_ref()?.receivers.get(&span)?)?;
        while let Some(p) = self.types.pointee(ty) {
            ty = p;
        }
        let TypeKind::Struct(s) = *self.types.kind(self.types.repr(ty)) else {
            return None;
        };
        if !self.ide_laid_out(ty) {
            return None;
        }
        let site = self.ide_find_field(s, name, 0, 0)?;
        Some(self.ide_field_layout(site))
    }

    /// Where `name` is in struct `s` (through `using` members held by value), `base` bytes
    /// into the struct the search started from.
    fn ide_find_field(
        &self,
        s: StructId,
        name: Sym,
        base: u64,
        depth: u32,
    ) -> Option<IdeFieldSite> {
        let fields = &self.types.struct_info(s).fields;
        if let Some(at) = fields.iter().position(|f| f.name == Some(name)) {
            return Some(IdeFieldSite {
                owner: s,
                declaring: s,
                field: at,
                offset: base + fields[at].offset,
            });
        }
        if depth > 16 {
            return None;
        }
        fields.iter().filter(|f| f.using).find_map(|f| {
            let TypeKind::Struct(inner) = *self.types.kind(self.types.repr(f.ty)) else {
                return None;
            };
            self.ide_find_field(inner, name, base + f.offset, depth + 1)
        })
    }

    /// A field's offset (in the struct it is reached through or declared in), size, alignment
    /// and the bytes skipped before it.
    fn ide_field_layout(&self, site: IdeFieldSite) -> IdeLayout {
        let info = self.types.struct_info(site.declaring);
        let f = &info.fields[site.field];
        let padding = if f.overlay || info.is_union {
            0
        } else {
            let before = info.fields[..site.field]
                .iter()
                .filter(|g| !g.overlay)
                .map(|g| g.offset + self.types.size_of(g.ty))
                .max()
                .unwrap_or(0);
            f.offset.saturating_sub(before)
        };
        IdeLayout {
            offset: Some(site.offset),
            size: self.types.size_of(f.ty),
            align: f.align,
            padding,
        }
    }

    /// `b` of `a.b` was checked with `a` being `base`.
    pub(super) fn ide_note_receiver(&mut self, span: Span, base: &Operand) {
        if matches!(
            base,
            Operand::Type(_) | Operand::Module(_) | Operand::Procs(_)
        ) || !self.ide_wants(span.file)
        {
            return;
        }
        let ty = base.ty();
        if let Some(ide) = self.ide.as_mut() {
            ide.receivers
                .entry(span)
                .and_modify(|t| {
                    if *t != Some(ty) {
                        *t = None;
                    }
                })
                .or_insert(Some(ty));
        }
    }

    /// The name of `name` within declaration `span` (a field's span can cover its type and
    /// initializer).
    fn ide_name_span(&self, span: Span, name: Sym) -> Span {
        let text = self.sources.snippet(span);
        let word = name.as_str();
        let boundary = |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        let mut from = 0;
        while let Some(at) = text[from..].find(word) {
            let start = from + at;
            let end = start + word.len();
            if boundary(text[..start].chars().next_back()) && boundary(text[end..].chars().next()) {
                return Span {
                    file: span.file,
                    start: span.start + start as u32,
                    end: span.start + end as u32,
                };
            }
            from = start + 1;
        }
        span
    }

    /// The declaration of the field or enum member `name` of `ty`'s struct or enum, through
    /// pointers and `using` members.
    fn ide_member_decl(&self, ty: TypeId, name: Sym) -> Option<Span> {
        let mut ty = ty;
        while let Some(p) = self.types.pointee(ty) {
            ty = p;
        }
        let ty = self.types.repr_struct(ty);
        match self.types.kind(ty) {
            TypeKind::Struct(s) => {
                let (s, field) = self.ide_find_field_decl(*s, name, &mut Vec::new())?;
                let span = self.types.struct_info(s).fields[field].span;
                (span.end > span.start).then(|| self.ide_name_span(span, name))
            }
            TypeKind::Enum(e) => {
                let e =
                    self.types
                        .enum_info(*e)
                        .loose_of
                        .map_or(*e, |t| match self.types.kind(t) {
                            TypeKind::Enum(strict) => *strict,
                            _ => *e,
                        });
                self.ide.as_ref()?.enum_decls.get(&(e, name)).copied()
            }
            _ => None,
        }
    }

    fn ide_find_field_decl(
        &self,
        s: StructId,
        name: Sym,
        searched: &mut Vec<StructId>,
    ) -> Option<(StructId, usize)> {
        if searched.contains(&s) {
            return None;
        }
        searched.push(s);
        let fields = &self.types.struct_info(s).fields;
        if let Some(at) = fields.iter().position(|f| f.name == Some(name)) {
            return Some((s, at));
        }
        fields.iter().filter(|f| f.using).find_map(|f| {
            let mut ty = f.ty;
            while let Some(p) = self.types.pointee(ty) {
                ty = p;
            }
            let TypeKind::Struct(inner) = *self.types.kind(self.types.repr_struct(ty)) else {
                return None;
            };
            self.ide_find_field_decl(inner, name, searched)
        })
    }

    /// `name` at `span` named a member of `ty` (a field through `a.name`, `T.{ name = v }`, an
    /// enum member through `Enum.NAME` or `.NAME`).
    pub(super) fn ide_note_member_use(&mut self, span: Span, ty: TypeId, name: Sym) {
        if self.ide.is_none() || !self.ide_wants(span.file) {
            return;
        }
        let Some(decl) = self.ide_member_decl(ty, name) else {
            return;
        };
        if let Some(ide) = self.ide.as_mut() {
            ide.member_uses.insert((span, decl));
        }
    }

    /// The label of `break label` / `continue label`: the loop's name, an entity in `scope`.
    pub(super) fn ide_note_label(&mut self, scope: ScopeId, label: ast::Ident) {
        if !self.ide_wants(label.span.file) {
            return;
        }
        let Ok(ids) = self.lookup(scope, label.name) else {
            return;
        };
        let &[id] = ids.as_slice() else {
            return;
        };
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            }
            | EntityKind::Const {
                ty, ..
            } => *ty,
            _ => return,
        };
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span: label.span,
                what: IdeWhat::Entity(id),
                ty,
                decl: false,
            });
        }
    }

    /// `name` at `span` names an authored member/parameter declaration at `decl`.
    pub(super) fn ide_note_named_field(&mut self, span: Span, name: Sym, decl: Span, ty: TypeId) {
        if !self.ide_wants(span.file) {
            return;
        }
        let decl = self.ide_name_span(decl, name);
        if let Some(ide) = self.ide.as_mut() {
            ide.member_uses.insert((span, decl));
        }
        self.ide_note_member_ref(span, name, ty);
    }

    /// The name of a procedure type's parameter (`#type (a: int)`), once however often the
    /// type is evaluated.
    pub(super) fn ide_note_param_name(&mut self, span: Span, name: Sym, ty: TypeId) {
        if self
            .ide
            .as_mut()
            .is_some_and(|i| i.param_names.insert(span))
        {
            self.ide_note_named_field(span, name, span, ty);
        }
    }

    /// Named arguments of a successfully selected struct instance. Its bound constants
    /// supply the actual parameter types, including dependent and baked parameters.
    pub(super) fn ide_note_struct_arguments(&mut self, ty: TypeId, args: &[ast::Arg]) {
        if self.ide.is_none() {
            return;
        }
        let Some(s) = self.types.as_struct(ty) else {
            return;
        };
        let Some(source) = self.struct_asts.get(&s) else {
            return;
        };
        let (lit, scope) = (source.lit.clone(), source.scope);
        for name in args.iter().filter_map(|a| a.name) {
            let Some(param) = lit
                .params
                .iter()
                .filter_map(|p| p.name)
                .find(|p| p.name == name.name)
            else {
                continue;
            };
            let Some(ids) = self.scope(scope).names.get(&name.name) else {
                continue;
            };
            let [id] = ids.as_slice() else {
                continue;
            };
            let EntityKind::Const {
                ty, ..
            } = self.entity(*id).kind
            else {
                continue;
            };
            self.ide_note_named_field(name.span, name.name, param.span, ty);
        }
    }

    /// `name` at `span` names a member of type `ty` where nothing else records it for hover
    /// (a field of a struct literal).
    pub(super) fn ide_note_member_ref(&mut self, span: Span, name: Sym, ty: TypeId) {
        if self.ide.is_none() || !self.ide_wants(span.file) {
            return;
        }
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Member(name),
                ty,
                decl: false,
            });
        }
    }

    /// The `$T` binders of a procedure header: each name with the span of the name after the
    /// `$` (its first binder when there are several).
    pub(super) fn poly_binders(&self, header: &ast::ProcHeader) -> Vec<(Sym, Span)> {
        let mut out: Vec<(Sym, Span)> = Vec::new();
        for_header_types(header, &mut |e| {
            if let ast::ExprKind::PolyVar {
                name, ..
            }
            | ast::ExprKind::PolyRestricted {
                name, ..
            } = &e.kind
                && !out.iter().any(|(n, _)| n == name)
            {
                out.push((*name, self.poly_name_span(e, *name)));
            }
        });
        out
    }

    /// The name of `$T`, `$$T` or `$T/R` at `e`, after its `$`s.
    fn poly_name_span(&self, e: &ast::Expr, name: Sym) -> Span {
        let text = self.ide_snippet(e.span);
        let skip = text.len() - text.trim_start_matches('$').len();
        let start = e.span.start + skip as u32;
        Span {
            file: e.span.file,
            start,
            end: (start + name.as_str().len() as u32).min(e.span.end.max(start)),
        }
    }

    /// The header of a polymorphic procedure was instantiated with `bound` (each `$T` and the
    /// constant it became): its `$T`s and `T`s name those constants, and the name of a
    /// restriction (`$T/Ring`) is looked up in `scope`.
    pub(super) fn ide_note_poly_header(
        &mut self,
        header: &ast::ProcHeader,
        bound: &[(Sym, EntityId)],
        scope: ScopeId,
    ) {
        if !self.ide_wants(header.span.file) {
            return;
        }
        let mut found: Vec<(Span, Sym, bool)> = Vec::new();
        for_header_types(header, &mut |e| match &e.kind {
            ast::ExprKind::Ident(name) => found.push((e.span, *name, false)),
            ast::ExprKind::PolyVar {
                name, ..
            }
            | ast::ExprKind::PolyRestricted {
                name, ..
            } => found.push((self.poly_name_span(e, *name), *name, true)),
            _ => {}
        });
        for (span, name, binder) in found {
            let what = match bound.iter().find(|(n, _)| *n == name) {
                Some(&(_, id)) => IdeWhat::Entity(id),
                // Only restrictions and other named types: types the header names elsewhere
                // are recorded when the instance's signature is checked.
                None if !binder => match self.lookup(scope, name) {
                    Ok(ids) if ids.len() == 1 => IdeWhat::Entity(ids[0]),
                    _ => continue,
                },
                None => continue,
            };
            let ty = match &what {
                IdeWhat::Entity(e) => match &self.entity(*e).kind {
                    EntityKind::Const {
                        ty, ..
                    } => *ty,
                    _ => TypeId::TYPE,
                },
                _ => TypeId::TYPE,
            };
            if let Some(ide) = self.ide.as_mut() {
                ide.refs.push(IdeRef {
                    span,
                    what,
                    ty,
                    decl: false,
                });
            }
        }
    }

    /// A polymorphic procedure or macro nothing used is not checked: still name the types
    /// its header mentions.
    pub(super) fn ide_note_unchecked_header(&mut self, id: ProcId) {
        let p = self.proc(id);
        if p.instances.is_empty() {
            let (header, scope) = (p.lit.header.clone(), p.scope);
            self.ide_note_poly_header(&header, &[], scope);
        }
    }

    /// The enum members an inferred `.NAME` at `offset` can stand for, from the type its
    /// context expected (a variable's declared type, a parameter, the other side of `==`).
    pub fn ide_inferred_members(&mut self, file: FileId, offset: u32) -> Vec<IdeName> {
        let Some(types) = self.ide.as_ref().and_then(|ide| {
            ide.inferred_expected
                .iter()
                .find(|(s, _)| s.file == file && s.start == offset)
                .map(|(_, t)| t.clone())
        }) else {
            return Vec::new();
        };
        let mut out: Vec<IdeName> = Vec::new();
        for t in types {
            let t = self.types.repr_struct(t);
            if matches!(self.types.kind(t), TypeKind::Enum(_)) {
                for n in self.ide_members(IdeReceiver::Type(t)) {
                    if !out.iter().any(|o| o.name == n.name) {
                        out.push(n);
                    }
                }
            }
        }
        out
    }

    /// A call nothing accepts (typically one being typed, with arguments missing): record the
    /// types its candidates take where an argument is an inferred `.NAME`, so completion can
    /// still list the enum's members.
    pub(super) fn ide_note_unmatched_inferred(
        &mut self,
        procs: &[ProcId],
        args: &[super::calls::CallArg],
        span: Span,
    ) {
        if !self.ide_wants(span.file) {
            return;
        }
        for (i, a) in args.iter().enumerate() {
            let Some(ast::Expr {
                kind: ast::ExprKind::InferredMember(name),
                ..
            }) = &a.expr
            else {
                continue;
            };
            for &p in procs {
                if self.proc(p).is_poly {
                    continue;
                }
                let header = self.proc(p).lit.header.clone();
                let at = match a.name {
                    Some(n) => header
                        .params
                        .iter()
                        .position(|q| q.name.map(|x| x.name) == Some(n)),
                    None => Some(i),
                };
                let Some(at) = at.filter(|&k| k < header.params.len() && !header.params[k].baked)
                else {
                    continue;
                };
                // Index among the runtime parameters.
                let k = header.params[..at].iter().filter(|q| !q.baked).count();
                let Ok(sig) = self.signature(p, span) else {
                    continue;
                };
                let Some(param) = sig.params.get(k) else {
                    continue;
                };
                let ty = param.ty;
                if let Some(ide) = self.ide.as_mut() {
                    let seen = ide.inferred_expected.entry(name.span).or_default();
                    if !seen.contains(&ty) {
                        seen.push(ty);
                    }
                }
            }
        }
    }

    /// An enum member's declaration.
    pub(super) fn ide_note_enum_member(&mut self, e: crate::types::EnumId, name: Sym, span: Span) {
        if self.ide.is_none() {
            return;
        }
        // Every file's: a use in a recorded file can name a member declared in a module.
        let wanted = self.ide_wants(span.file);
        if let Some(ide) = self.ide.as_mut() {
            ide.enum_decls.insert((e, name), span);
            if wanted {
                ide.member_uses.insert((span, span));
            }
        }
    }

    /// `Poly(args)` with `Poly` at `span` gave the struct `t`.
    fn ide_note_instance(&mut self, span: Span, t: TypeId) {
        if let Some(ide) = self.ide.as_mut() {
            ide.instances
                .entry(span)
                .and_modify(|old| {
                    if *old != Some(t) {
                        *old = None;
                    }
                })
                .or_insert(Some(t));
        }
    }

    /// Struct `s` was laid out: record where its named fields are, and move the fields of its
    /// anonymous `struct { ... }` / `union { ... }` members into it. The fields of a
    /// polymorphic struct's instance are recorded as having no one layout: their source is the
    /// polymorphic definition.
    pub(super) fn ide_note_layout(&mut self, s: StructId) {
        let info = self.types.struct_info(s);
        let instance = !info.poly_args.is_empty();
        let mut sites = Vec::new();
        let mut absorbed = Vec::new();
        for (field, f) in info.fields.iter().enumerate() {
            if f.name.is_some() {
                let site = IdeFieldSite {
                    owner: s,
                    declaring: s,
                    field,
                    offset: f.offset,
                };
                sites.push((f.span, site));
            } else if f.using
                && let TypeKind::Struct(inner) = *self.types.kind(f.ty)
                && self.types.struct_info(inner).name.as_str() == "anonymous"
            {
                absorbed.push((inner, f.offset));
            }
        }
        let sites: Vec<_> = sites
            .into_iter()
            .filter(|(span, _)| span.end > span.start && self.ide_wants(span.file))
            .collect();
        let names: Vec<Span> = sites
            .iter()
            .filter_map(|(span, site)| {
                let name = self.types.struct_info(site.declaring).fields[site.field].name?;
                Some(self.ide_name_span(*span, name))
            })
            .collect();
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        for name in names {
            ide.member_uses.insert((name, name));
        }
        for (span, site) in sites {
            let site = (!instance).then_some(site);
            ide.fields
                .entry(span)
                .and_modify(|old| {
                    if *old != site {
                        *old = None;
                    }
                })
                .or_insert(site);
        }
        for (inner, base) in absorbed {
            for entry in ide.fields.values_mut() {
                if let Some(site) = entry
                    && site.owner == inner
                {
                    site.owner = s;
                    site.offset += base;
                    if instance {
                        *entry = None;
                    }
                }
            }
        }
    }

    /// Declarations the identifier at `offset` names: an entity's declaration, each procedure
    /// of an overload set, a struct or enum. Spans are narrowed to the declared name when it
    /// starts the declaration (`name :: ...`).
    pub fn ide_definition(&mut self, file: FileId, offset: u32) -> Vec<Span> {
        if let Some((_, ids)) = self.ide_filter_targets(file, offset) {
            return self.ide_narrow_decls(
                ids.iter()
                    .map(|&id| (self.entity(id).span, self.entity(id).name.to_string()))
                    .collect(),
            );
        }
        if let Some(selection) = self.ide.as_ref().and_then(|ide| {
            (0..3).find_map(|delta| {
                let start = offset.checked_sub(delta)?;
                let end = start.checked_add(3)?;
                ide.for_expansions.get(&Span {
                    file,
                    start,
                    end,
                })
            })
        }) {
            let spans = selection
                .as_ref()
                .into_iter()
                .flatten()
                .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
                .collect();
            let mut definitions = self.ide_narrow_decls(spans);
            definitions.sort_unstable_by_key(|s| (s.file.0, s.start, s.end));
            definitions.dedup();
            return definitions;
        }
        // A struct field or enum member: the declaration its use was matched to.
        if let Some(ide) = self.ide.as_ref()
            && let Some((_, decl)) = ide.member_use_at(file, offset)
            && decl.end > decl.start
        {
            return vec![decl];
        }
        let Some(r) = self.ide.as_ref().and_then(|ide| {
            ide.refs_at(file, offset)
                .into_iter()
                .min_by_key(|r| r.span.end - r.span.start)
                .cloned()
        }) else {
            return Vec::new();
        };
        // A module's name (`B` of `B.print`): the start of its entry file.
        if let IdeWhat::Module(m) = r.what {
            return self.modules[m.0 as usize]
                .files
                .first()
                .map(|&file| Span {
                    file,
                    start: 0,
                    end: 0,
                })
                .into_iter()
                .collect();
        }
        let name = self.sources.snippet(r.span).to_string();
        let spans: Vec<(Span, String)> = match &r.what {
            IdeWhat::Entity(e) => vec![(self.entity(*e).span, name)],
            // Each under its own name: an overload set can include aliases.
            // A call that resolved to one overload goes to that one.
            IdeWhat::Procs(ps) => match self.ide_resolved_overload(r.span, ps) {
                Some(p) => vec![(self.proc(p).span, self.proc(p).name.to_string())],
                None => ps
                    .iter()
                    .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
                    .collect(),
            },
            IdeWhat::Type(t) => match self.types.kind(*t) {
                TypeKind::Struct(s) => vec![(self.types.struct_info(*s).span, name)],
                TypeKind::Enum(e) => vec![(self.types.enum_info(*e).span, name)],
                _ => match self.ide.as_ref().and_then(|i| i.type_names.get(&r.span)) {
                    Some(&e) => vec![(self.entity(e).span, name)],
                    None => Vec::new(),
                },
            },
            IdeWhat::Member(_) | IdeWhat::Module(_) => Vec::new(),
        };
        self.ide_narrow_decls(spans)
    }

    /// The name inside each declaration span (see `ide_definition`); spans in no file drop out.
    pub fn ide_narrow_decls(&self, spans: Vec<(Span, String)>) -> Vec<Span> {
        spans
            .into_iter()
            // Builtins such as `OS` are declared nowhere: `Span::NONE` has no file. The slicing
            // below needs a range of whole characters in it.
            .filter(|(s, _)| {
                self.sources
                    .try_get(s.file)
                    .is_some_and(|f| f.text.get(s.start as usize..s.end as usize).is_some())
            })
            .map(|(s, name)| {
                let text = self.sources.snippet(s);
                if let Some(at) = text.find(name.as_str())
                    && text[..at].trim().is_empty()
                {
                    return Span {
                        file: s.file,
                        start: s.start + at as u32,
                        end: s.start + (at + name.len()) as u32,
                    };
                }
                // A loop's span covers `for name: ...`: its `name :` inside.
                let word = |c: char| c == '_' || c.is_alphanumeric();
                if text.starts_with("for")
                    && let Some((at, _)) = text.match_indices(name.as_str()).find(|&(at, _)| {
                        text[at + name.len()..].trim_start().starts_with(':')
                            && !text[..at].ends_with(word)
                            && !text[at + name.len()..].starts_with(word)
                    })
                {
                    return Span {
                        file: s.file,
                        start: s.start + at as u32,
                        end: s.start + (at + name.len()) as u32,
                    };
                }
                // A procedure's span starts at its literal: find `name :` earlier on the line.
                let file = &self.sources.get(s.file).text;
                let line = file[..s.start as usize].rfind('\n').map_or(0, |n| n + 1);
                let found = file[line..s.start as usize]
                    .rmatch_indices(name.as_str())
                    .find(|&(at, _)| {
                        let rest = &file[line + at + name.len()..];
                        rest.trim_start().starts_with(':')
                            && !file[..line + at]
                                .ends_with(|c: char| c == '_' || c.is_alphanumeric())
                    });
                match found {
                    Some((at, _)) => Span {
                        file: s.file,
                        start: (line + at) as u32,
                        end: (line + at + name.len()) as u32,
                    },
                    None => Span {
                        end: s.start,
                        ..s
                    },
                }
            })
            .collect()
    }

    /// The declarations `names` (`print`, `Basic.print`, `Type.field`, `Enum.MEMBER`) name
    /// from `scope`, as the name spans `ide_definition` gives. Doc comment links use it.
    pub fn ide_resolve_path(&mut self, scope: ScopeId, names: &[Sym]) -> Vec<Span> {
        let Some((last, first)) = names.split_last() else {
            return Vec::new();
        };
        let found = if first.is_empty() {
            self.lookup_full(scope, *last).ok()
        } else {
            match self.ide_receiver(scope, first) {
                Some(IdeReceiver::Module(m)) => self.module_member_lookup(m, *last).ok(),
                Some(IdeReceiver::Type(t) | IdeReceiver::Value(t)) => {
                    let member = self.ide_member_decl(t, *last);
                    if member.is_some() {
                        return member.into_iter().collect();
                    }
                    return self.ide_struct_scope_decls(t, *last);
                }
                None => return self.ide_poly_struct_member(scope, first, *last),
            }
        };
        let Some(Found::Entities(ids)) = found else {
            return Vec::new();
        };
        let spans = ids
            .into_iter()
            .map(|id| (self.entity(id).span, last.to_string()))
            .collect();
        self.ide_narrow_decls(spans)
    }

    /// `[Ring.items]` where `Ring` is a polymorphic struct: a parameter or a declaration of its
    /// body, found in its source, so it resolves whether or not the program instantiates it.
    fn ide_poly_struct_member(&mut self, scope: ScopeId, path: &[Sym], name: Sym) -> Vec<Span> {
        let [only] = path else {
            return Vec::new();
        };
        let Ok(Found::Entities(ids)) = self.lookup_full(scope, *only) else {
            return Vec::new();
        };
        let Some(&id) = ids.first() else {
            return Vec::new();
        };
        let Ok(Resolved::PolyStruct(p)) = self.resolve_entity(id) else {
            return Vec::new();
        };
        let lit = self.poly_structs[p.0 as usize].lit.clone();
        let params = lit.params.iter().filter_map(|p| p.name.as_ref());
        let fields = lit.body.iter().flat_map(|stmt| match &stmt.kind {
            ast::StmtKind::Decl(decl) => decl.names.as_slice(),
            _ => &[],
        });
        params
            .chain(fields)
            .filter(|ident| ident.name == name)
            .map(|ident| ident.span)
            .take(1)
            .collect()
    }

    /// What `name` is inside the struct `t`: constants, procedures and types declared in its
    /// body (`Type :: struct { CONST :: 3; }` is `Type.CONST`).
    fn ide_struct_scope_decls(&self, t: TypeId, name: Sym) -> Vec<Span> {
        let t = self.ide_deref(t);
        let spans = self
            .struct_scope_entities(t, name)
            .into_iter()
            .map(|id| (self.entity(id).span, name.to_string()))
            .collect();
        self.ide_narrow_decls(spans)
    }

    /// The entities named `name` in the scopes of the struct `t` (a polymorphic struct has one
    /// per instance; they all declare the same names).
    fn struct_scope_entities(&self, t: TypeId, name: Sym) -> Vec<EntityId> {
        let mut out: Vec<EntityId> = Vec::new();
        for i in 0..self.scopes.len() {
            let s = ScopeId(i as u32);
            if self.scope(s).kind != ScopeKind::Struct(t) {
                continue;
            }
            if let Some(ids) = self.scope(s).names.get(&name) {
                out.extend(ids.iter().copied().filter(|&id| {
                    matches!(
                        &self.entity(id).kind,
                        EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Const
                    )
                }));
            }
        }
        out
    }

    fn ide_proc_header(&self, name: Sym, p: ProcId) -> String {
        format!("{name} :: {}", clip(&self.ide_header_text(p)))
    }

    /// A procedure's header as written, less the parameters `#bake_arguments` fixed.
    pub(crate) fn ide_header_text(&self, p: ProcId) -> String {
        let header = &self.proc(p).lit.header;
        let text = self.ide_snippet(header.span);
        if !header.baked {
            return text.to_string();
        }
        // The parameter list is the first parenthesized group; the rest follows it.
        let mut depth = 0;
        let close = text.char_indices().find_map(|(i, c)| {
            match c {
                '(' => depth += 1,
                ')' if depth == 1 => return Some(i),
                ')' => depth -= 1,
                _ => {}
            }
            None
        });
        let Some(close) = close else {
            return text.to_string();
        };
        let open = text.find('(').unwrap_or(0);
        let params: Vec<&str> = header
            .params
            .iter()
            .map(|param| self.ide_snippet(param.span).trim())
            .collect();
        format!(
            "{}({}){}",
            &text[..open],
            params.join(", "),
            &text[close + 1..]
        )
    }

    fn ide_type_hover(&self, name: &str, t: TypeId) -> String {
        match self.types.kind(t) {
            TypeKind::Struct(s) => {
                let info = self.types.struct_info(*s);
                let fields: Vec<String> = info
                    .fields
                    .iter()
                    .filter_map(|f| Some(format!("{}: {};", f.name?, self.types.name(f.ty))))
                    .take(16)
                    .collect();
                let keyword = if info.is_union {
                    "union"
                } else {
                    "struct"
                };
                format!("{name} :: {keyword} {{ {} }}", fields.join(" "))
            }
            TypeKind::Enum(e) => {
                let info = self.types.enum_info(*e);
                let members: Vec<String> = info
                    .members
                    .iter()
                    .take(16)
                    .map(|(n, _)| format!("{n};"))
                    .collect();
                let keyword = if info.is_flags {
                    "enum_flags"
                } else {
                    "enum"
                };
                format!(
                    "{name} :: {keyword} {} {{ {} }}",
                    self.types.name(info.base),
                    members.join(" ")
                )
            }
            _ => {
                let full = self.types.name(t);
                if full == name {
                    format!("{name} :: Type")
                } else {
                    format!("{name} :: {full}")
                }
            }
        }
    }

    fn ide_entity_hover(&mut self, id: EntityId, ty: Option<TypeId>) -> String {
        let name = self.entity(id).name;
        let resolved = match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty, ..
            } => return format!("{name}: {}", self.types.name(ty)),
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => return self.ide_type_hover(name.as_str(), t),
            EntityKind::Const {
                ty, ..
            } => return format!("{name} :: {}", self.types.name(ty)),
            EntityKind::Placeholder => return format!("{name} :: #placeholder"),
            EntityKind::Builtin(_) => {
                return format!(
                    "{name}: {}",
                    self.types.name(ty.unwrap_or(TypeId::COMPILE_TIME))
                );
            }
            _ => self.resolve_entity(id).ok(),
        };
        match resolved {
            Some(Resolved::Const {
                value: value::Value::Type(t),
                ..
            }) => self.ide_type_hover(name.as_str(), t),
            Some(Resolved::Const {
                value: value::Value::Proc(p),
                ..
            })
            | Some(Resolved::Proc(p)) => self.ide_proc_header(name, p),
            Some(Resolved::ProcSet(ps)) => ps
                .iter()
                .map(|&p| self.ide_proc_header(self.proc(p).name, p))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(Resolved::Const {
                ty,
                value,
            }) => format!(
                "{name} :: {} = {}",
                self.types.name(ty),
                value.render(&self.types)
            ),
            Some(Resolved::Global {
                ty, ..
            }) => format!("{name}: {}", self.types.name(ty)),
            Some(Resolved::Module(m)) => {
                format!("{name} :: #import \"{}\"", self.modules[m.0 as usize].name)
            }
            Some(Resolved::Library(_)) => format!("{name} :: #library"),
            Some(Resolved::PolyStruct(_)) => format!("{name} :: struct (polymorphic)"),
            None => format!(
                "{name}: {}",
                self.types.name(ty.unwrap_or(TypeId::COMPILE_TIME))
            ),
        }
    }

    /// The declaration to describe a name by: a procedure if the name has overloads (an alias
    /// like `print :: print_to_builder;` may come first).
    fn ide_pick(&self, ids: impl Iterator<Item = EntityId>) -> Option<EntityId> {
        let ids: Vec<EntityId> = ids.collect();
        let is_proc = |&e: &EntityId| match &self.entity(e).kind {
            EntityKind::Decl {
                decl, ..
            } => matches!(
                decl.value.as_ref().map(|v| &v.kind),
                Some(ast::ExprKind::Proc(_))
            ),
            _ => false,
        };
        ids.iter()
            .copied()
            .find(is_proc)
            .or_else(|| ids.first().copied())
    }

    /// Kind and one-line detail of an entity for a completion list. Declarations not resolved
    /// yet are classified by their syntax (resolving every export of a module would compile it).
    fn ide_entity_name(&self, id: EntityId) -> Option<IdeName> {
        let e = self.entity(id);
        let name = e.name.as_str();
        if name.is_empty() || name.starts_with(['\0', '`']) || name.contains(' ') {
            return None;
        }
        let (kind, detail) = match &e.kind {
            EntityKind::EnumMember {
                ty, ..
            } => (IdeKind::Constant, self.types.name(*ty)),
            EntityKind::Local {
                ty, ..
            } => (IdeKind::Variable, self.types.name(*ty)),
            EntityKind::Const {
                value: value::Value::Type(_),
                ..
            } => (IdeKind::Type, "Type".into()),
            EntityKind::Const {
                ty, ..
            } => (IdeKind::Constant, self.types.name(*ty)),
            EntityKind::Placeholder | EntityKind::UnboundGenericParameter => return None,
            EntityKind::Import(_) => (IdeKind::Module, "module".into()),
            EntityKind::Builtin(b) => match b {
                scope::Builtin::Type(_) => (IdeKind::Type, "builtin type".into()),
                scope::Builtin::Proc(_) => (IdeKind::Function, "builtin".into()),
                scope::Builtin::TargetConstant(_) => (IdeKind::Constant, "builtin constant".into()),
            },
            EntityKind::Decl {
                decl, ..
            } => match &e.state {
                EntityState::Done(Resolved::Proc(p))
                | EntityState::Done(Resolved::Const {
                    value: value::Value::Proc(p),
                    ..
                }) => (IdeKind::Function, clip(&self.ide_header_text(*p))),
                EntityState::Done(Resolved::ProcSet(procedures)) => (
                    IdeKind::Function,
                    procedures
                        .iter()
                        .map(|p| clip(&self.ide_header_text(*p)))
                        .collect::<Vec<_>>()
                        .join(" | "),
                ),
                EntityState::Done(Resolved::Global {
                    ty, ..
                }) => (IdeKind::Variable, self.types.name(*ty)),
                EntityState::Done(Resolved::Const {
                    value: value::Value::Type(_),
                    ..
                }) => (IdeKind::Type, "Type".into()),
                EntityState::Done(Resolved::Const {
                    ty, ..
                }) => (IdeKind::Constant, self.types.name(*ty)),
                EntityState::Done(Resolved::Module(_)) => (IdeKind::Module, "module".into()),
                _ => {
                    use ast::ExprKind as E;
                    let kind = match (decl.kind, decl.value.as_ref().map(|v| &v.kind)) {
                        (_, Some(E::Proc(_))) => IdeKind::Function,
                        (_, Some(E::Struct(_) | E::Enum(_) | E::ProcType(_))) => IdeKind::Type,
                        (ast::DeclKind::Var, _) => IdeKind::Variable,
                        _ => IdeKind::Constant,
                    };
                    let detail = match decl.value.as_ref().map(|v| &v.kind) {
                        Some(E::Proc(lit)) => clip(self.sources.snippet(lit.header.span)),
                        _ => clip(self.sources.snippet(e.span)),
                    };
                    (kind, detail)
                }
            },
        };
        Some(IdeName {
            name: name.to_string(),
            kind,
            detail,
            decl: e.span,
        })
    }

    /// Names visible at `offset` of `file` from `scope`: locals declared before the offset,
    /// enclosing declarations, imported modules' exports, Preload.
    pub fn ide_visible(&mut self, scope: ScopeId, file: FileId, offset: u32) -> Vec<IdeName> {
        let mut out: Vec<IdeName> = Vec::new();
        let mut seen: HashSet<Sym> = HashSet::default();
        let depth = self.scope(scope).proc_depth;
        let mut modules = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            let _ = self.expand_pending(sid);
            let mut names: Vec<(Sym, EntityId)> = self
                .scope(sid)
                .names
                .iter()
                .filter_map(|(n, ids)| Some((*n, self.ide_pick(ids.iter().copied())?)))
                .collect();
            names.sort_by_key(|(n, _)| n.as_str());
            for (name, id) in names {
                let e = self.entity(id);
                if let EntityKind::Local {
                    depth: d, ..
                } = e.kind
                {
                    // Locals of enclosing procedures are out of reach; later ones not declared yet.
                    if d != depth || (e.span.file == file && e.span.start > offset) {
                        continue;
                    }
                }
                if seen.contains(&name) {
                    continue;
                }
                if let Some(n) = self.ide_entity_name(id) {
                    seen.insert(name);
                    out.push(n);
                }
            }
            let imports = self.scope(sid).imports.len();
            for i in 0..imports {
                if let Ok(Some(m)) = self.import_module(sid, i) {
                    modules.push((m, Some(self.scope(sid).imports[i].filter.clone())));
                }
            }
            for entry in self.scope(sid).usings.clone() {
                match entry {
                    scope::UsingEntry::Module(m, filter) => {
                        modules.push((m, filter.as_deref().cloned()))
                    }
                    scope::UsingEntry::Place {
                        ty, ..
                    } => {
                        for n in self.ide_members(IdeReceiver::Value(ty)) {
                            if seen.insert(Sym::intern(&n.name)) {
                                out.push(n);
                            }
                        }
                    }
                    scope::UsingEntry::Type(ty) => {
                        for n in self.ide_members(IdeReceiver::Type(ty)) {
                            if seen.insert(Sym::intern(&n.name)) {
                                out.push(n);
                            }
                        }
                    }
                    scope::UsingEntry::Filtered(..) => {
                        for n in self.ide_using_members(&entry) {
                            if seen.insert(Sym::intern(&n.name)) {
                                out.push(n);
                            }
                        }
                    }
                }
            }
            if self.scope(sid).kind == ScopeKind::Module {
                modules.extend(
                    [self.preload, self.runtime_support]
                        .into_iter()
                        .flatten()
                        .map(|m| (m, None)),
                );
            }
            current = self.scope(sid).parent;
        }
        while let Some((m, filter)) = modules.pop() {
            let mut names = self.ide_module_exports(m, &mut HashSet::default());
            if let Some(filter) = filter {
                Self::ide_filter_names(&mut names, &filter);
            }
            for n in names {
                if seen.insert(Sym::intern(&n.name)) {
                    out.push(n);
                }
            }
        }
        out
    }

    /// Exported names, applying each edge filter before merging transitive names.
    /// Only the active path is guarded: another filtered path to the same module
    /// may expose additional names and must still be traversed.
    fn ide_module_exports(
        &mut self,
        m: ModuleId,
        visiting: &mut HashSet<ModuleId>,
    ) -> Vec<IdeName> {
        if !visiting.insert(m) {
            return Vec::new();
        }
        let scope = self.modules[m.0 as usize].scope;
        let _ = self.expand_pending(scope);
        let mut ids: Vec<EntityId> = self
            .scope(scope)
            .names
            .values()
            .filter_map(|ids| {
                self.ide_pick(ids.iter().copied().filter(|&e| self.entity(e).exported))
            })
            .collect();
        ids.sort_by_key(|&e| self.entity(e).name.as_str());
        let mut names = Vec::new();
        for id in ids {
            let entity = self.entity(id);
            if entity.file_private {
                continue;
            }
            // Facades often export procedures through aliases. Syntax alone calls
            // these constants, even when their targets are overload sets. Resolve
            // only simple aliases through the usual lookup/visibility rules; do
            // not eagerly compile every unrelated declaration or procedure body.
            if matches!(entity.state, EntityState::Unresolved)
                && let EntityKind::Decl {
                    decl, ..
                } = &entity.kind
                && decl.kind == ast::DeclKind::Const
                && matches!(
                    decl.value.as_ref().map(|v| &v.kind),
                    Some(ast::ExprKind::Ident(_) | ast::ExprKind::Member(..))
                )
            {
                let _ = self.resolve_entity(id);
            }
            if let Some(name) = self.ide_entity_name(id) {
                names.push(name);
            }
        }
        for index in self.modules[m.0 as usize].exported_using_imports.clone() {
            if let Ok(Some(inner)) = self.import_module(scope, index) {
                let mut inherited = self.ide_module_exports(inner, visiting);
                Self::ide_filter_names(&mut inherited, &self.scope(scope).imports[index].filter);
                names.extend(inherited);
            }
        }
        for entry in self.modules[m.0 as usize].exported_usings.clone() {
            names.extend(self.ide_using_members_inner(&entry, visiting));
        }
        visiting.remove(&m);
        let mut seen = HashSet::default();
        names.retain(|n| seen.insert(n.name.clone()));
        names
    }

    fn ide_filter_names(names: &mut Vec<IdeName>, filter: &ast::UsingFilter) {
        match filter {
            ast::UsingFilter::Map(pairs) => {
                *names = pairs
                    .iter()
                    .filter_map(|(new, old)| {
                        let mut n = names.iter().find(|n| n.name == old.name.as_str())?.clone();
                        n.name = new.name.to_string();
                        Some(n)
                    })
                    .collect();
            }
            filter => names.retain(|n| !scope::filter_hides(filter, Sym::intern(&n.name))),
        }
    }

    fn ide_using_members(&mut self, entry: &scope::UsingEntry) -> Vec<IdeName> {
        self.ide_using_members_inner(entry, &mut HashSet::default())
    }

    fn ide_filter_targets(&self, file: FileId, offset: u32) -> Option<(Span, Vec<EntityId>)> {
        self.ide
            .as_ref()?
            .filter_targets
            .iter()
            .filter(|(span, _)| span.file == file && span.start <= offset && offset <= span.end)
            .min_by_key(|(span, _)| span.end - span.start)
            .map(|(&span, ids)| (span, ids.clone()))
    }

    /// Static filters contain selector names, not ordinary expressions. Record an
    /// existing visible target without turning nonexistent selectors into errors
    /// (the language permits them) or inventing source declarations for builtins.
    pub(super) fn ide_note_filter_names(
        &mut self,
        entry: &scope::UsingEntry,
        filter: &ast::UsingFilter,
    ) {
        if self.ide.is_none() {
            return;
        }
        let (ast::UsingFilter::Only(names) | ast::UsingFilter::Except(names)) = filter else {
            return;
        };
        for &name in names {
            if self.ide_wants(name.span.file) {
                self.ide_note_filter_target(entry, name, &mut HashSet::default());
            }
        }
    }

    fn ide_note_filter_target(
        &mut self,
        entry: &scope::UsingEntry,
        name: ast::Ident,
        visiting: &mut HashSet<(ModuleId, Sym)>,
    ) {
        match entry {
            scope::UsingEntry::Module(module, filter) => {
                let target = match filter.as_deref() {
                    Some(filter) => super::using::filter_target(filter, name.name),
                    None => Some(name.name),
                };
                let Some(target) = target else {
                    return;
                };
                if !visiting.insert((*module, target)) {
                    return;
                }
                match self.module_member_lookup(*module, target) {
                    Ok(Found::Entities(ids)) if !ids.is_empty() => {
                        if let Some(ide) = self.ide.as_mut() {
                            if ids.len() > 1 {
                                ide.filter_targets.insert(name.span, ids);
                            } else {
                                ide.refs.push(IdeRef {
                                    span: name.span,
                                    what: IdeWhat::Entity(ids[0]),
                                    ty: TypeId::COMPILE_TIME,
                                    decl: false,
                                });
                            }
                        }
                    }
                    Ok(Found::Using(entry, target)) => self.ide_note_filter_target(
                        &entry,
                        ast::Ident {
                            name: target,
                            span: name.span,
                        },
                        visiting,
                    ),
                    _ => {}
                }
            }
            scope::UsingEntry::Filtered(base, filter) => {
                if let Some(target) = super::using::filter_target(filter, name.name) {
                    self.ide_note_filter_target(
                        base,
                        ast::Ident {
                            name: target,
                            span: name.span,
                        },
                        visiting,
                    );
                }
            }
            scope::UsingEntry::Type(ty)
            | scope::UsingEntry::Place {
                ty, ..
            } => {
                if let (Some(decl), Some(ty)) = (
                    self.ide_member_decl(*ty, name.name),
                    self.ide_field_type(*ty, name.name),
                ) {
                    self.ide_note_named_field(name.span, name.name, decl, ty);
                }
            }
        }
    }

    fn ide_using_members_inner(
        &mut self,
        entry: &scope::UsingEntry,
        visiting: &mut HashSet<ModuleId>,
    ) -> Vec<IdeName> {
        match entry {
            scope::UsingEntry::Place {
                ty, ..
            } => self.ide_members(IdeReceiver::Value(*ty)),
            scope::UsingEntry::Type(ty) => self.ide_members(IdeReceiver::Type(*ty)),
            scope::UsingEntry::Module(m, filter) => {
                let mut names = self.ide_module_exports(*m, visiting);
                if let Some(filter) = filter {
                    Self::ide_filter_names(&mut names, filter);
                }
                names
            }
            scope::UsingEntry::Filtered(base, filter) => {
                let mut names = self.ide_using_members_inner(base, visiting);
                Self::ide_filter_names(&mut names, filter);
                names
            }
        }
    }

    /// What `names[0].names[1]...` is, looked up from `scope`.
    pub fn ide_receiver(&mut self, scope: ScopeId, names: &[Sym]) -> Option<IdeReceiver> {
        let (first, rest) = names.split_first()?;
        let mut recv = match self.lookup_full(scope, *first).ok()? {
            Found::Entities(ids) => self.ide_entity_receiver(*ids.first()?)?,
            Found::Using(entry, member) => {
                let ty = entry.member_type()?;
                IdeReceiver::Value(self.ide_field_type(ty, member)?)
            }
        };
        for name in rest {
            recv = match recv {
                IdeReceiver::Value(t) => IdeReceiver::Value(self.ide_field_type(t, *name)?),
                IdeReceiver::Type(t) => {
                    // A type declared in the struct (`Outer.Inner`) is the receiver; else
                    // `Enum.MEMBER.` or a struct constant: the value has the type itself.
                    let nested = self
                        .struct_scope_entities(self.ide_deref(t), *name)
                        .first()
                        .and_then(|&id| self.ide_entity_receiver(id));
                    match nested {
                        Some(r @ IdeReceiver::Type(_)) => r,
                        _ => IdeReceiver::Value(t),
                    }
                }
                IdeReceiver::Module(m) => match self.module_member_lookup(m, *name).ok()? {
                    Found::Entities(ids) => self.ide_entity_receiver(*ids.first()?)?,
                    Found::Using(entry, member) => {
                        IdeReceiver::Value(self.ide_field_type(entry.member_type()?, member)?)
                    }
                },
            };
        }
        Some(recv)
    }

    fn ide_entity_receiver(&mut self, id: EntityId) -> Option<IdeReceiver> {
        Some(match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty, ..
            } => IdeReceiver::Value(ty),
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => IdeReceiver::Type(t),
            EntityKind::Const {
                ty, ..
            } => IdeReceiver::Value(ty),
            EntityKind::Builtin(scope::Builtin::Type(t)) => IdeReceiver::Type(t),
            _ => match self.resolve_entity(id).ok()? {
                Resolved::Const {
                    value: value::Value::Type(t),
                    ..
                } => IdeReceiver::Type(t),
                Resolved::Const {
                    ty, ..
                }
                | Resolved::Global {
                    ty, ..
                } => IdeReceiver::Value(ty),
                Resolved::Module(m) => IdeReceiver::Module(m),
                _ => return None,
            },
        })
    }

    /// Strip pointers and distinct wrappers: what `.` reaches.
    fn ide_deref(&self, mut t: TypeId) -> TypeId {
        for _ in 0..8 {
            t = match self.types.kind(t) {
                TypeKind::Pointer(inner) => *inner,
                TypeKind::Distinct(d) => self.types.distincts[d.0 as usize].base,
                _ => return t,
            };
        }
        t
    }

    fn ide_field_type(&mut self, t: TypeId, name: Sym) -> Option<TypeId> {
        self.ide_members(IdeReceiver::Value(t))
            .into_iter()
            .find(|n| n.name == name.as_str())
            .and_then(|_| self.ide_member_type(t, name))
    }

    fn ide_member_type(&mut self, t: TypeId, name: Sym) -> Option<TypeId> {
        let t = self.ide_deref(t);
        match self.types.kind(t) {
            TypeKind::Struct(s) => {
                for f in self.types.struct_info(*s).fields.clone() {
                    if f.name == Some(name) {
                        return Some(f.ty);
                    }
                    if f.using
                        && let Some(inner) = self.ide_member_type(f.ty, name)
                    {
                        return Some(inner);
                    }
                }
                None
            }
            TypeKind::String
            | TypeKind::Array {
                ..
            } => match name.as_str() {
                "count" | "allocated" => Some(TypeId::S64),
                "data" => Some(match self.types.kind(t).clone() {
                    TypeKind::Array {
                        elem, ..
                    } => self.types.pointer(elem),
                    _ => TypeId::U8_PTR,
                }),
                _ => None,
            },
            TypeKind::Any => match name.as_str() {
                "value_pointer" => Some(TypeId::VOID_PTR),
                _ => None,
            },
            _ => None,
        }
    }

    /// Members reachable with `.` on a value or type.
    pub fn ide_members(&mut self, recv: IdeReceiver) -> Vec<IdeName> {
        let mut out = Vec::new();
        match recv {
            IdeReceiver::Module(m) => return self.ide_module_exports(m, &mut HashSet::default()),
            IdeReceiver::Type(t) => {
                if let TypeKind::Enum(e) = self.types.kind(t) {
                    let info = self.types.enum_info(*e);
                    let strict = info.loose_of.map_or(*e, |t| match self.types.kind(t) {
                        TypeKind::Enum(strict) => *strict,
                        _ => *e,
                    });
                    for (n, v) in &info.members {
                        out.push(IdeName {
                            name: n.to_string(),
                            kind: IdeKind::EnumMember,
                            detail: format!("{} = {v}", info.name),
                            decl: self
                                .ide
                                .as_ref()
                                .and_then(|ide| ide.enum_decls.get(&(strict, *n)).copied())
                                .unwrap_or(Span::NONE),
                        });
                    }
                }
                // Struct constants live in the struct's scope.
                let scopes: Vec<ScopeId> = (0..self.scopes.len())
                    .map(|i| ScopeId(i as u32))
                    .filter(|&s| self.scope(s).kind == ScopeKind::Struct(t))
                    .collect();
                for s in scopes {
                    let ids: Vec<EntityId> = self
                        .scope(s)
                        .names
                        .values()
                        .filter_map(|v| v.first().copied())
                        .collect();
                    for id in ids {
                        if let EntityKind::Decl {
                            decl, ..
                        } = &self.entity(id).kind
                            && decl.kind == ast::DeclKind::Const
                            && let Some(n) = self.ide_entity_name(id)
                        {
                            out.push(n);
                        }
                    }
                }
            }
            IdeReceiver::Value(t) => {
                let t = self.ide_deref(t);
                match self.types.kind(t).clone() {
                    TypeKind::Struct(s) => {
                        let fields = self.types.struct_info(s).fields.clone();
                        for f in fields {
                            if let Some(n) = f.name {
                                out.push(IdeName {
                                    name: n.to_string(),
                                    kind: IdeKind::Field,
                                    detail: self.types.name(f.ty),
                                    decl: f.span,
                                });
                            }
                            if f.using {
                                out.extend(self.ide_members(IdeReceiver::Value(f.ty)));
                            }
                        }
                    }
                    TypeKind::String => {
                        for (n, d) in [("count", "s64"), ("data", "*u8")] {
                            out.push(field(n, d));
                        }
                    }
                    TypeKind::Array {
                        elem,
                        kind,
                    } => {
                        let elem = self.types.name(elem);
                        out.push(field("count", "s64"));
                        out.push(field("data", &format!("*{elem}")));
                        if kind == ArrayKind::Resizable {
                            out.push(field("allocated", "s64"));
                            out.push(field("allocator", "Allocator"));
                        }
                    }
                    TypeKind::Any => {
                        out.push(field("type", "*Type_Info"));
                        out.push(field("value_pointer", "*void"));
                    }
                    TypeKind::Enum(_) => return self.ide_members(IdeReceiver::Type(t)),
                    _ => {}
                }
            }
        }
        out
    }
}

fn field(name: &str, detail: &str) -> IdeName {
    IdeName {
        name: name.into(),
        kind: IdeKind::Field,
        detail: detail.into(),
        decl: Span::NONE,
    }
}

/// First line of a snippet, bounded.
fn clip(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    let line = line.trim_end_matches('{').trim_end();
    if line.len() > 160 {
        let end = line.floor_char_boundary(160);
        format!("{}…", &line[..end])
    } else {
        line.to_string()
    }
}

/// The same thing named (a polymorphic body checked again for another instance may not be).
fn same_what(a: &IdeWhat, b: &IdeWhat) -> bool {
    match (a, b) {
        (IdeWhat::Entity(x), IdeWhat::Entity(y)) => x == y,
        (IdeWhat::Member(x), IdeWhat::Member(y)) => x == y,
        (IdeWhat::Type(x), IdeWhat::Type(y)) => x == y,
        (IdeWhat::Procs(x), IdeWhat::Procs(y)) => x == y,
        (IdeWhat::Module(x), IdeWhat::Module(y)) => x == y,
        _ => false,
    }
}

/// Every expression of the parameter and return types of `header`, outermost first.
pub(super) fn for_header_types(header: &ast::ProcHeader, f: &mut dyn FnMut(&ast::Expr)) {
    fn walk(e: &ast::Expr, f: &mut dyn FnMut(&ast::Expr)) {
        use ast::ExprKind as E;
        f(e);
        match &e.kind {
            E::Unary(_, x) => walk(x, f),
            E::PolyRestricted {
                restriction, ..
            } => walk(restriction, f),
            E::ArrayType {
                size,
                elem,
            } => {
                if let ast::ArraySize::Fixed(n) = size {
                    walk(n, f);
                }
                walk(elem, f);
            }
            E::Call {
                callee,
                args,
                ..
            } => {
                walk(callee, f);
                args.iter().for_each(|a| walk(&a.value, f));
            }
            E::ProcType(h) => for_header_types(h, f),
            _ => {}
        }
    }
    for t in header
        .params
        .iter()
        .filter_map(|p| p.ty.as_ref())
        .chain(header.returns.iter().filter_map(|r| r.ty.as_ref()))
    {
        walk(t, f);
    }
}

/// Does a condition mention `OS` or `CPU`?
fn names_target(expr: &ast::Expr) -> bool {
    use ast::ExprKind as E;
    match &expr.kind {
        E::Ident(name) => matches!(name.as_str(), "OS" | "CPU"),
        E::Binary(_, a, b) => names_target(a) || names_target(b),
        E::Unary(_, e) => names_target(e),
        _ => false,
    }
}

#[cfg(test)]
mod body_failure_tests {
    use super::*;

    fn failed() -> (Compiler, ProcId) {
        let mut fs = super::super::VirtualFs::default();
        fs.insert("/audit/main.jai", b"broken :: missing; failed :: () -> s64 { nested :: () {}; nested(); return absent; } replacement :: () -> s64 { return 7; } main :: () {}".as_slice());
        let mut options = Options::host();
        options.runtime_support = false;
        options.stack_trace = false;
        let mut compiler = Compiler::new(options, Rc::new(fs));
        compiler.ide = Some(Box::new(IdeFacts::new(vec!["/audit/".into()])));
        assert!(
            compiler
                .compile_program(std::path::Path::new("/audit/main.jai"))
                .is_err()
        );
        assert!(
            compiler
                .ide_check_all()
                .iter()
                .any(|e| e.message.contains("absent"))
        );
        let at = compiler
            .procs
            .iter()
            .position(|p| p.name.as_str() == "failed")
            .unwrap();
        (compiler, ProcId(at as u32))
    }

    #[test]
    fn replacing_body_retries_without_source_epoch_change() {
        let (mut compiler, id) = failed();
        assert!(compiler.lower_body(id).is_err());
        let replacement = compiler
            .procs
            .iter()
            .find(|p| p.name.as_str() == "replacement")
            .unwrap()
            .lit
            .clone();
        let header = compiler.proc(id).lit.header.clone();
        compiler.procs[id.0 as usize].lit = Rc::new(ast::ProcLit {
            header,
            body: replacement.body.clone(),
        });
        compiler.lower_body(id).unwrap();
        let Some(super::super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
            panic!("compiled target");
        };
        assert!(compiler.program.funcs[func.0 as usize].is_some());
    }

    #[test]
    fn replacing_a_parked_body_retries_during_the_next_audit() {
        let (mut compiler, id) = failed();
        compiler.ide_check_all();
        assert!(compiler.parked_bodies.contains(&id));
        let replacement = compiler
            .procs
            .iter()
            .find(|p| p.name.as_str() == "replacement")
            .unwrap()
            .lit
            .clone();
        let header = compiler.proc(id).lit.header.clone();
        compiler.procs[id.0 as usize].lit = Rc::new(ast::ProcLit {
            header,
            body: replacement.body.clone(),
        });
        let errors = compiler.ide_check_all();
        assert!(
            !errors.iter().any(|e| e.message.contains("absent")),
            "{errors:?}"
        );
        let Some(super::super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
            panic!("compiled target");
        };
        assert!(compiler.program.funcs[func.0 as usize].is_some());
    }

    #[test]
    fn adding_visible_binding_retries_without_loading_a_file() {
        let (mut compiler, id) = failed();
        let scope = compiler.scope(compiler.proc(id).scope).parent.unwrap();
        let name = Sym::intern("absent");
        compiler.add_entity(
            scope,
            name,
            Span::NONE,
            EntityKind::Const {
                value: Value::Int(7),
                ty: TypeId::S64,
            },
            false,
        );
        compiler.lower_body(id).unwrap();
        let Some(super::super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
            panic!("compiled target");
        };
        assert!(compiler.program.funcs[func.0 as usize].is_some());
    }

    #[test]
    fn removing_ide_facts_keeps_strict_lowering_unmemoized() {
        let (mut compiler, id) = failed();
        let before = compiler
            .program
            .func_names
            .iter()
            .filter(|n| n.as_str() == "nested")
            .count();
        compiler.ide = None;
        assert!(
            compiler
                .lower_body(id)
                .unwrap_err()
                .message
                .contains("absent")
        );
        let after = compiler
            .program
            .func_names
            .iter()
            .filter(|n| n.as_str() == "nested")
            .count();
        assert!(
            after > before,
            "strict lowering must really execute, not consult IDE failure facts"
        );
    }
}
