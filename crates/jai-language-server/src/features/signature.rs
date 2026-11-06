//! Signature help.
use super::open_call;
use crate::MarkupKind;
use crate::semantic;
use crate::session::Session;
use crate::{DocumentUri, Error, Position, SignatureHelp, SignatureInformation};
use jaic::intern::Sym;
use std::path::Path;

impl Session {
    /// The signature of the call being typed at `position`, the chosen overload active.
    pub fn signature_help(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<SignatureHelp>, Error> {
        self.signature_help_as(uri, position, MarkupKind::PlainText)
    }

    /// Signature help with documentation written as `kind`.
    pub fn signature_help_as(
        &self,
        uri: &DocumentUri,
        position: Position,
        kind: MarkupKind,
    ) -> Result<Option<SignatureHelp>, Error> {
        self.projected_document_query(uri, |key| {
            self.signature_help_as_at_key(key, position, kind)
        })
    }

    fn signature_help_as_at_key(
        &self,
        uri: &DocumentUri,
        position: Position,
        kind: MarkupKind,
    ) -> Result<Option<SignatureHelp>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let byte = doc.index.byte(text, position);
        if let Some(help) = self.asm_signature_help(text, byte) {
            return Ok(Some(help));
        }
        let Some((open, chain, active_arg, named)) = open_call(text, byte) else {
            return Ok(None);
        };
        // Each name is followed by a `.` or the `(`; add the 1 first, since a callee at the start of
        // the document begins at byte 0.
        let callee_start = open + 1 - chain.iter().map(|c| c.len() + 1).sum::<usize>();
        // A call the check recorded at this position: its overloads, the chosen one active.
        let names: Vec<Sym> = chain.iter().map(|n| Sym::intern(n)).collect();
        let lookup = |a: &mut semantic::Analysis, path: &Path, start: usize, open: usize| {
            let file = a.file(path)?;
            if let Some(call) = a
                .compiler
                .ide_calls(file)
                .into_iter()
                .find(|c| c.span.start as usize == start && open < c.span.end as usize)
            {
                let docs = crate::doc_view::own_signature_documentation(a, &call.signatures);
                return Some((call.signatures, call.active, docs));
            }
            // The callee is looked up in the scope.
            let scope = a.compiler.ide_scope_at(file, start as u32)?;
            let procs = a.compiler.ide_callee(scope, &names);
            let found: Vec<_> = if procs.is_empty() {
                // A variable or member of procedure type.
                a.compiler
                    .ide_callee_value(scope, &names)
                    .into_iter()
                    .collect()
            } else {
                procs
                    .into_iter()
                    .map(|p| a.compiler.ide_signature(p))
                    .collect()
            };
            (!found.is_empty()).then(|| {
                let docs = crate::doc_view::own_signature_documentation(a, &found);
                (found, 0, docs)
            })
        };
        // Typing in the call has edited the text since the compile, which still describes it.
        let found = if self.may_compile() {
            self.with_semantic(uri, text, |a, path| lookup(a, path, callee_start, open))
        } else {
            self.on_older_compile(uri, text, |a, path, shift| {
                let start = shift.to_old(callee_start)?;
                lookup(a, path, start, shift.to_old_or_edit(open))
            })
        };
        let Some((mut signatures, active, docs)) = found else {
            return Ok(None);
        };
        // Under the name called: an alias or a `#bake_arguments` copy names the original.
        if let Some(called) = chain.last() {
            for sig in &mut signatures {
                if let Some((_, head)) = sig.label.split_once(" :: ") {
                    sig.label = format!("{called} :: {head}");
                }
            }
        }
        let parameter = |sig: &jaic::ide::meta::IdeSignature| -> usize {
            if let Some(name) = &named
                && let Some(i) = sig.param_names.iter().position(|n| n == name)
            {
                return i;
            }
            let last = sig.params.len().saturating_sub(1);
            active_arg.min(last)
        };
        let active_parameter = signatures.get(active).map_or(0, parameter);
        let documented = self.signature_docs(uri, &signatures, docs, kind);
        Ok(Some(SignatureHelp {
            signatures: signatures
                .into_iter()
                .zip(documented)
                .map(
                    |(s, (documentation, parameter_docs))| SignatureInformation {
                        label: s.label,
                        parameters: s.params,
                        documentation,
                        parameter_docs,
                    },
                )
                .collect(),
            active_signature: active,
            active_parameter,
        }))
    }
}
