//! Type hierarchy: a struct extends the types it embeds with `using` or `#as using`, so those
//! are its supertypes, and the structs that embed it are its subtypes. Read from the declaration
//! rows of the open files, so it never waits for a compile.
use crate::SymbolKind;
use crate::analysis::SymbolRow;
use crate::session::Session;
use crate::{DocumentUri, Error, Position, TypeHierarchyItem};

impl Session {
    fn type_item(&self, uri: &DocumentUri, row: &SymbolRow) -> Result<TypeHierarchyItem, Error> {
        let doc = self.document(uri)?;
        let detail = Self::source_detail(doc, row).lines().next().unwrap_or("");
        Ok(TypeHierarchyItem {
            name: row.name.as_str().into(),
            detail: detail.trim().into(),
            kind: SymbolKind::Struct,
            uri: uri.as_str().into(),
            range: doc.index.range(&doc.text, row.location)?,
            selection_range: doc.index.range(&doc.text, row.selection)?,
        })
    }

    /// The structs named at `position`: the one declared there, or the ones a use names.
    pub fn prepare_type_hierarchy(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<TypeHierarchyItem>, Error> {
        let mut items: Vec<TypeHierarchyItem> = Vec::new();
        for (target, row) in self.bindings(uri, position)? {
            if row.kind != SymbolKind::Struct || row.local {
                continue;
            }
            let item = self.type_item(target, row)?;
            if !items.contains(&item) {
                items.push(item);
            }
        }
        Ok(items)
    }

    /// The row of the struct `item` names, in the open document it is declared in.
    fn struct_of(&self, item: &TypeHierarchyItem) -> Option<(&DocumentUri, usize)> {
        let uri = DocumentUri::parse(&item.uri).ok()?;
        let (key, doc) = self.document_entry(&uri).ok()?;
        let start = doc.index.byte(&doc.text, item.selection_range.start);
        let at = self
            .analyses
            .get(key)?
            .rows
            .iter()
            .position(|row| row.selection.start == start && row.kind == SymbolKind::Struct)?;
        Some((key, at))
    }

    /// The types `item` embeds with `using` or `#as using`.
    pub fn supertypes(&self, item: &TypeHierarchyItem) -> Result<Vec<TypeHierarchyItem>, Error> {
        let Some((key, id)) = self.struct_of(item) else {
            return Ok(Vec::new());
        };
        let mut items: Vec<TypeHierarchyItem> = Vec::new();
        for (owner, base) in &self.analyses[key].bases {
            if *owner != id {
                continue;
            }
            for candidate in self.reachable(key) {
                for row in &self.analyses[candidate].rows {
                    if row.parent.is_none()
                        && !row.local
                        && row.kind == SymbolKind::Struct
                        && row.name == *base
                        && (candidate == key || !row.file_private)
                    {
                        let found = self.type_item(candidate, row)?;
                        if !items.contains(&found) {
                            items.push(found);
                        }
                    }
                }
            }
        }
        Ok(items)
    }

    /// The structs of the open files that embed `item` with `using` or `#as using`.
    pub fn subtypes(&self, item: &TypeHierarchyItem) -> Result<Vec<TypeHierarchyItem>, Error> {
        let Some((key, id)) = self.struct_of(item) else {
            return Ok(Vec::new());
        };
        let name = self.analyses[key].rows[id].name;
        let mut items: Vec<TypeHierarchyItem> = Vec::new();
        for (uri, analysis) in &self.analyses {
            if analysis.bases.iter().all(|(_, base)| *base != name) {
                continue;
            }
            if uri != key && !self.reachable(uri).contains(&key) {
                continue;
            }
            for (owner, base) in &analysis.bases {
                let row = &analysis.rows[*owner];
                if *base == name && row.parent.is_none() && !row.local {
                    let found = self.type_item(uri, row)?;
                    if !items.contains(&found) {
                        items.push(found);
                    }
                }
            }
        }
        Ok(items)
    }
}
