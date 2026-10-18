//! Records for files, modules and failed imports.
use super::constants::message_kind;
use super::state::FileEvent;
use crate::records::{Record, RecordTag, Records};
use crate::sema::*;

impl Compiler {
    /// FILE (and, before a module's first file, IMPORT) messages for files
    /// loaded since the last call.
    pub fn export_file_events(&mut self, r: &mut Records) -> Vec<FileEvent> {
        let mut events = Vec::new();
        while self.export.files_announced < self.files.len() {
            let info = &self.files[self.export.files_announced];
            let (file, module) = (info.id, info.module);
            self.export.files_announced += 1;
            if !self.export.modules.contains_key(&module) {
                let id = self.module_record(r, module);
                events.push((message_kind::IMPORT, id));
            }
            let id = self.file_record_in(r, file, module);
            events.push((message_kind::FILE, id));
        }
        events
    }

    /// One `Message_Failed_Import` record per import that found no module, once.
    pub fn export_failed_imports(&mut self, r: &mut Records) -> Vec<i64> {
        std::mem::take(&mut self.failed_imports)
            .into_iter()
            .map(|(host, target)| {
                let mut rec = Record::new(RecordTag::MessageFailedImport);
                rec.int("kind", message_kind::FAILED_IMPORT)
                    .int("workspace", self.workspace)
                    .int("status", 2) // NOT_FOUND
                    .str("host_module_name", host.as_bytes())
                    .str("target_module_name", target.as_bytes());
                r.add(rec)
            })
            .collect()
    }

    fn file_module(&self, file: FileId) -> ModuleId {
        self.files
            .iter()
            .find(|f| f.id == file)
            .map_or(ModuleId(0), |f| f.module)
    }

    fn module_record(&mut self, r: &mut Records, m: ModuleId) -> i64 {
        if let Some(&id) = self.export.modules.get(&m) {
            return id;
        }
        let module_type = if Some(m) == self.preload {
            1
        } else if Some(m) == self.runtime_support {
            2
        } else if Some(m) == self.main_module {
            3
        } else {
            4
        };
        let module = &self.modules[m.0 as usize];
        let entry = module
            .path
            .as_ref()
            .map(|p| self.canon(p).display().to_string())
            .or_else(|| {
                module
                    .files
                    .first()
                    .map(|&f| self.sources.get(f).path.to_string())
            })
            .unwrap_or_default();
        let mut rec = Record::new(RecordTag::MessageImport);
        rec.int("kind", message_kind::IMPORT)
            .int("workspace", self.workspace)
            .int("module_type", module_type)
            .str("module_name", module.name.as_bytes())
            .str(
                "fully_pathed_filename",
                crate::sema::expr::forward_slashes(&entry).as_bytes(),
            )
            .int("__module", m.0 as i64);
        let id = r.add(rec);
        self.export.modules.insert(m, id);
        id
    }

    pub fn file_record(&mut self, r: &mut Records, file: FileId) -> i64 {
        if let Some(&id) = self.export.files.get(&file) {
            return id;
        }
        let module = self.file_module(file);
        self.file_record_in(r, file, module)
    }

    /// `file_record` for a file whose module is known (finding it searches every file).
    pub(super) fn file_record_in(
        &mut self,
        r: &mut Records,
        file: FileId,
        module: ModuleId,
    ) -> i64 {
        if let Some(&id) = self.export.files.get(&file) {
            return id;
        }
        let import = self.module_record(r, module);
        let virtual_source = self.sources.get(file).is_virtual();
        let path = self.sources.get(file).path.to_string();
        let mut rec = Record::new(RecordTag::MessageFile);
        rec.int("kind", message_kind::FILE)
            .int("workspace", self.workspace)
            .str(
                "fully_pathed_filename",
                crate::sema::expr::forward_slashes(&path).as_bytes(),
            )
            .ptr("enclosing_import", import)
            .int("from_a_string", virtual_source as i64)
            .int("__file", file.0 as i64);
        let id = r.add(rec);
        self.export.files.insert(file, id);
        id
    }

    pub(in crate::sema) fn is_system_module(&mut self, m: ModuleId) -> bool {
        if Some(m) == self.preload || Some(m) == self.runtime_support {
            return true;
        }
        if self.export.system_dir.is_none() {
            let dir = self
                .options
                .stdlib_root()
                .map(|root| self.canon(root))
                .or_else(|| {
                    self.runtime_support_entry()
                        .and_then(|p| p.parent().map(|d| self.canon(d)))
                });
            self.export.system_dir = Some(dir);
        }
        if let Some(&system) = self.export.system_modules.get(&m) {
            return system;
        }
        let system = match (&self.export.system_dir, &self.modules[m.0 as usize].path) {
            (Some(Some(dir)), Some(path)) => self.canon(path).starts_with(dir),
            _ => false,
        };
        self.export.system_modules.insert(m, system);
        system
    }
}
