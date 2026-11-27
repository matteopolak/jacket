//! Calls out to foreign symbols and native code, and the thunks that let C call back in.
#[cfg(not(target_arch = "wasm32"))]
use super::crash;
use super::rets::Rets;
use super::trap::{Trap, TrapKind};
use super::{Interp, Res, native, probe, proc_addr, sandbox};
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

/// C library procedures whose arguments, results and effects mean the same in the Linux and
/// macOS C libraries: memory, strings, numbers, process exit, descriptor reads and writes,
/// `FILE*` streams (opaque handles, standard mode strings, `SEEK_*` 0 to 2, 64-bit `off_t`) and
/// path calls that take only paths, `mkdir`'s permission bits or `access`'s `F_OK` family.
/// Their failures show only through `errno`, which is each library's own and not here.
/// Compile-time code built for one of the two may call these natively on a host with the other.
const SAME_IN_EVERY_C_LIBRARY: &[&str] = &[
    "fopen",
    "fclose",
    "fread",
    "fwrite",
    "fflush",
    "fseeko",
    "ftello",
    "fseek",
    "ftell",
    "ferror",
    "feof",
    "realpath",
    "getcwd",
    "chdir",
    "access",
    "mkdir",
    "rmdir",
    "unlink",
    "rename",
    "link",
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
    "memchr",
    "strlen",
    "strcmp",
    "strncmp",
    "strchr",
    "strrchr",
    "strstr",
    "strcpy",
    "strncpy",
    "strtod",
    "strtof",
    "strtol",
    "strtoll",
    "strtoul",
    "strtoull",
    "atof",
    "atoi",
    "atol",
    "atoll",
    "getenv",
    "read",
    "write",
    "close",
    "isatty",
    "exit",
    "_exit",
    "abort",
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

/// The C library a foreign procedure of `program` comes from differs from the one the
/// compiler runs on: the C library of a Linux program on a macOS host, or the reverse, or
/// either on a Windows host. Their symbols, records (`struct stat`), flags (`O_CREAT`) and
/// `errno` numbers differ, so only `SAME_IN_EVERY_C_LIBRARY` may be called natively. Other
/// libraries keep their native lookup, which names a library the host lacks. Windows targets
/// reach their system API through `windows_native` instead.
fn foreign_c_library(program: &Program, foreign: &ir::Foreign) -> bool {
    use crate::target::TargetOs;
    let c_library = foreign.library.is_some_and(|library| {
        let library = &program.libraries[library];
        native::is_c_library(&library.name, library.system)
    });
    let posix = matches!(program.target_os, Some(TargetOs::Linux | TargetOs::MacOS));
    c_library
        && posix
        && program.target_os != Some(crate::sema::compiler_host().0)
        && !SAME_IN_EVERY_C_LIBRARY.contains(&foreign.symbol.as_str())
}

/// Is this a Linux `syscall(number, ...)` that never releases memory? `io_uring_enter` (426, the
/// same on every CPU with `io_uring`) runs once per scheduler poll in `Extensions/Io`, and the
/// completion-ring atomics after it would otherwise ask about their pages again, which means
/// reading all of `/proc/self/maps`: with thousands of fiber stacks that was most of the run.
fn keeps_memory_syscall(symbol: &str, args: &[u64]) -> bool {
    const IO_URING_ENTER: u64 = 426;
    cfg!(target_os = "linux") && symbol == "syscall" && args.first() == Some(&IO_URING_ENTER)
}

/// What calls to one foreign procedure share, worked out on the first call.
pub(super) struct ForeignPlan {
    scheduling: NativeScheduling,
    native: native::Plan,
}

/// Whether native code keeps the interpreter exclusive, lends its baton after the short-call
/// grace, or hands runnable work on before a known host wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativeScheduling {
    Exclusive,
    Short,
    Blocking,
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

/// The symbol is one the interpreter may answer itself or watch (`is_special`): the others go
/// straight to native code.
const SPECIAL: u8 = 16;

/// The call is `fork` or `vfork`.
const FORKS: u8 = 32;

/// Is `symbol` one `call_foreign` treats specially by name, before native code runs: the
/// executable-path queries, thread and synchronisation procedures, and the calls that end or
/// crash the process?
fn is_special(symbol: &str) -> bool {
    super::executable_path::handles(symbol)
        || super::threads_inline::ThreadFn::from_name(symbol).is_some()
        || win32_handles(symbol)
        || matches!(symbol, "exit" | "_exit" | "abort")
}

#[cfg(not(target_arch = "wasm32"))]
use super::threads::win32_handles;

#[cfg(target_arch = "wasm32")]
fn win32_handles(_symbol: &str) -> bool {
    false
}

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
    /// Open the libraries the program links always. Libraries are only ever added, so once
    /// `libraries_loaded` counts them all there is nothing to do: the case on every call.
    #[inline]
    pub(super) fn load_required_libraries(&mut self, program: &Program) -> Res<()> {
        if self.libraries_loaded == program.libraries.len() {
            return Ok(());
        }
        self.open_required_libraries(program)
    }

    #[cold]
    fn open_required_libraries(&mut self, program: &Program) -> Res<()> {
        if !self.host.native_linking() || (self.compile_time && program.cross_target) {
            return Ok(());
        }
        for (index, info) in program.libraries.iter().enumerate() {
            if !info.link_always || info.no_dll {
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
        self.libraries_loaded = program.libraries.len();
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
        // Checked where the address is resolved, so a call through a procedure pointer (a
        // variadic `..Any` foreign procedure, say) is refused as a direct call is.
        if self.compile_time && self.host.native_linking() && foreign_c_library(program, foreign) {
            return self.foreign_c_library_trap(program, id);
        }
        let counted = (self.host.native_linking() && !foreign.is_data)
            .then(|| crate::memory_limit::foreign_override(&foreign.symbol))
            .flatten();
        let addr = if counted.is_some() {
            counted
        } else if self.host.native_linking() {
            let lib = match foreign.library {
                Some(l) => {
                    let lib_info = &program.libraries[l];
                    if lib_info.no_dll {
                        return self.trap(format!(
                            "`{}` cannot be called while compiling: library `{}` has no dynamic library (`,no_dll`)",
                            foreign.symbol, lib_info.name
                        ));
                    }
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
                    #[cfg(not(windows))]
                    if library.is_none()
                        && let Some(note) = native::missing_library(&lib_info.name)
                    {
                        return self.trap(format!(
                            "native library `{}` could not be loaded: {note}",
                            lib_info.name
                        ));
                    }
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
                let (base, layout) = (self.base, self.layout);
                let data = sandbox::guest::with_memory(base, layout, || {
                    self.host.foreign_data(&foreign.symbol)
                });
                if let Some(addr) = data {
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
            None => proc_addr::foreign_word(self.proc_width(), id),
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
        if is_special(symbol) {
            flags |= SPECIAL;
        }
        if matches!(symbol, "fork" | "vfork") {
            flags |= FORKS;
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
        if effects & SPECIAL != 0 {
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
            // The process ends in this call: what ran must be written before it does.
            if matches!(symbol, "exit" | "_exit") && self.lines.is_some() {
                self.flush_profile();
                super::line_cover::write();
            }
            #[cfg(not(target_arch = "wasm32"))]
            if symbol == "abort" {
                crash::skip_core_dump();
            }
        }
        if self.host.answers_foreign() {
            let (base, layout) = (self.base, self.layout);
            if let Some(result) =
                sandbox::guest::with_memory(base, layout, || self.host.foreign(symbol, args, sig))
            {
                let exits = matches!(symbol, "exit" | "_exit");
                return result.map(Rets::from).map_err(|m| {
                    Trap::new(m).at(self.loc).of_kind(
                        exits.then(|| TrapKind::Exit(args.first().copied().unwrap_or(0) as i32)),
                    )
                });
            }
        }
        // Sandbox hosts answer above; only trusted native compile-time evaluation uses
        // real host files. Runtime and native Windows calls retain their normal dispatch.
        if self.compile_time
            && program.target_os == Some(crate::target::TargetOs::Windows)
            && !cfg!(windows)
            && self.host.native_linking()
            && let Some(result) = sandbox::guest::with_memory(self.base, self.layout, || {
                self.windows.foreign(symbol, args, sig)
            })
        {
            return Ok(Rets::from(vec![result]));
        }
        let addr = self.foreign_addr(program, id)?;
        if matches!(
            proc_addr::proc_of(self.proc_width(), addr),
            Some(proc_addr::Proc::Foreign(_))
        ) {
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
                        " (library `{}` was not found or does not define it; looked beside `{}` and in {}){}",
                        library.name,
                        library.base_dir,
                        if dirs.is_empty() {
                            "no native library directory".to_string()
                        } else {
                            dirs.join(", ")
                        },
                        if in_source_checkout(&library.base_dir) {
                            "; a source checkout builds the libraries the standard library binds with `just build`"
                        } else {
                            ""
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
        let forks = effects & FORKS != 0;
        let plan = self.foreign_plan(program, id, addr, sig, forks);
        let scheduling = plan.scheduling;
        #[cfg(target_os = "macos")]
        let result = if forks {
            native::main_thread::direct(|| {
                self.call_native_planned(
                    program,
                    addr,
                    args,
                    sig,
                    Some(symbol),
                    scheduling,
                    Some(&plan.native),
                )
            })?
        } else {
            self.call_native_planned(
                program,
                addr,
                args,
                sig,
                Some(symbol),
                scheduling,
                Some(&plan.native),
            )?
        };
        #[cfg(not(target_os = "macos"))]
        let result = self.call_native_planned(
            program,
            addr,
            args,
            sig,
            Some(symbol),
            scheduling,
            Some(&plan.native),
        )?;
        if effects & FORKS != 0 && symbol == "fork" && result.first() == Some(&0) {
            self.forked_child = true;
            #[cfg(target_os = "macos")]
            native::main_thread::disable();
        }
        Ok(result)
    }

    /// Compile-time code called a procedure of the target's C library, which the host's does
    /// not implement the same way (`foreign_c_library`).
    #[cold]
    fn foreign_c_library_trap(&mut self, program: &Program, id: ForeignId) -> Res<u64> {
        let name = |os| match os {
            crate::target::TargetOs::MacOS => "macOS",
            crate::target::TargetOs::Windows => "Windows",
            _ => "Linux",
        };
        let foreign = &program.foreigns[id.0 as usize];
        let library = foreign
            .library
            .map_or("", |library| program.libraries[library].name.as_str());
        let message = format!(
            "compile-time code built for {} calls `{}` of `{library}`, which the C library of \
             this {} host does not provide the same way",
            program.target_os.map_or("Linux", name),
            foreign.symbol,
            name(crate::sema::compiler_host().0),
        );
        let mut trap = self.trap::<()>(message).unwrap_err();
        trap.kind = Some(TrapKind::Unavailable);
        Err(trap)
    }

    /// The call plan of foreign procedure `id` (resolved to `addr`), made on its first call.
    fn foreign_plan(
        &mut self,
        program: &Program,
        id: ForeignId,
        addr: u64,
        sig: &ir::Sig,
        forks: bool,
    ) -> std::sync::Arc<ForeignPlan> {
        if let Some(Some(plan)) = self.foreign_plans.get(id.0 as usize) {
            return plan.clone();
        }
        let foreign = &program.foreigns[id.0 as usize];
        let scheduling = if forks {
            NativeScheduling::Exclusive
        } else if native::waits::known(
            foreign.library.map(|index| &program.libraries[index]),
            &foreign.symbol,
            addr,
        ) {
            NativeScheduling::Blocking
        } else {
            NativeScheduling::Short
        };
        let plan = std::sync::Arc::new(ForeignPlan {
            scheduling,
            native: native::Plan::new(sig),
        });
        if self.foreign_plans.len() <= id.0 as usize {
            self.foreign_plans.resize(id.0 as usize + 1, None);
        }
        self.foreign_plans[id.0 as usize] = Some(plan.clone());
        plan
    }

    /// Call C function `addr` (foreign procedure `symbol`, when it is one). Unless scheduling
    /// is `Exclusive`, other threads may run interpreted code until it returns.
    pub(super) fn call_native(
        &mut self,
        program: &Program,
        addr: u64,
        args: &[u64],
        sig: &ir::Sig,
        symbol: Option<&str>,
        scheduling: NativeScheduling,
    ) -> Res<Rets> {
        self.call_native_planned(program, addr, args, sig, symbol, scheduling, None)
    }

    /// `call_native`, with the call plan of the procedure when it has been made.
    #[allow(clippy::too_many_arguments)]
    fn call_native_planned(
        &mut self,
        program: &Program,
        addr: u64,
        args: &[u64],
        sig: &ir::Sig,
        symbol: Option<&str>,
        scheduling: NativeScheduling,
        plan: Option<&native::Plan>,
    ) -> Res<Rets> {
        self.read_only.seal();
        // Native code reached through a procedure pointer may release memory; a named foreign
        // procedure was classified by `call_foreign`.
        if symbol.is_none() {
            probe::invalidate();
        }
        // A `#c_call` procedure handed to C is a native thunk already (`proc_value`), unless C
        // cannot call it: that stayed tagged, and asking for its thunk again says why.
        let mut thunked: Vec<u64>;
        let narrow = self.proc_width();
        let func_of = |v: u64| match proc_addr::proc_of(narrow, v) {
            Some(proc_addr::Proc::Func(id)) => Some(id),
            _ => None,
        };
        let argv = if args.iter().any(|&v| func_of(v).is_some()) {
            thunked = args.to_vec();
            for v in &mut thunked {
                let Some(id) = func_of(*v) else {
                    continue;
                };
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
        let result = self.call_unlocked(addr, argv, sig, scheduling, plan);
        #[cfg(not(target_arch = "wasm32"))]
        drop(crash_report);
        self.put_exec_state(mine);
        result.map_err(|m| Trap::new(m).at(loc))
    }

    /// `native::call`, during which other threads may take the baton (unless scheduling is `Exclusive`) and C may call interpreted procedures back, on this thread or on others.
    #[cfg(not(target_arch = "wasm32"))]
    fn call_unlocked(
        &mut self,
        addr: u64,
        argv: &[u64],
        sig: &ir::Sig,
        scheduling: NativeScheduling,
        plan: Option<&native::Plan>,
    ) -> Result<Rets, String> {
        // The program reads the error its last native call left, whatever the interpreter did
        // in between: on Windows its first `GetLastError` is looked up lazily, which resets it.
        let call = || {
            super::last_error::restore_program();
            let result = match plan {
                Some(plan) => native::call_planned(addr, argv, sig, plan),
                None => native::call(addr, argv, sig),
            };
            super::last_error::save_program();
            result
        };
        let Some(shared) = self
            .shared
            .clone()
            .filter(|_| scheduling != NativeScheduling::Exclusive)
        else {
            let result = call();
            super::last_error::restore_program();
            return result;
        };
        let me = shared.enter_native(
            std::ptr::from_mut::<Interp>(self) as usize,
            scheduling == NativeScheduling::Blocking,
        );
        let result = native::calling_out((shared.key(), me), call);
        shared.leave_native(me);
        super::last_error::restore_program();
        result
    }

    #[cfg(target_arch = "wasm32")]
    fn call_unlocked(
        &mut self,
        addr: u64,
        argv: &[u64],
        sig: &ir::Sig,
        _scheduling: NativeScheduling,
        _plan: Option<&native::Plan>,
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
        let tagged = proc_addr::func_word(self.proc_width(), id);
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
            self.thunk_list.push(id);
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
        // Only a compile-time run that defers missing bodies inspects anything.
        if self.compile_time && self.defer_missing_bodies {
            for i in 0..self.thunk_list.len() {
                let function = self.thunk_list[i];
                self.inspect_callable(program, function)?;
            }
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
        if let Some(proc_addr::Proc::Func(id)) = proc_addr::proc_of(self.proc_width(), value) {
            return Some(id);
        }
        self.thunk_funcs.get(&value).copied()
    }
}

/// A library declared in a Jacket source checkout's `stdlib/`, whose native libraries and
/// helpers are built by `just build` rather than shipped beside it as in a release archive.
fn in_source_checkout(base_dir: &str) -> bool {
    std::path::Path::new(base_dir).ancestors().any(|dir| {
        dir.file_name().is_some_and(|name| name == "stdlib")
            && dir
                .parent()
                .is_some_and(|root| root.join("Justfile").is_file())
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_handoff_tests {
    use super::*;
    use crate::interp::ValueStack;

    fn signature(params: Vec<ir::Ty>, returns: Vec<ir::Ty>) -> ir::Sig {
        ir::Sig {
            c_fixed: params.len() as u32,
            params,
            returns,
            conv: ir::Conv::C,
            c_varargs: false,
            c_abi: None,
        }
    }

    extern "C" fn replace_word(word: *mut u64) -> u64 {
        // SAFETY: this test passes one live, naturally aligned stack word.
        unsafe {
            let previous = *word;
            *word = previous + 1;
            previous
        }
    }

    #[test]
    fn a_blocking_call_keeps_parked_arguments_and_execution_state_alive() {
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        interp.stack = ValueStack::heap(4);
        interp.stack[2] = 41;
        interp.sp = 3;
        interp.shared();
        // SAFETY: the stack owns four words, and native code accesses only word two.
        let word = unsafe { interp.stack.as_mut_ptr().add(2) } as u64;
        let result = interp
            .call_native(
                &Program::default(),
                replace_word as *const () as usize as u64,
                &[word],
                &signature(vec![ir::Ty::Ptr], vec![ir::Ty::I64]),
                Some("parked_argument_test"),
                NativeScheduling::Blocking,
            )
            .unwrap();
        assert_eq!(result.first(), Some(&41));
        assert_eq!(interp.stack[2], 42);
        assert_eq!(interp.sp, 3);
        assert!(interp.holds_baton());
    }

    #[test]
    fn a_native_argument_error_still_reacquires_and_restores_the_stack() {
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        interp.stack = ValueStack::heap(1);
        interp.stack[0] = 77;
        interp.shared();
        // The fixed-shape bridge must reject this oversized frame before calling C.
        let result = interp.call_native(
            &Program::default(),
            replace_word as *const () as usize as u64,
            &[0; 128],
            &signature(vec![ir::Ty::I64; 128], vec![ir::Ty::I64]),
            Some("oversized_argument_test"),
            NativeScheduling::Blocking,
        );
        assert!(result.unwrap_err().message.contains("too many"));
        assert_eq!(interp.stack[0], 77);
        assert!(interp.holds_baton());
    }

    #[cfg(target_os = "macos")]
    unsafe extern "C" {
        #[link_name = "__error"]
        fn errno_location() -> *mut i32;
    }

    #[cfg(target_os = "linux")]
    unsafe extern "C" {
        #[link_name = "__errno_location"]
        fn errno_location() -> *mut i32;
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    extern "C" fn failed_wait(error: i32) -> i32 {
        // SAFETY: the host SDK returns this calling OS thread's live errno slot.
        unsafe { *errno_location() = error };
        -1
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    extern "C" fn read_errno() -> i32 {
        // SAFETY: as in `failed_wait`.
        unsafe { *errno_location() }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn a_native_call_sees_errno_the_program_wrote_since_the_last_one() {
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        let failed = &signature(vec![ir::Ty::I32], vec![ir::Ty::I32]);
        let read = &signature(vec![], vec![ir::Ty::I32]);
        for scheduling in [NativeScheduling::Short, NativeScheduling::Exclusive] {
            interp
                .call_native(
                    &Program::default(),
                    failed_wait as *const () as usize as u64,
                    &[33],
                    failed,
                    Some("failed_wait_test"),
                    scheduling,
                )
                .unwrap();
            // The program's `set_errno(0)` before a `readdir`: a store through errno's address.
            // SAFETY: as in `failed_wait`.
            unsafe { *errno_location() = 0 };
            let seen = interp
                .call_native(
                    &Program::default(),
                    read_errno as *const () as usize as u64,
                    &[],
                    read,
                    Some("read_errno_test"),
                    scheduling,
                )
                .unwrap();
            assert_eq!(seen.first().copied().unwrap() as i32, 0, "{scheduling:?}");
        }
    }

    #[cfg(windows)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetLastError(code: u32);
        fn GetLastError() -> u32;
    }

    #[cfg(windows)]
    extern "C" fn fail_with(code: u32) -> i32 {
        // SAFETY: SetLastError only writes the calling thread's error slot.
        unsafe { SetLastError(code) };
        -1
    }

    #[cfg(windows)]
    extern "C" fn read_last_error() -> u32 {
        // SAFETY: GetLastError only reads the calling thread's error slot.
        unsafe { GetLastError() }
    }

    #[cfg(windows)]
    #[test]
    fn the_next_native_call_sees_the_last_error_the_last_one_left() {
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        let failed = &signature(vec![ir::Ty::I32], vec![ir::Ty::I32]);
        let read = &signature(vec![], vec![ir::Ty::I32]);
        for scheduling in [NativeScheduling::Short, NativeScheduling::Exclusive] {
            interp
                .call_native(
                    &Program::default(),
                    fail_with as *const () as usize as u64,
                    &[18],
                    failed,
                    Some("fail_with_test"),
                    scheduling,
                )
                .unwrap();
            // The interpreter's own work between the two calls, such as looking up
            // `GetLastError` on its first use.
            // SAFETY: as in `fail_with`.
            unsafe { SetLastError(0) };
            let seen = interp
                .call_native(
                    &Program::default(),
                    read_last_error as *const () as usize as u64,
                    &[],
                    read,
                    Some("read_last_error_test"),
                    scheduling,
                )
                .unwrap();
            assert_eq!(seen.first().copied().unwrap(), 18, "{scheduling:?}");
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn interrupted_and_timed_out_returns_preserve_errno_after_reacquiring() {
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        interp.shared();
        for error in [4, crate::interp::errno::ETIMEDOUT_HOST as i32] {
            let result = interp
                .call_native(
                    &Program::default(),
                    failed_wait as *const () as usize as u64,
                    &[error as u64],
                    &signature(vec![ir::Ty::I32], vec![ir::Ty::I32]),
                    Some("failed_wait_test"),
                    NativeScheduling::Blocking,
                )
                .unwrap();
            assert_eq!(result.first().copied().unwrap() as i32, -1);
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(error));
            assert!(interp.holds_baton());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_special;

    #[test]
    fn only_symbols_the_interpreter_watches_are_special() {
        for symbol in [
            "pthread_create",
            "pthread_mutex_lock",
            "nanosleep",
            "exit",
            "_exit",
            "abort",
            "readlink",
            "_NSGetExecutablePath",
        ] {
            assert!(is_special(symbol), "{symbol}");
        }
        for symbol in ["labs", "malloc", "write", "sqrt", "fork", "dlopen"] {
            assert!(!is_special(symbol), "{symbol}");
        }
    }
}
