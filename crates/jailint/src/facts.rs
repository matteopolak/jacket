//! What the compiler recorded while checking, arranged for the rules.
//!
//! Turn recording on with [`enable`] before compiling. The compiler then keeps, for files under
//! the given roots, each expression's type, each cast, what each identifier names, and which
//! variables any name reached (`jaic::sema::ide`). After `ide_check_all`, [`Facts::collect`]
//! indexes them by source span; rules look facts up by the spans of the syntax tree they walk.
use jaic::fxhash::{HashMap, HashSet};
use jaic::sema::Compiler;
use jaic::sema::EntityId;
use jaic::sema::ProcId;
use jaic::sema::ide::{IdeCast, IdeExprFact, IdeFacts, IdeWhat};
use jaic::sema::procs::BodyState;
use jaic::sema::scope::{EntityKind, EntityState, Resolved};
use jaic::source::{FileId, Span};
use jaic::types::TypeId;

/// Record lint facts for files whose path starts with one of `roots` (an existing editor
/// record is extended).
pub fn enable(compiler: &mut Compiler, roots: Vec<String>) {
    let ide = compiler
        .ide
        .get_or_insert_with(|| Box::new(IdeFacts::new(roots)));
    ide.lint = true;
}

pub struct Facts {
    /// Uses of names (not declarations): the entities each identifier resolved to (several
    /// when a polymorphic body was checked once per instance).
    pub idents: HashMap<Span, Vec<EntityId>>,
    /// Runtime variables (locals, parameters, loop variables) by their declaring span: a
    /// local's name, a parameter's whole declaration, a loop's whole statement.
    pub locals: HashMap<Span, Vec<EntityId>>,
    /// Procedure bodies (by the block's span) every instance of which checked to the end.
    clean: HashSet<Span>,
    /// Procedures by their header's span (instances of a polymorphic one share it).
    pub procs_by_header: HashMap<Span, Vec<ProcId>>,
    /// Where each procedure is named in the recorded files.
    pub proc_uses: HashMap<ProcId, Vec<Span>>,
}

impl Facts {
    pub fn collect(compiler: &Compiler, files: &[FileId]) -> Option<Facts> {
        let ide = compiler.ide.as_ref()?;
        if !ide.lint {
            return None;
        }
        let wanted: HashSet<FileId> = files.iter().copied().collect();
        let mut idents: HashMap<Span, Vec<EntityId>> = HashMap::default();
        for r in &ide.refs {
            if r.decl || !wanted.contains(&r.span.file) {
                continue;
            }
            if let IdeWhat::Entity(e) = r.what {
                let list = idents.entry(r.span).or_default();
                if !list.contains(&e) {
                    list.push(e);
                }
            }
        }
        let mut locals: HashMap<Span, Vec<EntityId>> = HashMap::default();
        for (i, e) in compiler.entities.iter().enumerate() {
            if wanted.contains(&e.span.file) && matches!(e.kind, EntityKind::Local { .. }) {
                locals.entry(e.span).or_default().push(EntityId(i as u32));
            }
        }
        let mut done: HashSet<Span> = HashSet::default();
        let mut failed: HashSet<Span> = HashSet::default();
        for p in &compiler.procs {
            let Some(body) = &p.lit.body else {
                continue;
            };
            if !wanted.contains(&body.span.file) {
                continue;
            }
            match p.body_state {
                BodyState::Done => {
                    done.insert(body.span);
                }
                BodyState::Queued | BodyState::Lowering => {
                    failed.insert(body.span);
                }
                BodyState::NotNeeded => {}
            }
        }
        let clean = done.difference(&failed).copied().collect();
        let mut procs_by_header: HashMap<Span, Vec<ProcId>> = HashMap::default();
        for (i, p) in compiler.procs.iter().enumerate() {
            if wanted.contains(&p.lit.header.span.file) {
                procs_by_header
                    .entry(p.lit.header.span)
                    .or_default()
                    .push(ProcId(i as u32));
            }
        }
        let mut proc_uses: HashMap<ProcId, Vec<Span>> = HashMap::default();
        for r in &ide.refs {
            if r.decl {
                continue;
            }
            let named: Vec<ProcId> = match &r.what {
                IdeWhat::Procs(ps) => ps.clone(),
                IdeWhat::Entity(e) => match &compiler.entity(*e).state {
                    EntityState::Done(Resolved::Proc(p))
                    | EntityState::Done(Resolved::Const {
                        value: jaic::sema::Value::Proc(p),
                        ..
                    }) => vec![*p],
                    EntityState::Done(Resolved::ProcSet(ps)) => ps.clone(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };
            for p in named {
                proc_uses.entry(p).or_default().push(r.span);
            }
        }
        Some(Facts {
            idents,
            locals,
            clean,
            procs_by_header,
            proc_uses,
        })
    }

    /// The body (a block's span) was checked completely, in every instance.
    pub fn clean(&self, body: Span) -> bool {
        self.clean.contains(&body)
    }
}

/// Lookups on the compiler's records.
pub trait Recorded {
    fn expr_fact(&self, span: Span) -> Option<IdeExprFact>;
    fn cast_fact(&self, span: Span) -> Option<IdeCast>;
    fn is_used(&self, e: EntityId) -> bool;

    /// The type of the expression at `span`, when every check of it agreed.
    fn type_at(&self, span: Span) -> Option<TypeId> {
        self.expr_fact(span)
            .filter(|f| !f.conflicting)
            .map(|f| f.ty)
    }
}

impl Recorded for Compiler {
    fn expr_fact(&self, span: Span) -> Option<IdeExprFact> {
        self.ide.as_ref()?.exprs.get(&span).copied()
    }

    fn cast_fact(&self, span: Span) -> Option<IdeCast> {
        self.ide.as_ref()?.casts.get(&span).copied()
    }

    fn is_used(&self, e: EntityId) -> bool {
        self.ide.as_ref().is_some_and(|i| i.used.contains(&e))
    }
}
