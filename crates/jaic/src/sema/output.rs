//! What this compile produces: the IR program, the constant data pooled into its globals, and
//! the output compile-time code printed.
use crate::fxhash::HashMap;
use crate::ir;
use crate::sema::{PackKey, driver, runtime_config};
use crate::types::TypeId;
use std::rc::Rc;

#[derive(Default)]
pub struct Output {
    pub program: ir::Program,
    pub(in crate::sema) registered_data_segments: HashMap<ir::GlobalId, u16>,
    /// `__runtime_info`, when the program references it.
    pub(in crate::sema) runtime_info: Option<ir::GlobalId>,
    pub(in crate::sema) runtime_storage_global: Option<ir::GlobalId>,
    pub(in crate::sema) runtime_storage_dependencies: runtime_config::RuntimeStorageDependencies,
    /// String literal globals, deduplicated.
    pub strings: HashMap<Rc<[u8]>, ir::GlobalId>,
    /// Static storage for aggregate constants whose address was taken, one per distinct image.
    pub const_addr_globals: HashMap<(TypeId, Vec<u8>, Vec<ir::Reloc>), ir::GlobalId>,
    /// The data of constant views made from baked variadic arguments, deduplicated by
    /// contents so that equal argument lists share one global (and one instance).
    pub packs: HashMap<PackKey, ir::GlobalId>,
    pub output: Vec<u8>,
    /// Spin lock word shared by `lock_cmpxchg8b`/`lock_cmpxchg16b` in `#asm` blocks.
    pub asm_pair_lock: Option<ir::GlobalId>,
    /// The AES S-boxes used by `aesenc` & co. in `#asm` blocks.
    pub asm_aes_tables: Option<ir::GlobalId>,
    /// The declaration of each global variable of a host twin, so a pointer to one that a run
    /// returns can be matched with the target's own (`host_import`). Empty elsewhere.
    pub(in crate::sema) declared_globals:
        HashMap<ir::GlobalId, (crate::intern::Sym, crate::source::Span)>,
    /// `#no_reset` globals with their types: compiled output starts from the values that
    /// compile-time code left in them (`prepare_compiled_output`).
    pub no_reset_globals: Vec<(ir::GlobalId, TypeId)>,
    /// How much code existed when `finish_program` began checking what the program does not
    /// reach: compiled output drops what was made after it and is still unreferenced.
    pub(in crate::sema) unreferenced_from: Option<driver::ProgramMark>,
}

impl Output {
    /// An empty program built for another platform than this compiler's when `cross_target`.
    pub(in crate::sema) fn new(cross_target: bool) -> Self {
        let mut out = Self::default();
        out.program.cross_target = cross_target;
        out
    }

    /// Drop what only running or emitting the program needs: the bodies of every procedure
    /// lowered so far (keeping their signatures) and the globals' initial contents (keeping
    /// their ids, sizes and names).
    pub(in crate::sema) fn release_lowered(&mut self) {
        for func in self.program.funcs.iter_mut().flatten() {
            func.blocks = Vec::new();
            func.vals = Vec::new();
            func.slots = Vec::new();
            func.trusted = Vec::new();
            func.debug = None;
        }
        for global in &mut self.program.globals {
            global.init = Vec::new();
            global.relocs = Vec::new();
        }
        self.program.debug_types = HashMap::default();
        self.program.debug_globals = Vec::new();
    }
}
