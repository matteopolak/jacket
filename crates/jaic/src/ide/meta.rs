//! Facts about metaprogramming and calls. While `Compiler::ed` records, checking records what
//! each `#insert`, `#run`, `#if` and `#expand` macro call of the user's files produced, and which
//! procedure and parameters each call resolved to. The language server builds expansion hovers,
//! inlay hints, signature help and format-string hovers from these.
mod call_info;
mod classes;
mod expansions;
mod notes;
mod references;
mod text;

pub use self::text::dedent;
use crate::intern::Sym;
use crate::sema::lower::Operand;
use crate::sema::scope::EntityId;
use crate::sema::value::ProcId;
use crate::sema::{Compiler, ModuleId, ProcTypeParams, Value};
use crate::source::FileId;
use crate::source::Span;
use crate::types::TypeId;
use std::rc::Rc;

/// Expansions kept per analysis: an edit that makes a metaprogram generate a lot stays bounded.
const MAX_EXPANSIONS: usize = 4096;

/// Calls kept per analysis.
const MAX_CALLS: usize = 16384;

/// Distinct results kept for one site (a polymorphic body can expand differently per instance).
const MAX_VARIANTS: usize = 4;

/// Longest expansion text kept.
const MAX_TEXT: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IdeExpansionKind {
    /// `#insert`: the code inserted.
    Insert,
    /// `#run`: the value computed.
    Run,
    /// `#if` / `#ifx` / `#assert`: whether the condition held.
    If,
    /// A call of an `#expand` procedure: its body with the arguments substituted.
    Macro,
}

#[derive(Clone, Debug)]
pub struct IdeExpansion {
    /// From the directive (or the macro call's callee) to the end of its operand.
    pub span: Span,
    pub kind: IdeExpansionKind,
    /// Distinct results, in the order they were seen (several when a polymorphic body or a
    /// macro inside one expands differently per instance).
    pub texts: Vec<String>,
    /// `#run`: the value's type. Macro: its name.
    pub detail: String,
    /// `#run`: what the compile-time code printed.
    pub output: String,
    /// `#run`: the value as source text, when it is a scalar, string or type (safe to inline).
    pub literal: Option<String>,
}

/// A string literal passed to a parameter of a procedure noted `@Interpolates`: its `%` holes
/// are code. Recorded even when the literal has none, so an editor knows what a `%` typed there
/// starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdeInterpolation {
    /// The literal, quotes (or the `#string` header and terminator) included.
    pub string: Span,
    pub mode: crate::interpolate::Mode,
}

#[derive(Clone, Debug)]
pub struct IdeCall {
    pub span: Span,
    /// The overload set the call chose from.
    pub candidates: Vec<ProcId>,
    /// The procedure called (an instance when the chosen one is polymorphic).
    pub chosen: ProcId,
    /// Several candidates fitted equally well and `chosen` is only the first of them.
    pub ambiguous: bool,
    pub args: Vec<IdeArg>,
}

#[derive(Clone, Debug)]
pub struct IdeArg {
    pub span: Span,
    /// Index into the chosen procedure's header parameters.
    pub param: usize,
    pub named: bool,
    pub variadic: bool,
    pub spread: bool,
    pub ty: Option<TypeId>,
}

/// A call, described for an editor.
#[derive(Clone, Debug)]
pub struct IdeCallInfo {
    pub span: Span,
    /// One label per candidate (`name :: (a: int, b: string) -> int`), and per candidate its
    /// parameters' source text.
    pub signatures: Vec<IdeSignature>,
    /// Index of the chosen procedure in `signatures`.
    pub active: usize,
    pub args: Vec<IdeArgInfo>,
    /// Header parameter that takes the format string, when the procedure is declared
    /// `@PrintLike`: its `string` parameter followed by a variadic `Any` one.
    pub format_param: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct IdeSignature {
    pub label: String,
    pub params: Vec<String>,
    pub param_names: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IdeArgInfo {
    pub span: Span,
    pub param: usize,
    pub param_name: String,
    pub named: bool,
    pub variadic: bool,
    pub spread: bool,
    pub ty: Option<String>,
}

/// How an editor colors an identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdeClass {
    Variable,
    Constant,
    Function,
    Macro,
    Type,
    Module,
    Field,
    EnumMember,
}

/// What a reference names, for find-references: overloads and aliases of one procedure, the
/// declaration of a type and its uses, the uses of a local.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Target {
    Entity(EntityId),
    /// A local, parameter or constant by where it is declared: a polymorphic procedure's or
    /// struct's parameter is one entity in each instance, and one more where a result type is
    /// checked.
    Declared(Span, Sym),
    Proc(ProcId),
    Type(TypeId),
    Module(ModuleId),
}

/// The checked callee's selected procedures and declaration-bound value header.
/// Structural type identity alone never supplies parameter names.
pub struct IdeCallTarget {
    pub procedures: Vec<ProcId>,
    pub parameters: Option<(TypeId, Rc<ProcTypeParams>)>,
}

impl Compiler {
    /// How to color the callee of the call whose `(` is at `open`: a macro if every candidate is
    /// one, a function otherwise. `None` when the call's target cannot be found.
    pub fn ide_callee_class(&mut self, file: FileId, open: u32) -> Option<IdeClass> {
        let procedures = self.ide_call_target_at(file, open)?.procedures;
        if procedures.is_empty() {
            return None;
        }
        Some(if procedures.iter().all(|&p| self.proc(p).is_macro) {
            IdeClass::Macro
        } else {
            IdeClass::Function
        })
    }

    /// Resolve only the actual callee at this call's opening parenthesis. The original
    /// program and its failed-body guards stay intact; no argument values are invented.
    pub fn ide_call_target_at(&mut self, file: FileId, open: u32) -> Option<IdeCallTarget> {
        // Code this compile skipped (an `#if` arm the target does not take) has no scope here:
        // its names are declared in the arm, so checking the callee would bind them to whatever
        // else is called that, and record the wrong reference for a later hover.
        if self
            .ed
            .facts()?
            .skipped
            .iter()
            .any(|s| s.file == file && s.start <= open && open <= s.end)
        {
            return None;
        }
        let text = self.src.sources.get(file).text.clone();
        let callee = crate::parser::parse_callee_ide(file, &text, open)?;
        let scope = self.ide_scope_at(file, callee.span.start)?;
        let operand = self.check_expr_no_emit(scope, &callee).ok()?;
        let parameters = match &operand {
            Operand::Callable {
                parameters, ..
            } => Some((operand.ty(), parameters.clone())),
            _ => None,
        };
        let procedures = match operand.without_callable() {
            Operand::Procs(ids) => ids,
            Operand::Const {
                value: Value::Proc(id),
                ..
            } => vec![id],
            _ => Vec::new(),
        };
        Some(IdeCallTarget {
            procedures,
            parameters,
        })
    }

    /// A callee is checked when something asks for it (`ide_call_target_at`, which the editor's
    /// token colouring does for every call), and checking it records the facts a use of its name
    /// has. Ask for them when none were recorded at `offset` and the name there is followed by
    /// `(`: an answer must not depend on whether the colouring ran against this analysis first.
    pub(crate) fn ide_note_callee_at(&mut self, file: FileId, offset: u32) {
        if self
            .ed
            .facts()
            .is_none_or(|ide| !ide.refs_at(file, offset).is_empty())
        {
            return;
        }
        let Some(source) = self.src.sources.try_get(file) else {
            return;
        };
        let bytes = source.text.as_bytes();
        let word = |at: usize| {
            bytes
                .get(at)
                .is_some_and(|&b| b == b'_' || b.is_ascii_alphanumeric())
        };
        let at = offset as usize;
        if !word(at) && (at == 0 || !word(at - 1)) {
            return;
        }
        let mut end = at;
        while word(end) {
            end += 1;
        }
        let mut open = end;
        while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
            open += 1;
        }
        if bytes.get(open) == Some(&b'(') {
            let _ = self.ide_call_target_at(file, open as u32);
        }
    }

    /// Direct-procedure view retained for existing IDE consumers.
    pub fn ide_callee_at(&mut self, file: FileId, open: u32) -> Vec<ProcId> {
        self.ide_call_target_at(file, open)
            .map_or_else(Vec::new, |t| t.procedures)
    }
}

/// An enum or a struct, as the editor lists its members.
#[derive(Clone, Debug)]
pub struct IdeMembers {
    pub name: String,
    pub is_enum: bool,
    /// Member names; for a struct with the value to write for the field.
    pub members: Vec<(String, String)>,
}
