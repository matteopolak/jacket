//! Calls out to foreign symbols and native code, and the thunks that let C call back in.
#[cfg(not(target_arch = "wasm32"))]
use super::crash;
use super::rets::Rets;
use super::trap::{Trap, TrapKind};
use super::{FOREIGN_TAG, FUNC_TAG, Interp, Res, TAG_MASK, native, probe};
use crate::ir::{self, ForeignId, FuncId, Program};

/// C functions whose calls leave nothing observable outside the interpreter's memory, so a
/// compile-time run that made only these can be repeated (see `Interp::effects`).
const UNOBSERVABLE_SYMBOLS: &[&str] = &[
    "malloc",
    "calloc",
    "realloc",
    "free",
    "posix_memalign",
    "aligned_alloc",
    "memcpy",
    "memmove",
    "memset",
    "memcmp",
    "strlen",
    "strcmp",
    "strncmp",
];

/// Foreign procedures that never release memory, so the pages `probe` found accessible stay so.
const KEEPS_MEMORY_SYMBOLS: &[&str] = &[
    "malloc",
    "calloc",
    "posix_memalign",
    "aligned_alloc",
    "memcpy",
    "memmove",
    "memset",
    "memcmp",
    "memchr",
    "strlen",
    "strcmp",
    "strncmp",
    "strchr",
    "strrchr",
    "strstr",
    "strcpy",
    "strncpy",
    "getenv",
    "read",
    "write",
    "clock_gettime",
    "gettimeofday",
    "mach_absolute_time",
    "abs",
    "labs",
    "sqrt",
    "sqrtf",
    "sin",
    "sinf",
    "cos",
    "cosf",
    "tan",
    "tanf",
    "atan2",
    "atan2f",
    "pow",
    "powf",
    "floor",
    "ceil",
    "fmod",
    "log",
    "exp",
];

/// Is this a Linux `syscall(number, ...)` that never releases memory? `io_uring_enter` (426, the
/// same on every CPU with `io_uring`) runs once per scheduler poll in `Extensions/Io`, and the
/// completion-ring atomics after it would otherwise ask about their pages again, which means
/// reading all of `/proc/self/maps`: with thousands of fiber stacks that was most of the run.
fn keeps_memory_syscall(symbol: &str, args: &[u64]) -> bool {
    const IO_URING_ENTER: u64 = 426;
    cfg!(target_os = "linux") && symbol == "syscall" && args.first() == Some(&IO_URING_ENTER)
}

/// What a foreign call does to the interpreter's view of the world (`Interp::foreign_effects`).
/// Set once the symbol has been looked at.
const CLASSIFIED: u8 = 1;

/// The call never releases memory.
const KEEPS_MEMORY: u8 = 2;

/// The call can have effects outside the interpreter's memory.
const OBSERVABLE: u8 = 4;

/// The call may write to the terminal, so the build status line goes down first.
const WRITES_OUTPUT: u8 = 8;

/// Does the call leave the terminal alone? Only these known calls do (`write` is in the memory
/// list but writes). Any other may print (`system`, `dprintf`, a spawned child, a library's own
/// output), so the status line goes down before it.
fn is_quiet(symbol: &str) -> bool {
    const QUIET: &[&str] = &[
        "nanosleep",
        "usleep",
        "sleep",
        "clock_nanosleep",
        "sched_yield",
    ];
    symbol != "write"
        && (KEEPS_MEMORY_SYMBOLS.contains(&symbol)
            || UNOBSERVABLE_SYMBOLS.contains(&symbol)
            || QUIET.contains(&symbol)
            || symbol.starts_with("pthread_"))
}

impl Interp {
    pub(super) fn load_required_libraries(&mut self, program: &Program) -> Res<()> {
        if !self.host.native_linking() || (self.compile_time && program.cross_target) {
            return Ok(());
        }
        for (index, info) in program.libraries.iter().enumerate() {
            if !info.link_always {
                continue;
            }
            let mut failure = None;
            let library =
                self.libraries
                    .entry(index)
                    .or_insert_with(|| {
                        match native::Library::open_checked(&info.name, info.system, &info.base_dir)
                        {
                            Ok(library) => Some(library),
                            Err(error) => {
                                failure = Some(error);
                                None
                            }
                        }
                    });
            if library.is_none() {
                return self.trap(format!(
                    "required native library `{}` could not be loaded{}",
                    info.name,
                    failure
                        .map(|reason| format!(": {reason}"))
                        .unwrap_or_default()
                ));
            }
        }
        Ok(())
    }

    pub fn foreign_addr(&mut self, program: &Program, id: ForeignId) -> Res<u64> {
        // A required library may register classes/constructors without supplying
        // the symbol being requested. Include libraries added since an earlier lookup.
        self.load_required_libraries(program)?;
        if let Some(&a) = self.foreign_addrs.get(id.0 as usize)
            && a != 0
        {
            return Ok(a);
        }
        let foreign = &program.foreigns[id.0 as usize];
        let counted = (self.host.native_linking() && !foreign.is_data)
            .then(|| crate::memory_limit::foreign_override(&foreign.symbol))
            .flatten();
        let addr = if counted.is_some() {
            counted
        } else if self.host.native_linking() {
            let lib = match foreign.library {
                Some(l) => {
                    let lib_info = &program.libraries[l];
                    #[cfg(windows)]
                    let mut failure = None;
                    let library = self
                        .libraries
                        .entry(l)
                        .or_insert_with(|| {
                            #[cfg(windows)]
                            {
                                match native::Library::open_checked(
                                    &lib_info.name,
                                    lib_info.system,
                                    &lib_info.base_dir,
                                ) {
                                    Ok(library) => Some(library),
                                    Err(error) => {
                                        failure = Some(error);
                                        None
                                    }
                                }
                            }
                            #[cfg(not(windows))]
                            {
                                native::Library::open(
                                    &lib_info.name,
                                    lib_info.system,
                                    &lib_info.base_dir,
                                )
                            }
                        })
                        .clone();
                    #[cfg(windows)]
                    if library.is_none() {
                        return self.trap(format!(
                            "native library `{}` could not be loaded: {}",
                            lib_info.name,
                            failure.unwrap_or_else(|| "an earlier load attempt failed".into())
                        ));
                    }
                    library
                }
                None => None,
            };
            match native::lookup_checked(lib.as_ref(), &foreign.symbol) {
                Ok(address) => address,
                Err(error) => return self.trap(error),
            }
        } else {
            None
        };
        let addr = match addr {
            Some(a) => a,
            None if foreign.is_data => {
                if let Some(addr) = self.host.foreign_data(&foreign.symbol) {
                    self.foreign_addrs
                        .resize(self.foreign_addrs.len().max(id.0 as usize + 1), 0);
                    self.foreign_addrs[id.0 as usize] = addr;
                    return Ok(addr);
                }
                return self.trap(format!(
                    "foreign variable `{}` is not available",
                    foreign.symbol
                ));
            }
            None => FOREIGN_TAG | id.0 as u64,
        };
        if self.foreign_addrs.len() <= id.0 as usize {
            self.foreign_addrs.resize(id.0 as usize + 1, 0);
        }
        self.foreign_addrs[id.0 as usize] = addr;
        Ok(addr)
    }

    /// The `KEEPS_MEMORY` and `OBSERVABLE` flags of foreign procedure `id`, from its symbol the
    /// first time.
    #[inline]
    fn foreign_effects(&mut self, program: &Program, id: ForeignId) -> u8 {
        match self.foreign_flags.get(id.0 as usize) {
            Some(&flags) if flags != 0 => flags,
            _ => self.classify_foreign(program, id),
        }
    }

    #[cold]
    fn classify_foreign(&mut self, program: &Program, id: ForeignId) -> u8 {
        let symbol = program.foreigns[id.0 as usize].symbol.as_str();
        let mut flags = CLASSIFIED;
        if KEEPS_MEMORY_SYMBOLS.contains(&symbol) {
            flags |= KEEPS_MEMORY;
        }
        if !UNOBSERVABLE_SYMBOLS.contains(&symbol) {
            flags |= OBSERVABLE;
        }
        if !is_quiet(symbol) {
            flags |= WRITES_OUTPUT;
        }
        if self.foreign_flags.len() <= id.0 as usize {
            self.foreign_flags.resize(id.0 as usize + 1, 0);
        }
        self.foreign_flags[id.0 as usize] = flags;
        flags
    }

    pub(super) fn call_foreign(
        &mut self,
        program: &Program,
        id: ForeignId,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Res<Rets> {
        let symbol = program.foreigns[id.0 as usize].symbol.as_str();
        let effects = self.foreign_effects(program, id);
        // Foreign code may release memory the compiler has found readable.
        if effects & KEEPS_MEMORY == 0 && !keeps_memory_syscall(symbol, args) {
            probe::invalidate();
        }
        if effects & OBSERVABLE != 0 {
            self.effects += 1;
        }
        if effects & WRITES_OUTPUT != 0 {
            crate::progress::before_output();
        }
        if !self.compile_time
            && let Some(result) = self.executable_path_foreign(symbol, args)
        {
            return result.map(Rets::from);
        }
        if self.host.cooperative_threads() {
            if let Some(result) = self.inline_thread_foreign(program, symbol, args) {
                return result.map(Rets::from);
            }
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(result) = self.thread_foreign(program, symbol, args) {
                return result.map(Rets::from);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if symbol == "abort" {
            crash::skip_core_dump();
        }
        if let Some(result) = self.host.foreign(symbol, args, sig) {
            let exits = matches!(symbol, "exit" | "_exit");
            return result.map(Rets::from).map_err(|m| {
                Trap::new(m).at(self.loc).of_kind(
                    exits.then(|| TrapKind::Exit(args.first().copied().unwrap_or(0) as i32)),
                )
            });
        }
        let addr = self.foreign_addr(program, id)?;
        if addr & TAG_MASK == FOREIGN_TAG {
            let foreign = &program.foreigns[id.0 as usize];
            let from = foreign
                .library
                .map(|l| &program.libraries[l])
                .filter(|library| !library.system)
                .map(|library| {
                    let dirs = native::library_dirs()
                        .iter()
                        .map(|dir| dir.display().to_string())
                        .collect::<Vec<_>>();
                    format!(
                        " (library `{}` was not found or does not define it; looked beside `{}` and in {})",
                        library.name,
                        library.base_dir,
                        if dirs.is_empty() {
                            "no native library directory".to_string()
                        } else {
                            dirs.join(", ")
                        }
                    )
                })
                .unwrap_or_default();
            let mut trap = self
                .trap::<()>(format!(
                    "foreign procedure `{symbol}` is not available here{from}"
                ))
                .unwrap_err();
            trap.kind = Some(TrapKind::Unavailable);
            return Err(trap);
        }
        // The child of a fork has only the forking thread: it must not wait for the others.
        let forks = matches!(symbol, "fork" | "vfork");
        #[cfg(target_os = "macos")]
        let result = if forks {
            native::main_thread::direct(|| {
                self.call_native(program, addr, args, sig, Some(symbol), false)
            })?
        } else {
            self.call_native(program, addr, args, sig, Some(symbol), true)?
        };
        #[cfg(not(target_os = "macos"))]
        let result = self.call_native(program, addr, args, sig, Some(symbol), !forks)?;
        if symbol == "fork" && result.first() == Some(&0) {
            self.forked_child = true;
            #[cfg(target_os = "macos")]
            native::main_thread::disable();
        }
        Ok(result)
    }

    /// Call C function `addr` (foreign procedure `symbol`, when it is one). Unless `release` is
    /// false, other threads may run interpreted code until it returns.
    pub(super) fn call_native(
        &mut self,
        program: &Program,
        addr: u64,
        args: &[u64],
        sig: &ir::Sig,
        symbol: Option<&str>,
        release: bool,
    ) -> Res<Rets> {
        // Native code reached through a procedure pointer may release memory; a named foreign
        // procedure was classified by `call_foreign`.
        if symbol.is_none() {
            probe::invalidate();
        }
        // A `#c_call` procedure handed to C is a native thunk already (`proc_value`), unless C
        // cannot call it: that stayed tagged, and asking for its thunk again says why.
        let mut thunked: Vec<u64>;
        let argv = if args.iter().any(|v| v & TAG_MASK == FUNC_TAG) {
            thunked = args.to_vec();
            for v in &mut thunked {
                if *v & TAG_MASK != FUNC_TAG {
                    continue;
                }
                let id = FuncId((*v & !TAG_MASK) as u32);
                let Some(sig) = program.func_sig(id).filter(|s| s.conv == ir::Conv::C) else {
                    continue;
                };
                let sig = sig.clone();
                *v = self
                    .thunk(program, id, &sig)
                    .map_err(|m| Trap::new(m).at(self.loc))?;
            }
            thunked.as_slice()
        } else {
            args
        };
        #[cfg(not(target_arch = "wasm32"))]
        self.hand_thunks_to_c(program, argv)?;
        let mine = self.take_exec_state();
        let loc = mine.loc;
        // A crash in C names `symbol` and this thread's interpreted call stack.
        #[cfg(not(target_arch = "wasm32"))]
        let crash_report = symbol.map(|symbol| crash::ForeignCall::enter(symbol, &mine, program));
        #[cfg(target_arch = "wasm32")]
        let _ = symbol;
        let result = self.call_unlocked(addr, argv, sig, release);
        #[cfg(not(target_arch = "wasm32"))]
        drop(crash_report);
        self.put_exec_state(mine);
        result.map_err(|m| Trap::new(m).at(loc))
    }

    /// `native::call`, during which other threads may take the baton (unless `release` is
    /// false) and C may call interpreted procedures back, on this thread or on others.
    #[cfg(not(target_arch = "wasm32"))]
    fn call_unlocked(
        &mut self,
        addr: u64,
        argv: &[u64],
        sig: &ir::Sig,
        release: bool,
    ) -> Result<Rets, String> {
        let Some(shared) = self.shared.clone().filter(|_| release) else {
            return native::call(addr, argv, sig);
        };
        let me = shared.enter_native(std::ptr::from_mut::<Interp>(self) as usize);
        let result = native::calling_out((shared.key(), me), || native::call(addr, argv, sig));
        shared.leave_native(me);
        result
    }

    #[cfg(target_arch = "wasm32")]
    fn call_unlocked(
        &mut self,
        addr: u64,
        argv: &[u64],
        sig: &ir::Sig,
        _release: bool,
    ) -> Result<Rets, String> {
        native::call(addr, argv, sig)
    }

    /// The value of procedure `id` in interpreted code (`FuncAddr`, relocations). A `#c_call`
    /// procedure is a native thunk wherever the host links natively, so C can call it
    /// wherever the program stores it (struct fields, globals, arrays); interpreted calls
    /// through a thunk address are mapped back to the procedure. Anything else is a tagged
    /// id. A procedure keeps one value for the interpreter's lifetime.
    pub(super) fn proc_value(&mut self, program: &Program, id: FuncId) -> u64 {
        let i = id.0 as usize;
        if let Some(&v) = self.proc_values.get(i)
            && v != 0
        {
            return v;
        }
        let tagged = FUNC_TAG | id.0 as u64;
        // The signature is known from the moment the procedure is reserved, so the value is
        // the same whether it is taken before or after the body is lowered.
        let value = match program.func_sig(id) {
            Some(sig) if sig.conv == ir::Conv::C && self.host.native_linking() => {
                // A procedure C cannot call (variadic, `long double`, out of thunks) stays
                // tagged; passing it to C directly reports why.
                let sig = sig.clone();
                self.thunk(program, id, &sig).unwrap_or(tagged)
            }
            _ => tagged,
        };
        if self.proc_values.len() <= i {
            self.proc_values.resize(i + 1, 0);
        }
        self.proc_values[i] = value;
        value
    }

    /// A C-callable thunk for `#c_call` procedure `id`.
    #[cfg(not(target_arch = "wasm32"))]
    fn thunk(&mut self, program: &Program, id: FuncId, sig: &ir::Sig) -> Result<u64, String> {
        let gate: std::sync::Arc<dyn native::Gate> = self.shared();
        let addr = native::callback_addr(
            &gate,
            std::ptr::from_ref::<Program>(program) as u64,
            id,
            sig,
        )?;
        if self.thunk_funcs.insert(addr, id).is_none() {
            self.unseen_thunks.push(addr);
        }
        Ok(addr)
    }

    /// A foreign call is about to run: C can reach every thunk made so far (as an argument, or
    /// through memory the program shares with it), so from now on a thread C started may call
    /// back, and the scheduler must not take all-threads-blocked for a deadlock at once.
    #[cfg(not(target_arch = "wasm32"))]
    fn hand_thunks_to_c(&mut self, program: &Program, arguments: &[u64]) -> Res<()> {
        // A retained procedure value is not itself an exposure to C. Require
        // readiness for callbacks actually passed at this native boundary.
        for &address in arguments {
            if let Some(function) = self.func_of(address) {
                self.require_callable(program, function)?;
            }
        }
        // Exposure bookkeeping can be cleared after a C call, but unresolved
        // readiness must survive until source addition permits another probe.
        let pending: Vec<_> = self.thunk_funcs.values().copied().collect();
        for function in pending {
            self.inspect_callable(program, function)?;
        }
        if !self.unseen_thunks.is_empty() {
            self.unseen_thunks.clear();
            self.shared().callbacks_possible();
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn thunk(&mut self, _program: &Program, _id: FuncId, _sig: &ir::Sig) -> Result<u64, String> {
        Err("C cannot call interpreted procedures here".into())
    }

    /// The interpreted procedure a procedure value names: a tagged id, or a thunk this
    /// interpreter made.
    pub fn func_of(&self, value: u64) -> Option<FuncId> {
        if value & TAG_MASK == FUNC_TAG {
            return Some(FuncId((value & 0xFFFF_FFFF) as u32));
        }
        self.thunk_funcs.get(&value).copied()
    }
}
