//! Global storage: zeroed blocks, materialising a global on first use and resetting them.
use super::host::PendingDataGlobal;
use super::trap::Trap;
use super::{Interp, Res, probe};
use crate::ir::{self, GlobalId, Program};

pub(super) struct GlobalMem {
    _storage: ZeroedBlock,
    addr: u64,
}

/// Zeroed host memory for program data (globals, sandbox `malloc`), released when dropped.
/// Allocation is fallible: a program asking for more than the host has gets an error or a null
/// pointer instead of aborting the compiler.
pub(crate) struct ZeroedBlock {
    ptr: std::ptr::NonNull<u8>,
    layout: std::alloc::Layout,
}

impl ZeroedBlock {
    pub(crate) fn new(size: usize, align: usize) -> Option<ZeroedBlock> {
        let layout = std::alloc::Layout::from_size_align(size.max(1), align).ok()?;
        // Zeroed pages are mapped lazily, so a large block nobody touches costs no memory.
        let ptr = std::ptr::NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) })?;
        Some(ZeroedBlock {
            ptr,
            layout,
        })
    }

    pub(crate) fn addr(&self) -> u64 {
        self.ptr.as_ptr() as u64
    }
}

impl Drop for ZeroedBlock {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), self.layout) };
        // Its pages may be unmapped now.
        probe::invalidate();
    }
}

impl Interp {
    /// Host address of a global, materializing it (and what it points to) on first use.
    pub fn global_addr(&mut self, program: &Program, g: GlobalId) -> Res<u64> {
        let i = g.0 as usize;
        if let Some(Some(m)) = self.globals.get(i) {
            return Ok(m.addr);
        }
        if self.globals.len() <= i {
            self.globals.resize_with(i + 1, || None);
        }
        let global = &program.globals[i];
        let align = global.align.max(8).next_power_of_two();
        let block = usize::try_from(global.size)
            .ok()
            .zip(usize::try_from(align).ok())
            .and_then(|(size, align)| ZeroedBlock::new(size, align));
        let Some(storage) = block else {
            return self.trap(format!(
                "cannot allocate {} bytes for a global variable",
                global.size
            ));
        };
        let addr = storage.addr();
        unsafe {
            std::ptr::copy_nonoverlapping(
                global.init.as_ptr(),
                addr as *mut u8,
                global.init.len().min(global.size as usize),
            )
        };
        self.globals[i] = Some(GlobalMem {
            _storage: storage,
            addr,
        });
        self.ranges.insert(addr, (addr + global.size.max(1), g));
        for r in &global.relocs {
            let target = match r.target {
                ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                ir::RelocTarget::Func(f) => self.proc_value(program, f),
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };
            let value = target.wrapping_add(r.addend as u64);
            unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
        }
        Ok(addr)
    }

    /// Reload an already materialized global from its (rebuilt) initial contents, in place.
    pub fn refresh_global(&mut self, program: &Program, g: GlobalId) -> Res<()> {
        let i = g.0 as usize;
        let Some(Some(m)) = self.globals.get(i) else {
            return Ok(());
        };
        let addr = m.addr;
        let global = &program.globals[i];
        unsafe {
            std::ptr::write_bytes(addr as *mut u8, 0, global.size as usize);
            std::ptr::copy_nonoverlapping(
                global.init.as_ptr(),
                addr as *mut u8,
                global.init.len().min(global.size as usize),
            )
        };
        for r in &global.relocs {
            let target = match r.target {
                ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                ir::RelocTarget::Func(f) => self.proc_value(program, f),
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };
            let value = target.wrapping_add(r.addend as u64);
            unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
        }
        Ok(())
    }

    /// Put every resettable global back to its initial contents (compile-time execution
    /// leaves no state behind in the program that then runs; `#no_reset` opts out).
    pub fn reset_globals(&mut self, program: &Program) -> Res<()> {
        for &g in &program.reset_globals {
            let i = g.0 as usize;
            let Some(Some(m)) = self.globals.get(i) else {
                continue;
            };
            let addr = m.addr;
            let global = &program.globals[i];
            let init = global.init.len().min(global.size as usize);
            unsafe {
                std::ptr::write_bytes(addr as *mut u8, 0, global.size as usize);
                std::ptr::copy_nonoverlapping(global.init.as_ptr(), addr as *mut u8, init);
            }
            for r in &global.relocs {
                let target = match r.target {
                    ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                    ir::RelocTarget::Func(f) => self.proc_value(program, f),
                    ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
                };
                let value = target.wrapping_add(r.addend as u64);
                unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
            }
        }
        Ok(())
    }

    /// Where global `g` lives in interpreter memory, if compile-time code has used it.
    pub fn materialized_global(&self, g: GlobalId) -> Option<u64> {
        self.globals.get(g.0 as usize)?.as_ref().map(|m| m.addr)
    }

    /// The global containing `addr`, with the offset into it.
    pub fn global_at(&self, addr: u64) -> Option<(GlobalId, u64)> {
        let (&start, &(end, g)) = self.ranges.range(..=addr).next_back()?;
        (addr < end).then_some((g, addr - start))
    }

    pub fn register_data_global(
        &mut self,
        bytes: &[u8],
        align: u64,
        read_only: bool,
        segment_tag: u16,
    ) -> Res<u64> {
        let storage = ZeroedBlock::new(bytes.len().max(1), align as usize)
            .ok_or_else(|| Trap::new("cannot allocate compiler-managed global data"))?;
        let address = storage.addr();
        if !self.write(address, bytes) {
            return Err(Trap::new("cannot initialize compiler-managed global data"));
        }
        self.pending_data_globals.push(PendingDataGlobal {
            storage,
            size: bytes.len() as u64,
            align,
            read_only,
            segment_tag,
        });
        Ok(address)
    }

    pub fn adopt_data_global(&mut self, global: GlobalId, data: PendingDataGlobal) {
        let address = data.storage.addr();
        self.globals
            .resize_with(self.globals.len().max(global.0 as usize + 1), || None);
        self.globals[global.0 as usize] = Some(GlobalMem {
            _storage: data.storage,
            addr: address,
        });
        self.ranges
            .insert(address, (address + data.size.max(1), global));
        self.data_segment_tags.insert(global, data.segment_tag);
    }

    /// Live global addresses in the running program, including registrations made
    /// earlier in this run that the compiler has not adopted yet. Snapshot records
    /// remain owned by this interpreter, while the data pointers retain storage identity.
    pub fn runtime_data_snapshot(&mut self, program: &Program) -> Res<u64> {
        let mut entries = Vec::new();
        for (index, data) in program.globals.iter().enumerate() {
            if data.size == 0 {
                continue;
            }
            let global = GlobalId(index as u32);
            let address = self.global_addr(program, global)?;
            let tag = self
                .data_segment_tags
                .get(&global)
                .copied()
                .unwrap_or(if data.read_only {
                    2
                } else {
                    1
                });
            entries.extend_from_slice(&[tag as u64, data.size, address]);
        }
        for data in &self.pending_data_globals {
            if data.size > 0 {
                entries.extend_from_slice(&[data.segment_tag as u64, data.size, data.address()]);
            }
        }
        let entries = entries.into_boxed_slice();
        let info = vec![0, entries.len() as u64 / 3, entries.as_ptr() as u64].into_boxed_slice();
        let address = info.as_ptr() as u64;
        self.runtime_data_snapshots.push(entries);
        self.runtime_data_snapshots.push(info);
        Ok(address)
    }
}
