//! Noting calls, macros and keyword calls for editors.
use super::text::{dedent, substitute};
use super::{IdeArg, IdeCall, IdeExpansionKind, MAX_CALLS};
use crate::sema::calls::{CallArg, Slot};
use crate::sema::ide::{IdeRef, IdeWhat};
use crate::sema::lower::Operand;
use crate::sema::value::ProcId;
use crate::sema::{Compiler, FileId, HashMap, Span, Sym, TypeId, Value, ast};

impl Compiler {
    /// A call at `span` expanded the macro `proc`: record its body with the arguments
    /// substituted for the parameters.
    pub(in crate::sema) fn ide_note_macro(
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
            let is_code = self.param_is_code(proc, param);
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
                        ast::CodeBody::Stmt(s) => self.ide_snippet(s.span).trim().to_string(),
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
    pub(in crate::sema) fn ide_note_call(
        &mut self,
        span: Span,
        candidates: &[ProcId],
        chosen: ProcId,
        slots: &[Slot],
        args: &[CallArg],
        ambiguous: bool,
    ) {
        self.ide_note_callee(span, chosen);
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
                    ty: arg.op.as_ref().map(super::super::lower::Operand::ty),
                });
            }
        }
        out.sort_by_key(|a| a.span.start);
        self.ide_note_named_args(chosen, slots, args);
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
            ambiguous,
            args: out,
        });
    }

    /// Remember which procedure the call at `span` named, whatever file it is in.
    fn ide_note_callee(&mut self, span: Span, chosen: ProcId) {
        if self.sketching() {
            return;
        }
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        ide.callees
            .entry(span)
            .and_modify(|seen| {
                if *seen != Some(chosen) {
                    *seen = None;
                }
            })
            .or_insert(Some(chosen));
    }

    /// Associate only the custom-loop keyword with the actual overload selection,
    /// before expansion can fail. Call-summary deduplication and its display budget
    /// must not discard selections from other real instantiations of this loop.
    pub(in crate::sema) fn ide_note_for_keyword(
        &mut self,
        span: Span,
        chosen: ProcId,
        ambiguous: bool,
    ) {
        if self.sketching() || !self.ide_wants_file(span.file) {
            return;
        }
        let keyword = Span {
            end: span.start + 3,
            ..span
        };
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        let selections = ide
            .for_expansions
            .entry(keyword)
            .or_insert_with(|| Some(Vec::new()));
        if ambiguous {
            *selections = None;
            return;
        }
        let Some(procs) = selections else {
            return;
        };
        if !procs.contains(&chosen) {
            procs.push(chosen);
        }
    }

    /// A `for` that does not name its value or index declares `it` and `it_index` anyway:
    /// remember their types for an inlay hint at the start of the iterated expression `at`.
    pub(in crate::sema) fn ide_note_implicit_loop_names(
        &mut self,
        for_: &ast::For,
        at: Span,
        types: [TypeId; 2],
    ) {
        if self.sketching() || !self.ide_wants_file(at.file) {
            return;
        }
        let mut names = Vec::new();
        if for_.it.is_none() {
            names.push(("it", types[0]));
        }
        if for_.index.is_none() {
            names.push(("it_index", types[1]));
        }
        if names.is_empty() {
            return;
        }
        let key = Span {
            end: at.start,
            ..at
        };
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        match ide.implicit_loop_names.entry(key) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(Some(names));
            }
            std::collections::hash_map::Entry::Occupied(mut slot) => {
                if slot.get().as_ref() != Some(&names) {
                    slot.insert(None);
                }
            }
        }
    }

    /// Each `name = value` argument of a call to `chosen`: hovering `name` shows the
    /// parameter and its type, and go to definition goes to the parameter.
    fn ide_note_named_args(&mut self, chosen: ProcId, slots: &[Slot], args: &[CallArg]) {
        let header = self.proc(chosen).lit.header.clone();
        let sig = self.proc(chosen).sig.clone();
        for (param, slot) in slots.iter().enumerate() {
            let (Slot::Arg(a) | Slot::Spread(a)) = slot else {
                continue;
            };
            let arg = &args[*a];
            let Some(name) = arg.name else {
                continue;
            };
            let Some(at) = self.ide_arg_name_span(arg.span, name) else {
                continue;
            };
            let ty = sig
                .as_ref()
                .and_then(|s| s.params.get(param))
                .map_or(TypeId::COMPILE_TIME, |p| p.ty);
            let decl = header
                .params
                .get(param)
                .and_then(|p| p.name)
                .filter(|n| n.span.end > n.span.start)
                .map(|n| n.span);
            let Some(ide) = self.ide.as_mut() else {
                return;
            };
            ide.refs.push(IdeRef {
                span: at,
                what: IdeWhat::Member(name),
                ty,
                decl: false,
            });
            if let Some(decl) = decl {
                ide.member_uses.insert((at, decl));
            }
        }
    }

    /// The `name` of a `name = value` argument whose value (or whole argument) is at `span`.
    fn ide_arg_name_span(&self, span: Span, name: Sym) -> Option<Span> {
        let text = &self.sources.try_get(span.file)?.text;
        let name = name.as_str();
        let start = span.start as usize;
        // The argument's span may start at the name or at the value.
        if text.get(start..)?.starts_with(name)
            && text[start + name.len()..].trim_start().starts_with('=')
        {
            return Some(Span {
                file: span.file,
                start: span.start,
                end: (start + name.len()) as u32,
            });
        }
        let before = text.get(..start)?.trim_end().strip_suffix('=')?.trim_end();
        let at = before.strip_suffix(name).map(str::len)?;
        Some(Span {
            file: span.file,
            start: at as u32,
            end: (at + name.len()) as u32,
        })
    }

    pub(super) fn ide_wants_file(&mut self, file: FileId) -> bool {
        self.ide.is_some() && self.ide_wants_pub(file)
    }
}
