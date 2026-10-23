//! Entering, running and leaving procedures, trace hooks and runtime hooks.
#[cfg(not(target_arch = "wasm32"))]
use super::frames::ExecState;
#[cfg(debug_assertions)]
use super::frames::poison_registers;
use super::frames::{Frame, ValueStack};
use super::host::Hook;
use super::rets::Rets;
use super::trap::TrapKind;
use super::{
    COMPILE_TIME_CALLS, COMPILE_TIME_NANOS, FUNC_TAG, Interp, MAX_DEPTH, Res, STACK_SIZE,
    exit_forked_child, record_coverage,
};
use crate::ir::{self, FuncId, Program};
use std::sync::atomic::Ordering;

/// What `exec` saved on entering a frame, to restore when it returns.
pub(super) struct FrameExit {
    pub(super) id: FuncId,
    /// `sp` before the frame.
    base: u64,
    saved_loc: Option<(u32, u32, u32)>,
    /// `trace_loc` before the frame, when the program keeps stack traces.
    saved_trace_loc: Option<Option<Option<(u32, u32, u32)>>>,
    /// The `context.stack_trace` slot and its previous top (`trace_enter`).
    pushed: Option<(u64, u64)>,
    /// The caller's profile counts.
    outer: (u64, u64),
}

/// A frame `Interp::enter` pushed.
struct Entered {
    /// `sp` before the frame.
    base: u64,
    /// Address of the frame's slots.
    stack_base: u64,
    /// Address of its registers.
    regs: *mut u64,
}

/// Whether `result` is a thread of the sandbox's scheduler switching out (`threads_inline.rs`).
pub(super) fn suspended<T>(result: &Res<T>) -> bool {
    matches!(result, Err(trap) if trap.suspends_frames())
}

impl Interp {
    /// Run `func` for C, on whichever thread now holds the baton: on a value stack of its own,
    /// since a thread inside a native call keeps its frames.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn run_callback(
        &mut self,
        program: &Program,
        func: FuncId,
        args: &[u64],
    ) -> Res<Vec<u64>> {
        let stack = self
            .callback_stacks
            .pop()
            .unwrap_or_else(|| ValueStack::heap(STACK_SIZE / 8));
        let outer = self.take_exec_state();
        self.put_exec_state(ExecState {
            stack,
            ..ExecState::default()
        });
        self.call_nesting += 1;
        let result = self.exec(program, func, args);
        self.call_nesting -= 1;
        let mine = self.take_exec_state();
        self.put_exec_state(outer);
        self.callback_stacks.push(mine.stack);
        result.map(Rets::into_vec)
    }

    // Execution

    /// Call a function with raw argument values.
    pub fn call(&mut self, program: &Program, func: FuncId, args: &[u64]) -> Res<Vec<u64>> {
        if self.closed {
            return self.trap("compile-time code no longer runs: the compile is over");
        }
        // Runtime startup loads linked requirements even when the program never
        // calls C. Compile-time pure code need not load another target's libraries;
        // its actual foreign lookups perform the required native initialization.
        if !self.compile_time {
            self.load_required_libraries(program)?;
        }
        if self.stack.is_empty() {
            self.stack = ValueStack::heap(STACK_SIZE / 8);
            self.sp = 0;
        }
        let (sp, depth, calls) = (self.sp, self.depth, self.calls.len());
        let started =
            (self.compile_time && self.call_nesting == 0).then(crate::sema::perf::Stopwatch::start);
        self.call_nesting += 1;
        let mut result = self.exec(program, func, args);
        if self.call_nesting == 1
            && matches!(&result, Err(trap) if trap.kind == Some(TrapKind::Suspended))
        {
            // A thread of the sandbox's scheduler blocked: run the others until this one ends.
            result = self.inline_schedule(program, result);
            if result.is_err() {
                (self.sp, self.depth) = (sp, depth);
                self.calls.truncate(calls);
            }
        }
        self.call_nesting -= 1;
        if let Some(started) = started {
            COMPILE_TIME_NANOS.fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
            COMPILE_TIME_CALLS.fetch_add(1, Ordering::Relaxed);
        }
        let result = result.map(Rets::into_vec);
        if self.forked_child {
            // Compile-time code forked and the child came back here (its `exec*` failed
            // or it trapped): it must not go on compiling alongside the parent.
            if let Err(t) = &result {
                eprintln!(
                    "error in a process forked by compile-time code: {}",
                    t.message
                );
            }
            exit_forked_child(if result.is_ok() {
                0
            } else {
                127
            });
        }
        result
    }

    /// The body of procedure `id`, or a trap when it has none (yet).
    #[inline(always)]
    fn body<'p>(&mut self, program: &'p Program, id: FuncId) -> Res<&'p ir::Func> {
        match program.funcs.get(id.0 as usize) {
            // Every lowered body has an entry block; one without was released
            // (`Compiler::release_lowered_code`).
            Some(Some(func)) if !func.blocks.is_empty() => Ok(func),
            _ => self.no_body(program, id),
        }
    }

    #[cold]
    pub(super) fn no_body<T>(&mut self, program: &Program, id: FuncId) -> Res<T> {
        self.missing_func = Some(id);
        let name = program
            .func_names
            .get(id.0 as usize)
            .cloned()
            .unwrap_or_default();
        if self.defer_missing_bodies && self.compile_time {
            if self.call_nesting != 1 || !self.compiler_threads_idle() {
                return self.trap(format!(
                    "cannot suspend compile-time body dependency `{name}` across a native callback or managed thread"
                ));
            }
            return self.trap_of(
                TrapKind::CompileDependency(id),
                format!("waiting for the body of `{name}`"),
            );
        }
        self.trap(format!(
            "procedure `{name}` has no body available at this point"
        ))
    }

    /// Call procedure `id` with `args`. This is the way in from outside the dispatch loop;
    /// calls by name in interpreted code use `call_by_name`.
    pub(super) fn exec(&mut self, program: &Program, id: FuncId, args: &[u64]) -> Res<Rets> {
        if let Some(hook) = self
            .hooks
            .get(id.0 as usize)
            .copied()
            .flatten()
            .filter(|hook| hook.active(self.compile_time))
        {
            return self.run_hook(program, hook, args);
        }
        let func = self.body(program, id)?;
        let frame = self.frame(program, id, func);
        let entered = self.enter(program, id, func, frame)?;
        let n = args.len().min(func.sig.params.len());
        // SAFETY: `enter` checked the frame's registers fit on the stack, and a procedure
        // has at least a register for each parameter.
        unsafe { std::ptr::copy_nonoverlapping(args.as_ptr(), entered.regs, n) };
        let rets = self.run_entered(program, id, func, frame, &entered)?;
        // SAFETY: the callee left `rets` results at the start of its registers.
        let results = unsafe { std::slice::from_raw_parts(entered.regs, rets as usize) };
        Ok(Rets::collect(results.iter().copied()))
    }

    /// Call a procedure by name from interpreted code: arguments from the registers
    /// `pool[..nargs]` of `regs`, results into the registers `pool[nargs..nargs + nrets]`.
    #[inline(always)]
    pub(super) fn call_by_name(
        &mut self,
        program: &Program,
        regs: *mut u64,
        pool: &[u32],
        callee: u32,
        nargs: usize,
        nrets: usize,
    ) -> Res<()> {
        let id = FuncId(callee);
        let (args, rets) = pool.split_at(nargs);
        // SAFETY (register accesses): `code::build` checked every register named in `pool`.
        if self
            .hooks
            .get(callee as usize)
            .copied()
            .flatten()
            .is_some_and(|hook| hook.active(self.compile_time))
        {
            let mut small = [0u64; 8];
            let argv: Vec<u64>;
            let argv = if nargs <= small.len() {
                for (slot, &a) in small.iter_mut().zip(args) {
                    // SAFETY: `code::build` checked every register named in `pool`, so each is a
                    // register of the caller's frame, which `regs` points at.
                    *slot = unsafe { *regs.add(a as usize) };
                }
                &small[..nargs]
            } else {
                argv = args
                    .iter()
                    // SAFETY: `code::build` checked every register named in `pool`, so each is a
                    // register of the caller's frame, which `regs` points at.
                    .map(|&a| unsafe { *regs.add(a as usize) })
                    .collect();
                &argv
            };
            let out = self.exec(program, id, argv)?;
            for (&r, &v) in rets[..nrets].iter().zip(out.iter()) {
                // SAFETY: `code::build` checked every register named in `pool`, so each is a
                // register of the caller's frame, which `regs` points at.
                unsafe { *regs.add(r as usize) = v };
            }
            return Ok(());
        }
        let func = self.body(program, id)?;
        let frame = self.frame(program, id, func);
        let entered = self.enter(program, id, func, frame)?;
        for (i, &a) in args.iter().enumerate().take(func.sig.params.len()) {
            // SAFETY: `i` is below the callee's parameter count, which `enter` gave registers for;
            // the argument register is one `code::build` checked in the caller's frame.
            unsafe { *entered.regs.add(i) = *regs.add(a as usize) };
        }
        let n = self.run_entered(program, id, func, frame, &entered)? as usize;
        for (i, &r) in rets[..nrets].iter().enumerate().take(n) {
            // SAFETY: `i` is below the results the callee returned, which sit at the start of its
            // registers; the result register is one `code::build` checked in the caller's frame.
            unsafe { *regs.add(r as usize) = *entered.regs.add(i) };
        }
        Ok(())
    }

    /// Push a frame for `id` on the stack: its slots, trace node and registers.
    #[inline(always)]
    fn enter(
        &mut self,
        program: &Program,
        id: FuncId,
        func: &ir::Func,
        frame: &Frame,
    ) -> Res<Entered> {
        if self.depth >= MAX_DEPTH {
            return self.trap_of(
                TrapKind::StackOverflow,
                "stack overflow (recursion too deep)",
            );
        }
        if let Some(covered) = self.covered.as_mut() {
            record_coverage(covered, program, id, func);
        }
        let base = self.sp;
        let stack_start = self.stack.as_mut_ptr() as u64;
        let align = frame.align - 1;
        let start = ((stack_start + base + align) & !align) - stack_start;
        let end = start + frame.size + frame.node_size + frame.code.regs as u64 * 8;
        if end > (self.stack.len() * 8) as u64 {
            return self.trap_of(TrapKind::StackOverflow, "interpreter stack overflow");
        }
        self.sp = end;
        let stack_base = stack_start + start;
        // Registers are not cleared: the IR defines a value before it reads it. Debug builds
        // fill them with a recognizable pattern to catch code that does not.
        #[cfg(debug_assertions)]
        // SAFETY: the registers were checked to fit on the stack above.
        unsafe {
            poison_registers(std::slice::from_raw_parts_mut(
                frame.regs(stack_base),
                frame.code.regs,
            ));
        };
        Ok(Entered {
            base,
            stack_base,
            regs: frame.regs(stack_base),
        })
    }

    /// Run the procedure in the frame `enter` pushed, whose arguments are in place. Its
    /// results are at the start of its registers; returns how many.
    #[inline(always)]
    fn run_entered(
        &mut self,
        program: &Program,
        id: FuncId,
        func: &ir::Func,
        frame: &Frame,
        entered: &Entered,
    ) -> Res<u32> {
        self.depth += 1;
        self.calls.push((id, self.loc));
        let saved_loc = self.loc;
        let (mut saved_trace_loc, mut pushed) = (None, None);
        if program.stack_trace.is_some() {
            saved_trace_loc = Some(self.trace_loc);
            if frame.node_size != 0 {
                // SAFETY: the callee's first registers are its parameters, which the caller
                // set (`context` is the first).
                let args =
                    unsafe { std::slice::from_raw_parts(entered.regs, func.sig.params.len()) };
                pushed = self.trace_enter(
                    program,
                    id,
                    func,
                    frame,
                    args,
                    entered.stack_base + frame.size,
                );
                self.trace_loc = None;
            } else if self.trace_loc.is_none() {
                self.trace_loc = Some(self.loc);
            }
        }
        let outer = if self.profile.is_some() {
            let outer = (self.frame_blocks, self.frame_insts);
            (self.frame_blocks, self.frame_insts) = (0, 0);
            outer
        } else {
            (0, 0)
        };
        let result = self.run_code(program, func, frame, entered.stack_base, entered.regs, None);
        let exit = FrameExit {
            id,
            base: entered.base,
            saved_loc,
            saved_trace_loc,
            pushed,
            outer,
        };
        if suspended(&result) {
            // The thread is switching out: the frame stays on its stack, to be resumed.
            (self.frame_blocks, self.frame_insts) = outer;
            self.capture_frame(exit, frame, entered.stack_base);
            return result;
        }
        self.leave_frame(func, exit, result)
    }

    /// What `run_entered` restores when a frame returns (also for a frame `resume_frames`
    /// resumed).
    #[inline(always)]
    pub(super) fn leave_frame(
        &mut self,
        func: &ir::Func,
        exit: FrameExit,
        mut result: Res<u32>,
    ) -> Res<u32> {
        if let Err(trap) = &mut result {
            trap.push_frame(func, self.loc);
        }
        if let Some(counts) = self.profile.as_mut() {
            counts.add(
                exit.id.0 as usize,
                &func.name,
                self.frame_blocks,
                self.frame_insts,
            );
            (self.frame_blocks, self.frame_insts) = exit.outer;
        }
        if let Some((slot, previous)) = exit.pushed {
            // SAFETY: `slot` is the stack-trace context word `trace_enter` wrote through, in the
            // program's live context memory, and `previous` is the node it held before.
            unsafe { std::ptr::write_unaligned(slot as *mut u64, previous) };
        }
        self.loc = exit.saved_loc;
        if let Some(trace_loc) = exit.saved_trace_loc {
            self.trace_loc = trace_loc;
        }
        self.depth -= 1;
        self.calls.pop();
        self.sp = exit.base;
        result
    }

    /// Push this call onto `context.stack_trace`: the node lives in the caller-visible stack
    /// frame at `node`, and the caller's node learns the line the call was made from.
    /// Returns the context slot and the previous top, to restore on return.
    #[inline]
    fn trace_enter(
        &mut self,
        program: &Program,
        id: FuncId,
        func: &ir::Func,
        frame: &Frame,
        args: &[u64],
        node: u64,
    ) -> Option<(u64, u64)> {
        let layout = program.stack_trace.as_ref()?;
        let n = &layout.node;
        let info = func.trace.as_ref()?;
        let context = *args.first()?;
        if context == 0 {
            return None;
        }
        let slot = context + layout.context;
        let line = self
            .trace_loc
            .unwrap_or(self.loc)
            .map_or(0, |(_, line, _)| line);
        // SAFETY: `context` is the program's live context pointer, so `slot` is its stack-trace
        // word; `previous` and `node` point at stack-trace nodes laid out as `layout.node`
        // describes, `node` inside the frame being entered.
        unsafe {
            let previous = std::ptr::read_unaligned(slot as *const u64);
            let (mut depth, mut hash) = (1u32, 0xcbf2_9ce4_8422_2325u64);
            if previous != 0 {
                std::ptr::write_unaligned((previous + n.line_number) as *mut u32, line);
                depth = std::ptr::read_unaligned((previous + n.call_depth) as *const u32)
                    .wrapping_add(1);
                hash = std::ptr::read_unaligned((previous + n.hash) as *const u64);
            }
            hash = (hash ^ (id.0 as u64) ^ ((line as u64) << 32)).wrapping_mul(0x0100_0000_01b3);
            let info_addr = match frame.trace_info.get() {
                0 => {
                    let addr = self.trace_info(program, id, info);
                    frame.trace_info.set(addr);
                    addr
                }
                addr => addr,
            };
            std::ptr::write_unaligned((node + n.next) as *mut u64, previous);
            std::ptr::write_unaligned((node + n.info) as *mut u64, info_addr);
            std::ptr::write_unaligned((node + n.hash) as *mut u64, hash);
            std::ptr::write_unaligned((node + n.call_depth) as *mut u32, depth);
            std::ptr::write_unaligned((node + n.line_number) as *mut u32, info.line);
            std::ptr::write_unaligned(slot as *mut u64, node);
            Some((slot, previous))
        }
    }

    /// The `Stack_Trace_Procedure_Info` of a procedure (name, declaration site, address).
    fn trace_info(&mut self, program: &Program, id: FuncId, info: &ir::TraceInfo) -> u64 {
        if let Some(&addr) = self.trace_infos.get(id.0 as usize)
            && addr != 0
        {
            return addr;
        }
        fn leak(text: &str) -> u64 {
            Box::leak(text.as_bytes().to_vec().into_boxed_slice()).as_ptr() as u64
        }
        let path = program
            .file_paths
            .get(info.file as usize)
            .map_or("", String::as_str);
        let Some(layout) = &program.stack_trace else {
            return 0;
        };
        let at = &layout.info;
        let words = vec![0u64; at.size.div_ceil(8) as usize];
        let addr = Box::leak(words.into_boxed_slice()).as_mut_ptr() as u64;
        // A string is {count, data}.
        for (offset, value) in [
            (at.name, info.name.len() as u64),
            (at.name + 8, leak(&info.name)),
            (at.path, path.len() as u64),
            (at.path + 8, leak(path)),
            (at.line, info.line as u64),
            (at.column, info.col as u64),
            (at.procedure_address, FUNC_TAG | id.0 as u64),
        ] {
            // SAFETY: `addr` is a freshly leaked block of `at.size` bytes and every offset written
            // lies within the `Stack_Trace_Procedure_Info` layout.
            unsafe { std::ptr::write_unaligned((addr + offset) as *mut u64, value) };
        }
        if self.trace_infos.len() <= id.0 as usize {
            self.trace_infos.resize(id.0 as usize + 1, 0);
        }
        self.trace_infos[id.0 as usize] = addr;
        addr
    }

    fn run_hook(&mut self, program: &Program, hook: Hook, args: &[u64]) -> Res<Rets> {
        match hook {
            Hook::PublicCompiler(index) => {
                let binding = self.public_bindings[index].clone();
                return crate::build::public_api::call(&binding, args, program, self);
            }
            Hook::UnsupportedCompiler(function) => {
                return self.trap(format!(
                    "unsupported compiler operation `{}`",
                    program.func_names[function.0 as usize]
                ));
            }
            Hook::WriteString => {
                let s = args[0];
                let count = self.fetch_u64(s)? as usize;
                let data = self.fetch_u64(s + 8)?;
                let bytes = self.fetch(data, count)?;
                self.effects += 1;
                self.host
                    .write(&bytes, args.get(1).is_some_and(|&v| v & 1 != 0));
            }
            Hook::WriteStrings => {
                let view = args[0];
                let count = self.fetch_u64(view)? as usize;
                let data = self.fetch_u64(view + 8)?;
                let to_stderr = args.get(1).is_some_and(|&v| v & 1 != 0);
                self.effects += 1;
                for i in 0..count {
                    let s = data + i as u64 * 16;
                    let n = self.fetch_u64(s)? as usize;
                    let p = self.fetch_u64(s + 8)?;
                    let bytes = self.fetch(p, n)?;
                    self.host.write(&bytes, to_stderr);
                }
            }
            Hook::DebugBreak => return self.trap("debug_break() was called"),
            Hook::Service => {
                let read = |this: &Self, s: u64| -> Res<Vec<u8>> {
                    let count = this.fetch_u64(s)? as usize;
                    let data = this.fetch_u64(s + 8)?;
                    this.fetch(data, count)
                };
                let service = String::from_utf8_lossy(&read(self, args[0])?).into_owned();
                let request = read(self, args[1])?;
                let response = args[2];
                self.effects += 1;
                let reply = self.host.service(&service, &request);
                let body = reply.body.into_boxed_slice();
                let (count, data) = (body.len() as u64, body.as_ptr() as u64);
                self.service_replies.push(body);
                if response != 0 {
                    self.store(ir::Ty::I64, response, count)?;
                    self.store(ir::Ty::Ptr, response + 8, data)?;
                }
                return Ok(Rets::one(reply.status as u64));
            }
            Hook::Unknown => {
                let s = args[0];
                let count = self.fetch_u64(s)? as usize;
                let data = self.fetch_u64(s + 8)?;
                let message = String::from_utf8_lossy(&self.fetch(data, count)?).into_owned();
                return self.trap_of(TrapKind::Unknown, message);
            }
            Hook::HostMonotonicNanos => {
                static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
                // Start above zero so a reading is never mistaken for an unset value.
                let nanos = ORIGIN
                    .get_or_init(std::time::Instant::now)
                    .elapsed()
                    .as_nanos();
                return Ok(Rets::one((nanos as u64).saturating_add(1_000_000_000)));
            }
            Hook::AssertionFailed {
                context,
                location,
            } => {
                let args = &args[usize::from(context)..];
                let string = |s: &Self, at: u64| -> Res<String> {
                    let count = s.fetch_u64(at)? as usize;
                    let data = s.fetch_u64(at + 8)?;
                    Ok(String::from_utf8_lossy(&s.fetch(data, count)?).into_owned())
                };
                let (loc, message) = (args[0], args[1]);
                let path = string(self, loc + location.path)?;
                let line = self.fetch_u64(loc + location.line)? as u32;
                let col = self.fetch_u64(loc + location.column)? as u32;
                let message = string(self, message)?;
                let mut trap = self
                    .trap::<()>(if message.is_empty() {
                        "assertion failed".to_string()
                    } else {
                        format!("assertion failed: {message}")
                    })
                    .unwrap_err();
                trap.assertion = Some(Box::new((path, line, col)));
                if message.is_empty() {
                    trap.kind = Some(TrapKind::BareAssertion);
                }
                return Err(trap);
            }
            Hook::Meta(op, has_context) => {
                let Some(workspaces) = self.workspaces.clone() else {
                    return self.trap("this build has no compiler workspaces (Compiler module)");
                };
                let loc = self.loc;
                return crate::build::call(&workspaces, op, has_context, args, self).map_err(
                    |mut t| {
                        t.loc = t.loc.or(loc);
                        t
                    },
                );
            }
        }
        Ok(Rets::default())
    }
}
