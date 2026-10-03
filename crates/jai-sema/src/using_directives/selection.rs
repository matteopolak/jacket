//! Selector text names declarations only at the source namespace boundary.
use super::*;

impl Resolver<'_> {
    pub(super) fn using_selected_names(
        &mut self,
        selection: &UsingSelection,
        names: &[Vec<u8>],
        span: Span,
    ) -> Result<Vec<Option<Vec<u8>>>, Diagnostic> {
        let only = matches!(selection, UsingSelection::Only(_));
        match selection {
            UsingSelection::All => Ok(names.iter().cloned().map(Some).collect()),
            UsingSelection::Only(selection) | UsingSelection::Except(selection) => {
                let selected = self.using_selector_names(selection, span)?;
                Ok(names
                    .iter()
                    .map(|name| (selected.contains(name) == only).then(|| name.clone()))
                    .collect())
            }
            UsingSelection::Map(mapper) => self.using_map_names(mapper, names, span).map(|names| {
                names
                    .into_iter()
                    .map(|name| (!name.is_empty()).then_some(name))
                    .collect()
            }),
        }
    }

    fn using_selector_names(
        &mut self,
        names: &UsingNames,
        span: Span,
    ) -> Result<HashSet<Vec<u8>>, Diagnostic> {
        let names = match names {
            UsingNames::Names(names) => names
                .iter()
                .map(|name| (self.symbols.name(name.name).as_bytes().to_vec(), name.span))
                .collect::<Vec<_>>(),
            UsingNames::Expression(expression) => self
                .using_computed_names(expression, span)?
                .into_iter()
                .map(|name| (name, span))
                .collect(),
        };
        let mut selected = HashSet::new();
        for (name, span) in names {
            if !selected.insert(name) {
                return Err(Diagnostic::new(span, "duplicate using selector name"));
            }
        }
        Ok(selected)
    }
}
