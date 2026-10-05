//! Native entry points for interpreted `#c_call` procedures handed to C code (`qsort`
//! comparators, GLFW and SDL callbacks...).
//!
//! A procedure value is a tagged address only the interpreter can call, so when one is passed
//! to a foreign procedure it is swapped for a thunk: a real C function with the same fixed
//! prototype `call_as` uses (8 integer and 8 float registers, then stack slots) that unpacks
//! its arguments the way `call` packs them and re-enters the interpreter. Thunks come in one
//! family per return shape, each with `SLOTS` entries assigned to procedures on first use.
use super::{FF, FFF, FFFF, FI, IF, II, STACK_SLOTS, X86_64, read_bytes, write_bytes};
use crate::abi::{self, Arch, Passing, Piece, PieceTy};
use crate::ir::{FuncId, Sig};
use std::cell::Cell;
use std::sync::Mutex;

/// Runs an interpreted procedure on behalf of a thunk.
pub type Reenter<'a> = dyn FnMut(FuncId, &[u64]) -> Result<Vec<u64>, String> + 'a;

thread_local! {
    /// The interpreter suspended in a native call on this thread, which thunks call back into.
    static REENTER: Cell<Option<*mut Reenter<'static>>> = const { Cell::new(None) };
}

/// Run `f` (a native call) with `reenter` available to the thunks it may call.
pub(super) fn with_reenter<T>(reenter: &mut Reenter<'_>, f: impl FnOnce() -> T) -> T {
    let ptr: *mut Reenter<'_> = reenter;
    // SAFETY: the pointer is only dereferenced while `f` runs, during which `reenter` is
    // exclusively borrowed by this frame.
    let ptr: *mut Reenter<'static> = unsafe { std::mem::transmute(ptr) };
    let previous = REENTER.with(|r| r.replace(Some(ptr)));
    let out = f();
    REENTER.with(|r| r.set(previous));
    out
}

/// Thunks per return shape.
const SLOTS: usize = 64;

/// Return shapes, in the order of `thunk_addr`'s families.
const INT: usize = 0;
const FLOAT: usize = 1;
const SHAPES: usize = 8;

struct Slot {
    program: u64,
    func: FuncId,
    sig: Sig,
    /// x86-64 hidden result pointer: the first integer argument, returned in `rax`.
    sret: bool,
}

static TABLE: Mutex<[Vec<Slot>; SHAPES]> = Mutex::new([const { Vec::new() }; SHAPES]);

/// The C-callable address for interpreted procedure `func` of `program` (an identity for the
/// program whose function ids these are).
pub fn callback_addr(program: u64, func: FuncId, sig: &Sig) -> Result<u64, String> {
    let arch = Arch::host().ok_or("native callbacks are not available on this CPU")?;
    // The register model below is System V / AAPCS64; Windows hosts do not load native
    // libraries in the interpreter yet (`Library::open`), so this is not reached there.
    if arch == Arch::Win64 {
        return Err(
            "the interpreter does not implement the Microsoft x64 calling convention".into(),
        );
    }
    if sig.c_varargs {
        return Err("a variadic procedure cannot be called from C in the interpreter".into());
    }
    let cabi = sig.c_abi.as_deref();
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
    let k = match slots
        .iter()
        .position(|s| s.program == program && s.func == func && s.sig == *sig)
    {
        Some(k) => k,
        None if slots.len() == SLOTS => {
            return Err(format!(
                "more than {SLOTS} interpreted procedures of one shape were passed to C"
            ));
        }
        None => {
            slots.push(Slot {
                program,
                func,
                sig: sig.clone(),
                sret,
            });
            slots.len() - 1
        }
    };
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
    let (func, sig, sret) = {
        let table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
        let slot = &table[shape][k];
        (slot.func, slot.sig.clone(), slot.sret)
    };
    let Some(reenter) = REENTER.with(Cell::get) else {
        fatal("C called an interpreted procedure outside a foreign call on its thread")
    };
    // SAFETY: set by `with_reenter` on this thread for the native call now in progress.
    let reenter = unsafe { &mut *reenter };
    invoke(reenter, func, &sig, sret, ints, floats, stack).unwrap_or_else(|m| fatal(&m))
}

/// Arguments in the order the C ABI assigns them, consumed like `Regs` fills them.
struct Incoming {
    ints: Vec<u64>,
    floats: [u64; 8],
    stack: Vec<u64>,
    ni: usize,
    nf: usize,
    ns: usize,
}

impl Incoming {
    fn stack(&mut self) -> u64 {
        let v = self.stack.get(self.ns).copied().unwrap_or(0);
        self.ns += 1;
        v
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
) -> Result<Vec<u64>, String> {
    let arch = Arch::host().ok_or("native callbacks are not available on this CPU")?;
    // The register model below is System V / AAPCS64; Windows hosts do not load native
    // libraries in the interpreter yet (`Library::open`), so this is not reached there.
    if arch == Arch::Win64 {
        return Err(
            "the interpreter does not implement the Microsoft x64 calling convention".into(),
        );
    }
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
    };
    // By-value aggregates are rebuilt in memory; the IR passes their addresses.
    let mut buffers: Vec<Vec<u64>> = Vec::new();
    let mut args = Vec::with_capacity(sig.params.len());
    let count = sig.params.len() - ret_layout.is_some() as usize;
    for (i, param) in sig.params[..count].iter().enumerate() {
        let Some(layout) = cabi.and_then(|c| c.params.get(i)).and_then(Option::as_ref) else {
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
