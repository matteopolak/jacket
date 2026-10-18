//! Signature help.
use super::open_call;
use crate::MarkupKind;
use crate::session::Session;
use crate::{DocumentUri, Error, Position, SignatureHelp, SignatureInformation};
use jaic::intern::Sym;

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
        let doc = self.document(uri)?;
        let text = &doc.text;
        let byte = doc.index.byte(text, position)?;
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
        let recorded = self
            .calls(uri)
            .into_iter()
            .find(|c| c.span.start as usize == callee_start && open < c.span.end as usize);
        let (mut signatures, active) = match recorded {
            Some(call) => (call.signatures, call.active),
            None => {
                // The callee is looked up in the scope.
                let names: Vec<Sym> = chain.iter().map(|n| Sym::intern(n)).collect();
                let found = self.with_semantic(uri, text, |a, path| {
                    let file = a.file(path)?;
                    let scope = a.compiler.ide_scope_at(file, callee_start as u32)?;
                    let procs = a.compiler.ide_callee(scope, &names);
                    if procs.is_empty() {
                        // A variable or member of procedure type.
                        return Some(
                            a.compiler
                                .ide_callee_value(scope, &names)
                                .into_iter()
                                .collect::<Vec<_>>(),
                        );
                    }
                    Some(
                        procs
                            .into_iter()
                            .map(|p| a.compiler.ide_signature(p))
                            .collect::<Vec<_>>(),
                    )
                });
                match found {
                    Some(s) if !s.is_empty() => (s, 0),
                    _ => return Ok(None),
                }
            }
        };
        // Under the name called: an alias or a `#bake_arguments` copy names the original.
        if let Some(called) = chain.last() {
            for sig in &mut signatures {
                if let Some((_, head)) = sig.label.split_once(" :: ") {
                    sig.label = format!("{called} :: {head}");
                }
            }
        }
        let parameter = |sig: &jaic::sema::ide_meta::IdeSignature| -> usize {
            if let Some(name) = &named
                && let Some(i) = sig.param_names.iter().position(|n| n == name)
            {
                return i;
            }
            let last = sig.params.len().saturating_sub(1);
            active_arg.min(last)
        };
        let active_parameter = signatures.get(active).map_or(0, parameter);
        let documented = self.signature_docs(uri, &signatures, kind);
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
