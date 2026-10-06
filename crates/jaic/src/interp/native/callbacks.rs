//! Native entry points for interpreted `#c_call` procedures handed to C code (`qsort`
//! comparators, GLFW and SDL callbacks...).
//!
//! The value of a `#c_call` procedure is a thunk (`Interp::proc_value`): a real C function with
//! the same fixed prototype `call_as` uses (8 integer and 8 float registers, then stack slots)
//! that unpacks its arguments the way `call` packs them and re-enters the interpreter through
//! its `Gate` (the thread scheduler), from whichever thread C calls it on. Thunks come in one family per return
//! shape, each with `SLOTS` entries assigned to procedures on first use.
//! Windows x64 passes arguments by position instead, which needs its own thunks (`win64.rs`).
use super::{FF, FFF, FFFF, FI, IF, II, STACK_SLOTS, X86_64, read_bytes, write_bytes};
use crate::abi::{self, Arch, Passing, Piece, PieceTy};
use crate::interp::Trap;
use crate::ir::{FuncId, Sig};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

#[cfg(all(windows, target_arch = "x86_64"))]
mod win64;

/// Runs an interpreted procedure on behalf of a thunk.
pub type Reenter<'a> = dyn FnMut(FuncId, &[u64]) -> Result<Vec<u64>, Trap> + 'a;

thread_local! {
    /// The interpreter (its scheduler's `key`) and scheduler thread whose native call is in
    /// progress on this OS thread; `(0, 0)` outside one. A thunk entered anywhere else was
    /// called on a thread C started (an audio thread).
    static CALLER: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

/// Run `f`, a native call that scheduler thread `caller.1` of interpreter `caller.0` makes, on
/// this OS thread.
pub fn calling_out<T>(caller: (usize, usize), f: impl FnOnce() -> T) -> T {
    let outer = CALLER.with(|c| c.replace(caller));
    let out = f();
    CALLER.with(|c| c.set(outer));
    out
}

/// The native call in progress on this OS thread (see `CALLER`).
pub fn caller() -> (usize, usize) {
    CALLER.with(Cell::get)
}

/// What a thunk enters: the interpreter's thread scheduler (`interp/threads.rs`). Only the
/// thread holding its baton runs interpreted code, whichever thread C calls the thunk on.
pub trait Gate: Send + Sync {
    /// Take the baton on this thread, call `f` with a way to run procedures of `program`, and
    /// give the baton back.
    fn run(&self, program: u64, f: &mut dyn FnMut(&mut Reenter<'_>)) -> Result<(), String>;

    /// `trap`, raised by a procedure of `program` that C called, as the text of a report.
    fn describe(&self, program: u64, trap: &Trap) -> String;
}

/// Run `f` through `gate`. A trap in the procedure ends the process: it cannot unwind through
/// the C code that called it.
fn enter<T>(
    gate: &dyn Gate,
    program: u64,
    f: impl FnOnce(&mut Reenter<'_>) -> Result<T, Trap>,
) -> T {
    let mut f = Some(f);
    let mut out = None;
    let entered = gate.run(program, &mut |reenter| {
        if let Some(f) = f.take() {
            out = Some(f(reenter));
        }
    });
    if let Err(m) = entered {
        fatal(&m)
    }
    match out {
        Some(Ok(value)) => value,
        Some(Err(trap)) => fatal(&gate.describe(program, &trap)),
        None => fatal("the interpreter did not run a procedure C called"),
    }
}

fn same_gate(a: &Arc<dyn Gate>, b: &Arc<dyn Gate>) -> bool {
    std::ptr::addr_eq(Arc::as_ptr(a), Arc::as_ptr(b))
}

/// Free every thunk made for `gate`'s interpreter.
pub fn release(gate: &Arc<dyn Gate>) {
    let mut table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    for family in table.iter_mut() {
        free_slots(family, |s| same_gate(&s.gate, gate));
    }
    drop(table);
    #[cfg(all(windows, target_arch = "x86_64"))]
    win64::release(gate);
}

/// The slot of a thunk family for a procedure: the one already serving it (`serves`), else a
/// free one, else a new one while there are fewer than `limit`.
fn slot_for<S>(
    slots: &mut Vec<Option<S>>,
    limit: usize,
    serves: impl Fn(&S) -> bool,
) -> Option<usize> {
    let reused = slots
        .iter()
        .position(|s| s.as_ref().is_some_and(&serves))
        .or_else(|| slots.iter().position(Option::is_none));
    if reused.is_some() {
        return reused;
    }
    if slots.len() == limit {
        return None;
    }
    slots.push(None);
    Some(slots.len() - 1)
}

/// Empty the slots `owned` picks.
fn free_slots<S>(slots: &mut [Option<S>], owned: impl Fn(&S) -> bool) {
    for slot in slots {
        if slot.as_ref().is_some_and(&owned) {
            *slot = None;
        }
    }
}

/// Thunks per return shape.
const SLOTS: usize = 64;

/// Return shapes, in the order of `thunk_addr`'s families.
const INT: usize = 0;

const FLOAT: usize = 1;
const SHAPES: usize = 8;

struct Slot {
    program: u64,
    gate: Arc<dyn Gate>,
    func: FuncId,
    sig: Sig,
    /// x86-64 hidden result pointer: the first integer argument, returned in `rax`.
    sret: bool,
}

static TABLE: Mutex<[Vec<Option<Slot>>; SHAPES]> = Mutex::new([const { Vec::new() }; SHAPES]);

/// The C-callable address for interpreted procedure `func` of `program` (an identity for the
/// program whose function ids these are), run by the interpreter behind `gate`. The same
/// procedure always gets the same address while that interpreter lives.
pub fn callback_addr(
    gate: &Arc<dyn Gate>,
    program: u64,
    func: FuncId,
    sig: &Sig,
) -> Result<u64, String> {
    let arch = Arch::host().ok_or("native callbacks are not available on this CPU")?;
    if sig.c_varargs {
        return Err("a variadic procedure cannot be called from C in the interpreter".into());
    }
    // The thunks below receive System V / AAPCS64 registers; Microsoft x64 has its own.
    if arch == Arch::Win64 {
        #[cfg(all(windows, target_arch = "x86_64"))]
        return win64::callback_addr(gate, program, func, sig);
        #[cfg(not(all(windows, target_arch = "x86_64")))]
        return Err("Microsoft x64 callbacks need a Windows x64 host".into());
    }
    let cabi = sig.c_abi.as_deref();
    // The thunks receive 64-bit float registers and cannot return in `st(0)`.
    let wide = |l: &crate::ir::AggLayout| {
        l.fields
            .iter()
            .any(|&(_, t)| matches!(t, crate::ir::Ty::F80 | crate::ir::Ty::F128))
    };
    if cabi.is_some_and(|c| c.ret.iter().chain(c.params.iter().flatten()).any(wide)) {
        return Err(
            "a procedure passing a long double wider than float64 cannot be called from C in the interpreter (it works in a native build)"
                .into(),
        );
    }
    let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
    let (shape, sret) = match cabi.and_then(|c| c.ret.as_ref()) {
        None if sig.returns.first().is_some_and(|t| t.is_float()) => (FLOAT, false),
        None => (INT, false),
        Some(layout) => match (!forced_sret)
            .then(|| abi::classify_ret(arch, layout))
            .flatten()
        {
            Some(pieces) => (shape_of(&pieces)?, false),
            None if X86_64 => (INT, true),
            None => {
                return Err(
                    "a procedure returning a large struct cannot be called from C in the interpreter on this CPU"
                        .into(),
                );
            }
        },
    };
    let mut table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    let slots = &mut table[shape];
    let k = slot_for(slots, SLOTS, |s| {
        s.program == program && s.func == func && s.sig == *sig
    })
    .ok_or_else(|| {
        format!("more than {SLOTS} interpreted procedures of one shape were passed to C")
    })?;
    // A later interpreter running the same program takes the thunk over.
    slots[k] = Some(Slot {
        program,
        gate: gate.clone(),
        func,
        sig: sig.clone(),
        sret,
    });
    Ok(thunk_addr(shape, k) as u64)
}

fn shape_of(pieces: &[Piece]) -> Result<usize, String> {
    let int = |p: &Piece| p.ty == PieceTy::I64;
    Ok(match pieces {
        [] => INT,
        [a] if int(a) => INT,
        [_] => FLOAT,
        [a, b] => match (int(a), int(b)) {
            (true, true) => 2,
            (true, false) => 3,
            (false, true) => 4,
            (false, false) => 5,
        },
        [_, _, _] => 6,
        [_, _, _, _] => 7,
        _ => return Err("unsupported C aggregate return shape".into()),
    })
}

/// A thunk's result, built from the words `invoke` returns.
trait Ret {
    const SHAPE: usize;
    fn from_words(w: &[u64]) -> Self;
}

fn word(w: &[u64], i: usize) -> u64 {
    w.get(i).copied().unwrap_or(0)
}

fn float(w: &[u64], i: usize) -> f64 {
    f64::from_bits(word(w, i))
}

impl Ret for u64 {
    const SHAPE: usize = INT;

    fn from_words(w: &[u64]) -> Self {
        word(w, 0)
    }
}

impl Ret for f64 {
    const SHAPE: usize = FLOAT;

    fn from_words(w: &[u64]) -> Self {
        float(w, 0)
    }
}

impl Ret for II {
    const SHAPE: usize = 2;

    fn from_words(w: &[u64]) -> Self {
        II(word(w, 0), word(w, 1))
    }
}

impl Ret for IF {
    const SHAPE: usize = 3;

    fn from_words(w: &[u64]) -> Self {
        IF(word(w, 0), float(w, 1))
    }
}

impl Ret for FI {
    const SHAPE: usize = 4;

    fn from_words(w: &[u64]) -> Self {
        FI(float(w, 0), word(w, 1))
    }
}

impl Ret for FF {
    const SHAPE: usize = 5;

    fn from_words(w: &[u64]) -> Self {
        FF(float(w, 0), float(w, 1))
    }
}

impl Ret for FFF {
    const SHAPE: usize = 6;

    fn from_words(w: &[u64]) -> Self {
        FFF(float(w, 0), float(w, 1), float(w, 2))
    }
}

impl Ret for FFFF {
    const SHAPE: usize = 7;

    fn from_words(w: &[u64]) -> Self {
        FFFF(float(w, 0), float(w, 1), float(w, 2), float(w, 3))
    }
}

/// Thunk `K` of the family returning `R`. Stack parameters a caller did not pass read its
/// own frame and are ignored.
#[rustfmt::skip]
extern "C" fn thunk<R: Ret, const K: usize>(
    i0: u64, i1: u64, i2: u64, i3: u64, i4: u64, i5: u64, i6: u64, i7: u64,
    f0: f64, f1: f64, f2: f64, f3: f64, f4: f64, f5: f64, f6: f64, f7: f64,
    s0: u64, s1: u64, s2: u64, s3: u64, s4: u64, s5: u64, s6: u64, s7: u64,
    s8: u64, s9: u64, s10: u64, s11: u64, s12: u64, s13: u64, s14: u64, s15: u64,
) -> R {
    let ints = [i0, i1, i2, i3, i4, i5, i6, i7];
    let floats = [f0, f1, f2, f3, f4, f5, f6, f7].map(f64::to_bits);
    let stack = [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15];
    R::from_words(&dispatch(R::SHAPE, K, ints, floats, stack))
}

#[rustfmt::skip]
type Thunk<R> = extern "C" fn(
    u64, u64, u64, u64, u64, u64, u64, u64,
    f64, f64, f64, f64, f64, f64, f64, f64,
    u64, u64, u64, u64, u64, u64, u64, u64,
    u64, u64, u64, u64, u64, u64, u64, u64,
) -> R;

macro_rules! family {
    ($r:ty; $($k:literal)*) => {
        [$(thunk::<$r, $k> as Thunk<$r> as usize),*]
    };
}

fn thunk_addr(shape: usize, k: usize) -> usize {
    let family: [usize; SLOTS] = match shape {
        INT => {
            family!(u64; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        FLOAT => {
            family!(f64; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        2 => {
            family!(II; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        3 => {
            family!(IF; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        4 => {
            family!(FI; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        5 => {
            family!(FF; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        6 => {
            family!(FFF; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
        _ => {
            family!(FFFF; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63)
        }
    };
    family[k]
}

/// Abandon the process: an error inside a callback cannot unwind through C frames.
fn fatal(message: &str) -> ! {
    eprintln!("error: in a procedure called from C: {message}");
    std::process::exit(1)
}

fn dispatch(
    shape: usize,
    k: usize,
    ints: [u64; 8],
    floats: [u64; 8],
    stack: [u64; STACK_SLOTS],
) -> Vec<u64> {
    let (gate, program, func, sig, sret) = {
        let table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(slot) = &table[shape][k] else {
            drop(table);
            fatal("C called a procedure of an interpreter that has finished")
        };
        (
            slot.gate.clone(),
            slot.program,
            slot.func,
            slot.sig.clone(),
            slot.sret,
        )
    };
    enter(&*gate, program, |reenter| {
        invoke(reenter, func, &sig, sret, ints, floats, stack)
    })
}

/// Arguments in the order the C ABI assigns them, consumed like `Regs` fills them.
struct Incoming {
    ints: Vec<u64>,
    floats: [u64; 8],
    stack: Vec<u64>,
    ni: usize,
    nf: usize,
    ns: usize,
    /// As `Regs::tail`: bytes of the last stack slot packed arguments used.
    tail: usize,
}

impl Incoming {
    fn stack(&mut self) -> u64 {
        let v = self.stack.get(self.ns).copied().unwrap_or(0);
        self.ns += 1;
        self.tail = 0;
        v
    }

    /// A stack argument smaller than 8 bytes, where Apple's arm64 ABI packs it (`Regs::packed`).
    fn packed(&mut self, size: usize) -> u64 {
        let at = self.tail.next_multiple_of(size);
        let mask = (1u64 << (size * 8)) - 1;
        if self.tail == 0 || at + size > 8 {
            let v = self.stack();
            self.tail = size;
            return v & mask;
        }
        self.tail = at + size;
        let slot = self.stack.get(self.ns - 1).copied().unwrap_or(0);
        (slot >> (at * 8)) & mask
    }

    fn int(&mut self) -> u64 {
        if self.ni == self.ints.len() {
            return self.stack();
        }
        self.ni += 1;
        self.ints[self.ni - 1]
    }

    fn float(&mut self) -> u64 {
        if self.nf == 8 {
            return self.stack();
        }
        self.nf += 1;
        self.floats[self.nf - 1]
    }

    fn fits(&self, pieces: &[Piece]) -> bool {
        let ints = pieces.iter().filter(|p| p.ty == PieceTy::I64).count();
        self.ni + ints <= self.ints.len() && self.nf + pieces.len() - ints <= 8
    }

    /// As `Regs::exhaust`.
    fn exhaust(&mut self, pieces: &[Piece]) {
        if X86_64 {
            return;
        }
        if pieces.iter().any(|p| p.ty == PieceTy::I64) {
            self.ni = self.ints.len();
        } else {
            self.nf = 8;
        }
    }
}

fn piece_size(p: &Piece, size: u64) -> u64 {
    let width = if p.ty == PieceTy::F32 {
        4
    } else {
        8
    };
    width.min(size - p.offset)
}

fn invoke(
    reenter: &mut Reenter<'_>,
    func: FuncId,
    sig: &Sig,
    sret: bool,
    ints: [u64; 8],
    floats: [u64; 8],
    stack: [u64; STACK_SLOTS],
) -> Result<Vec<u64>, Trap> {
    // `callback_addr` makes thunks only on CPUs it knows.
    let arch = Arch::host().expect("a thunk on an unknown CPU");
    let cabi = sig.c_abi.as_deref();
    let ret_layout = cabi.and_then(|c| c.ret.as_ref());
    // x86-64 has six integer argument registers (the hidden result pointer takes the first);
    // the prototype's remaining integer parameters are the first stack slots.
    let int_regs = if X86_64 {
        6
    } else {
        8
    };
    let first = sret as usize;
    let mut incoming = Incoming {
        ints: ints[first..int_regs].to_vec(),
        floats,
        stack: ints[int_regs..].iter().chain(&stack).copied().collect(),
        ni: 0,
        nf: 0,
        ns: 0,
        tail: 0,
    };
    // By-value aggregates are rebuilt in memory; the IR passes their addresses.
    let mut buffers: Vec<Vec<u64>> = Vec::new();
    let mut args = Vec::with_capacity(sig.params.len());
    let count = sig.params.len() - ret_layout.is_some() as usize;
    for (i, param) in sig.params[..count].iter().enumerate() {
        let Some(layout) = cabi.and_then(|c| c.params.get(i)).and_then(Option::as_ref) else {
            let full = if param.is_float() {
                incoming.nf == 8
            } else {
                incoming.ni == incoming.ints.len()
            };
            if full && super::PACKED_STACK && param.size() < 8 {
                args.push(incoming.packed(param.size() as usize));
                continue;
            }
            args.push(if param.is_float() {
                incoming.float()
            } else {
                incoming.int()
            });
            continue;
        };
        let mut buffer = vec![0u64; layout.size.div_ceil(8).max(1) as usize];
        let base = buffer.as_mut_ptr() as u64;
        match abi::classify_arg(arch, layout) {
            Passing::Registers(pieces) if incoming.fits(&pieces) => {
                for p in &pieces {
                    let v = if p.ty == PieceTy::I64 {
                        incoming.int()
                    } else {
                        incoming.float()
                    };
                    // SAFETY: the buffer holds `layout.size` bytes.
                    unsafe { write_bytes(base + p.offset, v, piece_size(p, layout.size)) };
                }
            }
            // The caller's copy, passed by address.
            Passing::Indirect => {
                args.push(incoming.int());
                continue;
            }
            Passing::Registers(pieces) => {
                incoming.exhaust(&pieces);
                for w in buffer.iter_mut() {
                    *w = incoming.stack();
                }
            }
            Passing::ByVal => {
                for w in buffer.iter_mut() {
                    *w = incoming.stack();
                }
            }
        }
        args.push(base);
        buffers.push(buffer);
    }
    let Some(layout) = ret_layout else {
        return reenter(func, &args);
    };
    let mut result = vec![0u64; layout.size.div_ceil(8).max(1) as usize];
    let base = result.as_mut_ptr() as u64;
    args.push(base);
    reenter(func, &args)?;
    drop(buffers);
    if sret {
        let out = ints[0];
        // SAFETY: the caller's result buffer holds `layout.size` bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(base as *const u8, out as *mut u8, layout.size as usize)
        };
        return Ok(vec![out]);
    }
    let pieces = abi::classify_ret(arch, layout).unwrap_or_default();
    // SAFETY: `result` holds `layout.size` bytes.
    Ok(pieces
        .iter()
        .map(|p| unsafe { read_bytes(base + p.offset, piece_size(p, layout.size)) })
        .collect())
}
