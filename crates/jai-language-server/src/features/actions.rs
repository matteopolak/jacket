//! Code actions and inlay hints.
use super::{
    HINT_CHARS, ambiguous_params, clip, inferred_name_ends, inline_insert, literal, span_of,
};
use crate::session::{Session, contains};
use crate::{CodeAction, Command, DocumentUri, Error, InlayHint, InlayHintKind, Range, TextEdit};
use jaic::sema::ide_meta::IdeExpansionKind;

impl Session {
    /// Show an expansion; inline an `#insert` or a `#run` value; add an `#import` an unknown name
    /// needs; apply a lint's fix, or all.
    pub fn code_actions(&self, uri: &DocumentUri, range: Range) -> Result<Vec<CodeAction>, Error> {
        self.code_actions_in(uri, range, &crate::lints::ActionContext::default())
    }

    /// [`Session::code_actions`] limited to what `context` asks for.
    pub fn code_actions_in(
        &self,
        uri: &DocumentUri,
        range: Range,
        context: &crate::lints::ActionContext,
    ) -> Result<Vec<CodeAction>, Error> {
        let mut actions = self.all_code_actions(uri, range, context)?;
        actions.retain(|a| match a.kind {
            Some(kind) => context.wants(kind),
            None => context.only.is_none(),
        });
        Ok(actions)
    }

    fn all_code_actions(
        &self,
        uri: &DocumentUri,
        range: Range,
        context: &crate::lints::ActionContext,
    ) -> Result<Vec<CodeAction>, Error> {
        let doc = self.document(uri)?;
        let start = doc.index.byte(&doc.text, range.start)?;
        let end = doc.index.byte(&doc.text, range.end)?;
        let text = &doc.text;
        let mut fixes = self.import_actions(uri, start, end, context);
        fixes.extend(self.lint_actions(uri, start, end, context));
        let switch_fixes = self.complete_switch_actions(uri, start, end, context);
        let fixed_switch = !switch_fixes.is_empty();
        fixes.extend(switch_fixes);
        if context
            .only
            .as_ref()
            .is_none_or(|only| only.iter().any(|k| k.starts_with("refactor")))
        {
            // The quick fix already adds the cases of a switch with an error.
            fixes.extend(self.refactor_actions(
                uri,
                start,
                end,
                if fixed_switch {
                    crate::refactor::Pick::AllButFillSwitch
                } else {
                    crate::refactor::Pick::All
                },
            ));
        }
        let Some(e) = self
            .expansions(uri)
            .into_iter()
            .filter(|e| {
                let s = span_of(e);
                contains(s, start) || (start <= s.start && s.end <= end && start < end)
            })
            .min_by_key(|e| e.span.end - e.span.start)
        else {
            return Ok(fixes);
        };
        let span = span_of(&e);
        let Some(at) = self.range_of(uri, span) else {
            return Ok(fixes);
        };
        let mut actions = Vec::new();
        if e.kind != IdeExpansionKind::If {
            let what = match e.kind {
                IdeExpansionKind::Macro => format!("Show expansion of {}", e.detail),
                IdeExpansionKind::Run => "Show #run result".into(),
                _ => "Show #insert expansion".into(),
            };
            actions.push(CodeAction {
                title: what.clone(),
                kind: None,
                edit: None,
                command: Some(Command {
                    title: what,
                    command: "jai.showExpansion".into(),
                    target: Some((uri.as_str().into(), at.start)),
                }),
                ..CodeAction::default()
            });
        }
        let single = e.texts.len() == 1;
        let edit = match e.kind {
            IdeExpansionKind::Insert if single => {
                inline_insert(text, span, &e.texts[0]).map(|(s, t)| ("Inline #insert", s, t))
            }
            IdeExpansionKind::Run if single => e
                .literal
                .clone()
                .filter(|_| e.output.is_empty())
                .map(|lit| ("Replace #run with its value", span, lit)),
            _ => None,
        };
        if let Some((title, span, new_text)) = edit
            && let Some(range) = self.range_of(uri, span)
        {
            actions.push(CodeAction {
                title: title.into(),
                kind: Some("refactor.inline"),
                edit: Some((
                    uri.as_str().into(),
                    vec![TextEdit {
                        range,
                        new_text,
                    }],
                )),
                command: None,
                ..CodeAction::default()
            });
        }
        actions.extend(fixes);
        Ok(actions)
    }

    // Inlay hints

    /// Inferred types of `x := value`, parameter names of literal arguments, and `#run` values.
    pub fn typed_hints(&self, uri: &DocumentUri, range: Range) -> Result<Vec<InlayHint>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let first = doc.index.byte(text, range.start)?;
        let last = doc.index.byte(text, range.end)?;
        let Some((types, calls, expansions)) = self.checked(uri, |a, f| {
            (
                a.compiler.ide_declared_types(f),
                a.compiler.ide_calls(f),
                a.compiler.ide_expansions(f),
            )
        }) else {
            return Ok(Vec::new());
        };
        let mut hints: Vec<(usize, InlayHint)> = Vec::new();
        let mut hint = |at: usize, label: String, kind, left, right, tooltip| {
            if at < first || at > last || at > text.len() || !text.is_char_boundary(at) {
                return;
            }
            if let Ok(position) = doc.index.position(text, at) {
                hints.push((
                    at,
                    InlayHint {
                        position,
                        label,
                        kind,
                        padding_left: left,
                        padding_right: right,
                        tooltip,
                    },
                ));
            }
        };
        let inferred = inferred_name_ends(&doc.ide_text());
        for (span, ty) in types {
            let end = span.end as usize;
            // `?` stands in for a type nothing could name: say nothing, not that.
            if ty.as_str() == "?" || end > text.len() || !inferred.contains(&(end as u32)) {
                continue;
            }
            // `t := Thing.{...}` and `p := cast(*u8) q` already say the type.
            let value = text[end..]
                .trim_start()
                .trim_start_matches([',', ':', '='])
                .trim_start();
            let value = value.split(['\n', ';']).next().unwrap_or("");
            if value.starts_with(ty.as_str()) || value.contains(&format!("cast({ty})")) {
                continue;
            }
            hint(
                end,
                format!(": {}", clip(&ty, HINT_CHARS)),
                Some(InlayHintKind::Type),
                false,
                false,
                None,
            );
        }
        for call in &calls {
            let signature = &call.signatures[call.active];
            let ambiguous = ambiguous_params(&signature.params);
            for arg in &call.args {
                let (s, e) = (arg.span.start as usize, arg.span.end as usize);
                if arg.named
                    || arg.variadic
                    || arg.spread
                    || arg.param_name.is_empty()
                    || e > text.len()
                {
                    continue;
                }
                let source = &text[s..e];
                if !literal(source)
                    || source == arg.param_name
                    || !ambiguous.get(arg.param).copied().unwrap_or(false)
                {
                    continue;
                }
                let tooltip = signature.params.get(arg.param).cloned();
                hint(
                    s,
                    format!("{}:", arg.param_name),
                    Some(InlayHintKind::Parameter),
                    false,
                    true,
                    tooltip,
                );
            }
        }
        for e in &expansions {
            if e.kind != IdeExpansionKind::Run || e.texts.len() != 1 || e.texts[0].is_empty() {
                continue;
            }
            let span = span_of(e);
            if span.end > text.len() {
                continue;
            }
            // `#run 3` needs no hint.
            let operand = text[span.start..span.end].trim_start_matches("#run").trim();
            if operand == e.texts[0] {
                continue;
            }
            hint(
                span.end,
                format!("= {}", clip(&e.texts[0], HINT_CHARS)),
                None,
                true,
                false,
                Some(format!("computed at compile time ({})", e.detail)),
            );
        }
        hints.sort_by_key(|(at, _)| *at);
        Ok(hints.into_iter().map(|(_, h)| h).collect())
    }
}
