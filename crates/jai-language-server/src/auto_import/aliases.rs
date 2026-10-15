//! Callable documentation for simple aliases in the tolerant export index.
use super::*;

type AliasKey = (PathBuf, PathBuf, Vec<String>, bool);

struct Resolver<'a, 'env> {
    sources: &'a Sources<'env>,
    index: &'a mut Index,
    import_dirs: Vec<PathBuf>,
    visiting: BTreeSet<AliasKey>,
}

impl Sources<'_> {
    pub(super) fn alias_documentation(
        &self,
        index: &mut Index,
        root: &Path,
        file: &Path,
        decl: &Decl,
        import_dir: &Path,
    ) -> Vec<Decl> {
        let Some(path) = &decl.alias else {
            return vec![decl.clone()];
        };
        let mut import_dirs = vec![import_dir.to_path_buf()];
        import_dirs.extend((self.env.options)(root).import_paths);
        let mut resolver = Resolver {
            sources: self,
            index,
            import_dirs,
            visiting: BTreeSet::new(),
        };
        let mut resolved = Vec::new();
        for target in resolver.resolve(root, file, path, false) {
            let Some(header) = target.signature.strip_prefix(&target.name) else {
                continue;
            };
            let mut public = decl.clone();
            public.kind = CompletionKind::Function;
            public.signature = format!("{}{header}", decl.name);
            public.alias = None;
            if public.doc.is_empty() {
                public.doc = target.doc;
            }
            if !resolved
                .iter()
                .any(|d: &Decl| d.signature == public.signature)
            {
                resolved.push(public);
            }
        }
        if resolved.is_empty() {
            vec![decl.clone()]
        } else {
            resolved
        }
    }
}

impl Resolver<'_, '_> {
    fn module(&self, name: &str, file: &Path) -> Option<PathBuf> {
        let local = file.parent()?.join("modules");
        std::iter::once(&local)
            .chain(self.import_dirs.iter())
            .flat_map(|dir| {
                [
                    dir.join(name).join("module.jai"),
                    dir.join(format!("{name}.jai")),
                ]
            })
            .find(|path| self.sources.is_file(path))
            .map(|path| project::normalize(&path))
    }

    fn resolve(&mut self, root: &Path, file: &Path, path: &[String], external: bool) -> Vec<Decl> {
        if path.is_empty() || self.visiting.len() >= 64 {
            return Vec::new();
        }
        let key = (root.into(), file.into(), path.into(), external);
        if !self.visiting.insert(key.clone()) {
            return Vec::new();
        }
        let result = self.resolve_inner(root, file, path, external);
        self.visiting.remove(&key);
        result
    }

    fn resolve_inner(
        &mut self,
        root: &Path,
        file: &Path,
        path: &[String],
        external: bool,
    ) -> Vec<Decl> {
        let name = &path[0];
        let current = self.sources.scan(self.index, file);
        let private = !external
            && (current
                .local_decls
                .iter()
                .any(|d| d.name == *name && d.visibility == Visibility::File)
                || current
                    .imports
                    .iter()
                    .any(|d| d.name.as_ref() == Some(name) && d.visibility == Visibility::File));
        let files = if private {
            BTreeSet::from([file.to_path_buf()])
        } else {
            self.sources.closure(self.index, root)
        };
        let mut result = Vec::new();
        for part in files {
            let scan = self.sources.scan(self.index, &part);
            let visible = |visibility: Visibility| {
                if external {
                    visibility == Visibility::Export
                } else if private {
                    visibility == Visibility::File
                } else {
                    visibility != Visibility::File
                }
            };
            for decl in scan
                .local_decls
                .iter()
                .filter(|d| d.name == *name && visible(d.visibility))
            {
                if let Some(alias) = &decl.alias {
                    let mut target = alias.clone();
                    target.extend_from_slice(&path[1..]);
                    result.extend(self.resolve(root, &part, &target, false));
                } else if path.len() == 1 && decl.kind == CompletionKind::Function {
                    result.push(decl.clone());
                }
            }
            for import in scan
                .imports
                .iter()
                .filter(|d| d.name.as_ref() == Some(name) && visible(d.visibility))
            {
                if let Some(module) = self.module(&import.path, &part) {
                    result.extend(self.resolve(&module, &module, &path[1..], true));
                }
            }
        }
        result
    }
}
