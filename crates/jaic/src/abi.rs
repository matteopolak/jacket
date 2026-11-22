//! C ABI classification for aggregates passed or returned by value.
//!
//! Covers the 64-bit ABIs the compiler targets: `AArch64` AAPCS64 (Apple and
//! Linux), its Windows variant, x86-64 System V, the Microsoft x64 convention
//! (Windows) and the WebAssembly C ABI (wasm64). The IR
//! describes an aggregate only by its flattened scalar fields (`AggLayout`),
//! which is all these ABIs look at.
use crate::ir::{AggLayout, Ty};
use crate::layout::DataLayout;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arch {
    Aarch64,
    /// 32-bit x86 System V (Linux): every argument on the stack, aggregates by value in the
    /// argument area, small aggregates returned in memory.
    X86,
    /// x86-64 System V (macOS, Linux).
    X86_64,
    /// IBM z/Architecture ELF (s390x Linux, big-endian): an aggregate of 1, 2, 4 or 8 bytes
    /// travels as an integer of that size (a lone float as that float), a larger one by
    /// reference to a caller copy, and every aggregate result comes back through a hidden
    /// pointer in `r2`. Integer arguments take `r2`-`r6`, floating-point ones `f0`, `f2`, `f4`, `f6`.
    S390x,
    /// 64-bit big-endian PowerPC, ELFv1 (powerpc64 Linux): every aggregate argument is passed
    /// by value (the backend assigns its doublewords to registers and the stack) and every
    /// aggregate result comes back through a hidden pointer in `r3`.
    PowerPc64,
    /// 32-bit ARM, AAPCS with the VFP (hard-float) variant: an aggregate of at most 4 bytes
    /// travels as one integer, a homogeneous float aggregate in VFP registers, any other as
    /// 4- or 8-byte integers in core registers and then on the stack; a larger-than-4-byte
    /// result comes back through a hidden pointer in `r0`.
    Arm32,
    /// 8-bit AVR, avr-gcc's convention as Clang lowers it: arguments take the 18 registers
    /// `r8`-`r25` (each rounded up to an even number of bytes, so an 8-bit one takes two), a
    /// whole aggregate in registers or on the stack by value, never split; a result of at most
    /// 8 bytes comes back in registers, a larger one through a hidden pointer.
    Avr,
    /// x86-64 with the Microsoft x64 calling convention (Windows, MSVC and MinGW alike).
    Win64,
    /// `AArch64` Windows (MSVC and MinGW alike): AAPCS64, except that a variadic procedure takes
    /// every argument, fixed ones included, in x0-x7 and then on the stack, with no
    /// floating-point aggregate treatment (`classify_vararg`).
    Win64Arm,
    /// WebAssembly (32-bit memory or Memory64), Clang's basic C ABI: an aggregate holding
    /// a single scalar travels as that scalar, any other goes by `byval` pointer and comes
    /// back through a hidden result pointer.
    Wasm,
}

impl Arch {
    /// The architecture this compiler runs on (for the interpreter's native calls).
    pub fn host() -> Option<Arch> {
        if cfg!(all(target_arch = "aarch64", windows)) {
            Some(Arch::Win64Arm)
        } else if cfg!(target_arch = "aarch64") {
            Some(Arch::Aarch64)
        } else if cfg!(all(target_arch = "x86_64", windows)) {
            Some(Arch::Win64)
        } else if cfg!(target_arch = "x86_64") {
            Some(Arch::X86_64)
        } else {
            None
        }
    }

    /// Whether the CPU is x86-64 (either calling convention).
    pub fn is_x86_64(self) -> bool {
        matches!(self, Arch::X86_64 | Arch::Win64)
    }

    /// Whether the CPU is `AArch64` (either flavour).
    pub fn is_aarch64(self) -> bool {
        matches!(self, Arch::Aarch64 | Arch::Win64Arm)
    }

    /// Whether the target is Windows (either CPU).
    pub fn is_windows(self) -> bool {
        matches!(self, Arch::Win64 | Arch::Win64Arm)
    }

    /// Whether the target is WebAssembly.
    pub fn is_wasm(self) -> bool {
        self == Arch::Wasm
    }
}

/// One register-sized chunk of an aggregate passed in registers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceTy {
    I64,
    F32,
    F64,
    /// Two packed `f32`s sharing one SSE eightbyte (x86-64 only).
    V2F32,
    /// An x87 `long double` (x86-64 System V): arguments go to the stack, 16-byte aligned,
    /// and results come back in `st(0)`.
    X87,
    /// A binary128 `long double` in a 128-bit vector register (`AArch64` outside Apple).
    F128,
    /// An integer of this many bytes (1, 2, 4 or 8): a small aggregate on s390x, which keeps
    /// its exact size so a big-endian register holds the right bytes.
    Int(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    /// Byte offset of the chunk inside the aggregate.
    pub offset: u64,
    pub ty: PieceTy,
}

/// How a by-value aggregate crosses the call boundary.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Passing {
    /// Split into register-class chunks, each its own LLVM argument/return element.
    Registers(Vec<Piece>),
    /// Copied to the stack by the callee prologue/caller (`byval`, x86-64).
    ByVal,
    /// Caller makes a copy and passes its address (`AArch64` large aggregates, and every
    /// Win64 aggregate that is not 1, 2, 4 or 8 bytes).
    Indirect,
}

impl Passing {
    /// Whether this argument is written as one LLVM array of its pieces (`AArch64` coerces a
    /// multi-register aggregate to `[N x iM]`) rather than as one element per piece. Windows on
    /// `AArch64` does not for a variadic procedure, whose arguments straddle registers and the
    /// stack.
    pub fn as_array(&self, arch: Arch, variadic: bool) -> bool {
        (arch.is_aarch64() || arch == Arch::Arm32)
            && !(variadic && arch == Arch::Win64Arm)
            && matches!(self, Passing::Registers(p) if p.len() > 1)
    }
}

/// Whether the hidden result pointer carries `inreg`: MSVC on arm64 passes a forced (non-POD C++)
/// result pointer in `x0`, not `x8`, and `inreg` selects that.
pub fn sret_is_inreg(arch: Arch, forced: bool) -> bool {
    forced && arch == Arch::Win64Arm
}

/// Whether the hidden result pointer takes the first integer argument register, leaving one
/// fewer for the parameters (x86-64 always; arm64 only where it is `inreg`).
pub fn sret_takes_int_register(arch: Arch, forced: bool) -> bool {
    arch.is_x86_64()
        || matches!(arch, Arch::S390x | Arch::PowerPc64 | Arch::Avr)
        || sret_is_inreg(arch, forced)
}

/// Classify a by-value aggregate argument.
pub fn classify_arg(arch: Arch, layout: &AggLayout, data: &DataLayout) -> Passing {
    match classify_registers(arch, layout, data) {
        Some(pieces) => Passing::Registers(pieces),
        None => match arch {
            Arch::X86_64 | Arch::X86 | Arch::Wasm => Passing::ByVal,
            Arch::Aarch64 | Arch::Win64 | Arch::Win64Arm | Arch::S390x => Passing::Indirect,
            Arch::PowerPc64 => Passing::ByVal,
            // AAPCS copies every aggregate into core registers and then the stack.
            Arch::Arm32 | Arch::Avr => {
                unreachable!("every aggregate on this target classifies as registers")
            }
        },
    }
}

/// Classify a by-value aggregate argument of a C-variadic procedure (`varargs`), whether
/// it is one of the fixed parameters or a variadic one.
///
/// Only Windows on `AArch64` differs from [`classify_arg`]: there a variadic procedure's
/// arguments all travel in general registers and on the stack, so a floating-point
/// aggregate is no different from any other: up to 16 bytes as one or two integer pieces,
/// larger ones by reference. (The backend itself moves scalar `float`/`double` arguments
/// of a variadic call into the general registers.)
pub fn classify_vararg(arch: Arch, layout: &AggLayout, data: &DataLayout) -> Passing {
    // AAPCS: a variadic call uses the base standard, so a float aggregate goes in core registers.
    if arch == Arch::Arm32 {
        return Passing::Registers(arm32_int_pieces(layout));
    }
    if arch != Arch::Win64Arm {
        return classify_arg(arch, layout, data);
    }
    if layout.size <= 16 {
        Passing::Registers(int_pieces(layout.size))
    } else {
        Passing::Indirect
    }
}

/// The argument registers still free while a parameter list is laid out, which decides whether
/// an aggregate classified as `Registers` really travels in them. System V passes an aggregate
/// that does not fit in the registers left entirely in memory (never half and half), which
/// separate LLVM parameters would not do. (AAPCS64 does the same, but the backend does it itself
/// for the array parameter the LLVM lowering uses there.)
pub struct RegBudget {
    arch: Arch,
    ints: u32,
    vectors: u32,
}

impl RegBudget {
    /// A fresh budget; `hidden_in_int` is whether the hidden result pointer takes an integer
    /// register (x86-64, and Windows on arm64 for a non-POD C++ result).
    pub fn new(arch: Arch, hidden_in_int: bool) -> RegBudget {
        let ints = match arch {
            Arch::X86_64 => 6,
            Arch::X86 => 0,
            Arch::S390x => 5,
            // Register bytes, not registers; the hidden result pointer takes two.
            Arch::Avr => 18 - 2 * hidden_in_int as u32,
            _ => 8,
        };
        RegBudget {
            arch,
            ints: if arch == Arch::Avr {
                ints
            } else {
                ints - hidden_in_int as u32
            },
            vectors: if arch == Arch::S390x {
                4
            } else {
                8
            },
        }
    }

    /// A scalar parameter of type `ty`.
    pub fn scalar(&mut self, ty: Ty) {
        if self.arch == Arch::Avr {
            let bytes: u32 = match ty {
                Ty::I8 => 1,
                Ty::I16 | Ty::Ptr => 2,
                Ty::I32 | Ty::F32 => 4,
                Ty::I64 | Ty::F64 => 8,
                Ty::F80 | Ty::F128 => 16,
            };
            self.ints = self.ints.saturating_sub(bytes.next_multiple_of(2));
            return;
        }
        match ty {
            Ty::F80 => {}
            Ty::F128 => self.vectors = self.vectors.saturating_sub(1),
            t if t.is_float() => self.vectors = self.vectors.saturating_sub(1),
            _ => self.ints = self.ints.saturating_sub(1),
        }
    }

    /// AVR: a whole aggregate goes in the registers still free, each member rounded up to an
    /// even number of bytes together, or on the stack by value; it is never split.
    fn avr_aggregate(&mut self, passing: Passing) -> Passing {
        let Passing::Registers(pieces) = &passing else {
            return passing;
        };
        let bytes: u32 = pieces
            .iter()
            .map(|p| match p.ty {
                PieceTy::Int(n) => n as u32,
                PieceTy::F32 => 4,
                _ => 8,
            })
            .sum();
        let bytes = bytes.next_multiple_of(2);
        if bytes <= self.ints {
            self.ints -= bytes;
            return passing;
        }
        self.ints = 0;
        Passing::ByVal
    }

    /// An aggregate parameter classified as `passing`; returns how it really travels.
    pub fn aggregate(&mut self, passing: Passing) -> Passing {
        if self.arch == Arch::Avr {
            return self.avr_aggregate(passing);
        }
        // Only System V needs the count: Windows passes by position and has no multi-register
        // aggregates, WebAssembly has no registers, and AArch64's arrays are the backend's job.
        if self.arch != Arch::X86_64 {
            return passing;
        }
        match passing {
            Passing::Indirect => {
                self.ints = self.ints.saturating_sub(1);
                passing
            }
            Passing::Registers(pieces) if pieces.iter().any(|p| p.ty == PieceTy::X87) => {
                Passing::Registers(pieces)
            }
            Passing::Registers(pieces) => {
                let ints = pieces.iter().filter(|p| p.ty == PieceTy::I64).count() as u32;
                let vectors = pieces.len() as u32 - ints;
                if ints <= self.ints && vectors <= self.vectors {
                    self.ints -= ints;
                    self.vectors -= vectors;
                    return Passing::Registers(pieces);
                }
                Passing::ByVal
            }
            Passing::ByVal => passing,
        }
    }
}

/// Classify an aggregate return value; `None` means returned through a hidden
/// `sret` pointer.
pub fn classify_ret(arch: Arch, layout: &AggLayout, data: &DataLayout) -> Option<Vec<Piece>> {
    // s390x returns every aggregate in memory, whatever its size.
    if arch == Arch::S390x && layout.size > 0 {
        return None;
    }
    // AAPCS: only a float aggregate or one of at most 4 bytes comes back in registers.
    if arch == Arch::Arm32 && layout.size > 4 && hfa(layout).is_none() {
        return None;
    }
    if arch == Arch::Avr && layout.size > 8 {
        return None;
    }
    classify_registers(arch, layout, data)
}

fn classify_registers(arch: Arch, layout: &AggLayout, data: &DataLayout) -> Option<Vec<Piece>> {
    if layout.size == 0 {
        return Some(Vec::new());
    }
    // Microsoft x64: an aggregate of exactly 1, 2, 4 or 8 bytes travels as an integer of that
    // size (in a general register even when its members are floats); anything else goes by
    // reference to a caller copy, and is returned through a hidden pointer.
    if arch == Arch::Win64 {
        return matches!(layout.size, 1 | 2 | 4 | 8).then(|| int_pieces(layout.size));
    }
    // i386 System V (Linux): no argument or result travels in a register but a scalar one. An
    // aggregate is copied onto the stack as an argument and returned through a hidden pointer,
    // whatever its size or members (`-fno-reg-struct-return`, the Linux default). ELFv1
    // powerpc64 is the same by Clang's lowering: by-value arguments, results in memory.
    if arch == Arch::X86 || arch == Arch::PowerPc64 {
        return None;
    }
    if arch == Arch::Wasm {
        return wasm_single_scalar(layout);
    }
    if arch == Arch::S390x {
        return s390x_small(layout);
    }
    if arch == Arch::Arm32 {
        return Some(arm32_pieces(layout));
    }
    if arch == Arch::Avr {
        return Some(avr_pieces(layout, data));
    }
    if layout.size > 16 && !(arch.is_aarch64() && hfa(layout).is_some()) {
        return None;
    }
    // System V classes an x87 `long double` X87/X87UP: alone (a scalar, or a struct holding just
    // one) it is passed in memory and returned in `st(0)`; mixed with anything else, the whole
    // aggregate is MEMORY.
    if arch == Arch::X86_64 && layout.fields.iter().any(|&(_, t)| t == Ty::F80) {
        return (layout.size == 16 && layout.fields == [(0, Ty::F80)]).then(|| {
            vec![Piece {
                offset: 0,
                ty: PieceTy::X87,
            }]
        });
    }
    match arch {
        Arch::Aarch64 | Arch::Win64Arm => {
            // Homogeneous floating-point aggregates (up to four members) use vector registers.
            if let Some(ty) = hfa(layout) {
                return Some(
                    layout
                        .fields
                        .iter()
                        .map(|&(offset, _)| Piece {
                            offset,
                            ty,
                        })
                        .collect(),
                );
            }
            Some(int_pieces(layout.size))
        }
        Arch::Win64
        | Arch::Wasm
        | Arch::X86
        | Arch::S390x
        | Arch::PowerPc64
        | Arch::Arm32
        | Arch::Avr => {
            unreachable!("handled above")
        }
        Arch::X86_64 => {
            // System V: an aggregate with a member that is not naturally aligned (a packed
            // struct, or `#no_padding`) is MEMORY, however small.
            if layout
                .fields
                .iter()
                .any(|&(off, t)| t.size(data) > 0 && off % t.size(data) != 0)
            {
                return None;
            }
            let count = layout.size.div_ceil(8);
            let mut pieces = Vec::new();
            for k in 0..count {
                let fields: Vec<Ty> = layout
                    .fields
                    .iter()
                    .filter(|&&(off, _)| off / 8 == k)
                    .map(|&(_, ty)| ty)
                    .collect();
                // An eightbyte is SSE only when every scalar in it is a float.
                let sse = !fields.is_empty() && fields.iter().all(|t| t.is_float());
                let ty = if !sse {
                    PieceTy::I64
                } else if fields.contains(&Ty::F64) {
                    PieceTy::F64
                } else if fields.len() >= 2 {
                    PieceTy::V2F32
                } else {
                    PieceTy::F32
                };
                pieces.push(Piece {
                    offset: k * 8,
                    ty,
                });
            }
            Some(pieces)
        }
    }
}

/// The one scalar a wasm aggregate is passed as, when it holds exactly one. Only 8-byte
/// integers and pointers, `f32` and `f64` have a piece type; a struct around a smaller
/// integer goes by pointer here, where Clang would pass an `i32`.
fn wasm_single_scalar(layout: &AggLayout) -> Option<Vec<Piece>> {
    let [(0, ty)] = layout.fields[..] else {
        return None;
    };
    let ty = match ty {
        Ty::I64 | Ty::Ptr => PieceTy::I64,
        Ty::F64 => PieceTy::F64,
        Ty::F32 => PieceTy::F32,
        _ => return None,
    };
    (layout.size == ty_size(ty)).then(|| {
        vec![Piece {
            offset: 0,
            ty,
        }]
    })
}

fn ty_size(ty: PieceTy) -> u64 {
    if ty == PieceTy::F32 {
        4
    } else {
        8
    }
}

/// s390x: an aggregate holding one float travels as that float, any other of 1, 2, 4 or 8 bytes
/// as an integer of its size; the rest go by reference.
fn s390x_small(layout: &AggLayout) -> Option<Vec<Piece>> {
    let ty = match layout.fields.as_slice() {
        [(0, Ty::F32)] if layout.size == 4 => PieceTy::F32,
        [(0, Ty::F64)] if layout.size == 8 => PieceTy::F64,
        _ if matches!(layout.size, 1 | 2 | 4 | 8) => PieceTy::Int(layout.size as u8),
        _ => return None,
    };
    Some(vec![Piece {
        offset: 0,
        ty,
    }])
}

/// AAPCS32 (VFP): a homogeneous float aggregate travels in VFP registers, any other as 4-byte
/// integers, or 8-byte ones when it is 8-aligned (which makes it start at an even register).
fn arm32_pieces(layout: &AggLayout) -> Vec<Piece> {
    match hfa(layout) {
        Some(ty @ (PieceTy::F32 | PieceTy::F64)) => layout
            .fields
            .iter()
            .map(|&(offset, _)| Piece {
                offset,
                ty,
            })
            .collect(),
        _ => arm32_int_pieces(layout),
    }
}

fn arm32_int_pieces(layout: &AggLayout) -> Vec<Piece> {
    let (step, ty) = if layout.align > 4 {
        (8, PieceTy::I64)
    } else {
        (4, PieceTy::Int(4))
    };
    (0..layout.size.div_ceil(step))
        .map(|k| Piece {
            offset: k * step,
            ty,
        })
        .collect()
}

/// AVR: each scalar member is its own register group, in memory order (there is no padding).
fn avr_pieces(layout: &AggLayout, data: &DataLayout) -> Vec<Piece> {
    layout
        .fields
        .iter()
        .map(|&(offset, ty)| Piece {
            offset,
            ty: match ty {
                Ty::F32 => PieceTy::F32,
                Ty::F64 => PieceTy::F64,
                other => PieceTy::Int(other.size(data) as u8),
            },
        })
        .collect()
}

fn int_pieces(size: u64) -> Vec<Piece> {
    (0..size.div_ceil(8))
        .map(|k| Piece {
            offset: k * 8,
            ty: PieceTy::I64,
        })
        .collect()
}

/// If the aggregate is a homogeneous float aggregate of 1..=4 members,
/// returns the member type. A binary128 `long double` counts as a float member (a quad HFA).
fn hfa(layout: &AggLayout) -> Option<PieceTy> {
    let first = layout.fields.first()?.1;
    if layout.fields.len() > 4 || !(first.is_float() || first == Ty::F128) {
        return None;
    }
    if layout.fields.iter().any(|&(_, t)| t != first) {
        return None;
    }
    let stride = first.numeric_size();
    // Members must be laid out back to back, or an HFA's registers would not
    // line up with its memory image.
    if layout
        .fields
        .iter()
        .enumerate()
        .any(|(i, &(off, _))| off != i as u64 * stride)
    {
        return None;
    }
    let ty = match first {
        Ty::F32 => PieceTy::F32,
        Ty::F128 => PieceTy::F128,
        _ => PieceTy::F64,
    };
    Some(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LP64: DataLayout = DataLayout::lp64_little();

    fn target_arch(triple: &str) -> Option<Arch> {
        crate::target::Target::parse(triple).ok()?.arch()
    }

    fn layout(size: u64, fields: &[(u64, Ty)]) -> AggLayout {
        AggLayout {
            size,
            align: 8,
            fields: fields.to_vec(),
        }
    }

    /// i386 System V psABI: an aggregate is passed in the argument area and returned in
    /// memory, however small, and whatever its members.
    #[test]
    fn i386_aggregates_go_on_the_stack_and_come_back_in_memory() {
        let i686 = crate::target::Target::parse("i686-unknown-linux-gnu").unwrap();
        assert_eq!(i686.arch(), Some(Arch::X86));
        let data = i686.data_layout();
        for l in [
            layout(4, &[(0, Ty::I32)]),
            layout(8, &[(0, Ty::F64)]),
            layout(4, &[(0, Ty::F32)]),
            layout(8, &[(0, Ty::I32), (4, Ty::I32)]),
            layout(12, &[(0, Ty::F80)]),
        ] {
            assert_eq!(classify_arg(Arch::X86, &l, &data), Passing::ByVal, "{l:?}");
            assert!(classify_ret(Arch::X86, &l, &data).is_none(), "{l:?}");
        }
        assert_eq!(
            classify_ret(Arch::X86, &layout(0, &[]), &data),
            Some(vec![])
        );
        // No argument register, and the result pointer is on the stack too.
        assert!(!sret_takes_int_register(Arch::X86, false));
    }

    /// s390x ELF psABI: aggregates of 1, 2, 4 or 8 bytes are integers of that size (a lone float
    /// stays a float), larger ones go by reference, and no aggregate is returned in registers.
    #[test]
    fn s390x_passes_small_aggregates_as_exact_integers() {
        let target = crate::target::Target::parse("s390x-unknown-linux-gnu").unwrap();
        assert_eq!(target.arch(), Some(Arch::S390x));
        let data = target.data_layout();
        let piece = |ty| {
            Passing::Registers(vec![Piece {
                offset: 0,
                ty,
            }])
        };
        assert_eq!(
            classify_arg(
                Arch::S390x,
                &layout(4, &[(0, Ty::I16), (2, Ty::I16)]),
                &data
            ),
            piece(PieceTy::Int(4))
        );
        assert_eq!(
            classify_arg(Arch::S390x, &layout(1, &[(0, Ty::I8)]), &data),
            piece(PieceTy::Int(1))
        );
        assert_eq!(
            classify_arg(Arch::S390x, &layout(8, &[(0, Ty::F64)]), &data),
            piece(PieceTy::F64)
        );
        assert_eq!(
            classify_arg(Arch::S390x, &layout(4, &[(0, Ty::F32)]), &data),
            piece(PieceTy::F32)
        );
        for l in [
            layout(16, &[(0, Ty::I64), (8, Ty::I64)]),
            layout(3, &[(0, Ty::I8), (1, Ty::I8), (2, Ty::I8)]),
        ] {
            assert_eq!(
                classify_arg(Arch::S390x, &l, &data),
                Passing::Indirect,
                "{l:?}"
            );
        }
        assert!(classify_ret(Arch::S390x, &layout(8, &[(0, Ty::I64)]), &data).is_none());
        assert!(sret_takes_int_register(Arch::S390x, false));
    }

    /// ELFv1 powerpc64: every aggregate argument goes by value on the stack image (the backend
    /// puts it in registers) and every aggregate result through a hidden pointer in `r3`.
    #[test]
    fn powerpc64_passes_aggregates_by_value_and_returns_them_in_memory() {
        let target = crate::target::Target::parse("powerpc64-unknown-linux-gnu").unwrap();
        assert_eq!(target.arch(), Some(Arch::PowerPc64));
        let data = target.data_layout();
        for l in [
            layout(4, &[(0, Ty::F32)]),
            layout(8, &[(0, Ty::I64)]),
            layout(16, &[(0, Ty::F64), (8, Ty::F64)]),
        ] {
            assert_eq!(
                classify_arg(Arch::PowerPc64, &l, &data),
                Passing::ByVal,
                "{l:?}"
            );
            assert!(classify_ret(Arch::PowerPc64, &l, &data).is_none(), "{l:?}");
        }
        assert!(sret_takes_int_register(Arch::PowerPc64, false));
    }

    /// AAPCS32 (VFP): small aggregates are one integer, float aggregates use VFP registers,
    /// the rest are core-register words (8-byte ones for an 8-aligned aggregate), and a result of
    /// more than 4 bytes goes through a hidden pointer unless it is a float aggregate.
    #[test]
    fn arm32_follows_the_vfp_variant_of_aapcs() {
        let target = crate::target::Target::parse("armv7-unknown-linux-gnueabihf").unwrap();
        assert_eq!(target.arch(), Some(Arch::Arm32));
        let data = target.data_layout();
        let l = |size, align, fields: &[(u64, Ty)]| AggLayout {
            size,
            align,
            fields: fields.to_vec(),
        };
        let words = |n: u64| {
            Passing::Registers(
                (0..n)
                    .map(|k| Piece {
                        offset: k * 4,
                        ty: PieceTy::Int(4),
                    })
                    .collect(),
            )
        };
        let small = l(2, 1, &[(0, Ty::I8), (1, Ty::I8)]);
        assert_eq!(classify_arg(Arch::Arm32, &small, &data), words(1));
        assert_eq!(
            classify_ret(Arch::Arm32, &small, &data).map(|p| p.len()),
            Some(1)
        );
        let triple = l(12, 4, &[(0, Ty::I32), (4, Ty::I32), (8, Ty::I32)]);
        assert_eq!(classify_arg(Arch::Arm32, &triple, &data), words(3));
        assert!(classify_ret(Arch::Arm32, &triple, &data).is_none());
        let eight = l(8, 8, &[(0, Ty::I64)]);
        assert_eq!(
            classify_arg(Arch::Arm32, &eight, &data),
            Passing::Registers(vec![Piece {
                offset: 0,
                ty: PieceTy::I64
            }])
        );
        let floats = l(8, 4, &[(0, Ty::F32), (4, Ty::F32)]);
        let in_vfp = |ty| {
            Passing::Registers(
                [0, 4]
                    .iter()
                    .map(|&offset| Piece {
                        offset,
                        ty,
                    })
                    .collect(),
            )
        };
        assert_eq!(
            classify_arg(Arch::Arm32, &floats, &data),
            in_vfp(PieceTy::F32)
        );
        assert!(classify_ret(Arch::Arm32, &floats, &data).is_some());
        // A variadic call uses core registers whatever the members are.
        assert_eq!(classify_vararg(Arch::Arm32, &floats, &data), words(2));
    }

    /// AVR (avr-gcc): members travel as register groups while 18 register bytes last, an
    /// aggregate that does not fit goes on the stack whole, and a result over 8 bytes is returned
    /// through a hidden pointer.
    #[test]
    fn avr_passes_aggregates_in_registers_while_they_fit() {
        let target = crate::target::Target::parse("avr-none-elf").unwrap();
        assert_eq!(target.arch(), Some(Arch::Avr));
        let data = target.data_layout();
        let l = |size, fields: &[(u64, Ty)]| AggLayout {
            size,
            align: 1,
            fields: fields.to_vec(),
        };
        let pair = l(3, &[(0, Ty::I16), (2, Ty::I8)]);
        let Passing::Registers(pieces) = classify_arg(Arch::Avr, &pair, &data) else {
            panic!("a small aggregate travels in registers");
        };
        assert_eq!(
            pieces.iter().map(|p| p.ty).collect::<Vec<_>>(),
            [PieceTy::Int(2), PieceTy::Int(1)]
        );
        let mut budget = RegBudget::new(Arch::Avr, false);
        // Nine 16-bit scalars use all 18 registers: the next aggregate goes on the stack.
        for _ in 0..9 {
            budget.scalar(Ty::I16);
        }
        assert_eq!(
            budget.aggregate(classify_arg(Arch::Avr, &pair, &data)),
            Passing::ByVal
        );
        let mut budget = RegBudget::new(Arch::Avr, false);
        budget.scalar(Ty::I8);
        assert!(matches!(
            budget.aggregate(classify_arg(Arch::Avr, &pair, &data)),
            Passing::Registers(_)
        ));
        let eight = l(8, &[(0, Ty::I32), (4, Ty::I32)]);
        assert!(classify_ret(Arch::Avr, &eight, &data).is_some());
        let nine = l(9, &[(0, Ty::I64), (8, Ty::I8)]);
        assert!(classify_ret(Arch::Avr, &nine, &data).is_none());
        assert!(sret_takes_int_register(Arch::Avr, false));
    }

    #[test]
    fn sysv_unaligned_member_forces_memory() {
        // `char; int` packed: the int sits at offset 1.
        let l = layout(5, &[(0, Ty::I8), (1, Ty::I32)]);
        assert!(classify_ret(Arch::X86_64, &l, &LP64).is_none());
        assert_eq!(classify_arg(Arch::X86_64, &l, &LP64), Passing::ByVal);
        // `int; double` packed: the double is at offset 4, inside one eightbyte.
        let l = layout(12, &[(0, Ty::I32), (4, Ty::F64)]);
        assert!(classify_ret(Arch::X86_64, &l, &LP64).is_none());
        // Naturally aligned members of a packed struct are still register class.
        let l = layout(12, &[(0, Ty::F64), (8, Ty::I32)]);
        assert_eq!(classify_ret(Arch::X86_64, &l, &LP64).unwrap().len(), 2);
        // The other ABIs size-code packed structs like any other.
        let l = layout(5, &[(0, Ty::I8), (1, Ty::I32)]);
        assert_eq!(classify_ret(Arch::Aarch64, &l, &LP64).unwrap().len(), 1);
        assert_eq!(classify_ret(Arch::Win64Arm, &l, &LP64).unwrap().len(), 1);
        assert_eq!(classify_arg(Arch::Win64, &l, &LP64), Passing::Indirect);
    }

    #[test]
    fn sysv_aggregate_without_enough_registers_goes_to_memory() {
        let two = layout(16, &[(0, Ty::I64), (8, Ty::I64)]);
        let mut budget = RegBudget::new(Arch::X86_64, false);
        for _ in 0..5 {
            budget.scalar(Ty::I64);
        }
        // One integer register left: the two-eightbyte struct is MEMORY, a scalar still fits.
        assert_eq!(
            budget.aggregate(classify_arg(Arch::X86_64, &two, &LP64)),
            Passing::ByVal
        );
        let one = layout(8, &[(0, Ty::I64)]);
        assert!(matches!(
            budget.aggregate(classify_arg(Arch::X86_64, &one, &LP64)),
            Passing::Registers(_)
        ));
        // A hidden result pointer takes the first register.
        let mut budget = RegBudget::new(Arch::X86_64, true);
        for _ in 0..4 {
            budget.scalar(Ty::I64);
        }
        assert_eq!(
            budget.aggregate(classify_arg(Arch::X86_64, &two, &LP64)),
            Passing::ByVal
        );
        // AArch64 leaves it to the backend's array parameter.
        let mut budget = RegBudget::new(Arch::Aarch64, false);
        for _ in 0..7 {
            budget.scalar(Ty::I64);
        }
        assert!(matches!(
            budget.aggregate(classify_arg(Arch::Aarch64, &two, &LP64)),
            Passing::Registers(_)
        ));
    }

    #[test]
    fn aarch64_packed_floats_stay_homogeneous() {
        let l = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        let p = classify_ret(Arch::Aarch64, &l, &LP64).unwrap();
        assert!(p.iter().all(|p| p.ty == PieceTy::F32) && p.len() == 2);
        // Not back to back, or not one type: integer pieces.
        let l = layout(9, &[(0, Ty::I8), (1, Ty::F64)]);
        assert!(
            classify_ret(Arch::Aarch64, &l, &LP64)
                .unwrap()
                .iter()
                .all(|p| p.ty == PieceTy::I64)
        );
    }

    #[test]
    fn aarch64_small_int_struct_uses_x_registers() {
        let l = layout(12, &[(0, Ty::I64), (8, Ty::I32)]);
        assert_eq!(classify_ret(Arch::Aarch64, &l, &LP64).unwrap().len(), 2);
    }

    #[test]
    fn aarch64_hfa_uses_float_registers() {
        let l = layout(
            16,
            &[(0, Ty::F32), (4, Ty::F32), (8, Ty::F32), (12, Ty::F32)],
        );
        let p = classify_ret(Arch::Aarch64, &l, &LP64).unwrap();
        assert!(p.iter().all(|p| p.ty == PieceTy::F32) && p.len() == 4);
    }

    #[test]
    fn aarch64_large_aggregate_is_indirect() {
        let l = layout(24, &[(0, Ty::I64), (8, Ty::I64), (16, Ty::I64)]);
        assert_eq!(classify_arg(Arch::Aarch64, &l, &LP64), Passing::Indirect);
        assert!(classify_ret(Arch::Aarch64, &l, &LP64).is_none());
    }

    #[test]
    fn sysv_mixed_eightbytes() {
        let l = layout(16, &[(0, Ty::I64), (8, Ty::F64)]);
        let p = classify_ret(Arch::X86_64, &l, &LP64).unwrap();
        assert_eq!(p[0].ty, PieceTy::I64);
        assert_eq!(p[1].ty, PieceTy::F64);
        let v = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        assert_eq!(
            classify_ret(Arch::X86_64, &v, &LP64).unwrap()[0].ty,
            PieceTy::V2F32
        );
    }

    #[test]
    fn win64_power_of_two_sizes_use_one_integer_register() {
        let floats = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        let p = classify_ret(Arch::Win64, &floats, &LP64).unwrap();
        assert_eq!(
            p,
            vec![Piece {
                offset: 0,
                ty: PieceTy::I64
            }]
        );
        let small = layout(4, &[(0, Ty::I16), (2, Ty::I16)]);
        assert!(
            matches!(classify_arg(Arch::Win64, &small, &LP64), Passing::Registers(p) if p.len() == 1)
        );
    }

    #[test]
    fn win64_other_sizes_go_by_reference() {
        let odd = layout(12, &[(0, Ty::I32), (4, Ty::I32), (8, Ty::I32)]);
        assert_eq!(classify_arg(Arch::Win64, &odd, &LP64), Passing::Indirect);
        assert!(classify_ret(Arch::Win64, &odd, &LP64).is_none());
        let pair = layout(16, &[(0, Ty::F64), (8, Ty::F64)]);
        assert_eq!(classify_arg(Arch::Win64, &pair, &LP64), Passing::Indirect);
        let three = layout(3, &[(0, Ty::I8), (1, Ty::I8), (2, Ty::I8)]);
        assert_eq!(classify_arg(Arch::Win64, &three, &LP64), Passing::Indirect);
    }

    #[test]
    fn windows_triples_select_win64() {
        assert_eq!(target_arch("x86_64-pc-windows-msvc"), Some(Arch::Win64));
        assert_eq!(target_arch("x86_64-w64-windows-gnu"), Some(Arch::Win64));
        assert_eq!(target_arch("x86_64-w64-mingw32"), Some(Arch::Win64));
        assert_eq!(target_arch("x86_64-unknown-linux-gnu"), Some(Arch::X86_64));
    }

    /// Piece types of a register classification, for comparing with Clang's IR.
    fn shape(passing: Passing) -> Option<Vec<PieceTy>> {
        match passing {
            Passing::Registers(pieces) => Some(pieces.iter().map(|p| p.ty).collect()),
            Passing::ByVal | Passing::Indirect => None,
        }
    }

    // Expected shapes below are what `clang -target aarch64-pc-windows-msvc -S -emit-llvm`
    // emits for the same C structs: `[2 x float]` is two F32 pieces, `[2 x i64]` two I64,
    // `ptr` (or `sret`) by reference.

    #[test]
    fn win64arm_fixed_arguments_follow_aapcs64() {
        use PieceTy::*;
        let arch = Arch::Win64Arm;
        let f2 = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        assert_eq!(shape(classify_arg(arch, &f2, &LP64)), Some(vec![F32, F32]));
        let d4 = layout(
            32,
            &[(0, Ty::F64), (8, Ty::F64), (16, Ty::F64), (24, Ty::F64)],
        );
        // An HFA of up to four members uses vector registers even beyond 16 bytes.
        assert_eq!(shape(classify_arg(arch, &d4, &LP64)), Some(vec![F64; 4]));
        assert_eq!(classify_ret(arch, &d4, &LP64).map(|p| p.len()), Some(4));
        let i12 = layout(12, &[(0, Ty::I32), (4, Ty::I32), (8, Ty::I32)]);
        assert_eq!(shape(classify_arg(arch, &i12, &LP64)), Some(vec![I64, I64]));
        let c3 = layout(3, &[(0, Ty::I8), (1, Ty::I8), (2, Ty::I8)]);
        assert_eq!(shape(classify_arg(arch, &c3, &LP64)), Some(vec![I64]));
        let mix = layout(16, &[(0, Ty::F64), (8, Ty::I64)]);
        assert_eq!(shape(classify_arg(arch, &mix, &LP64)), Some(vec![I64, I64]));
        // Over 16 bytes and not an HFA: by reference, and returned through x8.
        let i3 = layout(24, &[(0, Ty::I64), (8, Ty::I64), (16, Ty::I64)]);
        assert_eq!(classify_arg(arch, &i3, &LP64), Passing::Indirect);
        assert!(classify_ret(arch, &i3, &LP64).is_none());
    }

    #[test]
    fn win64arm_variadic_arguments_ignore_hfas() {
        use PieceTy::*;
        let arch = Arch::Win64Arm;
        let f2 = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        assert_eq!(shape(classify_vararg(arch, &f2, &LP64)), Some(vec![I64]));
        let f4 = layout(
            16,
            &[(0, Ty::F32), (4, Ty::F32), (8, Ty::F32), (12, Ty::F32)],
        );
        assert_eq!(
            shape(classify_vararg(arch, &f4, &LP64)),
            Some(vec![I64, I64])
        );
        let d2 = layout(16, &[(0, Ty::F64), (8, Ty::F64)]);
        assert_eq!(
            shape(classify_vararg(arch, &d2, &LP64)),
            Some(vec![I64, I64])
        );
        let d4 = layout(
            32,
            &[(0, Ty::F64), (8, Ty::F64), (16, Ty::F64), (24, Ty::F64)],
        );
        assert_eq!(classify_vararg(arch, &d4, &LP64), Passing::Indirect);
        // Results are unaffected: a variadic procedure still returns an HFA in v0-v3.
        assert_eq!(classify_ret(arch, &f4, &LP64).map(|p| p.len()), Some(4));
        // Elsewhere a variadic aggregate is classified like any other.
        assert_eq!(
            shape(classify_vararg(Arch::Aarch64, &f2, &LP64)),
            Some(vec![F32, F32])
        );
    }

    #[test]
    fn windows_arm64_triples_select_win64arm() {
        for triple in [
            "aarch64-pc-windows-msvc",
            "aarch64-w64-windows-gnu",
            "aarch64-w64-mingw32",
            "arm64-pc-windows-msvc",
        ] {
            assert_eq!(target_arch(triple), Some(Arch::Win64Arm), "{triple}");
        }
        assert_eq!(
            target_arch("aarch64-unknown-linux-gnu"),
            Some(Arch::Aarch64)
        );
        assert!(Arch::Win64Arm.is_aarch64() && Arch::Win64Arm.is_windows());
    }

    /// Checked against `clang -target x86_64-linux-gnu` / `aarch64-linux-gnu` /
    /// `x86_64-w64-windows-gnu -S -emit-llvm` for `long double f(long double)` and structs
    /// `{ long double }`, `{ char; long double }`, `{ long double a, b; }`.
    #[test]
    fn long_double_classes() {
        let x87 = AggLayout {
            size: 16,
            align: 16,
            fields: vec![(0, Ty::F80)],
        };
        let one = vec![Piece {
            offset: 0,
            ty: PieceTy::X87,
        }];
        assert_eq!(
            classify_arg(Arch::X86_64, &x87, &LP64),
            Passing::Registers(one.clone())
        );
        assert_eq!(classify_ret(Arch::X86_64, &x87, &LP64), Some(one));
        let tagged = AggLayout {
            size: 32,
            align: 16,
            fields: vec![(0, Ty::I8), (16, Ty::F80)],
        };
        assert_eq!(classify_arg(Arch::X86_64, &tagged, &LP64), Passing::ByVal);
        assert!(classify_ret(Arch::X86_64, &tagged, &LP64).is_none());
        // MinGW: by reference, returned through a hidden pointer.
        assert_eq!(classify_arg(Arch::Win64, &x87, &LP64), Passing::Indirect);
        assert!(classify_ret(Arch::Win64, &x87, &LP64).is_none());
        let quad = |n: u64| AggLayout {
            size: 16 * n,
            align: 16,
            fields: (0..n).map(|i| (i * 16, Ty::F128)).collect(),
        };
        let p = classify_ret(Arch::Aarch64, &quad(2), &LP64).unwrap();
        assert!(p.len() == 2 && p.iter().all(|p| p.ty == PieceTy::F128));
        assert!(
            matches!(classify_arg(Arch::Aarch64, &quad(1), &LP64), Passing::Registers(p) if p.len() == 1)
        );
        let mixed = AggLayout {
            size: 32,
            align: 16,
            fields: vec![(0, Ty::I8), (16, Ty::F128)],
        };
        assert_eq!(
            classify_arg(Arch::Aarch64, &mixed, &LP64),
            Passing::Indirect
        );
    }

    #[test]
    fn sysv_large_aggregate_is_byval() {
        let l = layout(24, &[(0, Ty::I64), (8, Ty::I64), (16, Ty::I64)]);
        assert_eq!(classify_arg(Arch::X86_64, &l, &LP64), Passing::ByVal);
    }

    #[test]
    fn wasm64_passes_single_scalars_directly_and_the_rest_by_pointer() {
        assert_eq!(target_arch("wasm64-unknown-unknown"), Some(Arch::Wasm));
        let one = layout(8, &[(0, Ty::F64)]);
        assert_eq!(
            classify_arg(Arch::Wasm, &one, &LP64),
            Passing::Registers(vec![Piece {
                offset: 0,
                ty: PieceTy::F64
            }])
        );
        let pair = layout(16, &[(0, Ty::I64), (8, Ty::I64)]);
        assert_eq!(classify_arg(Arch::Wasm, &pair, &LP64), Passing::ByVal);
        assert!(classify_ret(Arch::Wasm, &pair, &LP64).is_none());
        let small = layout(4, &[(0, Ty::I32)]);
        assert_eq!(classify_arg(Arch::Wasm, &small, &LP64), Passing::ByVal);
    }
}
