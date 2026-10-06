//! Callback thunks for the Microsoft x64 convention (Windows on x64).
//!
//! The System V thunks in `callbacks.rs` rely on integer and float arguments using separate
//! register files, so one Rust prototype receives both. Microsoft x64 assigns positional slots
//! instead: argument `n < 4` is in RCX/RDX/R8/R9 or, if it is floating point, in XMM`n`, and
//! the rest are on the stack above a 32-byte home area. No Rust prototype can read "XMM1 or
//! RDX" depending on the callee, so the entry points are a little assembly:
//!
//! - `SLOTS` stubs, 16 bytes apart, each loading its index into EAX and jumping to `common`;
//! - `common` spills RCX..R9 to the caller's home area (which is contiguous with the stack
//!   arguments, giving one array of positional slots) and XMM0-XMM3 to its own frame, then
//!   calls `dispatch` with both arrays, the index and a 16-byte result area;
//! - `dispatch` decodes the arguments from the procedure's signature, runs it, and fills the
//!   result area, which `common` loads into RAX and XMM0 before returning.
//!
//! Unlike System V there is a single family: the return shape is decided in `dispatch` (an
//! integer or aggregate of 1, 2, 4 or 8 bytes in RAX, a float in XMM0, anything else through
//! the hidden pointer in the first slot, also returned in RAX).
use super::{Gate, Reenter, enter, fatal, same_gate};
use crate::abi::{self, Arch, Passing};
use crate::interp::native::{read_bytes, write_bytes};
use crate::ir::{FuncId, Sig, Ty};
use std::sync::{Arc, Mutex};

/// Interpreted procedures that can be handed to C at once.
const SLOTS: usize = 256;

/// Bytes between two stubs (`.p2align 4` pads each one).
const STUB_STRIDE: usize = 16;

struct Slot {
    program: u64,
    gate: Arc<dyn Gate>,
    func: FuncId,
    sig: Sig,
}

static TABLE: Mutex<Vec<Option<Slot>>> = Mutex::new(Vec::new());

macro_rules! stubs {
    ($($k:literal)*) => {
        std::arch::global_asm!(
            ".text",
            ".p2align 4",
            ".globl jaic_win64_callback_stubs",
            "jaic_win64_callback_stubs:",
            $(
                concat!("mov eax, ", stringify!($k)),
                "jmp {common}",
                ".p2align 4",
            )*
            common = sym common,
        );
    };
}

stubs!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95
    96 97 98 99 100 101 102 103 104 105 106 107 108 109 110 111 112 113 114 115 116 117 118 119
    120 121 122 123 124 125 126 127 128 129 130 131 132 133 134 135 136 137 138 139 140 141 142 143
    144 145 146 147 148 149 150 151 152 153 154 155 156 157 158 159 160 161 162 163 164 165 166 167
    168 169 170 171 172 173 174 175 176 177 178 179 180 181 182 183 184 185 186 187 188 189 190 191
    192 193 194 195 196 197 198 199 200 201 202 203 204 205 206 207 208 209 210 211 212 213 214 215
    216 217 218 219 220 221 222 223 224 225 226 227 228 229 230 231 232 233 234 235 236 237 238 239
    240 241 242 243 244 245 246 247 248 249 250 251 252 253 254 255
);

unsafe extern "C" {
    /// The first stub; stub `k` is `STUB_STRIDE * k` bytes further.
    static jaic_win64_callback_stubs: u8;
}

/// The shared body of the stubs, with EAX holding the stub's index. Its frame (88 bytes, which
/// re-aligns RSP to 16 after the return address) holds the callee's 32-byte home area for
/// `dispatch`, the four XMM arguments at +32 and the result at +64; the caller's home area,
/// and after it the caller's stack arguments, start at +96.
#[unsafe(naked)]
unsafe extern "C" fn common() {
    std::arch::naked_asm!(
        ".seh_proc {name}",
        "mov [rsp + 8], rcx",
        "mov [rsp + 16], rdx",
        "mov [rsp + 24], r8",
        "mov [rsp + 32], r9",
        "sub rsp, 88",
        ".seh_stackalloc 88",
        ".seh_endprologue",
        "movsd [rsp + 32], xmm0",
        "movsd [rsp + 40], xmm1",
        "movsd [rsp + 48], xmm2",
        "movsd [rsp + 56], xmm3",
        "lea rcx, [rsp + 96]",
        "lea rdx, [rsp + 32]",
        "mov r8d, eax",
        "lea r9, [rsp + 64]",
        "call {dispatch}",
        "mov rax, [rsp + 64]",
        "movsd xmm0, [rsp + 72]",
        "add rsp, 88",
        "ret",
        ".seh_endproc",
        name = sym common,
        dispatch = sym dispatch,
    )
}

/// The C-callable address for interpreted procedure `func` of `program`, run by the
/// interpreter behind `gate`.
pub(super) fn callback_addr(
    gate: &Arc<dyn Gate>,
    program: u64,
    func: FuncId,
    sig: &Sig,
) -> Result<u64, String> {
    let mut table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    let existing = table.iter().position(|s| {
        s.as_ref()
            .is_some_and(|s| s.program == program && s.func == func && s.sig == *sig)
    });
    let k = match existing.or_else(|| table.iter().position(Option::is_none)) {
        Some(k) => k,
        None if table.len() == SLOTS => {
            return Err(format!(
                "more than {SLOTS} interpreted procedures were passed to C"
            ));
        }
        None => {
            table.push(None);
            table.len() - 1
        }
    };
    table[k] = Some(Slot {
        program,
        gate: gate.clone(),
        func,
        sig: sig.clone(),
    });
    let first = &raw const jaic_win64_callback_stubs as usize;
    Ok((first + k * STUB_STRIDE) as u64)
}

/// Called by `common`: `slots` are the positional argument slots (the caller's home area, then
/// its stack arguments), `xmm` the first four XMM registers, `out` RAX and XMM0 to return.
unsafe extern "C" fn dispatch(slots: *const u64, xmm: *const u64, k: u32, out: *mut [u64; 2]) {
    let (gate, program, func, sig) = {
        let table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(slot) = &table[k as usize] else {
            drop(table);
            fatal("C called a procedure of an interpreter that has finished")
        };
        (slot.gate.clone(), slot.program, slot.func, slot.sig.clone())
    };
    // SAFETY: `common` passes its caller's argument area and its own spill area; `invoke`
    // reads only the positions the procedure's signature declares.
    let incoming = Incoming {
        slot: |i| unsafe { slots.add(i).read() },
        xmm: |i| unsafe { xmm.add(i).read() },
    };
    let result = enter(&*gate, program, |reenter| {
        invoke(reenter, func, &sig, &incoming)
    })
    .unwrap_or_else(|m| fatal(&m));
    // SAFETY: `out` is the 16-byte result area in `common`'s frame.
    unsafe { out.write(result) };
}

/// Free every stub assigned to `gate`'s interpreter.
pub(super) fn release(gate: &Arc<dyn Gate>) {
    let mut table = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    for slot in table.iter_mut() {
        if slot.as_ref().is_some_and(|s| same_gate(&s.gate, gate)) {
            *slot = None;
        }
    }
}

struct Incoming<S, X> {
    slot: S,
    xmm: X,
}

impl<S: Fn(usize) -> u64, X: Fn(usize) -> u64> Incoming<S, X> {
    /// The scalar in position `i`: an XMM register for the first four floats.
    fn scalar(&self, i: usize, ty: Ty) -> u64 {
        if !ty.is_float() {
            return (self.slot)(i);
        }
        let bits = if i < 4 {
            (self.xmm)(i)
        } else {
            (self.slot)(i)
        };
        if ty == Ty::F32 {
            bits & 0xffff_ffff
        } else {
            bits
        }
    }
}

/// Run `func` with the arguments the C caller passed; returns the words for RAX and XMM0.
fn invoke<S: Fn(usize) -> u64, X: Fn(usize) -> u64>(
    reenter: &mut Reenter<'_>,
    func: FuncId,
    sig: &Sig,
    incoming: &Incoming<S, X>,
) -> Result<[u64; 2], String> {
    let cabi = sig.c_abi.as_deref();
    let ret_layout = cabi.and_then(|c| c.ret.as_ref());
    let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
    let in_rax = ret_layout
        .filter(|_| !forced_sret)
        .is_some_and(|l| abi::classify_ret(Arch::Win64, l).is_some());
    let sret = ret_layout.is_some() && !in_rax;
    // The hidden result pointer takes the first slot.
    let mut position = sret as usize;
    // By-value aggregates in a slot are rebuilt in memory; the IR passes their addresses.
    let mut buffers: Vec<Vec<u64>> = Vec::new();
    let mut args = Vec::with_capacity(sig.params.len());
    let count = sig.params.len() - ret_layout.is_some() as usize;
    for (i, &param) in sig.params[..count].iter().enumerate() {
        let Some(layout) = cabi.and_then(|c| c.params.get(i)).and_then(Option::as_ref) else {
            args.push(incoming.scalar(position, param));
            position += 1;
            continue;
        };
        let mut buffer = vec![0u64; layout.size.div_ceil(8).max(1) as usize];
        match abi::classify_arg(Arch::Win64, layout) {
            // An empty struct takes no slot.
            Passing::Registers(pieces) if pieces.is_empty() => {}
            Passing::Registers(_) => {
                let value = (incoming.slot)(position);
                position += 1;
                // SAFETY: the buffer holds `layout.size` (at most 8) bytes.
                unsafe { write_bytes(buffer.as_mut_ptr() as u64, value, layout.size) };
            }
            // The caller's copy, passed by address.
            Passing::Indirect | Passing::ByVal => {
                args.push((incoming.slot)(position));
                position += 1;
                continue;
            }
        }
        args.push(buffer.as_ptr() as u64);
        buffers.push(buffer);
    }
    let Some(layout) = ret_layout else {
        let value = reenter(func, &args)?.first().copied().unwrap_or(0);
        // The caller reads RAX or XMM0 according to its own declaration; fill both.
        return Ok([value, value]);
    };
    let mut result = vec![0u64; layout.size.div_ceil(8).max(1) as usize];
    let base = result.as_mut_ptr() as u64;
    args.push(base);
    reenter(func, &args)?;
    drop(buffers);
    if sret {
        let out = (incoming.slot)(0);
        // SAFETY: the caller's result buffer holds `layout.size` bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(base as *const u8, out as *mut u8, layout.size as usize)
        };
        return Ok([out, 0]);
    }
    // SAFETY: `result` holds `layout.size` (at most 8) bytes.
    Ok([unsafe { read_bytes(base, layout.size) }, 0])
}
