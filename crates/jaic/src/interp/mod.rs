//! IR interpreter. Runs `#run` code at compile time and whole programs in
//! scripting mode and in the browser.
//!
//! Memory is real: pointers are host addresses, globals and stack slots live
//! in host allocations, and foreign procedures are called natively (or
//! through `Host` shims where no dynamic linker exists, e.g. wasm). Procedure
//! values are tagged non-canonical addresses that only the interpreter can
//! call.
#![allow(unsafe_code)]

mod native;

use crate::ir::{
    self, BinOp, Callee, CmpOp, ConvOp, ForeignId, FuncId, GlobalId, Inst, Program, Term, Ty, UnOp,
};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

/// Tag bits marking an interpreted procedure address.
pub const FUNC_TAG: u64 = 0xFEED_0000_0000_0000;
/// Tag bits for foreign procedures that have no native address.
pub const FOREIGN_TAG: u64 = 0xFEEE_0000_0000_0000;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

const STACK_SIZE: usize = 32 << 20;
const MAX_DEPTH: usize = 20_000;

/// Interpreter-implemented `#compiler` procedures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    /// `write_string(s: string, to_standard_error: bool)`
    WriteString,
    /// `write_strings(strings: ..string, to_standard_error: bool)`
    WriteStrings,
    DebugBreak,
    /// A `Compiler` module primitive (`__jaic_*`); the flag says whether the
    /// procedure takes a context pointer.
    Meta(crate::build::MetaOp, bool),
}

/// Where program output and platform services go.
pub trait Host {
    fn write(&mut self, bytes: &[u8], to_stderr: bool);
    /// Implement a foreign procedure without native linking. `None` = not provided.
    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>>;
    /// Whether foreign symbols may be resolved through the native dynamic linker.
    fn native_linking(&self) -> bool;
}

/// Writes to the process's stdout/stderr and links natively.
pub struct NativeHost;
impl Host for NativeHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        use std::io::Write;
        if to_stderr {
            let _ = std::io::stderr().write_all(bytes);
        } else {
            let mut out = std::io::stdout();
            let _ = out.write_all(bytes);
            let _ = out.flush();
        }
    }
    fn foreign(
        &mut self,
        _symbol: &str,
        _args: &[u64],
        _sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        None
    }
    fn native_linking(&self) -> bool {
        cfg!(unix)
    }
}

/// Collects output in memory and implements a small libc for sandboxed runs.
#[derive(Default)]
pub struct SandboxHost {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    allocations: HashMap<u64, (Box<[u64]>, usize)>,
}

impl Host for SandboxHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        if to_stderr {
            self.stderr.extend_from_slice(bytes);
        } else {
            self.stdout.extend_from_slice(bytes);
        }
    }
    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        _sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        Some(Ok(vec![match symbol {
            "write" => {
                let (fd, ptr, len) = (arg(0), arg(1), arg(2) as usize);
                let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
                self.write(&bytes, fd == 2);
                len as u64
            }
            "wasm_write_string" => {
                let (len, ptr, to_stderr) = (arg(0) as usize, arg(1), arg(2) & 1 != 0);
                let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
                self.write(&bytes, to_stderr);
                0
            }
            "malloc" => self.alloc(arg(0) as usize),
            "calloc" => self.alloc(arg(0) as usize * arg(1) as usize),
            "realloc" => {
                let (old, size) = (arg(0), arg(1) as usize);
                let new = self.alloc(size);
                if let Some((_, old_size)) = self.allocations.get(&old) {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            old as *const u8,
                            new as *mut u8,
                            (*old_size).min(size),
                        )
                    };
                    self.allocations.remove(&old);
                }
                new
            }
            "free" => {
                self.allocations.remove(&arg(0));
                0
            }
            "memcpy" | "memmove" => {
                unsafe { std::ptr::copy(arg(1) as *const u8, arg(0) as *mut u8, arg(2) as usize) };
                arg(0)
            }
            "memset" => {
                unsafe { std::ptr::write_bytes(arg(0) as *mut u8, arg(1) as u8, arg(2) as usize) };
                arg(0)
            }
            "strlen" => {
                let mut n = 0;
                while unsafe { *((arg(0) + n) as *const u8) } != 0 {
                    n += 1;
                }
                n
            }
            "exit" | "_exit" => return Some(Err(format!("exit({})", arg(0) as i32))),
            "abort" => return Some(Err("abort()".into())),
            "pthread_mutex_lock"
            | "pthread_mutex_unlock"
            | "pthread_mutex_init"
            | "pthread_mutex_destroy" => 0,
            "isatty" => 0,
            _ => return None,
        }]))
    }
    fn native_linking(&self) -> bool {
        false
    }
}

impl SandboxHost {
    fn alloc(&mut self, size: usize) -> u64 {
        let words = size.div_ceil(8).max(1) + 1;
        let mut block = vec![0u64; words].into_boxed_slice();
        // 16-byte alignment: skip one word when needed.
        let base = block.as_mut_ptr() as u64;
        let addr = if base % 16 == 0 {
            base
        } else {
            base + 8
        };
        self.allocations.insert(addr, (block, size));
        addr
    }
}

/// A runtime failure inside interpreted code.
#[derive(Debug, Clone)]
pub struct Trap {
    pub message: String,
    /// (file, line, column) of the last executed statement.
    pub loc: Option<(u32, u32, u32)>,
}

type Res<T> = std::result::Result<T, Trap>;

struct Frame {
    offsets: Vec<u64>,
    size: u64,
}

struct GlobalMem {
    _storage: Box<[u64]>,
    addr: u64,
}

pub struct Interp {
    stack: Box<[u64]>,
    sp: u64,
    globals: Vec<Option<GlobalMem>>,
    /// start address -> (end, global) for address lookups.
    ranges: BTreeMap<u64, (u64, GlobalId)>,
    frames: Vec<Option<Rc<Frame>>>,
    foreign_addrs: HashMap<ForeignId, u64>,
    libraries: HashMap<usize, Option<native::Library>>,
    pub hooks: HashMap<FuncId, Hook>,
    pub host: Box<dyn Host>,
    depth: usize,
    loc: Option<(u32, u32, u32)>,
    /// True while evaluating compile-time code (`#compile_time`).
    pub compile_time: bool,
    /// Workspace registry for the `Compiler` module, when the embedder has one.
    pub workspaces: Option<crate::build::SharedWorkspaces>,
    /// Bodies and source text of the compiler's `Code` values, by `CodeId`.
    pub codes: Vec<(std::rc::Rc<crate::ast::CodeBody>, std::rc::Rc<str>)>,
}

impl Default for Interp {
    fn default() -> Self {
        Self::new(Box::new(NativeHost))
    }
}

impl Interp {
    pub fn new(host: Box<dyn Host>) -> Self {
        Self {
            stack: Vec::new().into_boxed_slice(),
            sp: 0,
            globals: Vec::new(),
            ranges: BTreeMap::new(),
            frames: Vec::new(),
            foreign_addrs: HashMap::new(),
            libraries: HashMap::new(),
            hooks: HashMap::new(),
            host,
            depth: 0,
            loc: None,
            compile_time: true,
            workspaces: None,
            codes: Vec::new(),
        }
    }

    fn trap<T>(&self, message: impl Into<String>) -> Res<T> {
        Err(Trap {
            message: message.into(),
            loc: self.loc,
        })
    }

    // -----------------------------------------------------------------------
    // Memory
    // -----------------------------------------------------------------------

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
        let align = global.align.max(8);
        let words = (global.size + align).div_ceil(8) as usize;
        let mut storage = vec![0u64; words.max(1)].into_boxed_slice();
        let base = storage.as_mut_ptr() as u64;
        let addr = base.next_multiple_of(align);
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
                ir::RelocTarget::Func(f) => FUNC_TAG | f.0 as u64,
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };
            let value = target.wrapping_add(r.addend as u64);
            unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
        }
        Ok(addr)
    }

    /// The global containing `addr`, with the offset into it.
    pub fn global_at(&self, addr: u64) -> Option<(GlobalId, u64)> {
        let (&start, &(end, g)) = self.ranges.range(..=addr).next_back()?;
        (addr < end).then_some((g, addr - start))
    }

    pub fn read(&self, addr: u64, len: usize) -> Vec<u8> {
        if len == 0 {
            return Vec::new();
        }
        unsafe { std::slice::from_raw_parts(addr as *const u8, len) }.to_vec()
    }
    pub fn read_u64(&self, addr: u64) -> u64 {
        unsafe { std::ptr::read_unaligned(addr as *const u64) }
    }
    pub fn write(&mut self, addr: u64, bytes: &[u8]) {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()) };
    }

    fn load(&self, ty: Ty, addr: u64) -> Res<u64> {
        if addr < 4096 {
            return self.trap(format!(
                "invalid memory read at address {addr:#x} (null pointer?)"
            ));
        }
        if addr & TAG_MASK == FUNC_TAG || addr & TAG_MASK == FOREIGN_TAG {
            return self.trap("read through a procedure address");
        }
        let p = addr as *const u8;
        Ok(unsafe {
            match ty {
                Ty::I8 => *p as u64,
                Ty::I16 => std::ptr::read_unaligned(p as *const u16) as u64,
                Ty::I32 | Ty::F32 => std::ptr::read_unaligned(p as *const u32) as u64,
                Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::read_unaligned(p as *const u64),
            }
        })
    }

    fn store(&self, ty: Ty, addr: u64, v: u64) -> Res<()> {
        if addr < 4096 {
            return self.trap(format!(
                "invalid memory write at address {addr:#x} (null pointer?)"
            ));
        }
        let p = addr as *mut u8;
        unsafe {
            match ty {
                Ty::I8 => *p = v as u8,
                Ty::I16 => std::ptr::write_unaligned(p as *mut u16, v as u16),
                Ty::I32 | Ty::F32 => std::ptr::write_unaligned(p as *mut u32, v as u32),
                Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::write_unaligned(p as *mut u64, v),
            }
        }
        Ok(())
    }

    fn frame(&mut self, program: &Program, id: FuncId) -> Rc<Frame> {
        let i = id.0 as usize;
        if let Some(Some(f)) = self.frames.get(i) {
            return f.clone();
        }
        let func = program.funcs[i].as_ref().unwrap();
        let mut offsets = Vec::with_capacity(func.slots.len());
        let mut size = 0u64;
        for slot in &func.slots {
            let align = slot.align.clamp(8, 4096);
            size = size.next_multiple_of(align);
            offsets.push(size);
            size += slot.size.max(1);
        }
        let frame = Rc::new(Frame {
            offsets,
            size: size.next_multiple_of(16),
        });
        if self.frames.len() <= i {
            self.frames.resize_with(i + 1, || None);
        }
        self.frames[i] = Some(frame.clone());
        frame
    }

    // -----------------------------------------------------------------------
    // Foreign symbols
    // -----------------------------------------------------------------------

    pub fn foreign_addr(&mut self, program: &Program, id: ForeignId) -> Res<u64> {
        if let Some(&a) = self.foreign_addrs.get(&id) {
            return Ok(a);
        }
        let foreign = &program.foreigns[id.0 as usize];
        let addr = if self.host.native_linking() {
            let lib = match foreign.library {
                Some(l) => {
                    let lib_info = &program.libraries[l];
                    self.libraries
                        .entry(l)
                        .or_insert_with(|| {
                            native::Library::open(
                                &lib_info.name,
                                lib_info.system,
                                &lib_info.base_dir,
                            )
                        })
                        .clone()
                }
                None => None,
            };
            native::lookup(lib.as_ref(), &foreign.symbol)
        } else {
            None
        };
        let addr = match addr {
            Some(a) => a,
            None if foreign.is_data => {
                return self.trap(format!(
                    "foreign variable '{}' is not available",
                    foreign.symbol
                ));
            }
            None => FOREIGN_TAG | id.0 as u64,
        };
        self.foreign_addrs.insert(id, addr);
        Ok(addr)
    }

    fn call_foreign(
        &mut self,
        program: &Program,
        id: ForeignId,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Res<Vec<u64>> {
        let symbol = program.foreigns[id.0 as usize].symbol.clone();
        if let Some(result) = self.host.foreign(&symbol, args, sig) {
            return result.map_err(|m| Trap {
                message: m,
                loc: self.loc,
            });
        }
        let addr = self.foreign_addr(program, id)?;
        if addr & TAG_MASK == FOREIGN_TAG {
            return self.trap(format!(
                "foreign procedure '{symbol}' is not available here"
            ));
        }
        self.call_native(addr, args, sig)
    }

    fn call_native(&mut self, addr: u64, args: &[u64], sig: &ir::Sig) -> Res<Vec<u64>> {
        if let Some(abi) = &sig.c_abi
            && (abi.ret.is_some() || abi.params.iter().any(Option::is_some))
        {
            return self.trap("calling foreign procedures with by-value structs is not supported by the interpreter yet");
        }
        native::call(addr, args, sig).map_err(|m| Trap {
            message: m,
            loc: self.loc,
        })
    }

    // -----------------------------------------------------------------------
    // Execution
    // -----------------------------------------------------------------------

    /// Call a function with raw argument values.
    pub fn call(&mut self, program: &Program, func: FuncId, args: &[u64]) -> Res<Vec<u64>> {
        if self.stack.is_empty() {
            self.stack = vec![0u64; STACK_SIZE / 8].into_boxed_slice();
            self.sp = 0;
        }
        self.exec(program, func, args)
    }

    fn exec(&mut self, program: &Program, id: FuncId, args: &[u64]) -> Res<Vec<u64>> {
        if let Some(&hook) = self.hooks.get(&id) {
            return self.run_hook(hook, args);
        }
        let Some(func) = program.funcs.get(id.0 as usize).and_then(Option::as_ref) else {
            let name = program
                .func_names
                .get(id.0 as usize)
                .cloned()
                .unwrap_or_default();
            return self.trap(format!(
                "procedure '{name}' has no body available at this point"
            ));
        };
        if self.depth >= MAX_DEPTH {
            return self.trap("stack overflow (recursion too deep)");
        }
        let frame = self.frame(program, id);
        let base = self.sp;
        if base + frame.size > (self.stack.len() * 8) as u64 {
            return self.trap("interpreter stack overflow");
        }
        self.sp += frame.size;
        self.depth += 1;
        let stack_base = self.stack.as_mut_ptr() as u64 + base;
        let result = self.run(program, func, &frame, stack_base, args);
        self.depth -= 1;
        self.sp = base;
        result
    }

    fn run_hook(&mut self, hook: Hook, args: &[u64]) -> Res<Vec<u64>> {
        match hook {
            Hook::WriteString => {
                let s = args[0];
                let count = self.read_u64(s) as usize;
                let data = self.read_u64(s + 8);
                let bytes = self.read(data, count);
                self.host
                    .write(&bytes, args.get(1).is_some_and(|&v| v & 1 != 0));
            }
            Hook::WriteStrings => {
                let view = args[0];
                let count = self.read_u64(view) as usize;
                let data = self.read_u64(view + 8);
                let to_stderr = args.get(1).is_some_and(|&v| v & 1 != 0);
                for i in 0..count {
                    let s = data + i as u64 * 16;
                    let n = self.read_u64(s) as usize;
                    let p = self.read_u64(s + 8);
                    let bytes = self.read(p, n);
                    self.host.write(&bytes, to_stderr);
                }
            }
            Hook::DebugBreak => return self.trap("debug_break() was called"),
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
        Ok(Vec::new())
    }

    fn run(
        &mut self,
        program: &Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        args: &[u64],
    ) -> Res<Vec<u64>> {
        let mut vals = vec![0u64; func.vals.len()];
        vals[..args.len().min(func.sig.params.len())]
            .copy_from_slice(&args[..args.len().min(func.sig.params.len())]);
        let mut block = 0usize;
        loop {
            let b = &func.blocks[block];
            for inst in &b.insts {
                self.step(program, inst, &mut vals, frame, stack_base)?;
            }
            match &b.term {
                Term::Jump(t) => block = t.0 as usize,
                Term::Branch {
                    cond,
                    then_block,
                    else_block,
                } => {
                    block = if vals[cond.0 as usize] & 0xff != 0 {
                        then_block.0
                    } else {
                        else_block.0
                    } as usize;
                }
                Term::Switch {
                    value,
                    ty,
                    cases,
                    default,
                } => {
                    let v = mask(*ty, vals[value.0 as usize]);
                    block = cases
                        .iter()
                        .find(|(c, _)| mask(*ty, *c) == v)
                        .map_or(default.0, |(_, t)| t.0) as usize;
                }
                Term::Ret(values) => {
                    return Ok(values.iter().map(|v| vals[v.0 as usize]).collect());
                }
                Term::Unreachable => {
                    return self.trap(format!("reached unreachable code in '{}'", func.name));
                }
            }
        }
    }

    fn step(
        &mut self,
        program: &Program,
        inst: &Inst,
        vals: &mut [u64],
        frame: &Frame,
        stack_base: u64,
    ) -> Res<()> {
        match inst {
            Inst::IConst {
                dst,
                ty,
                value,
            } => vals[dst.0 as usize] = mask(*ty, *value),
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                vals[dst.0 as usize] = if *ty == Ty::F32 {
                    (*value as f32).to_bits() as u64
                } else {
                    value.to_bits()
                };
            }
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (vals[a.0 as usize], vals[b.0 as usize]);
                vals[dst.0 as usize] = self.bin(*op, *ty, x, y)?;
            }
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let x = vals[a.0 as usize];
                vals[dst.0 as usize] = match op {
                    UnOp::Neg => mask(*ty, x.wrapping_neg()),
                    UnOp::Not => mask(*ty, !x),
                    UnOp::FNeg => {
                        if *ty == Ty::F32 {
                            (-f32::from_bits(x as u32)).to_bits() as u64
                        } else {
                            (-f64::from_bits(x)).to_bits()
                        }
                    }
                };
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (vals[a.0 as usize], vals[b.0 as usize]);
                vals[dst.0 as usize] = cmp(*op, *ty, x, y) as u64;
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => vals[dst.0 as usize] = conv(*op, *from, *to, vals[src.0 as usize]),
            Inst::SlotAddr {
                dst,
                slot,
            } => vals[dst.0 as usize] = stack_base + frame.offsets[slot.0 as usize],
            Inst::GlobalAddr {
                dst,
                global,
            } => vals[dst.0 as usize] = self.global_addr(program, *global)?,
            Inst::FuncAddr {
                dst,
                func,
            } => vals[dst.0 as usize] = FUNC_TAG | func.0 as u64,
            Inst::ForeignAddr {
                dst,
                foreign,
            } => vals[dst.0 as usize] = self.foreign_addr(program, *foreign)?,
            Inst::Load {
                dst,
                ty,
                addr,
            } => vals[dst.0 as usize] = self.load(*ty, vals[addr.0 as usize])?,
            Inst::Store {
                ty,
                addr,
                value,
            } => self.store(*ty, vals[addr.0 as usize], vals[value.0 as usize])?,
            Inst::PtrAdd {
                dst,
                base,
                offset,
            } => vals[dst.0 as usize] = vals[base.0 as usize].wrapping_add(vals[offset.0 as usize]),
            Inst::Copy {
                dst,
                src,
                size,
            } => {
                let (d, s) = (vals[dst.0 as usize], vals[src.0 as usize]);
                if d < 4096 || s < 4096 {
                    return self.trap("copy through a null pointer");
                }
                unsafe { std::ptr::copy(s as *const u8, d as *mut u8, *size as usize) };
            }
            Inst::Zero {
                dst,
                size,
            } => {
                let d = vals[dst.0 as usize];
                if d < 4096 {
                    return self.trap("write through a null pointer");
                }
                unsafe { std::ptr::write_bytes(d as *mut u8, 0, *size as usize) };
            }
            Inst::Call {
                results,
                callee,
                args,
            } => {
                let argv: Vec<u64> = args.iter().map(|a| vals[a.0 as usize]).collect();
                let out = match callee {
                    Callee::Func(f) => self.exec(program, *f, &argv)?,
                    Callee::Foreign(f) => {
                        let sig = program.foreigns[f.0 as usize].sig.clone();
                        self.call_foreign(program, *f, &argv, &sig)?
                    }
                    Callee::Indirect(target, sig) => {
                        let addr = vals[target.0 as usize];
                        match addr & TAG_MASK {
                            FUNC_TAG => {
                                self.exec(program, FuncId((addr & !TAG_MASK) as u32), &argv)?
                            }
                            FOREIGN_TAG => self.call_foreign(
                                program,
                                ForeignId((addr & !TAG_MASK) as u32),
                                &argv,
                                sig,
                            )?,
                            _ if addr < 4096 => {
                                return self.trap("call through a null procedure pointer");
                            }
                            _ => self.call_native(addr, &argv, sig)?,
                        }
                    }
                };
                for (r, v) in results.iter().zip(out) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Intrinsic {
                results,
                op,
                args,
            } => {
                let argv: Vec<u64> = args.iter().map(|a| vals[a.0 as usize]).collect();
                let tys: Vec<Ty> = args.iter().map(|_| Ty::I64).collect();
                let _ = tys;
                let out = self.intrinsic(*op, &argv, results.first().map(|_| ()).is_some())?;
                for (r, v) in results.iter().zip(out) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Loc {
                line,
                col,
                file,
            } => self.loc = Some((*file, *line, *col)),
        }
        Ok(())
    }

    fn bin(&self, op: BinOp, ty: Ty, x: u64, y: u64) -> Res<u64> {
        let bits = ty.size() as u32 * 8;
        Ok(match op {
            BinOp::Add => mask(ty, x.wrapping_add(y)),
            BinOp::Sub => mask(ty, x.wrapping_sub(y)),
            BinOp::Mul => mask(ty, x.wrapping_mul(y)),
            BinOp::SDiv | BinOp::SRem => {
                let (a, b) = (sext(ty, x), sext(ty, y));
                if b == 0 {
                    return self.trap("integer division by zero");
                }
                let r = if op == BinOp::SDiv {
                    a.wrapping_div(b)
                } else {
                    a.wrapping_rem(b)
                };
                mask(ty, r as u64)
            }
            BinOp::UDiv | BinOp::URem => {
                if y == 0 {
                    return self.trap("integer division by zero");
                }
                if op == BinOp::UDiv {
                    x / y
                } else {
                    x % y
                }
            }
            BinOp::And => x & y,
            BinOp::Or => x | y,
            BinOp::Xor => x ^ y,
            BinOp::Shl => {
                if y >= bits as u64 {
                    0
                } else {
                    mask(ty, x << y)
                }
            }
            BinOp::LShr => {
                if y >= bits as u64 {
                    0
                } else {
                    x >> y
                }
            }
            BinOp::AShr => mask(ty, (sext(ty, x) >> y.min(63)) as u64),
            BinOp::Rotl | BinOp::Rotr => {
                let s = (y % bits as u64) as u32;
                let s = if op == BinOp::Rotr {
                    (bits - s) % bits
                } else {
                    s
                };
                if s == 0 {
                    x
                } else {
                    mask(ty, (x << s) | (x >> (bits - s)))
                }
            }
            BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv => {
                if ty == Ty::F32 {
                    let (a, b) = (f32::from_bits(x as u32), f32::from_bits(y as u32));
                    (match op {
                        BinOp::FAdd => a + b,
                        BinOp::FSub => a - b,
                        BinOp::FMul => a * b,
                        _ => a / b,
                    })
                    .to_bits() as u64
                } else {
                    let (a, b) = (f64::from_bits(x), f64::from_bits(y));
                    (match op {
                        BinOp::FAdd => a + b,
                        BinOp::FSub => a - b,
                        BinOp::FMul => a * b,
                        _ => a / b,
                    })
                    .to_bits()
                }
            }
        })
    }

    fn intrinsic(&mut self, op: ir::Intrinsic, a: &[u64], _has_result: bool) -> Res<Vec<u64>> {
        use ir::Intrinsic as I;
        let f64_of = |x: u64| f64::from_bits(x);
        Ok(match op {
            I::Memcpy => {
                if a[2] > 0 {
                    unsafe { std::ptr::copy(a[1] as *const u8, a[0] as *mut u8, a[2] as usize) };
                }
                vec![]
            }
            I::Memset => {
                if a[2] > 0 {
                    unsafe { std::ptr::write_bytes(a[0] as *mut u8, a[1] as u8, a[2] as usize) };
                }
                vec![]
            }
            I::Memcmp => {
                let n = a[2] as usize;
                let (x, y) = (self.read(a[0], n), self.read(a[1], n));
                let r: i16 = match x.cmp(&y) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                vec![r as u16 as u64]
            }
            I::CompareAndSwap => {
                // (ptr, old, new, width) -> (success, previous)
                let width = a.get(3).copied().unwrap_or(8);
                let ty = Ty::int(width);
                let current = self.load(ty, a[0])?;
                let success = current == mask(ty, a[1]);
                if success {
                    self.store(ty, a[0], a[2])?;
                }
                vec![success as u64, current]
            }
            I::DebugBreak => return self.trap("debug_break() was called"),
            I::Trap => return self.trap("runtime check failed"),
            I::CompilerWrite => {
                let bytes = self.read(a[0], a[1] as usize);
                self.host.write(&bytes, a.get(2).is_some_and(|&v| v != 0));
                vec![]
            }
            I::Sqrt => vec![f64_of(a[0]).sqrt().to_bits()],
            I::Sin => vec![f64_of(a[0]).sin().to_bits()],
            I::Cos => vec![f64_of(a[0]).cos().to_bits()],
            I::Floor => vec![f64_of(a[0]).floor().to_bits()],
            I::Ceil => vec![f64_of(a[0]).ceil().to_bits()],
            I::Round => vec![f64_of(a[0]).round().to_bits()],
            I::Trunc => vec![f64_of(a[0]).trunc().to_bits()],
            I::Fabs => vec![f64_of(a[0]).abs().to_bits()],
            I::ReturnAddress => vec![0],
            I::CycleCounter => vec![
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos() as u64),
            ],
            I::Pause => vec![],
            I::Popcount => vec![a[0].count_ones() as u64],
            I::Ctlz => {
                let bits = a[1] as u32;
                let lead = if a[0] == 0 {
                    64
                } else {
                    a[0].leading_zeros()
                };
                vec![(lead - (64 - bits)) as u64]
            }
            I::Cttz => vec![(a[0].trailing_zeros()).min(a[1] as u32) as u64],
            I::Bswap => vec![a[0].swap_bytes() >> (64 - a[1] as u32)],
            I::IsCompileTime => vec![self.compile_time as u64],
        })
    }
}

fn mask(ty: Ty, v: u64) -> u64 {
    match ty {
        Ty::I8 => v & 0xff,
        Ty::I16 => v & 0xffff,
        Ty::I32 | Ty::F32 => v & 0xffff_ffff,
        _ => v,
    }
}

fn sext(ty: Ty, v: u64) -> i64 {
    match ty {
        Ty::I8 => v as u8 as i8 as i64,
        Ty::I16 => v as u16 as i16 as i64,
        Ty::I32 => v as u32 as i32 as i64,
        _ => v as i64,
    }
}

fn cmp(op: CmpOp, ty: Ty, x: u64, y: u64) -> bool {
    let (ux, uy) = (mask(ty, x), mask(ty, y));
    let (sx, sy) = (sext(ty, x), sext(ty, y));
    let float = |x: u64| {
        if ty == Ty::F32 {
            f32::from_bits(x as u32) as f64
        } else {
            f64::from_bits(x)
        }
    };
    match op {
        CmpOp::Eq => ux == uy,
        CmpOp::Ne => ux != uy,
        CmpOp::SLt => sx < sy,
        CmpOp::SLe => sx <= sy,
        CmpOp::SGt => sx > sy,
        CmpOp::SGe => sx >= sy,
        CmpOp::ULt => ux < uy,
        CmpOp::ULe => ux <= uy,
        CmpOp::UGt => ux > uy,
        CmpOp::UGe => ux >= uy,
        CmpOp::FEq => float(x) == float(y),
        CmpOp::FNe => float(x) != float(y),
        CmpOp::FLt => float(x) < float(y),
        CmpOp::FLe => float(x) <= float(y),
        CmpOp::FGt => float(x) > float(y),
        CmpOp::FGe => float(x) >= float(y),
    }
}

fn conv(op: ConvOp, from: Ty, to: Ty, v: u64) -> u64 {
    let f = |v: u64| {
        if from == Ty::F32 {
            f32::from_bits(v as u32) as f64
        } else {
            f64::from_bits(v)
        }
    };
    let out_f = |x: f64| {
        if to == Ty::F32 {
            (x as f32).to_bits() as u64
        } else {
            x.to_bits()
        }
    };
    match op {
        ConvOp::Trunc => mask(to, v),
        ConvOp::ZExt => mask(from, v),
        ConvOp::SExt => mask(to, sext(from, v) as u64),
        ConvOp::FToS => mask(
            to,
            match to {
                Ty::I8 => f(v) as i8 as u64,
                Ty::I16 => f(v) as i16 as u64,
                Ty::I32 => f(v) as i32 as u64,
                _ => f(v) as i64 as u64,
            },
        ),
        ConvOp::FToU => mask(
            to,
            match to {
                Ty::I8 => f(v) as u8 as u64,
                Ty::I16 => f(v) as u16 as u64,
                Ty::I32 => f(v) as u32 as u64,
                _ => f(v) as u64,
            },
        ),
        ConvOp::SToF => out_f(sext(from, v) as f64),
        ConvOp::UToF => out_f(mask(from, v) as f64),
        ConvOp::FExt | ConvOp::FTrunc => out_f(f(v)),
        ConvOp::Bitcast => mask(to, v),
    }
}
