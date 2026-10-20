//! Source loading: files, `#load`, `#import`, module instances and the
//! declaration pass that enters top-level statements into scopes.
mod declare;
mod filesystem;
mod imports;
mod loading;
mod not_found;
mod parameters;

pub(super) use self::declare::file_scope_for_eval;
pub use self::filesystem::{
    BootstrapModule, FileSystem, NativeFs, VirtualFs, find_module_in, import_entry,
};
pub(super) use self::loading::FileLoadState;
pub use self::parameters::ParameterContract;
use super::scope::{EntityKind, ScopeKind};
use super::{Compiler, EntityId, ModuleId, PathBuf, Result, ScopeId, Span, ast};
use crate::paths::normalize;
use crate::sema::SourceState;
use std::path::Path;

impl Compiler {
    /// Load every module reachable through imports and expand all pending
    /// top-level items, until nothing changes.
    pub fn expand_all(&mut self) -> Result<()> {
        // First the plain `#if`s and imports everywhere (they decide which files and
        // `#add_context`s exist), then everything else.
        // Only unsettled module and file scopes are visited, in scope order (scopes made on the
        // way are visited too); a scope whose items and imports are all done has nothing to do.
        let mut next = 0;
        while let Some(sid) = self.ns.unsettled.range(ScopeId(next)..).next().copied() {
            next = sid.0 + 1;
            self.validate_parameter_contracts(self.scope(sid).module)?;
            self.expand_plain_ifs(sid)?;
            for i in 0..self.scope(sid).imports.len() {
                self.import_module(sid, i)?;
            }
        }
        let mut next = 0;
        loop {
            while let Some(sid) = self.ns.unsettled.range(ScopeId(next)..).next().copied() {
                next = sid.0 + 1;
                self.validate_parameter_contracts(self.scope(sid).module)?;
                self.expand_pending(sid)?;
                for i in 0..self.scope(sid).imports.len() {
                    self.import_module(sid, i)?;
                }
                if self.settled(sid) {
                    self.ns.unsettled.remove(&sid);
                }
            }
            // Named imports (`X :: #import`) load eagerly too, so every module's
            // `#add_context` is known before the Context type is laid out. The mark moves past
            // an import only once it resolved: one that failed (in a nested call whose caller
            // went on) is tried, and reports its error, again.
            let mut index = self.ns.named_imports_done;
            while index < self.ns.entities.len() {
                let id = EntityId(index as u32);
                let e = self.entity(id);
                let required_library = match &e.kind {
                    EntityKind::Decl {
                        decl, ..
                    } => matches!(
                        decl.value.as_ref().map(|value| &value.kind),
                        Some(ast::ExprKind::UnknownDirective(directive))
                            if directive.kind.is_library()
                                && directive.flags.iter().any(|flag| flag.name.as_str() == "link_always")
                    ),
                    _ => false,
                };
                // Required library metadata must exist before a #run native
                // lookup, just like unnamed link_always declarations. Resolving
                // a literal library name does not lower exports or freeze Context.
                if (matches!(e.kind, EntityKind::Import(_)) || required_library)
                    && matches!(
                        self.scope(e.scope).kind,
                        ScopeKind::Module | ScopeKind::File
                    )
                {
                    self.resolve_entity(id)?;
                }
                index += 1;
                self.ns.named_imports_done = self.ns.named_imports_done.max(index);
            }
            // Deferred items wait until no body is mid-lowering (an outer `expand_all`).
            let lowering = self
                .pr
                .procs
                .iter()
                .any(|p| p.body_state == super::procs::BodyState::Lowering);
            if !self.ns.deferred_pending.is_empty() && !lowering {
                let deferred = std::mem::take(&mut self.ns.deferred_pending);
                self.diag.deferred_errors.clear();
                let retrying = std::mem::replace(&mut self.epochs.retrying_pending, true);
                let result = deferred
                    .into_iter()
                    .try_for_each(|sid| self.expand_pending(sid));
                self.epochs.retrying_pending = retrying;
                result?;
                continue;
            }
            // Resolving a named import may have loaded a module: its scopes come next.
            if self.ns.unsettled.range(ScopeId(next)..).next().is_none() {
                let mut index = 0;
                while index < self.ns.modules.len() {
                    if !self.ns.modules[index].loading {
                        self.validate_module_arguments(ModuleId(index as u32), Span::NONE)?;
                        self.validate_parameter_contracts(ModuleId(index as u32))?;
                    }
                    index += 1;
                }
                // A contract may import its declaring interface or a hook type.
                // Newly registered scopes must participate in normal expansion.
                if self.ns.unsettled.range(ScopeId(next)..).next().is_some() {
                    continue;
                }
                return Ok(());
            }
        }
    }
}

impl SourceState {
    /// `self.fs.canonical(path)`, remembered for the rest of the run: the standard library's
    /// modules are named over and over, and every uncached answer is a `realpath` walk.
    pub(super) fn canon(&self, path: &Path) -> PathBuf {
        if let Some(real) = self.canon_cache.get(path) {
            return real;
        }
        match self.fs.canonical_exact(path) {
            Some(real) => {
                self.canon_cache.remember(path, real.clone());
                real
            }
            None => normalize(path),
        }
    }
}
