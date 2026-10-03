//! Effective source capture keeps pending shadows and scoped imports explicit.
use super::*;

enum CapturedBinding {
    Ready(Binding),
    Pending,
    Placeholder(jai_modules::PlaceholderId),
}

impl LocalScopes {
    pub(crate) fn insertion_capture_bindings(
        &self,
        bindings: &[HashMap<Symbol, Binding>],
        scope: crate::modules::FileScope<'_>,
        symbols: &Symbols,
        span: Span,
    ) -> Result<HashMap<Symbol, Binding>, Diagnostic> {
        if self.frames.len() > bindings.len() {
            return Err(Diagnostic::new(
                span,
                "declaration insertion has an unpaired lexical source frame",
            ));
        }
        let mut effective = HashMap::<Symbol, CapturedBinding>::new();
        for (depth, bindings) in bindings.iter().enumerate() {
            if let Some(frame) = self.frames.get(depth) {
                if !frame.operators.is_empty()
                    || !frame.operator_imports.is_empty()
                    || !frame.using_operator_declarations.is_empty()
                {
                    return Err(Diagnostic::new(
                        span,
                        "declaration insertion requires portable scoped operator capture metadata",
                    ));
                }
                for (name, placeholder) in frame.imports.placeholders() {
                    effective.insert(name, CapturedBinding::Placeholder(placeholder));
                }
                for (&name, &binding) in frame.imports.iter() {
                    effective.insert(name, CapturedBinding::Ready(Binding::Imported(binding)));
                }
                for (&name, &placeholder) in &frame.using_placeholders {
                    effective.insert(name, CapturedBinding::Placeholder(placeholder));
                }
                for (&name, binding) in &frame.using_bindings {
                    effective.insert(name, CapturedBinding::Ready(binding.clone()));
                }
                for &name in frame
                    .declarations
                    .keys()
                    .chain(frame.runtime.keys())
                    .chain(frame.runtime_symbols.iter())
                    .chain(frame.using_pending.iter())
                {
                    effective.insert(name, CapturedBinding::Pending);
                }
            }
            for (&name, binding) in bindings {
                effective.insert(name, CapturedBinding::Ready(binding.clone()));
            }
            if effective.len() > 65_536 {
                return Err(Diagnostic::new(
                    span,
                    "declaration insertion capture exceeds its name limit",
                ));
            }
        }
        let mut effective = effective.into_iter().collect::<Vec<_>>();
        effective.sort_by_key(|(name, _)| symbols.name(*name));
        effective
            .into_iter()
            .map(|(name, binding)| {
                let binding = match binding {
                    CapturedBinding::Ready(binding) => binding,
                    CapturedBinding::Placeholder(placeholder) => {
                        Binding::Imported(scope.imported_placeholder_binding(placeholder, span)?)
                    }
                    CapturedBinding::Pending => {
                        return Err(Diagnostic::new(span, "declaration insertion has a pending lexical declaration or using shadow that cannot be transported as a ready source binding"));
                    }
                };
                Ok((name, binding))
            })
            .collect()
    }
}
