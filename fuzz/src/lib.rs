//! Bounded, host-independent fuzz entrypoints shared with deterministic regressions.
#![forbid(unsafe_code)]
mod checked_ir;
mod source;
pub use checked_ir::checked_ir_vm;
pub use source::{constant_sema, lexer_utf8, module_vfs, parser};

pub const MAX_SOURCE_BYTES: usize = 128 * 1024;
pub const MAX_PARSE_TOKENS: usize = 256;
pub const MAX_RECURSIVE_TOKENS: usize = 32;
pub const MAX_VIRTUAL_FILES: usize = 4;

pub fn limits() -> jai_vm::Limits {
    jai_vm::Limits {
        fuel: 50_000,
        stack_depth: 32,
        evaluation_depth: 64,
        allocations: 64,
        value_cells: 4096,
    }
}

struct Bytes<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Bytes<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            at: 0,
        }
    }

    fn byte(&mut self) -> u8 {
        let byte = self.data.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        byte
    }

    fn signed(&mut self) -> i128 {
        i128::from(self.byte() as i8)
    }
}
