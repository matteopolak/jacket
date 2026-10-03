//! Export names come from validated IR identities, never debug labels or source text.
use jai_ir::{EntryPoint, ExportTarget, GlobalId, Library, ProcedureId};
use std::{borrow::Cow, collections::HashMap};

pub(super) struct Symbols<'a> {
    procedures: HashMap<ProcedureId, &'a str>,
    globals: HashMap<GlobalId, &'a str>,
    exported_main: Option<ProcedureId>,
}
impl<'a> Symbols<'a> {
    pub(super) fn new(library: &'a Library) -> Self {
        let mut symbols = Self {
            procedures: HashMap::new(),
            globals: HashMap::new(),
            exported_main: None,
        };
        for export in library.program_exports() {
            let name = export.symbol.as_str();
            match export.target {
                ExportTarget::Procedure(id) => {
                    symbols.procedures.insert(id, name);
                    if name == "main" {
                        symbols.exported_main = Some(id);
                    }
                }
                ExportTarget::Global(id) => {
                    symbols.globals.insert(id, name);
                }
            }
        }
        symbols
    }
    pub(super) fn procedure(&self, id: ProcedureId) -> Cow<'a, str> {
        self.procedures.get(&id).map_or_else(
            || Cow::Owned(format!("jai.p{}", id.index())),
            |name| Cow::Borrowed(*name),
        )
    }
    pub(super) fn global(&self, id: GlobalId) -> Cow<'a, str> {
        self.globals.get(&id).map_or_else(
            || Cow::Owned(format!("jai.g{}", id.index())),
            |name| Cow::Borrowed(*name),
        )
    }
    pub(super) fn synthetic_entry(&self, entry: Option<EntryPoint>) -> Option<EntryPoint> {
        if self.exported_main.is_some() {
            None
        } else {
            entry
        }
    }
}
