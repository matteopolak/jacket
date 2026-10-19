//! What an editor analysis records: references, hovers, layouts and the indexes over them.
use crate::ast;
use crate::fxhash::HashMap;
use crate::fxhash::HashSet;
use crate::intern::Sym;
use crate::sema::ModuleId;
use crate::sema::scope::{EntityId, ScopeId};
use crate::sema::value::ProcId;
use crate::source::Diagnostic;
use crate::source::FileId;
use crate::source::Span;
use crate::types::StructId;
use crate::types::TypeId;
use std::path::PathBuf;
use std::rc::Rc;

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

/// Which sources an analysis records facts for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdeRoot {
    /// A file, or every file in a folder and below it. Compared by path components, both as
    /// the source was named and canonically, so separators and `.`/`..` spellings agree.
    Under(PathBuf),
    /// Every source without a file on disk (`#import,string`, `add_build_string`, `<input>`).
    Virtual,
}

impl IdeRoot {
    pub fn under(path: impl Into<PathBuf>) -> Self {
        IdeRoot::Under(path.into())
    }
}

#[derive(Default)]
pub struct IdeFacts {
    /// Per-analysis missing filesystem discovery; never a placeholder or failed module body.
    pub(super) missing_modules: HashMap<(ScopeId, Span), IdeMissingModule>,
    /// The sources facts are recorded for.
    pub(super) roots: Vec<IdeRoot>,
    /// Files recorded wherever they are: the open documents, as `FileSystem::canonical` keys.
    pub(super) files: Vec<PathBuf>,
    pub(super) wanted: HashMap<FileId, bool>,
    /// The exact body associated with an IDE-final lenient failure. Replacing a
    /// procedure's AST invalidates that failure even without another source load.
    pub(crate) failed_body_lits: HashMap<ProcId, Rc<ast::ProcLit>>,
    pub refs: Vec<IdeRef>,
    /// Static filter selectors may denote an entire exported overload set.
    pub(super) filter_targets: HashMap<Span, Vec<EntityId>>,
    pub scopes: Vec<(Span, ScopeId)>,
    /// What `#insert`, `#run`, `#if` and macro calls produced (see `ide::meta`).
    pub expansions: Vec<crate::ide::meta::IdeExpansion>,
    /// Position in `expansions` of each (span, kind), so recording one is not a scan.
    pub(crate) expansion_index: HashMap<(Span, crate::ide::meta::IdeExpansionKind), usize>,
    /// Calls and the procedure and parameters they resolved to.
    pub calls: Vec<crate::ide::meta::IdeCall>,
    pub(crate) call_spans: HashSet<Span>,
    /// The procedure each direct call expression resolved to, in any file (`None`: checks of it
    /// disagreed, as instances of a polymorphic procedure can). See `never_returns.rs`.
    pub(crate) callees: HashMap<Span, Option<ProcId>>,
    /// Procedures whose body expanded a macro that returns from its caller.
    pub(crate) macro_returners: HashSet<ProcId>,
    /// Actual custom-iteration selections per keyword, across real instantiations.
    /// `None` means an ambiguous selection; sketches never populate this map.
    pub(crate) for_expansions: HashMap<Span, Option<Vec<ProcId>>>,
    /// The implicit `it` and `it_index` of loops that name neither, by the start of the iterated
    /// expression, with their types. `None`: instances of the loop disagree.
    pub(crate) implicit_loop_names: HashMap<Span, Option<Vec<(&'static str, TypeId)>>>,
    /// The compile-time output channel, to tell what each `#run` printed.
    pub output: Option<Rc<std::cell::RefCell<crate::interp::SandboxHost>>>,
    /// Entity the named expression being checked resolved to (set by identifier
    /// lookup or explicit module-member lookup, after resolving its operand).
    pub(crate) last_entity: Option<(Span, EntityId)>,
    /// Field the identifier being checked reached through a `using` (set by `check_ident`).
    pub(crate) last_using_member: Option<Sym>,
    /// Names of a type that is neither a struct nor an enum (a polymorph's `T`, an alias of a
    /// builtin): the constant each resolved to, for go to definition.
    pub(crate) type_names: HashMap<Span, EntityId>,
    /// Procedure type parameter names already recorded.
    pub(crate) param_names: HashSet<Span>,
    /// Where `ide_sketch_polymorphs` began: the refs, entities and procedures from there on
    /// come from sketches with stand-in types, which lints skip.
    pub sketch_from: Option<SketchMark>,
    /// The stand-in types of sketches: anything may be done with a value of one (see
    /// `ide_sketch`).
    pub(crate) stand_ins: HashSet<TypeId>,
    /// The stand-in for member `name` of stand-in type `t`.
    pub(crate) stand_in_members: HashMap<(TypeId, Sym), TypeId>,
    /// Macros (and instances of polymorphic ones) that were expanded somewhere.
    pub(crate) expanded_macros: HashSet<ProcId>,
    /// The sketch's own instance is being made, which binds stand-ins.
    pub(crate) sketch_root: bool,
    /// A sketch is being checked: an expression that fails on a stand-in becomes one.
    pub(crate) sketching: bool,
    /// The type of each expression the current sketch checked.
    pub(crate) sketch_types: HashMap<Span, TypeId>,
    /// Each expression the current sketch began to check, failed or not.
    pub(crate) sketch_tried: HashSet<Span>,
    /// Expressions whose check failed and has not succeeded since: the recovery of an enclosing
    /// expression does not check them again (a chain of `n` failing operators would take `2^n`).
    pub(crate) failed_exprs: HashSet<Span>,
    /// Instances a sketch made with stand-ins for a call's types: only their signatures are
    /// needed, so their bodies are never checked (a body would call on, without end).
    pub(crate) sketch_signatures: HashSet<ProcId>,
    /// The code an `#if` on `OS` or `CPU` did not take: another target's compile has its facts.
    pub skipped: Vec<Span>,
    /// The branch of each `#if` and `ifx` of the recorded files, and whether any evaluation
    /// ran it.
    pub static_branches: HashMap<Span, bool>,
    /// What each checked integer or bool expression of the recorded files can evaluate to
    /// (see `reachability.rs`), for the last evaluation of it.
    pub(crate) ainfo: HashMap<Span, crate::sema::reachability::AInfo>,
    /// Where each local was assigned.
    pub(crate) assigned: HashMap<EntityId, Vec<Span>>,
    /// Locals whose address was taken, or that a macro may write.
    pub(crate) addr_taken: HashSet<EntityId>,
    /// How many times each procedure body (by span) was analysed for reachability, and for
    /// each run of dead code (body, run) how many of those found it dead.
    pub body_evals: HashMap<Span, u32>,
    pub dead_runs: HashMap<(Span, Span), u32>,
    /// `refs` by file, sorted by start, for position lookups (`refs_at`). Built on first use
    /// and again whenever `refs` has changed since (its length and buffer are the key).
    by_position: std::cell::RefCell<RefIndex>,
    /// Lint mode (`jailint`): also record the facts below.
    pub lint: bool,
    /// Workspace trust is withheld: no compile-time code runs (`call_thunk` refuses with
    /// `DiagnosticKind::Untrusted`), so `#run` results, computed `#insert`s and build
    /// metaprograms stay unknown.
    pub untrusted: bool,
    /// Where compile-time code was refused for that reason.
    pub refused: Vec<Span>,
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
    pub(crate) enum_decls: HashMap<(crate::types::EnumId, Sym), Span>,
    /// The types an inferred member (`.NAME`) of the recorded files was expected to have, by
    /// the name's span. Recorded whether or not the name exists, for completion.
    pub(crate) inferred_expected: HashMap<Span, Vec<TypeId>>,
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

    /// Also record `files` (open documents), whichever folder they are in.
    #[must_use]
    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        self.files = files;
        self
    }

    pub fn new(roots: Vec<IdeRoot>) -> Self {
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
    /// How a procedure is called, for completion to insert the call.
    pub call: Option<IdeCall>,
}

/// The parameters of a procedure (every overload of a name) as far as inserting a call needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdeCall {
    /// No overload takes a parameter.
    pub parameterless: bool,
    /// The names of the leading parameters a call must pass (those before the first one with a
    /// default value or `..` variadic), for one procedure; `None` when the name is overloaded
    /// or its parameters are not known.
    pub required: Option<Vec<String>>,
}

impl IdeCall {
    /// A procedure whose parameters are not known (a builtin).
    pub fn unknown() -> Self {
        Self {
            parameterless: false,
            required: None,
        }
    }

    /// The call shape of a name that is both `self` and `other` (overloads).
    #[must_use]
    pub fn overloaded(&self, other: &Self) -> Self {
        Self {
            parameterless: self.parameterless && other.parameterless,
            required: None,
        }
    }

    pub fn of_headers<'a>(headers: impl IntoIterator<Item = &'a ast::ProcHeader>) -> Self {
        let headers: Vec<&ast::ProcHeader> = headers.into_iter().collect();
        let required = match headers.as_slice() {
            [one] => Some(
                one.params
                    .iter()
                    .take_while(|p| p.default.is_none() && !p.variadic)
                    .map(|p| {
                        p.name
                            .as_ref()
                            .map_or_else(|| "arg".to_owned(), |n| n.name.as_str().to_owned())
                    })
                    .collect(),
            ),
            _ => None,
        };
        Self {
            parameterless: headers.iter().all(|h| h.params.is_empty()),
            required,
        }
    }
}

/// What `a.b.` refers to, for member completion.
#[derive(Clone, Copy, Debug)]
pub enum IdeReceiver {
    Value(TypeId),
    Type(TypeId),
    Module(ModuleId),
}


pub(super) type IdeMissingModuleEpoch = (usize, usize, usize, usize, u64);

pub(super) struct IdeMissingModule {
    pub(super) epoch: IdeMissingModuleEpoch,
    pub(super) wanted: String,
    pub(super) dir: PathBuf,
    pub(super) import_paths: Vec<PathBuf>,
    pub(super) preload: Option<PathBuf>,
    pub(super) error: Box<Diagnostic>,
}

/// The same thing named (a polymorphic body checked again for another instance may not be).
pub(super) fn same_what(a: &IdeWhat, b: &IdeWhat) -> bool {
    match (a, b) {
        (IdeWhat::Entity(x), IdeWhat::Entity(y)) => x == y,
        (IdeWhat::Member(x), IdeWhat::Member(y)) => x == y,
        (IdeWhat::Type(x), IdeWhat::Type(y)) => x == y,
        (IdeWhat::Procs(x), IdeWhat::Procs(y)) => x == y,
        (IdeWhat::Module(x), IdeWhat::Module(y)) => x == y,
        _ => false,
    }
}
