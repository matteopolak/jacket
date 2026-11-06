//! Conditional foreign preservation declarations, never complete memory frames.
use super::{Compiler, ProcId, Summary};
use jaic::{ast, ir};
use std::sync::Arc;

const NOTE: &str = "PreservesUnpassedStorage";
const MAX_NOTES: usize = 64;
const MAX_BYTES: usize = 4096;

/// A declared promise about separately proved fresh, unescaped allocation
/// instances. It supplies neither such an instance nor a provider identity.
#[derive(Clone, Debug)]
pub(in crate::rules) struct UnpassedStorage {
    procedure: ProcId,
    declaration: Arc<ast::ProcLit>,
    declaration_notes: Vec<ast::Note>,
    foreign: ir::ForeignId,
    signature: jaic::types::TypeId,
    library: usize,
    library_name: String,
    library_system: bool,
    library_base: String,
    symbol: String,
    authentication_work: usize,
}
impl PartialEq for UnpassedStorage {
    fn eq(&self, other: &Self) -> bool {
        self.procedure == other.procedure
            && Arc::ptr_eq(&self.declaration, &other.declaration)
            && self.declaration_notes == other.declaration_notes
            && self.foreign == other.foreign
            && self.signature == other.signature
            && self.library == other.library
            && self.library_name == other.library_name
            && self.library_system == other.library_system
            && self.library_base == other.library_base
            && self.symbol == other.symbol
            && self.authentication_work == other.authentication_work
    }
}
impl Eq for UnpassedStorage {
}

fn compatible(s: &Summary) -> bool {
    s.trusted.is_none()
        && s.may.is_empty()
        && s.guaranteed.is_empty()
        && s.returns.is_empty()
        && s.elements.is_empty()
        && s.allocation_effects.is_empty()
        && s.allocations.is_empty()
        && s.native_allocation.is_none()
        && s.locks.is_none()
        && !s.changes_allocator
        && !s.pushes_context
        && !s.observable.exit
        && !s.observable.output
        && s.memory.is_none()
        && s.return_fields.is_empty()
        && s.may_return_fields.is_empty()
}
fn bounded_notes(c: &Compiler, id: ProcId) -> Option<&[ast::Note]> {
    let p = c.pr.procs.get(id.0 as usize)?;
    let notes = c.pr.proc_decl_notes.get(&id).map_or(&[][..], |n| &n[..]);
    if p.lit.header.notes.len() + notes.len() > MAX_NOTES {
        return None;
    }
    let bytes = p
        .lit
        .header
        .notes
        .iter()
        .chain(notes)
        .try_fold(0usize, |n, note| n.checked_add(note.text.len()))?;
    (bytes <= MAX_BYTES).then_some(notes)
}
impl UnpassedStorage {
    pub(super) fn capture(c: &Compiler, id: ProcId, s: &Summary) -> Option<Self> {
        let notes = bounded_notes(c, id)?;
        if !compatible(s) || !c.proc_has_note(id, NOTE) {
            return None;
        }
        let b = super::super::primitives::selected_c_boundary(c, id)?;
        if !b.signature.params.is_empty() || b.signature.returns != [jaic::types::TypeId::S32] {
            return None;
        }
        let bytes = b
            .library
            .name
            .len()
            .checked_add(b.library.base_dir.len())?
            .checked_add(b.foreign.symbol.len())?;
        if bytes > MAX_BYTES || jaic::interrupt::should_stop() {
            return None;
        }
        let note_bytes = c
            .proc(id)
            .lit
            .header
            .notes
            .iter()
            .chain(notes)
            .map(|n| n.text.len())
            .sum::<usize>();
        let authentication_work = 256 + 2 * (note_bytes + bytes);
        Some(Self {
            procedure: id,
            declaration: c.proc(id).lit.clone(),
            declaration_notes: notes.to_vec(),
            foreign: b.foreign_id,
            signature: b.signature.ty,
            library: b.foreign.library?,
            library_name: b.library.name.clone(),
            library_system: b.library.system,
            library_base: b.library.base_dir.clone(),
            symbol: b.foreign.symbol.clone(),
            authentication_work,
        })
    }
    /// Authenticate the promise at this exact selected declaration. A caller
    /// must still prove each protected instance fresh, live and unexposed.
    pub(in crate::rules::lifetime_summary) fn current(
        &self,
        c: &Compiler,
        id: ProcId,
        s: &Summary,
    ) -> bool {
        if id != self.procedure || !compatible(s) {
            return false;
        }
        let Some(p) = c.pr.procs.get(id.0 as usize) else {
            return false;
        };
        if !Arc::ptr_eq(&p.lit, &self.declaration) {
            return false;
        }
        let Some(notes) = bounded_notes(c, id) else {
            return false;
        };
        if notes != self.declaration_notes || !c.proc_has_note(id, NOTE) {
            return false;
        }
        let Some(b) = super::super::primitives::selected_c_boundary(c, id) else {
            return false;
        };
        b.foreign_id == self.foreign
            && b.signature.ty == self.signature
            && b.signature.params.is_empty()
            && b.signature.returns == [jaic::types::TypeId::S32]
            && b.foreign.library == Some(self.library)
            && b.library.name == self.library_name
            && b.library.system == self.library_system
            && b.library.base_dir == self.library_base
            && b.foreign.symbol == self.symbol
            && !jaic::interrupt::should_stop()
    }
    /// Charge before authentication, including bounded current-note preflight,
    /// note/string comparisons and selected/retained ABI rows.
    pub(in crate::rules::lifetime_summary) fn authentication_work(&self) -> usize {
        self.authentication_work
    }
    /// Owned copies only; parsed syntax and note text remain shared with the compiler.
    pub(in crate::rules::lifetime_summary) fn payload_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.declaration_notes.capacity() * std::mem::size_of::<ast::Note>()
            + self.library_name.capacity()
            + self.library_base.capacity()
            + self.symbol.capacity()
    }
}
