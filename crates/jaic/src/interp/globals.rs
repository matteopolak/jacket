//! Global storage: zeroed blocks, materialising a global on first use and resetting them.
use super::host::PendingDataGlobal;
use super::trap::Trap;
use super::{Interp, Res, probe};
use crate::ir::{self, GlobalId, Program};

pub(super) struct GlobalMem {
    /// The block a writable global lives in; `None` in read-only data (`Interp::read_only`).
    _storage: Option<ZeroedBlock>,
    addr: u64,
}

/// Zeroed memory for program data (globals, sandbox `malloc`), released when dropped. It comes
/// from the interpreter's window when the program's pointers are narrower than the host's, and
/// from the host's allocator otherwise. Allocation is fallible: a program asking for more than
/// the host has gets an error or a null pointer instead of aborting the compiler.
pub(crate) struct ZeroedBlock {
    place: Place,
}

enum Place {
    Host {
        ptr: std::ptr::NonNull<u8>,
        layout: std::alloc::Layout,
    },
    Window {
        window: std::sync::Arc<super::Window>,
        offset: u64,
        size: usize,
    },
}

impl ZeroedBlock {
    /// Host memory, for a program whose addresses are host addresses.
    pub(crate) fn new(size: usize, align: usize) -> Option<ZeroedBlock> {
        let layout = std::alloc::Layout::from_size_align(size.max(1), align).ok()?;
        // Zeroed pages are mapped lazily, so a large block nobody touches costs no memory.
        // SAFETY: `layout` has a non-zero size (`size.max(1)`).
        let ptr = std::ptr::NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) })?;

        Some(ZeroedBlock {
            place: Place::Host {
                ptr,
                layout,
            },
        })
    }

    /// Memory from `window` when there is one, else host memory.
    pub(crate) fn in_window(
        window: Option<&std::sync::Arc<super::Window>>,
        size: usize,
        align: usize,
    ) -> Option<ZeroedBlock> {
        let Some(window) = window else {
            return Self::new(size, align);
        };
        let offset = window.alloc(size, align)?;

        Some(ZeroedBlock {
            place: Place::Window {
                window: window.clone(),
                offset,
                size,
            },
        })
    }

    /// The block's program address: its host address, or its offset in the window.
    pub(crate) fn addr(&self) -> u64 {
        match &self.place {
            Place::Host {
                ptr, ..
            } => ptr.as_ptr() as u64,
            Place::Window {
                offset, ..
            } => *offset,
        }
    }
}

impl Drop for ZeroedBlock {
    fn drop(&mut self) {
        match &self.place {
            Place::Host {
                ptr,
                layout,
            } => {
                // SAFETY: `ptr` came from `alloc_zeroed` with this `layout` in `new` and is
                // freed once, here.
                unsafe { std::alloc::dealloc(ptr.as_ptr(), *layout) }
            }
            Place::Window {
                window,
                offset,
                size,
            } => window.free(*offset, *size),
        }

        // Its pages may be unmapped now.
        probe::invalidate();
    }
}

impl Interp {
    /// An array of the pointers `words` in program memory that lives as long as the
    /// interpreter, each as wide as the target's pointer. `None` when memory is out.
    pub fn keep_pointers(&mut self, words: &[u64]) -> Option<u64> {
        let width = self.layout.pointer_size as usize;
        let mut bytes = vec![0; (words.len() * width).max(1)];

        for (word, slot) in words.iter().zip(bytes.chunks_mut(width)) {
            self.layout.write_ptr(slot, *word);
        }

        self.keep(&bytes, width.max(8))
    }

    /// A copy of `bytes` in program memory that lives as long as the interpreter: what the
    /// interpreter hands the program a pointer to (names, tables). `None` when memory is out.
    pub fn keep(&mut self, bytes: &[u8], align: usize) -> Option<u64> {
        let block = ZeroedBlock::in_window(self.window.as_ref(), bytes.len().max(1), align)?;
        let addr = block.addr();

        // SAFETY: the block is `bytes.len().max(1)` bytes of fresh memory that nothing else uses.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.host_addr(addr) as *mut u8,
                bytes.len(),
            );
        }
        self.kept.push(block);
        Some(addr)
    }

    /// Zeroed program memory (window memory when the target's pointers are narrow) the caller
    /// owns.
    pub(crate) fn zeroed_block(&self, size: usize, align: usize) -> Option<ZeroedBlock> {
        ZeroedBlock::in_window(self.window.as_ref(), size, align)
    }

    /// A copy of `bytes` in program memory for a target with narrow pointers; `None` when
    /// program addresses are host addresses and the host's own bytes can be handed out.
    pub(crate) fn keep_narrow(&mut self, bytes: &[u8]) -> Option<u64> {
        self.window.as_ref()?;
        self.keep(bytes, 1)
    }

    /// Host address of a global, materializing it (and what it points to) on first use.
    /// `read_only` globals go where interpreted code cannot write them, as in a native build.
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
        let size_align = usize::try_from(global.size)
            .ok()
            .zip(usize::try_from(align).ok());
        let placed = size_align
            .filter(|_| global.read_only)
            .and_then(|(size, align)| self.read_only.place(size, align))
            .map(|host| self.program_addr(host));
        let mem = if let Some(addr) = placed {
            GlobalMem {
                _storage: None,
                addr,
            }
        } else {
            let Some(storage) = size_align.and_then(|(size, align)| {
                ZeroedBlock::in_window(self.window.as_ref(), size, align)
            }) else {
                return self.trap(format!(
                    "cannot allocate {} bytes for a global variable",
                    global.size
                ));
            };

            GlobalMem {
                addr: storage.addr(),
                _storage: Some(storage),
            }
        };
        let addr = mem.addr;

        // Registered before its relocations are resolved, so globals pointing at each other
        // are made once.
        self.globals[i] = Some(mem);
        self.ranges.insert(addr, (addr + global.size.max(1), g));
        self.initialize_global(program, g, addr, true)?;

        // A note's text from type info: reads of it are reads of that note.
        if let Some(place) = program.note_globals.get(&g)
            && let Some(watch) = self.note_watch.as_mut()
        {
            watch.add(addr, global.size.saturating_sub(1), place.clone());
        }

        Ok(addr)
    }

    /// Write global `g`'s initial contents at `addr`, its live block: zeroes, then `init`, then
    /// the relocated pointers.
    fn initialize_global(
        &mut self,
        program: &Program,
        g: GlobalId,
        addr: u64,
        fresh: bool,
    ) -> Res<()> {
        let global = &program.globals[g.0 as usize];
        let mut pointers = Vec::with_capacity(global.relocs.len());

        for r in &global.relocs {
            let target = match r.target {
                ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                ir::RelocTarget::Func(f) => self.proc_value(program, f),
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };

            pointers.push((r.offset, target.wrapping_add(r.addend as u64)));
        }

        let size = global.size as usize;
        let host = self.host_addr(addr);
        let layout = self.layout;

        self.read_only.fill(host, size, || {
            // SAFETY: `addr` is the live block of this global, `global.size` bytes long (see
            // `global_addr`), writable for now; the copy is no longer than that, and each
            // relocation addresses a pointer-sized slot inside it (the IR only records
            // relocations that fit). Unaligned writes need no alignment.
            unsafe {
                // Newly made blocks are zeroed already.
                if !fresh {
                    std::ptr::write_bytes(host as *mut u8, 0, size);
                }

                std::ptr::copy_nonoverlapping(
                    global.init.as_ptr(),
                    host as *mut u8,
                    global.init.len().min(size),
                );

                for (offset, value) in pointers {
                    let mut word = [0; 8];

                    layout.write_ptr(&mut word, value);
                    std::ptr::copy_nonoverlapping(
                        word.as_ptr(),
                        (host + offset) as *mut u8,
                        layout.pointer_size as usize,
                    );
                }
            }
        });
        Ok(())
    }

    /// Write the initial contents of global `g` from byte `from` on, into its live block and
    /// leaving the bytes before `from` as they are (members appended to a record in use).
    pub fn initialize_global_tail(&mut self, program: &Program, g: GlobalId, from: u64) -> Res<()> {
        let Some(Some(m)) = self.globals.get(g.0 as usize) else {
            return Ok(());
        };
        let addr = m.addr;
        let global = &program.globals[g.0 as usize];
        let mut pointers = Vec::new();

        for r in global.relocs.iter().filter(|r| r.offset >= from) {
            let target = match r.target {
                ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                ir::RelocTarget::Func(f) => self.proc_value(program, f),
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };

            pointers.push((r.offset, target.wrapping_add(r.addend as u64)));
        }

        let from = from as usize;
        let end = (global.size as usize).min(global.init.len());
        let host = self.host_addr(addr);
        let layout = self.layout;

        self.read_only.fill(host, global.size as usize, || {
            // SAFETY: `addr` is the live block of this global, `global.size` bytes long (see
            // `global_addr`); `from..end` lies inside it and inside `init`, and each relocation
            // addresses a pointer-sized slot inside it. Unaligned writes need no alignment.
            unsafe {
                if from < end {
                    std::ptr::copy_nonoverlapping(
                        global.init.as_ptr().add(from),
                        (host as *mut u8).add(from),
                        end - from,
                    );
                }

                for (offset, value) in pointers {
                    let mut word = [0; 8];

                    layout.write_ptr(&mut word, value);
                    std::ptr::copy_nonoverlapping(
                        word.as_ptr(),
                        (host + offset) as *mut u8,
                        layout.pointer_size as usize,
                    );
                }
            }
        });
        Ok(())
    }

    /// Reload an already materialized global from its (rebuilt) initial contents, in place.
    pub fn refresh_global(&mut self, program: &Program, g: GlobalId) -> Res<()> {
        match self.globals.get(g.0 as usize) {
            Some(Some(m)) => self.initialize_global(program, g, m.addr, false),
            _ => Ok(()),
        }
    }

    /// Put every resettable global back to its initial contents (compile-time execution
    /// leaves no state behind in the program that then runs; `#no_reset` opts out).
    pub fn reset_globals(&mut self, program: &Program) -> Res<()> {
        for &g in &program.reset_globals {
            if let Some(Some(m)) = self.globals.get(g.0 as usize) {
                self.initialize_global(program, g, m.addr, false)?;
            }
        }

        Ok(())
    }

    /// Where global `g` lives in interpreter memory, if compile-time code has used it.
    pub fn materialized_global(&self, g: GlobalId) -> Option<u64> {
        self.globals.get(g.0 as usize)?.as_ref().map(|m| m.addr)
    }

    /// Whether the pointer value `p` cannot address compile-time memory, so it is an integer
    /// cast to a pointer (`cast(*void) 32512`, `SIG_ERR`): below 64 KiB, past a narrow
    /// program's window, or, with host-width pointers, an address with the top bit set, which
    /// is never user-space memory on the 64-bit hosts jaic runs on
    /// (`cast(HKEY) cast(s32) 0x80000000`).
    pub fn is_integer_address(&self, p: u64) -> bool {
        match &self.window {
            Some(window) => window.never_memory(p),
            None => p < 0x10000 || (p as i64) < 0,
        }
    }

    /// The global containing `addr`, with the offset into it.
    pub fn global_at(&self, addr: u64) -> Option<(GlobalId, u64)> {
        let (&start, &(end, g)) = self.ranges.range(..=addr).next_back()?;
        (addr < end).then_some((g, addr - start))
    }

    /// Allocates zeroed storage for compiler-managed global data, copies `bytes` into it and
    /// returns its address.
    /// The global stays pending in `pending_data_globals` until `adopt_data_global` binds it.
    ///
    /// Fails when the storage cannot be allocated or written.
    pub fn register_data_global(
        &mut self,
        bytes: &[u8],
        align: u64,
        read_only: bool,
        segment_tag: u16,
    ) -> Res<u64> {
        let storage =
            ZeroedBlock::in_window(self.window.as_ref(), bytes.len().max(1), align as usize)
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

    /// Binds the storage of `data` (from `register_data_global`) to `global`, so addresses inside
    /// it resolve to that global.
    pub fn adopt_data_global(&mut self, global: GlobalId, data: PendingDataGlobal) {
        let address = data.storage.addr();

        self.globals
            .resize_with(self.globals.len().max(global.0 as usize + 1), || None);
        self.globals[global.0 as usize] = Some(GlobalMem {
            _storage: Some(data.storage),
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

        // `Global_Data_Info { version_stamp: u64; segment_info: [] Global_Data_Segment_Info }` and
        // `Global_Data_Segment_Info { segment_tag: u16; data: [] u8 }`: the tag, the data's
        // count and its pointer each start a word, and the pointer is the target's width.
        let view = self.layout.builtins().view;
        let word = self.layout.pointer_size as usize;
        let count = entries.len() / 3;
        let Some(table) = self.keep(&vec![0; (count * 24).max(1)], 16) else {
            return self.trap("cannot allocate the global data snapshot");
        };
        let Some(address) = self.keep(&[0; 24], 16) else {
            return self.trap("cannot allocate the global data snapshot");
        };

        for (i, entry) in entries.chunks(3).enumerate() {
            let at = table + i as u64 * 24;

            // SAFETY: `table` is a block of `count * 24` zeroed bytes.
            unsafe {
                self.poke(at, 8, entry[0]);
                self.poke(at + 8 + view.count, 8, entry[1]);
                self.poke(at + 8 + view.data, word, entry[2]);
            }
        }

        // SAFETY: `address` is a block of 24 zeroed bytes.
        unsafe {
            self.poke(address + 8 + view.count, 8, count as u64);
            self.poke(address + 8 + view.data, word, table);
        }
        Ok(address)
    }
}
