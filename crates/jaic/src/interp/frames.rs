//! Call frames, the value stack and the saved execution state of a suspended run.
use super::{Call, Interp, Res, code, last_error, profile};
use crate::ir::{self, FuncId, Program};

pub(super) struct Frame {
    pub(super) offsets: Vec<u64>,
    /// Bytes of slots, a multiple of 16.
    pub(super) size: u64,
    /// Largest slot alignment: the frame's absolute start address is a multiple of it, so
    /// `#align 64` locals land aligned (offsets alone only align relative to the frame).
    pub(super) align: u64,
    /// Bytes after the slots for the stack trace node, when the procedure keeps one.
    pub(super) node_size: u64,
    /// The body in the interpreter's form (`code.rs`).
    pub(super) code: code::Code,
    /// The procedure's `Stack_Trace_Procedure_Info` (`trace_info`), once made; 0 before.
    pub(super) trace_info: std::cell::Cell<u64>,
}

impl Frame {
    /// A frame's registers sit after its slots and trace node, on the same stack.
    #[inline(always)]
    pub(super) fn regs(&self, stack_base: u64) -> *mut u64 {
        (stack_base + self.size + self.node_size) as *mut u64
    }
}

/// Preserve the debug poison exactly while copying constant blocks. In an
/// unoptimized compiler, `slice::fill` otherwise executes one Rust iterator step
/// per register on every interpreted call.
#[cfg(debug_assertions)]
pub(super) fn poison_registers(registers: &mut [u64]) {
    const BLOCK: [u64; 32] = [0xdead_beef_dead_beef; 32];
    if registers.len() <= BLOCK.len() {
        registers.copy_from_slice(&BLOCK[..registers.len()]);
        return;
    }
    for chunk in registers.chunks_mut(BLOCK.len()) {
        chunk.copy_from_slice(&BLOCK[..chunk.len()]);
    }
}

#[cfg(all(test, debug_assertions))]
mod register_poison_tests {
    #[test]
    #[allow(clippy::disallowed_methods, reason = "checks host stack words")]
    fn every_register_keeps_the_exact_debug_pattern_and_slice_boundaries() {
        for size in [0, 1, 7, 8, 31, 32, 33, 63, 64, 65, 127, 128, 129, 1024] {
            let mut guarded = vec![0x1234_5678_9abc_def0; size + 2];
            super::poison_registers(&mut guarded[1..=size]);
            assert_eq!(guarded[0], 0x1234_5678_9abc_def0);
            assert_eq!(guarded[size + 1], 0x1234_5678_9abc_def0);
            for word in &guarded[1..=size] {
                assert_eq!(word.to_ne_bytes(), 0xdead_beef_dead_beefu64.to_ne_bytes());
            }
        }
    }
}

/// The memory interpreted frames live on: the words of an allocation of its own, or of a mapping
/// somebody else owns (a fiber's, `fibers.rs`). It dereferences to the words like the box it
/// replaces, and frees only what it allocated.
pub(super) struct ValueStack {
    words: *mut u64,
    len: usize,
    owner: Owner,
}

/// What a stack has to give back.
enum Owner {
    /// Somebody else's memory.
    Borrowed,
    /// A boxed slice made by `heap`.
    Heap,
    /// A block of the program's window, freed when the stack goes.
    Block(
        #[allow(dead_code, reason = "owning it is the point: dropping it frees it")]
        super::ZeroedBlock,
    ),
}

impl Default for ValueStack {
    fn default() -> Self {
        ValueStack {
            words: std::ptr::NonNull::dangling().as_ptr(),
            len: 0,
            owner: Owner::Borrowed,
        }
    }
}

impl ValueStack {
    /// `len` zeroed words from the heap.
    pub(super) fn heap(len: usize) -> Self {
        let words = Box::into_raw(vec![0u64; len].into_boxed_slice());
        ValueStack {
            words: words.cast(),
            len,
            owner: Owner::Heap,
        }
    }

    /// `len` zeroed words inside `window`, so the program may point at them; host memory when
    /// there is no window. `None` when the memory cannot be had.
    pub(super) fn in_window(
        window: Option<&std::sync::Arc<super::Window>>,
        len: usize,
    ) -> Option<Self> {
        let Some(window) = window else {
            return Some(Self::heap(len));
        };
        let block = super::ZeroedBlock::in_window(Some(window), len * 8, 4096)?;
        Some(ValueStack {
            words: window.addresses().host(block.addr()) as *mut u64,
            len,
            owner: Owner::Block(block),
        })
    }

    /// `len` words of memory that outlives the stack and is zeroed.
    #[cfg(all(
        any(unix, windows),
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    pub(super) fn borrowed(words: *mut u64, len: usize) -> Self {
        ValueStack {
            words,
            len,
            owner: Owner::Borrowed,
        }
    }
}

// SAFETY: a stack is used by one thread at a time, like the box it stands in for.
unsafe impl Send for ValueStack {
}

impl std::ops::Deref for ValueStack {
    type Target = [u64];

    fn deref(&self) -> &[u64] {
        // SAFETY: `words` points at `len` words that live as long as `self` (see `heap`/`borrowed`).
        unsafe { std::slice::from_raw_parts(self.words, self.len) }
    }
}

impl std::ops::DerefMut for ValueStack {
    fn deref_mut(&mut self) -> &mut [u64] {
        // SAFETY: as above, and `&mut self` is the only way in.
        unsafe { std::slice::from_raw_parts_mut(self.words, self.len) }
    }
}

impl ValueStack {
    /// Is this the empty stack `take_exec_state` leaves behind? It owns nothing, so it needs no drop.
    #[inline(always)]
    fn is_placeholder(&self) -> bool {
        self.len == 0 && matches!(self.owner, Owner::Borrowed)
    }
}

impl Drop for ValueStack {
    fn drop(&mut self) {
        if matches!(self.owner, Owner::Heap) {
            // SAFETY: made by `heap` from a boxed slice of `len` words.
            drop(unsafe {
                Box::from_raw(std::ptr::slice_from_raw_parts_mut(self.words, self.len))
            });
        }
    }
}

/// What a thread running interpreted code keeps of `Interp` while another thread has it, or
/// while it is inside a native call.
#[derive(Default)]
pub(super) struct ExecState {
    pub(super) stack: ValueStack,
    pub(super) sp: u64,
    pub(super) depth: usize,
    pub(super) calls: Vec<Call>,
    pub(super) loc: Option<(u32, u32, u32)>,
    pub(super) trace_loc: Option<Option<(u32, u32, u32)>>,
}

impl Interp {
    /// The bytes of window a stack of `wanted` bytes takes: a 64 KiB window (2-byte pointers)
    /// has room for a 16 KiB stack beside the program's data, not a megabytes-deep one.
    pub(super) fn stack_bytes(&self, wanted: usize) -> usize {
        match self.window.as_ref().map(|w| w.pointer_size()) {
            Some(2) => wanted.min(4 << 10),
            _ => wanted,
        }
    }

    /// A fresh interpreter stack of `words` words, in the program's window when it has one.
    pub(super) fn new_stack(&mut self, words: usize) -> Res<ValueStack> {
        match ValueStack::in_window(self.window.as_ref(), words) {
            Some(stack) => Ok(stack),
            None => self.trap("cannot allocate the interpreter stack"),
        }
    }

    /// The frame of procedure `id`, made on its first call (and again when its body was
    /// replaced). The reference stays valid for as long as the interpreter lives: frames are
    /// boxed and a replaced one is kept in `retired`, since it may still be running.
    #[inline(always)]
    pub(super) fn frame<'f>(
        &mut self,
        program: &Program,
        id: FuncId,
        func: &ir::Func,
    ) -> &'f Frame {
        let i = id.0 as usize;
        if let Some(Some(f)) = self.frames.get(i)
            && f.code.source == code::fingerprint(func)
        {
            let frame: *const Frame = &raw const **f;
            // SAFETY: see above.
            return unsafe { &*frame };
        }
        self.make_frame(program, id, func)
    }

    #[inline(never)]
    fn make_frame<'f>(&mut self, program: &Program, id: FuncId, func: &ir::Func) -> &'f Frame {
        // The program may be between a failed foreign call and reading its error code.
        let _error = last_error::keep();
        let i = id.0 as usize;
        let mut offsets = Vec::with_capacity(func.slots.len());
        let mut size = 0u64;
        let mut frame_align = 16u64;
        for slot in &func.slots {
            let align = slot.align.clamp(8, 4096);
            frame_align = frame_align.max(align);
            size = size.next_multiple_of(align);
            offsets.push(size);
            size += slot.size.max(1);
        }
        // `#c_call` procedures may be thunks (`proc_value`).
        let native = self.host.native_linking();
        let late_value = |f: FuncId| {
            native
                && program
                    .funcs
                    .get(f.0 as usize)
                    .is_none_or(|g| g.as_ref().is_none_or(|g| g.sig.conv == ir::Conv::C))
        };
        // Straightened control flow skips some block entries, which `-profile` and `-coverage` count.
        let code = code::build(
            func,
            &offsets,
            &late_value,
            &|g| {
                program
                    .globals
                    .get(g.0 as usize)
                    .is_some_and(|g| g.read_only)
            },
            (self.profile.is_none() || !profile::exact()) && self.lines.is_none(),
            &self.layout,
            self.addresses,
        );
        let node_size = match &program.stack_trace {
            Some(layout) if func.trace.is_some() => layout.node.size.next_multiple_of(8),
            _ => 0,
        };
        let frame = Box::new(Frame {
            offsets,
            size: size.next_multiple_of(16),
            align: frame_align,
            node_size,
            code,
            trace_info: std::cell::Cell::new(0),
        });
        if self.frames.len() <= i {
            self.frames.resize_with(i + 1, || None);
        }
        let ptr: *const Frame = &raw const *frame;
        if let Some(old) = self.frames[i].replace(frame) {
            self.retired.push(old);
        }
        // SAFETY: see `frame`.
        unsafe { &*ptr }
    }

    #[inline]
    pub(super) fn take_exec_state(&mut self) -> ExecState {
        ExecState {
            stack: std::mem::take(&mut self.stack),
            sp: self.sp,
            depth: self.depth,
            calls: std::mem::take(&mut self.calls),
            loc: self.loc,
            trace_loc: self.trace_loc,
        }
    }

    #[inline]
    pub(super) fn put_exec_state(&mut self, state: ExecState) {
        // What was left behind is almost always the placeholder, which owns nothing: skip the drop.
        let old = std::mem::replace(&mut self.stack, state.stack);
        if old.is_placeholder() {
            std::mem::forget(old);
        } else {
            drop(old);
        }
        self.sp = state.sp;
        self.depth = state.depth;
        self.calls = state.calls;
        self.loc = state.loc;
        self.trace_loc = state.trace_loc;
    }
}
