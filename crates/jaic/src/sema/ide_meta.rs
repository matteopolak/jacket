//! Editor facts about metaprogramming and calls. With `Compiler::ide` set, checking records what
//! each `#insert`, `#run`, `#if` and `#expand` macro call of the user's files produced, and which
//! procedure and parameters each call resolved to. The language server builds expansion hovers,
//! inlay hints, signature help and format-string hovers from these.
use super::calls::{CallArg, Slot};
use super::ide::{IdeRef, IdeWhat};
use super::lower::Operand;
use super::scope::{EntityId, EntityKind, EntityState, Found, Resolved, ScopeId};
use super::value::{CodeId, ProcId};
use super::*;
use crate::lexer::{P, Tok};
use crate::types::TypeKind;

/// Expansions kept per analysis: an edit that makes a metaprogram generate a lot stays bounded.
const MAX_EXPANSIONS: usize = 4096;
/// Calls kept per analysis.
const MAX_CALLS: usize = 16384;
/// Distinct results kept for one site (a polymorphic body can expand differently per instance).
const MAX_VARIANTS: usize = 4;
/// Longest expansion text kept.
const MAX_TEXT: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
pub struct IdeCall {
    pub span: Span,
    /// The overload set the call chose from.
    pub candidates: Vec<ProcId>,
    /// The procedure called (an instance when the chosen one is polymorphic).
    pub chosen: ProcId,
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
    /// Header parameter that takes the format string, when the procedure is print-like: a
    /// `string` parameter followed by a variadic `Any` one.
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
    Proc(ProcId),
    Type(TypeId),
    Module(ModuleId),
}

impl Compiler {
    // -------------------------------------------------------------------------------------
    // Recording
    // -------------------------------------------------------------------------------------

    fn ide_note_expansion(
        &mut self,
        span: Span,
        kind: IdeExpansionKind,
        text: String,
        detail: String,
        output: String,
        literal: Option<String>,
    ) {
        let text = bounded(text);
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        if let Some(e) = ide
            .expansions
            .iter_mut()
            .find(|e| e.span == span && e.kind == kind)
        {
            if !e.texts.contains(&text) && e.texts.len() < MAX_VARIANTS {
                e.texts.push(text);
            }
            if e.output.is_empty() {
                e.output = output;
            }
            return;
        }
        if ide.expansions.len() >= MAX_EXPANSIONS {
            return;
        }
        ide.expansions.push(IdeExpansion {
            span,
            kind,
            texts: vec![text],
            detail,
            output: bounded(output),
            literal,
        });
    }

    /// Bytes compile-time code has printed so far (to tell what one `#run` printed).
    pub(super) fn ide_output_mark(&self) -> usize {
        self.ide
            .as_ref()
            .and_then(|i| i.output.as_ref())
            .map_or(0, |o| {
                o.try_borrow()
                    .map_or(0, |h| h.stdout.len() + h.stderr.len())
            })
    }

    fn ide_output_since(&self, mark: usize) -> String {
        let Some(out) = self.ide.as_ref().and_then(|i| i.output.as_ref()) else {
            return String::new();
        };
        let Ok(host) = out.try_borrow() else {
            return String::new();
        };
        // Both streams in write order; `mark` counted bytes of both.
        let mut all = Vec::new();
        let (mut o, mut e) = (0, 0);
        for &(to_stderr, n) in &host.order {
            if to_stderr {
                all.extend_from_slice(&host.stderr[e..(e + n).min(host.stderr.len())]);
                e += n;
            } else {
                all.extend_from_slice(&host.stdout[o..(o + n).min(host.stdout.len())]);
                o += n;
            }
        }
        String::from_utf8_lossy(all.get(mark..).unwrap_or(&[])).into_owned()
    }

    /// `span` widened back to the directive (`#insert`, `#if`...) written before it, when one
    /// of `names` is: the operand's span is what the compiler has.
    fn ide_widen(&self, span: Span, names: &[&str]) -> Option<Span> {
        let text = &self.sources.get(span.file).text;
        let start = span.start as usize;
        if start > text.len() {
            return None;
        }
        let from = text.floor_char_boundary(start.saturating_sub(512));
        let before = &text[from..start];
        let mut at = before.len();
        while let Some(hash) = before[..at].rfind('#') {
            let between = &before[hash..];
            if between.contains([';', '{', '}']) {
                return None;
            }
            let word: String = between[1..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if names.contains(&word.as_str()) {
                return Some(Span {
                    start: (from + hash) as u32,
                    ..span
                });
            }
            at = hash;
        }
        None
    }

    /// Source text of a `Code` value: an expression, or a block's statements without braces.
    pub(super) fn ide_code_text(&self, code: CodeId) -> String {
        match &*self.codes[code.0 as usize] {
            ast::CodeBody::Expr(e) => self.ide_snippet(e.span).trim().to_string(),
            ast::CodeBody::Block(b) => {
                let text = self.ide_snippet(b.span).trim();
                match text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
                    Some(inner) => dedent(inner.trim_matches('\n')),
                    None => text.to_string(),
                }
            }
        }
    }

    fn ide_snippet(&self, span: Span) -> &str {
        let text = &self.sources.get(span.file).text;
        let end = (span.end as usize).min(text.len());
        let start = (span.start as usize).min(end);
        if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return "";
        }
        &text[start..end]
    }

    /// `#insert` evaluated `value` (a string or `Code`) to `op`.
    pub(super) fn ide_note_insert(&mut self, value: &ast::Expr, op: &Operand) {
        if !self.ide_wants_file(value.span.file) {
            return;
        }
        let text = match op {
            Operand::Const {
                value: Value::String(s),
                ..
            } => String::from_utf8_lossy(s).into_owned(),
            Operand::Const {
                value: Value::Code(c),
                ..
            } => self.ide_code_text(*c),
            Operand::Void => String::new(),
            _ => return,
        };
        let span = self
            .ide_widen(value.span, &["insert"])
            .unwrap_or(value.span);
        self.ide_note_expansion(
            span,
            IdeExpansionKind::Insert,
            text,
            String::new(),
            String::new(),
            None,
        );
    }

    /// `#run` at `span` produced `op`; compile-time output past `mark` is what it printed.
    pub(super) fn ide_note_run(&mut self, span: Span, op: &Operand, mark: usize) {
        if !self.ide_wants_file(span.file) {
            return;
        }
        let output = self.ide_output_since(mark);
        let (text, detail, literal) = match op {
            Operand::Const {
                value,
                ty,
                ..
            } => {
                let literal = self.ide_literal(value);
                let text = literal.clone().unwrap_or_else(|| match value {
                    Value::Proc(p) => self.proc(*p).name.to_string(),
                    Value::Code(c) => format!("#code {}", self.ide_code_text(*c)),
                    Value::Bytes(_) => format!("{}.{{…}}", self.types.name(*ty)),
                    other => other.render(&self.types),
                });
                (text, self.types.name(*ty), literal)
            }
            Operand::Type(t) => {
                let name = self.types.name(*t);
                (name.clone(), "Type".into(), Some(name))
            }
            Operand::Procs(ps) => (
                ps.first()
                    .map(|&p| self.proc(p).name.to_string())
                    .unwrap_or_default(),
                "procedure".into(),
                None,
            ),
            Operand::Void => (String::new(), "void".into(), None),
            _ => return,
        };
        self.ide_note_expansion(span, IdeExpansionKind::Run, text, detail, output, literal);
    }

    /// A constant as Jai source, when it has a literal form.
    fn ide_literal(&self, value: &Value) -> Option<String> {
        Some(match value {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => {
                let text = format!("{v}");
                if text.contains(['.', 'e', 'i', 'N']) {
                    text
                } else {
                    format!("{text}.0")
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".into(),
            Value::String(s) => quote(&String::from_utf8_lossy(s)),
            Value::Type(t) => self.types.name(*t),
            _ => return None,
        })
    }

    /// A static condition (`#if`, `#ifx`, `#assert`) at `cond` evaluated to `holds`.
    pub(super) fn ide_note_condition(&mut self, cond: &ast::Expr, holds: bool) {
        if !self.ide_wants_file(cond.span.file) {
            return;
        }
        let Some(span) = self.ide_widen(cond.span, &["if", "ifx", "assert"]) else {
            return;
        };
        self.ide_note_expansion(
            span,
            IdeExpansionKind::If,
            holds.to_string(),
            String::new(),
            String::new(),
            None,
        );
    }

    /// A call at `span` expanded the macro `proc`: record its body with the arguments
    /// substituted for the parameters.
    pub(super) fn ide_note_macro(
        &mut self,
        proc: ProcId,
        slots: &[Slot],
        args: &[CallArg],
        body: &ast::Block,
        span: Span,
    ) {
        if !self.ide_wants_file(span.file) {
            return;
        }
        let header = self.proc(proc).lit.header.clone();
        let mut subst: HashMap<Sym, (String, bool, bool)> = HashMap::default();
        for (i, param) in header.params.iter().enumerate() {
            let Some(name) = param.name else {
                continue;
            };
            let is_code = matches!(&param.ty, Some(ast::Expr { kind: ast::ExprKind::Ident(n), .. }) if n.as_str() == "Code");
            let arg_text = |a: usize| -> String {
                let arg = &args[a];
                if let Some(Operand::Const {
                    value: Value::Code(c),
                    ..
                }) = &arg.op
                {
                    return self.ide_code_text(*c);
                }
                match &arg.expr {
                    Some(ast::Expr {
                        kind: ast::ExprKind::Code(body),
                        ..
                    }) => match &**body {
                        ast::CodeBody::Expr(e) => self.ide_snippet(e.span).trim().to_string(),
                        ast::CodeBody::Block(b) => self.ide_snippet(b.span).trim().to_string(),
                    },
                    Some(e) => self.ide_snippet(e.span).trim().to_string(),
                    None => self.ide_snippet(arg.span).trim().to_string(),
                }
            };
            let text = match slots.get(i) {
                Some(Slot::Arg(a)) => arg_text(*a),
                Some(Slot::Spread(a)) => format!("..{}", arg_text(*a)),
                Some(Slot::Variadic(list)) => list
                    .iter()
                    .map(|&a| arg_text(a))
                    .collect::<Vec<_>>()
                    .join(", "),
                Some(Slot::Default) | None => match &param.default {
                    Some(d) => self.ide_snippet(d.span).trim().to_string(),
                    None => continue,
                },
            };
            subst.insert(name.name, (text, is_code, param.variadic));
        }
        let source = self.ide_snippet(body.span).trim().to_string();
        let text = match substitute(&source, &subst) {
            Some(text) => text,
            None => source,
        };
        let text = match text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
            Some(inner) => dedent(inner.trim_matches('\n')),
            None => text,
        };
        let name = self.proc(proc).name.to_string();
        self.ide_note_expansion(
            span,
            IdeExpansionKind::Macro,
            text,
            name,
            String::new(),
            None,
        );
    }

    /// A call at `span` resolved to `chosen` among `candidates`.
    pub(super) fn ide_note_call(
        &mut self,
        span: Span,
        candidates: &[ProcId],
        chosen: ProcId,
        slots: &[Slot],
        args: &[CallArg],
    ) {
        if !self.ide_wants_file(span.file) {
            return;
        }
        let header = self.proc(chosen).lit.header.clone();
        let mut out = Vec::new();
        for (param, slot) in slots.iter().enumerate() {
            let variadic = header.params.get(param).is_some_and(|p| p.variadic);
            let list: Vec<usize> = match slot {
                Slot::Arg(a) | Slot::Spread(a) => vec![*a],
                Slot::Variadic(l) => l.clone(),
                Slot::Default => Vec::new(),
            };
            for a in list {
                let arg = &args[a];
                out.push(IdeArg {
                    span: arg.span,
                    param,
                    named: arg.name.is_some(),
                    variadic,
                    spread: arg.spread,
                    ty: arg.op.as_ref().map(|o| o.ty()),
                });
            }
        }
        out.sort_by_key(|a| a.span.start);
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        if ide.calls.len() >= MAX_CALLS || !ide.call_spans.insert(span) {
            return;
        }
        ide.calls.push(IdeCall {
            span,
            candidates: candidates.to_vec(),
            chosen,
            args: out,
        });
    }

    fn ide_wants_file(&mut self, file: FileId) -> bool {
        self.ide.is_some() && self.ide_wants_pub(file)
    }

    // -------------------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------------------

    /// Expansions recorded in `file`.
    pub fn ide_expansions(&self, file: FileId) -> Vec<IdeExpansion> {
        self.ide
            .as_ref()
            .map(|ide| {
                ide.expansions
                    .iter()
                    .filter(|e| e.span.file == file)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Calls recorded in `file`, described.
    pub fn ide_calls(&mut self, file: FileId) -> Vec<IdeCallInfo> {
        let calls: Vec<IdeCall> = self
            .ide
            .as_ref()
            .map(|ide| {
                ide.calls
                    .iter()
                    .filter(|c| c.span.file == file)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        calls.into_iter().map(|c| self.ide_call_info(&c)).collect()
    }

    /// A procedure's header, for signature help.
    pub fn ide_signature(&self, p: ProcId) -> IdeSignature {
        let info = self.proc(p);
        let header = &info.lit.header;
        let params: Vec<String> = header
            .params
            .iter()
            .map(|param| self.ide_snippet(param.span).trim().to_string())
            .collect();
        let param_names = header
            .params
            .iter()
            .map(|param| param.name.map(|n| n.name.to_string()).unwrap_or_default())
            .collect();
        let head = self.ide_snippet(header.span);
        let head = head.lines().next().unwrap_or("").trim();
        let head = head.trim_end_matches('{').trim_end();
        IdeSignature {
            label: format!("{} :: {}", info.name, head),
            params,
            param_names,
            span: info.span,
        }
    }

    fn ide_call_info(&mut self, call: &IdeCall) -> IdeCallInfo {
        let chosen_span = self.proc(call.chosen).span;
        let mut signatures: Vec<IdeSignature> = call
            .candidates
            .iter()
            .map(|&p| self.ide_signature(p))
            .collect();
        let active = match signatures.iter().position(|s| s.span == chosen_span) {
            Some(i) => i,
            None => {
                signatures.push(self.ide_signature(call.chosen));
                signatures.len() - 1
            }
        };
        let header = self.proc(call.chosen).lit.header.clone();
        let format_param = header.params.windows(2).position(|w| {
            let is_string = matches!(&w[0].ty, Some(ast::Expr { kind: ast::ExprKind::Ident(n), .. }) if n.as_str() == "string");
            let any_args = w[1].variadic
                && matches!(&w[1].ty, Some(ast::Expr { kind: ast::ExprKind::Ident(n), .. }) if n.as_str() == "Any");
            is_string && any_args
        });
        let args = call
            .args
            .iter()
            .map(|a| IdeArgInfo {
                span: a.span,
                param: a.param,
                param_name: header
                    .params
                    .get(a.param)
                    .and_then(|p| p.name)
                    .map(|n| n.name.to_string())
                    .unwrap_or_default(),
                named: a.named,
                variadic: a.variadic,
                spread: a.spread,
                ty: a
                    .ty
                    .filter(|&t| t != TypeId::COMPILE_TIME)
                    .map(|t| self.types.name(t)),
            })
            .collect();
        IdeCallInfo {
            span: call.span,
            signatures,
            active,
            args,
            format_param,
        }
    }

    /// Procedures `chain` (`name` or `Module.name`) names from `scope`, for signature help
    /// while the call is being typed (and does not parse yet).
    pub fn ide_callee(&mut self, scope: ScopeId, chain: &[Sym]) -> Vec<ProcId> {
        let Some((last, first)) = chain.split_last() else {
            return Vec::new();
        };
        let ids = if first.is_empty() {
            match self.lookup_full(scope, *last) {
                Ok(Found::Entities(ids)) => ids,
                _ => return Vec::new(),
            }
        } else {
            match self.ide_receiver(scope, first) {
                Some(super::ide::IdeReceiver::Module(m)) => match self.module_lookup(m, *last) {
                    Ok(Found::Entities(ids)) => ids,
                    _ => return Vec::new(),
                },
                _ => return Vec::new(),
            }
        };
        let mut out = Vec::new();
        for id in ids {
            match self.resolve_entity(id) {
                Ok(Resolved::Proc(p))
                | Ok(Resolved::Const {
                    value: Value::Proc(p),
                    ..
                }) => out.push(p),
                Ok(Resolved::ProcSet(ps)) => out.extend(ps),
                _ => {}
            }
        }
        out.dedup();
        out
    }

    /// Inferred types of declarations `x := value` in `file`: (name span, type).
    pub fn ide_declared_types(&self, file: FileId) -> Vec<(Span, String)> {
        let Some(ide) = self.ide.as_ref() else {
            return Vec::new();
        };
        let mut out: Vec<(Span, String)> = Vec::new();
        let mut conflicting = Vec::new();
        for r in ide.refs.iter().filter(|r| r.decl && r.span.file == file) {
            let IdeWhat::Entity(e) = r.what else {
                continue;
            };
            let EntityKind::Local {
                ty, ..
            } = self.entity(e).kind
            else {
                continue;
            };
            let name = self.types.name(ty);
            match out.iter().find(|(s, _)| *s == r.span) {
                Some((_, n)) if *n != name => conflicting.push(r.span),
                Some(_) => {}
                None => out.push((r.span, name)),
            }
        }
        out.retain(|(s, _)| !conflicting.contains(s));
        out
    }

    fn ide_class_of_entity(&self, id: EntityId) -> Option<IdeClass> {
        let e = self.entity(id);
        Some(match &e.kind {
            EntityKind::Local {
                ..
            } => IdeClass::Variable,
            EntityKind::Const {
                value: Value::Type(_),
                ..
            } => IdeClass::Type,
            EntityKind::Const {
                ..
            } => IdeClass::Constant,
            EntityKind::Import(_) => IdeClass::Module,
            EntityKind::Builtin(b) => match b {
                scope::Builtin::Type(_) => IdeClass::Type,
                scope::Builtin::Proc(_) => IdeClass::Function,
                scope::Builtin::TargetConstant(_) => IdeClass::Constant,
            },
            EntityKind::Placeholder => return None,
            EntityKind::Decl {
                decl, ..
            } => match &e.state {
                EntityState::Done(Resolved::Proc(p))
                | EntityState::Done(Resolved::Const {
                    value: Value::Proc(p),
                    ..
                }) => {
                    if self.proc(*p).is_macro {
                        IdeClass::Macro
                    } else {
                        IdeClass::Function
                    }
                }
                EntityState::Done(Resolved::ProcSet(_)) => IdeClass::Function,
                EntityState::Done(Resolved::Global {
                    ..
                }) => IdeClass::Variable,
                EntityState::Done(Resolved::Const {
                    value: Value::Type(_),
                    ..
                })
                | EntityState::Done(Resolved::PolyStruct(_)) => IdeClass::Type,
                EntityState::Done(Resolved::Const {
                    ..
                }) => IdeClass::Constant,
                EntityState::Done(Resolved::Module(_) | Resolved::Library(_)) => IdeClass::Module,
                _ => {
                    use ast::ExprKind as E;
                    match (decl.kind, decl.value.as_ref().map(|v| &v.kind)) {
                        (_, Some(E::Proc(lit))) if lit.header.flags.expand => IdeClass::Macro,
                        (_, Some(E::Proc(_))) => IdeClass::Function,
                        (_, Some(E::Struct(_) | E::Enum(_) | E::ProcType(_))) => IdeClass::Type,
                        (ast::DeclKind::Var, _) => IdeClass::Variable,
                        _ => IdeClass::Constant,
                    }
                }
            },
        })
    }

    /// How to color each recorded identifier of `file`.
    pub fn ide_classes(&self, file: FileId) -> Vec<(Span, IdeClass)> {
        let Some(ide) = self.ide.as_ref() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for r in ide.refs.iter().filter(|r| r.span.file == file) {
            let class = match &r.what {
                IdeWhat::Entity(e) => match self.ide_class_of_entity(*e) {
                    Some(c) => c,
                    None => continue,
                },
                IdeWhat::Procs(ps) => {
                    if !ps.is_empty() && ps.iter().all(|&p| self.proc(p).is_macro) {
                        IdeClass::Macro
                    } else {
                        IdeClass::Function
                    }
                }
                IdeWhat::Type(_) => IdeClass::Type,
                IdeWhat::Module(_) => IdeClass::Module,
                IdeWhat::Member(name) => match self.types.kind(r.ty) {
                    TypeKind::Enum(en)
                        if self
                            .types
                            .enum_info(*en)
                            .members
                            .iter()
                            .any(|(m, _)| m == name) =>
                    {
                        IdeClass::EnumMember
                    }
                    _ => IdeClass::Field,
                },
            };
            out.push((r.span, class));
        }
        out
    }

    fn ide_ref_at(&self, file: FileId, offset: u32) -> Option<IdeRef> {
        self.ide
            .as_ref()?
            .refs
            .iter()
            .filter(|r| r.span.file == file && r.span.start <= offset && offset <= r.span.end)
            .min_by_key(|r| r.span.end - r.span.start)
            .cloned()
    }

    fn ide_targets(&self, r: &IdeRef) -> Vec<Target> {
        match &r.what {
            IdeWhat::Entity(e) => match &self.entity(*e).state {
                EntityState::Done(Resolved::Proc(p))
                | EntityState::Done(Resolved::Const {
                    value: Value::Proc(p),
                    ..
                }) => vec![Target::Proc(*p)],
                EntityState::Done(Resolved::Const {
                    value: Value::Type(t),
                    ..
                }) => vec![Target::Type(*t)],
                EntityState::Done(Resolved::Module(m)) => vec![Target::Module(*m)],
                _ => match &self.entity(*e).kind {
                    EntityKind::Const {
                        value: Value::Type(t),
                        ..
                    } => vec![Target::Type(*t)],
                    _ => vec![Target::Entity(*e)],
                },
            },
            IdeWhat::Procs(ps) => ps.iter().map(|&p| Target::Proc(p)).collect(),
            IdeWhat::Type(t) => vec![Target::Type(*t)],
            IdeWhat::Module(m) => vec![Target::Module(*m)],
            IdeWhat::Member(_) => Vec::new(),
        }
    }

    /// Every recorded reference to what the identifier at `offset` names (declarations
    /// included), as (span, is the declaration). Struct members are not tracked.
    pub fn ide_references(&self, file: FileId, offset: u32) -> Vec<(Span, bool)> {
        let Some(at) = self.ide_ref_at(file, offset) else {
            return Vec::new();
        };
        let wanted = self.ide_targets(&at);
        if wanted.is_empty() {
            return Vec::new();
        }
        let Some(ide) = self.ide.as_ref() else {
            return Vec::new();
        };
        let mut out: Vec<(Span, bool)> = Vec::new();
        for r in &ide.refs {
            if self.ide_targets(r).iter().any(|t| wanted.contains(t))
                && !out.iter().any(|(s, _)| *s == r.span)
            {
                out.push((r.span, r.decl));
            }
        }
        out.sort_by_key(|(s, _)| (s.file.0, s.start));
        out
    }

    /// Where the type of the expression at `offset` is declared (through pointers and arrays).
    pub fn ide_type_definition(&self, file: FileId, offset: u32) -> Vec<Span> {
        let Some(r) = self.ide_ref_at(file, offset) else {
            return Vec::new();
        };
        let mut t = match &r.what {
            IdeWhat::Type(t) => *t,
            IdeWhat::Entity(e) => match &self.entity(*e).kind {
                EntityKind::Const {
                    value: Value::Type(t),
                    ..
                } => *t,
                _ => r.ty,
            },
            _ => r.ty,
        };
        for _ in 0..8 {
            t = match self.types.kind(t) {
                TypeKind::Pointer(inner) => *inner,
                TypeKind::Array {
                    elem, ..
                } => *elem,
                _ => break,
            };
        }
        let span = match self.types.kind(t) {
            TypeKind::Struct(s) => self.types.struct_info(*s).span,
            TypeKind::Enum(e) => self.types.enum_info(*e).span,
            _ => return Vec::new(),
        };
        if span.end <= span.start {
            return Vec::new();
        }
        vec![span]
    }

    /// Polymorphic procedures declared in `file` with the instances checking created: (the
    /// procedure's span, each instance's bindings as `T = s64`).
    pub fn ide_polymorphs(&self, file: FileId) -> Vec<(Span, String, Vec<String>)> {
        let mut out = Vec::new();
        for p in &self.procs {
            if !p.is_poly || p.span.file != file || p.bindings.is_some() {
                continue;
            }
            let mut instances: Vec<String> = p
                .instances
                .values()
                .map(|&inst| self.ide_bindings_text(inst))
                .collect();
            instances.sort();
            out.push((p.span, p.name.to_string(), instances));
        }
        out
    }

    fn ide_bindings_text(&self, inst: ProcId) -> String {
        let Some(scope) = self.proc(inst).bindings else {
            return String::new();
        };
        let mut names: Vec<(Sym, EntityId)> = self
            .scope(scope)
            .names
            .iter()
            .filter_map(|(n, ids)| Some((*n, *ids.first()?)))
            .collect();
        names.sort_by_key(|(_, e)| self.entity(*e).span.start);
        names
            .into_iter()
            .map(|(n, e)| match &self.entity(e).kind {
                EntityKind::Const {
                    value, ..
                } => format!(
                    "{n} = {}",
                    self.ide_literal(value)
                        .unwrap_or_else(|| value.render(&self.types))
                ),
                _ => n.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn bounded(mut text: String) -> String {
    if text.len() > MAX_TEXT {
        let end = text.floor_char_boundary(MAX_TEXT);
        text.truncate(end);
        text.push('…');
    }
    text
}

/// `text` as a Jai string literal.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '%' => out.push_str("\\%"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `text` without the indentation its lines share (the first line, often unindented after a
/// brace, does not count).
pub fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let indent = lines
        .iter()
        .skip(1)
        .chain(lines.first())
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.len() >= indent && l[..indent].trim().is_empty() {
                &l[indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// A substituted argument needs parentheses unless it is one token-like atom.
fn atom(text: &str) -> bool {
    let simple = |c: char| c.is_alphanumeric() || c == '_' || c == '.';
    if text.chars().all(simple) {
        return true;
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return !text[1..text.len() - 1].contains('"');
    }
    // `name(...)` / `(...)` with one outer pair of parentheses.
    let open = text.find('(');
    if let Some(open) = open
        && text.ends_with(')')
        && text[..open].chars().all(simple)
    {
        let mut depth = 0i32;
        for (i, c) in text[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return open + i == text.len() - 1;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// `source` (a macro body) with each parameter replaced by its argument's text: `#insert c` of
/// a `Code` parameter by the code, `..args` of a variadic one by the arguments, and the
/// backtick of a caller-scope name dropped (after expansion it is the caller's name).
fn substitute(source: &str, subst: &HashMap<Sym, (String, bool, bool)>) -> Option<String> {
    let tokens = crate::lexer::lex(FileId(0), source).ok()?;
    let mut out = String::with_capacity(source.len());
    let mut copied = 0usize;
    let mut i = 0;
    let keyword = |s: Sym| {
        matches!(
            s.as_str(),
            "return" | "defer" | "break" | "continue" | "remove"
        )
    };
    while i < tokens.len() {
        let t = &tokens[i];
        let (start, end) = (t.span.start as usize, t.span.end as usize);
        let prev = i.checked_sub(1).map(|p| &tokens[p].tok);
        let next = tokens.get(i + 1).map(|n| &n.tok);
        match &t.tok {
            // `#insert code_param`: the code itself.
            Tok::Directive(d) if d.as_str() == "insert" => {
                if let Some(Tok::Ident(n)) = next
                    && let Some((text, true, _)) = subst.get(n)
                {
                    out.push_str(&source[copied..start]);
                    out.push_str(text);
                    copied = tokens[i + 1].span.end as usize;
                    i += 2;
                    continue;
                }
            }
            Tok::Punct(P::Backtick) => {
                if let Some(Tok::Ident(n)) = next
                    && !keyword(*n)
                {
                    out.push_str(&source[copied..start]);
                    copied = end;
                }
            }
            Tok::Punct(P::DotDot) => {
                if let Some(Tok::Ident(n)) = next
                    && let Some((text, false, true)) = subst.get(n)
                {
                    out.push_str(&source[copied..start]);
                    out.push_str(text);
                    copied = tokens[i + 1].span.end as usize;
                    i += 2;
                    continue;
                }
            }
            Tok::Ident(n) => {
                let member = matches!(prev, Some(Tok::Punct(P::Dot | P::Backtick)));
                let declared = matches!(
                    next,
                    Some(Tok::Punct(P::ColonEq | P::ColonColon | P::Colon))
                ) && !matches!(prev, Some(Tok::Punct(P::Dot)));
                if !member
                    && !declared
                    && let Some((text, code, variadic)) = subst.get(n)
                    && !code
                {
                    out.push_str(&source[copied..start]);
                    if *variadic {
                        out.push_str(&format!(".[{text}]"));
                    } else if atom(text) {
                        out.push_str(text);
                    } else {
                        out.push('(');
                        out.push_str(text);
                        out.push(')');
                    }
                    copied = end;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out.push_str(&source[copied.min(source.len())..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitution_replaces_parameters_and_drops_caller_backticks() {
        let mut subst = HashMap::default();
        subst.insert(Sym::intern("a"), ("x + 1".to_string(), false, false));
        subst.insert(Sym::intern("b"), ("y".to_string(), false, false));
        subst.insert(
            Sym::intern("body"),
            ("print(\"hi\");".to_string(), true, false),
        );
        let text = substitute(
            "{ t := a; `total = t * b; p.a = 2; #insert body; `return; }",
            &subst,
        )
        .unwrap();
        assert_eq!(
            text,
            "{ t := (x + 1); total = t * y; p.a = 2; print(\"hi\");; `return; }"
        );
    }

    #[test]
    fn dedent_keeps_relative_indentation() {
        assert_eq!(
            dedent("    a;\n    if b {\n        c;\n    }"),
            "a;\nif b {\n    c;\n}"
        );
    }

    #[test]
    fn atoms_need_no_parentheses() {
        assert!(atom("x"));
        assert!(atom("a.b"));
        assert!(atom("f(1, g(2))"));
        assert!(atom("\"text\""));
        assert!(!atom("x + 1"));
        assert!(!atom("f(1) + g(2)"));
    }
}
