//! Call, signature and callee queries.
use super::{IdeArgInfo, IdeCall, IdeCallInfo, IdeExpansion, IdeSignature};
use crate::sema::scope::{Found, Resolved, ScopeId};
use crate::sema::value::ProcId;
use crate::sema::*;
use crate::types::TypeKind;

impl Compiler {
    // Queries

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

    /// The overload the call whose callee name is at `name` resolved to, out of the set `ps`
    /// that name stands for. `None` when no call was recorded there, or when several
    /// candidates fitted equally well.
    pub(in crate::sema) fn ide_resolved_overload(
        &self,
        name: Span,
        ps: &[ProcId],
    ) -> Option<ProcId> {
        if ps.len() < 2 {
            return None;
        }
        let ide = self.ide.as_ref()?;
        let call = ide
            .calls
            .iter()
            .filter(|c| {
                c.span.file == name.file
                    && c.span.start <= name.start
                    && name.end <= c.span.end
                    && c.candidates == ps
                    && !self
                        .sources
                        .snippet(Span {
                            end: name.end,
                            ..c.span
                        })
                        .contains('(')
            })
            .min_by_key(|c| c.span.end - c.span.start)?;
        if call.ambiguous {
            return None;
        }
        let chosen = self.proc(call.chosen).span;
        ps.iter().copied().find(|&p| self.proc(p).span == chosen)
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
        let head = self.ide_header_text(p);
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
        let format_param = self
            .format_string_param(call.chosen, chosen_span)
            .filter(|_| self.is_print_like(call.chosen));
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

    /// The signature of the procedure-typed variable, constant or member `chain` (`f`,
    /// `s.callback`) names from `scope`. A procedure type carries no parameter names, so the
    /// parameters are shown by type.
    pub fn ide_callee_value(&mut self, scope: ScopeId, chain: &[Sym]) -> Option<IdeSignature> {
        let Some(crate::sema::ide::IdeReceiver::Value(mut ty)) = self.ide_receiver(scope, chain)
        else {
            return None;
        };
        while let Some(inner) = self.types.pointee(ty) {
            ty = inner;
        }
        let TypeKind::Proc(proc) = self.types.kind(ty).clone() else {
            return None;
        };
        let mut params: Vec<String> = proc.params.iter().map(|&t| self.types.name(t)).collect();
        if proc.variadic
            && let Some(last) = proc.params.last()
            && let TypeKind::Array {
                elem, ..
            } = self.types.kind(*last)
        {
            let elem = *elem;
            if let Some(slot) = params.last_mut() {
                *slot = format!("..{}", self.types.name(elem));
            }
        }
        let returns: Vec<String> = proc.returns.iter().map(|&t| self.types.name(t)).collect();
        let mut label = format!("{}: ({})", chain.last()?, params.join(", "));
        if !returns.is_empty() {
            label.push_str(&format!(" -> {}", returns.join(", ")));
        }
        Some(IdeSignature {
            label,
            param_names: vec![String::new(); params.len()],
            params,
            span: Span::NONE,
        })
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
                Some(crate::sema::ide::IdeReceiver::Module(m)) => {
                    match self.module_member_lookup(m, *last) {
                        Ok(Found::Entities(ids)) => ids,
                        _ => return Vec::new(),
                    }
                }
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
}
