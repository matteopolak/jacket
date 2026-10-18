//! One-line descriptions of each instruction.
use super::decode::Kind;
use crate::ir::{BinOp, Ty};
use crate::sema::asm::mask::KOp;
use crate::sema::asm::scalar::{Rep, StrOp, XOp};
use crate::sema::asm::simd::{AesOp, B2, Conv, DupKind, FmaKind, GfOp, SOp, Sat, ShaOp, U1};
use crate::sema::asm::vec::{Cvt, Lane, Shift, VOp};
use crate::sema::asm::{Alu, BitTest, Cond, Op, ShiftKind};

fn lanes(ty: Ty) -> &'static str {
    match ty {
        Ty::I8 => "bytes",
        Ty::I16 => "words",
        Ty::I32 => "dwords",
        Ty::I64 => "qwords",
        Ty::F32 => "single-precision floats",
        Ty::F64 => "double-precision floats",
        _ => "elements",
    }
}

fn float_lanes(ty: Ty, scalar: bool) -> String {
    let precision = if matches!(ty, Ty::F32 | Ty::I32) {
        "single"
    } else {
        "double"
    };
    if scalar {
        format!("the low {precision}-precision float (upper lanes from the first source)")
    } else {
        format!("packed {precision}-precision floats")
    }
}

fn bits_of(ty: Ty) -> u64 {
    ty.size() * 8
}

fn condition(c: Cond) -> &'static str {
    match c {
        Cond::E => "equal / zero (ZF=1)",
        Cond::Ne => "not equal / not zero (ZF=0)",
        Cond::B => "below, unsigned (CF=1)",
        Cond::Ae => "above or equal, unsigned (CF=0)",
        Cond::Be => "below or equal, unsigned (CF=1 or ZF=1)",
        Cond::A => "above, unsigned (CF=0 and ZF=0)",
        Cond::S => "negative (SF=1)",
        Cond::Ns => "not negative (SF=0)",
        Cond::O => "overflow (OF=1)",
        Cond::No => "no overflow (OF=0)",
        Cond::L => "less, signed (SF!=OF)",
        Cond::Ge => "greater or equal, signed (SF=OF)",
        Cond::Le => "less or equal, signed (ZF=1 or SF!=OF)",
        Cond::G => "greater, signed (ZF=0 and SF=OF)",
        Cond::P => "parity even (PF=1)",
        Cond::Np => "parity odd (PF=0)",
    }
}

pub(super) fn describe(name: &str, kind: Kind) -> String {
    if let Some(base) = name.strip_prefix("lock_") {
        return format!(
            "Atomic with a memory destination: {}",
            lower_first(&describe(base, kind))
        );
    }
    match kind {
        Kind::Core(op) => describe_core(name, op),
        Kind::Scalar(op) => describe_scalar(name, op),
        Kind::Mask(op, ty) => describe_mask(op, ty),
        Kind::Vector(op) => describe_vector(name.strip_prefix('v').unwrap_or(name), op),
    }
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_lowercase().chain(c).collect(),
        None => String::new(),
    }
}

fn describe_core(name: &str, op: Op) -> String {
    let s: &str = match op {
        Op::Mov if name == "movnti" => {
            "Store a register to memory, hinting that it bypass the cache."
        }
        Op::Mov => "Copy the source into the destination.",
        Op::Movzx => "Copy a narrower source into the destination, zero-filling the upper bits.",
        Op::Movsx => "Copy a narrower source into the destination, sign-extending it.",
        Op::Movbe => "Copy with the byte order reversed (big-endian loads and stores).",
        Op::Lea => "Compute the address of a memory operand without accessing memory.",
        Op::Xchg => "Swap the two operands; with a memory operand the swap is atomic.",
        Op::Xadd => {
            "Add the source into the destination and hand the old destination back in the source."
        }
        Op::Cmpxchg => {
            "Store the source if the destination equals the accumulator, else load the destination into it; ZF tells which."
        }
        Op::Alu(alu) => match alu {
            Alu::Add => "Add the source to the destination, setting the flags.",
            Alu::Sub => "Subtract the source from the destination, setting the flags.",
            Alu::Adc => "Add the source and the carry flag to the destination.",
            Alu::Sbb => "Subtract the source and the carry flag from the destination.",
            Alu::And => "Bitwise AND into the destination; clears CF and OF.",
            Alu::Or => "Bitwise OR into the destination; clears CF and OF.",
            Alu::Xor => "Bitwise exclusive OR into the destination; clears CF and OF.",
            Alu::Cmp => "Set the flags from a - b without storing the difference.",
            Alu::Test => "Set the flags from a AND b without storing the result.",
        },
        Op::Inc => "Add one; CF is left as it was.",
        Op::Dec => "Subtract one; CF is left as it was.",
        Op::Neg => "Replace the operand with its two's-complement negation.",
        Op::Not => "Invert every bit; the flags are untouched.",
        Op::Shift(kind) => match kind {
            ShiftKind::Shl if name == "sal" => {
                "Shift left (same as shl), filling with zeros; the count defaults to 1."
            }
            ShiftKind::Shl => "Shift left, filling with zeros; the count defaults to 1.",
            ShiftKind::Shr => {
                "Shift right, filling with zeros (unsigned divide by a power of two)."
            }
            ShiftKind::Sar => {
                "Shift right, filling with copies of the sign bit (signed divide, rounding down)."
            }
            ShiftKind::Rol => "Rotate the bits left; bits leaving the top re-enter at the bottom.",
            ShiftKind::Ror => "Rotate the bits right; bits leaving the bottom re-enter at the top.",
        },
        Op::Bt(kind) => match kind {
            BitTest::Bt => "Copy the selected bit into CF.",
            BitTest::Bts => "Copy the selected bit into CF, then set it.",
            BitTest::Btr => "Copy the selected bit into CF, then clear it.",
            BitTest::Btc => "Copy the selected bit into CF, then flip it.",
        },
        Op::Bsf => {
            "Index of the lowest set bit; ZF=1 and the destination unchanged when the source is zero."
        }
        Op::Bsr => {
            "Index of the highest set bit; ZF=1 and the destination unchanged when the source is zero."
        }
        Op::Popcnt => "Count the set bits of the source.",
        Op::Lzcnt => "Count the zero bits above the highest set bit (the width for zero).",
        Op::Tzcnt => "Count the zero bits below the lowest set bit (the width for zero).",
        Op::Bswap => "Reverse the order of the bytes in a register.",
        Op::Blsr => "Clear the lowest set bit of the source.",
        Op::Blsi => "Keep only the lowest set bit of the source.",
        Op::Blsmsk => "Set every bit up to and including the lowest set bit of the source.",
        Op::Imul => {
            "Signed multiply: two-operand and immediate forms keep the low half; `hi, lo, src` gives the full product."
        }
        Op::Mul => "Unsigned widening multiply: hi:lo = lo * src.",
        Op::Setcc(c) => return format!("Set a byte to 1 when {}, else to 0.", condition(c)),
        Op::Cmovcc(c) => {
            return format!(
                "Copy the source into the destination when {}.",
                condition(c)
            );
        }
        Op::Nop => "Do nothing.",
        Op::Fence(crate::ir::X86Fence::Load) => {
            "Execute LFENCE: order loads and instruction execution; preserve flags."
        }
        Op::Fence(crate::ir::X86Fence::Store) => "Execute SFENCE: order stores; preserve flags.",
        Op::Fence(crate::ir::X86Fence::Full) => {
            "Execute MFENCE: order loads and stores; preserve flags."
        }
        Op::Pause => "Spin-loop hint: lets the core relax while waiting.",
        Op::Int3 => "Breakpoint: stop in the debugger.",
        Op::SetCarry(Some(false)) => "Clear the carry flag.",
        Op::SetCarry(Some(true)) => "Set the carry flag.",
        Op::SetCarry(None) => "Flip the carry flag.",
        Op::Rdtsc => {
            "Read the actual x86 time-stamp counter into hi:lo; not a serializing instruction."
        }
        Op::Rdtscp => {
            "Read the actual x86 time-stamp counter into hi:lo and IA32_TSC_AUX into the third output."
        }
        Op::Rdseed => {
            "One RDSEED attempt: CF=1 on success, other arithmetic flags clear; no automatic retry."
        }
        Op::Rdrand => {
            "One RDRAND attempt: CF=1 on success, other arithmetic flags clear; no automatic retry."
        }
        Op::Syscall => {
            "Linux x86-64 system call: rax is the number in and the raw result out, with arguments in the registers pinned to di, si, d, 10, 8 and 9; clobbers rcx and r11."
        }
        Op::Cpuid => {
            "Query the executing x86-64 processor: a/eax and c/ecx select leaf/subleaf; a,b,c,d receive the actual 32-bit registers, zero-extended into 64-bit destinations."
        }
    };
    s.to_owned()
}

fn describe_scalar(name: &str, op: XOp) -> String {
    let s: &str = match op {
        XOp::Div(false) => {
            "Unsigned divide hi:lo by the divisor: quotient into lo, remainder into hi; traps on zero or overflow."
        }
        XOp::Div(true) => {
            "Signed divide hi:lo by the divisor: quotient into lo, remainder into hi; traps on zero or overflow."
        }
        XOp::WidenA(ty) => {
            return format!(
                "Sign-extend the low half of the accumulator to {} bits.",
                bits_of(ty)
            );
        }
        XOp::SignFill(ty) => {
            return format!(
                "Fill d with copies of the sign bit of the {}-bit accumulator (sets up a signed divide).",
                bits_of(ty)
            );
        }
        XOp::DoubleShift(true) => {
            "Shift the destination left, filling from the top bits of the source."
        }
        XOp::DoubleShift(false) => {
            "Shift the destination right, filling from the low bits of the source."
        }
        XOp::RotateCarry(true) => "Rotate left through the carry flag.",
        XOp::RotateCarry(false) => "Rotate right through the carry flag.",
        XOp::Mulx => "Unsigned multiply d by src into hi:lo without touching the flags.",
        XOp::AddCarry(true) => "Add with carry using only CF (the other flags are kept).",
        XOp::AddCarry(false) => "Add with carry using OF as the carry (the other flags are kept).",
        XOp::Andn => "AND b with the inverted a.",
        XOp::Bextr => {
            "Extract a bit field: start in bits 0-7 and length in bits 8-15 of the control."
        }
        XOp::Bzhi => "Clear the bits of the source from the given index upwards.",
        XOp::Pdep => {
            "Scatter the low bits of the source to the positions of the set bits of the mask."
        }
        XOp::Pext => "Gather the source bits under the set bits of the mask into the low bits.",
        XOp::ShiftX(ShiftKind::Shl) => "Shift left by a register count without touching the flags.",
        XOp::ShiftX(ShiftKind::Shr) => {
            "Logical right shift by a register count without touching the flags."
        }
        XOp::ShiftX(ShiftKind::Sar) => {
            "Arithmetic right shift by a register count without touching the flags."
        }
        XOp::ShiftX(ShiftKind::Rol | ShiftKind::Ror) | XOp::Rorx => {
            "Rotate right by an immediate without touching the flags."
        }
        XOp::Crc32 => "Fold the data into a CRC-32C (Castagnoli) checksum accumulator.",
        XOp::Lahf => "Load SF, ZF, PF and CF into the second byte (AH) of the operand.",
        XOp::Sahf => "Store the second byte (AH) of the operand into SF, ZF, PF and CF.",
        XOp::Xlat => "Table lookup: replace the low byte of a with table[a].",
        XOp::Hint => match name {
            "prefetcht0" | "prefetcht1" | "prefetcht2" | "prefetchnta" => {
                "Execute the x86 cache prefetch hint for the address; preserve flags; no guaranteed data fetch."
            }
            _ => {
                "Recognized cache operation; architectural implementation is unavailable and checking diagnoses it."
            }
        },
        XOp::Direction(false) => "Clear the direction flag: string instructions walk upwards.",
        XOp::Direction(true) => {
            "Set the direction flag: string instructions in this block walk downwards."
        }
        XOp::CmpxchgPair(Ty::I32) => {
            "Compare d:a with 8 bytes of memory; store c:b if equal, else load them into d:a."
        }
        XOp::CmpxchgPair(_) => {
            "Compare d:a with 16 bytes of memory; store c:b if equal, else load them into d:a."
        }
        XOp::Str(op, rep) => {
            let what = match op {
                StrOp::Movs => "copy an element from [si] to [di]",
                StrOp::Stos => "store a to [di]",
                StrOp::Lods => "load [si] into a",
                StrOp::Cmps => "compare [si] with [di]",
                StrOp::Scas => "compare a with [di]",
            };
            let how = match rep {
                Rep::Once => "once",
                Rep::Count => "c times",
                Rep::Repe => "while equal, at most c times",
                Rep::Repne => "while not equal, at most c times",
            };
            return format!(
                "String instruction ({}): {what}, {how}, advancing the pointers by the element size.",
                if name.starts_with("rep") {
                    "repeated"
                } else {
                    "single"
                }
            );
        }
        XOp::Int => "Software interrupt; only `int 3` (a breakpoint) is meaningful.",
        XOp::Xgetbv => {
            "Read an extended control register into d:a (0 here: no extended state is reported)."
        }
        XOp::Stmxcsr => "Store MXCSR to memory (the default 0x1f80 here).",
        XOp::Ldmxcsr => "Load MXCSR from memory (accepted and ignored).",
        XOp::Rdpid => "Read the processor id (0 here).",
    };
    s.to_owned()
}

fn describe_mask(op: KOp, ty: Ty) -> String {
    let bits = bits_of(ty);
    match op {
        KOp::Mov => format!(
            "Move {bits} mask bits between mask registers, general-purpose registers and memory."
        ),
        KOp::Bin(BinOp::And) => format!("AND two {bits}-bit masks."),
        KOp::Bin(BinOp::Or) => format!("OR two {bits}-bit masks."),
        KOp::Bin(_) => format!("Exclusive OR two {bits}-bit masks."),
        KOp::AndNot => format!("AND the second {bits}-bit mask with the inverted first."),
        KOp::Xnor => format!("Exclusive NOR of two {bits}-bit masks."),
        KOp::Add => format!("Add two {bits}-bit masks as integers."),
        KOp::Not => format!("Invert a {bits}-bit mask."),
        KOp::ShiftLeft => format!("Shift a {bits}-bit mask left by an immediate."),
        KOp::ShiftRight => format!("Shift a {bits}-bit mask right by an immediate."),
        KOp::Test => {
            format!("Set ZF if a AND b is zero and CF if a AND NOT b is zero ({bits} bits).")
        }
        KOp::OrTest => {
            format!("Set ZF if a OR b is all zeros and CF if it is all ones ({bits} bits).")
        }
        KOp::Unpack => format!("Concatenate the low halves of two masks into a {bits}-bit mask."),
    }
}

fn describe_vector(name: &str, op: VOp) -> String {
    match op {
        VOp::Move if name.starts_with("movnt") => {
            "Move a whole vector to or from memory with a non-temporal (cache-bypassing) hint.".into()
        }
        VOp::Move if name.starts_with("mova") || name.starts_with("movdqa") => {
            "Move a whole vector between registers and memory; hardware wants memory aligned to the vector size.".into()
        }
        VOp::Move => "Move a whole vector between registers and memory, with no alignment requirement.".into(),
        VOp::MovScalarInt(bytes) => format!(
            "Move {} bits between a vector's low lane and a general-purpose register or memory.",
            bytes * 8
        ),
        VOp::MovScalarFloat(ty) => format!(
            "Move one {} to or from the low lane; the three-operand form merges into a copy of the first source.",
            if ty == Ty::F32 { "single-precision float" } else { "double-precision float" }
        ),
        VOp::Broadcast(bytes) => format!(
            "Repeat the first {bytes} bytes of the source across the whole destination."
        ),
        VOp::Bin(lane, ty, scalar) => {
            let f = float_lanes(ty, scalar);
            let i = lanes(ty);
            match lane {
                Lane::And => "Bitwise AND of the sources.".into(),
                Lane::Or => "Bitwise OR of the sources.".into(),
                Lane::Xor => "Bitwise exclusive OR of the sources.".into(),
                Lane::AndNot => "Bitwise AND of the second source with the inverted first source.".into(),
                Lane::Add => format!("Add packed {i}, wrapping on overflow."),
                Lane::Sub => format!("Subtract packed {i}, wrapping on overflow."),
                Lane::Mul => format!("Multiply packed {i}, keeping the low half of each product."),
                Lane::CmpEq => format!("Compare packed {i} for equality: all ones where equal (a mask bit per lane into an omr)."),
                Lane::CmpGt => format!("Compare packed signed {i} for greater-than: all ones where true (a mask bit per lane into an omr)."),
                Lane::MinS => format!("Lane-wise minimum of packed signed {i}."),
                Lane::MaxS => format!("Lane-wise maximum of packed signed {i}."),
                Lane::MinU => format!("Lane-wise minimum of packed unsigned {i}."),
                Lane::MaxU => format!("Lane-wise maximum of packed unsigned {i}."),
                Lane::FAdd => format!("Add {f}."),
                Lane::FSub => format!("Subtract {f}."),
                Lane::FMul => format!("Multiply {f}."),
                Lane::FDiv => format!("Divide {f}."),
                Lane::FMin => format!("Minimum of {f} (the second source when either is NaN)."),
                Lane::FMax => format!("Maximum of {f} (the second source when either is NaN)."),
            }
        }
        VOp::Sqrt(ty, scalar) => format!("Square root of {}.", float_lanes(ty, scalar)),
        VOp::Abs(ty) => format!("Absolute value of packed signed {}.", lanes(ty)),
        VOp::Shift(kind, ty) => {
            let what = match kind {
                Shift::Left => "left, filling with zeros",
                Shift::Logical => "right, filling with zeros",
                Shift::Arith => "right, filling with the sign bit",
            };
            format!(
                "Shift packed {} {what}, by an immediate or the low qword of a vector.",
                lanes(ty)
            )
        }
        VOp::Pshufd => "Rearrange the dwords within each 128-bit block as chosen by the immediate's 2-bit fields.".into(),
        VOp::Shufps => "Take two floats per 128-bit block from each source, as chosen by the immediate.".into(),
        VOp::Cvt(c) => match c {
            Cvt::IntToFloat => "Convert packed signed dwords to single-precision floats.".into(),
            Cvt::FloatToInt => "Convert packed single-precision floats to signed dwords, rounding to nearest even (or the `!n/!d/!u/!z` mode).".into(),
            Cvt::FloatToIntTrunc => "Convert packed single-precision floats to signed dwords, truncating toward zero.".into(),
        },
        VOp::Movmsk(bytes) => format!(
            "Collect the sign bit of each {bytes}-byte lane into the low bits of a general-purpose register."
        ),
        VOp::Gather(index, elem) => format!(
            "Load {elem}-byte elements from base + index*scale for each {index}-byte index whose mask lane is set, clearing the mask."
        ),
        VOp::Nop if name == "emms" => "Leave MMX state (no effect here).".into(),
        VOp::Nop => "Clear the upper parts of the vector registers (no effect here: declared registers are locals).".into(),
        VOp::Ext(s) => describe_simd(name, s),
    }
}

fn describe_simd(name: &str, op: SOp) -> String {
    match op {
        SOp::Bin(b, ty) => {
            let i = lanes(ty);
            match b {
                B2::AddSatS => format!("Add packed signed {i}, clamping to the signed range."),
                B2::AddSatU => format!("Add packed unsigned {i}, clamping to the unsigned range."),
                B2::SubSatS => format!("Subtract packed signed {i}, clamping to the signed range."),
                B2::SubSatU => format!("Subtract packed unsigned {i}, clamping at zero."),
                B2::Avg => format!("Average packed unsigned {i}, rounding up."),
                B2::MulHiS => format!("Multiply packed signed {i}, keeping the high half of each product."),
                B2::MulHiU => format!("Multiply packed unsigned {i}, keeping the high half of each product."),
                B2::MulHrs => "Multiply packed signed words as fixed point, rounding and keeping bits 15-30 of each product.".into(),
                B2::Sign => format!("Negate, zero or keep each of the packed {i} by the sign of the second source."),
                B2::Shlv => format!("Shift each of the packed {i} left by its own count."),
                B2::Shrv => format!("Shift each of the packed {i} right (zero-filling) by its own count."),
                B2::Sarv => format!("Shift each of the packed {i} right (sign-filling) by its own count."),
                B2::Rolv => format!("Rotate each of the packed {i} left by its own count."),
                B2::Rorv => format!("Rotate each of the packed {i} right by its own count."),
            }
        }
        SOp::RotImm(left, ty) => format!(
            "Rotate packed {} {} by an immediate.",
            lanes(ty),
            if left { "left" } else { "right" }
        ),
        SOp::Unary(u, ty) => match u {
            U1::Popcnt => format!("Count the set bits in each of the packed {}.", lanes(ty)),
            U1::Lzcnt => format!("Count the leading zero bits in each of the packed {}.", lanes(ty)),
            U1::Conflict => format!("For each of the packed {}, a bit mask of the earlier lanes holding the same value.", lanes(ty)),
            U1::Rcp => format!("Reciprocal of {} (exact here; hardware approximates).", float_lanes(ty, false)),
            U1::Rsqrt => format!("Reciprocal square root of {} (exact here; hardware approximates).", float_lanes(ty, false)),
        },
        SOp::Round(ty, scalar) => format!(
            "Round {} to an integral value with the mode in the immediate.",
            float_lanes(ty, scalar)
        ),
        SOp::Fma(kind, order, ty, scalar) => {
            let (x, y, z) = match order {
                132 => ("dst", "b", "a"),
                213 => ("a", "dst", "b"),
                _ => ("a", "b", "dst"),
            };
            let what = match kind {
                FmaKind::Madd => format!("dst = {x}*{y} + {z}"),
                FmaKind::Msub => format!("dst = {x}*{y} - {z}"),
                FmaKind::Nmadd => format!("dst = -({x}*{y}) + {z}"),
                FmaKind::Nmsub => format!("dst = -({x}*{y}) - {z}"),
                FmaKind::MaddSub => format!("dst = {x}*{y} - {z} in even lanes, + {z} in odd lanes"),
                FmaKind::MsubAdd => format!("dst = {x}*{y} + {z} in even lanes, - {z} in odd lanes"),
            };
            format!("Fused multiply-add with one rounding on {}: {what}.", float_lanes(ty, scalar))
        }
        SOp::Horizontal(sub, sat, ty) => format!(
            "{} adjacent pairs of {} from both sources{}.",
            if sub { "Subtract" } else { "Add" },
            lanes(ty),
            if sat { " with signed saturation" } else { "" }
        ),
        SOp::AddSub(ty) => format!(
            "Subtract in even lanes and add in odd lanes of {}.",
            float_lanes(ty, false)
        ),
        SOp::Pmaddwd => "Multiply signed words and add adjacent products into dwords.".into(),
        SOp::Pmaddubsw => "Multiply unsigned bytes by signed bytes and add adjacent products into saturated words.".into(),
        SOp::Mpsadbw => "Sums of absolute differences of 4-byte groups of the first source against 4 bytes of the second, chosen by the immediate, into 8 words per 128 bits.".into(),
        SOp::Madd52(high) => format!(
            "Multiply the low 52 bits of each qword and add the {} 52 bits of the product to the accumulator.",
            if high { "high" } else { "low" }
        ),
        SOp::Psadbw => "Sum of absolute byte differences over each group of 8 bytes, into a qword.".into(),
        SOp::MulEven(signed) => format!(
            "Multiply the even {} dwords into full 64-bit products.",
            if signed { "signed" } else { "unsigned" }
        ),
        SOp::DotAcc(Ty::I8) => "Dot product of groups of 4 unsigned by signed bytes, added into the dword accumulator.".into(),
        SOp::DotAcc(_) => "Dot product of pairs of signed words, added into the dword accumulator.".into(),
        SOp::Phminposuw => "Find the smallest unsigned word and its index, into the low dword.".into(),
        SOp::Unpack(high, ty) => format!(
            "Interleave the {} {} of each 128-bit block of the two sources.",
            if high { "upper" } else { "lower" },
            if name.starts_with('u') { lanes(if ty == Ty::I32 { Ty::F32 } else { Ty::F64 }) } else { lanes(ty) }
        ),
        SOp::ShufHalf(high) => format!(
            "Rearrange the four {} words of each 128-bit block by the immediate; the rest is copied.",
            if high { "upper" } else { "lower" }
        ),
        SOp::Shufpd => "Pick one double per 128-bit block from each source, as chosen by the immediate.".into(),
        SOp::Palignr => "Concatenate each pair of 128-bit blocks and extract 16 bytes at the immediate byte offset.".into(),
        SOp::Valign(ty) => format!(
            "Concatenate the sources and extract a vector shifted right by the immediate in {}.",
            lanes(ty)
        ),
        SOp::ByteShift(left) => format!(
            "Shift each 128-bit block {} by the immediate number of bytes.",
            if left { "left" } else { "right" }
        ),
        SOp::BlendImm(ty) => format!("Choose each of the {} from either source by the immediate's bits.", lanes(ty)),
        SOp::BlendVar(ty) => format!("Choose each of the {} from either source by the sign bit of the mask vector.", lanes(ty)),
        SOp::BlendMask(ty) => format!("Choose each of the {} from either source by the `&k` mask.", lanes(ty)),
        SOp::Pshufb => "Shuffle bytes within each 128-bit block by the indexes in the second source; a set top bit zeroes.".into(),
        SOp::Perm(Ty::I64) => "Permute qwords across the whole vector by an index vector, or within each 256-bit half by the immediate.".into(),
        SOp::Perm(ty) => format!("Permute {} across the whole vector by an index vector.", lanes(ty)),
        SOp::PermTwo(ty, overwrite_index) => format!(
            "Permute {} from a table of two vectors, overwriting the {}.",
            lanes(ty),
            if overwrite_index { "index operand" } else { "first table operand" }
        ),
        SOp::PermilImm(ty) => format!(
            "Permute {} within each 128-bit block by an immediate or a control vector.",
            lanes(if ty == Ty::I32 { Ty::F32 } else { Ty::F64 })
        ),
        SOp::Perm2x128 => "Build each 128-bit half from any half of either source, or zero, by the immediate.".into(),
        SOp::Shuf128(_) => "Pick 128-bit blocks from both sources by the immediate.".into(),
        SOp::Insert(bytes, _) => format!(
            "Copy the first source and replace the {}-bit block selected by the immediate with the second.",
            bytes * 8
        ),
        SOp::Extract(bytes, _) => format!("Extract the {}-bit block selected by the immediate.", bytes * 8),
        SOp::Pinsr(ty) => format!("Insert one of the {} from a register or memory at the lane in the immediate.", lanes(ty)),
        SOp::Pextr(ty) => format!("Extract one of the {} at the lane in the immediate.", lanes(ty)),
        SOp::Insertps => "Insert a float from the source at a lane, and zero lanes, as the immediate says.".into(),
        SOp::Extractps => "Extract the float at the lane in the immediate as 32 bits.".into(),
        SOp::MovHalf(high) => format!(
            "Load or store the {} 64 bits of a vector, keeping the rest.",
            if high { "upper" } else { "lower" }
        ),
        SOp::Movhlps => "Move the upper 64 bits of the source to the lower 64 bits of the destination.".into(),
        SOp::Movlhps => "Move the lower 64 bits of the source to the upper 64 bits of the destination.".into(),
        SOp::Dup(kind) => match kind {
            DupKind::Low64 => "Duplicate the even doubles.".into(),
            DupKind::OddF32 => "Duplicate the odd single-precision floats.".into(),
            DupKind::EvenF32 => "Duplicate the even single-precision floats.".into(),
        },
        SOp::Extend(from, to, signed) => format!(
            "{} packed {} to {}.",
            if signed { "Sign-extend" } else { "Zero-extend" },
            lanes(from),
            lanes(to)
        ),
        SOp::Narrow(from, to, sat) => format!(
            "Narrow packed {} to {}{}.",
            lanes(from),
            lanes(to),
            match sat {
                Sat::Wrap => ", truncating",
                Sat::Signed => " with signed saturation",
                Sat::Unsigned => " with unsigned saturation",
            }
        ),
        SOp::Pack(from, unsigned) => format!(
            "Narrow the {} of both sources to half width with {} saturation.",
            lanes(from),
            if unsigned { "unsigned" } else { "signed" }
        ),
        SOp::Cvt(conv) => match conv {
            Conv::IntToFloat(from, to, unsigned) => format!(
                "Convert packed {} {} to {}.",
                if unsigned { "unsigned" } else { "signed" },
                lanes(from),
                lanes(to)
            ),
            Conv::FloatToInt(from, to, trunc, unsigned) => format!(
                "Convert packed {} to {} {}, {}.",
                lanes(from),
                if unsigned { "unsigned" } else { "signed" },
                lanes(to),
                if trunc { "truncating" } else { "rounding to nearest even" }
            ),
            Conv::FloatToFloat(from, to) | Conv::ScalarFloat(from, to) => format!(
                "Convert {} to {}.",
                lanes(from),
                lanes(to)
            ),
            Conv::GprToScalar(to, unsigned) => format!(
                "Convert a {} integer to the low {} lane.",
                if unsigned { "unsigned" } else { "signed" },
                lanes(to).trim_end_matches('s')
            ),
            Conv::ScalarToGpr(from, trunc, unsigned) => format!(
                "Convert the low {} lane to a {} integer, {}.",
                lanes(from).trim_end_matches('s'),
                if unsigned { "unsigned" } else { "signed" },
                if trunc { "truncating" } else { "rounding to nearest even" }
            ),
        },
        SOp::FCmp(ty, scalar) => format!(
            "Compare {} with the predicate in the immediate: all ones where true (a mask bit per lane into an omr).",
            float_lanes(ty, scalar)
        ),
        SOp::IntCmp(ty, unsigned, _) => format!(
            "Compare packed {} {} into a mask register, with the predicate in the immediate or the name.",
            if unsigned { "unsigned" } else { "signed" },
            lanes(ty)
        ),
        SOp::Comis(ty) => format!(
            "Compare the low {} lanes and set ZF, PF and CF.",
            lanes(ty).trim_end_matches('s')
        ),
        SOp::Ptest => "Set ZF if a AND b is zero and CF if b AND NOT a is zero.".into(),
        SOp::Pcmpstr(explicit, mask) => format!(
            "Compare two strings of bytes or words ({} lengths) as the control byte says and \
             write {}; CF, ZF, SF and OF describe the result.",
            if explicit { "explicit" } else { "zero-terminated" },
            if mask { "a mask into xmm0" } else { "an index into ecx" }
        ),
        SOp::Testm(ty, not) => format!(
            "Set a mask bit for each of the {} where a AND b is {}.",
            lanes(ty),
            if not { "zero" } else { "non-zero" }
        ),
        SOp::MaskToVec(ty) => format!("Expand each mask bit to all ones or zeros in the {}.", lanes(ty)),
        SOp::VecToMask(ty) => format!("Collect the sign bit of each of the {} into a mask register.", lanes(ty)),
        SOp::BroadcastMask(ty) => format!("Repeat the low bits of a mask register into every one of the {}.", lanes(ty)),
        SOp::Compress(ty) => format!("Pack the {} selected by the `&k` mask contiguously into the destination.", lanes(ty)),
        SOp::Expand(ty) => format!("Spread contiguous {} from the source to the lanes selected by the `&k` mask.", lanes(ty)),
        SOp::Ternlog(_) => "Any three-input bitwise function, given as an 8-bit truth table in the immediate.".into(),
        SOp::Dp(ty) => format!("Dot product of {} with lanes selected by the immediate.", float_lanes(ty, false)),
        SOp::Pclmul(_) => "Carry-less multiply of two qwords (selected by the immediate or the name) into 128 bits.".into(),
        SOp::Aes(aes) => match aes {
            AesOp::Enc => "One AES encryption round on each 128-bit block.".into(),
            AesOp::EncLast => "The last AES encryption round (no MixColumns).".into(),
            AesOp::Dec => "One AES decryption round on each 128-bit block.".into(),
            AesOp::DecLast => "The last AES decryption round.".into(),
            AesOp::Imc => "Apply inverse MixColumns to a round key (for decryption).".into(),
            AesOp::KeygenAssist => "Help expand an AES key: S-box and rotate words, XOR the round constant in the immediate.".into(),
        },
        SOp::Half(false) => "Convert packed half-precision floats to single precision.".into(),
        SOp::Half(true) => "Convert packed single-precision floats to half precision with the immediate's rounding mode.".into(),
        SOp::Sha(sha) => match sha {
            ShaOp::Sha1Rnds4 => "Four SHA-1 rounds, with the round function chosen by the immediate.",
            ShaOp::Sha1Nexte => "Compute the SHA-1 state variable E after four rounds.",
            ShaOp::Sha1Msg1 => "First step of the SHA-1 message schedule.",
            ShaOp::Sha1Msg2 => "Final step of the SHA-1 message schedule.",
            ShaOp::Sha256Rnds2 => "Two SHA-256 rounds; the message plus constants operand is the implicit xmm0.",
            ShaOp::Sha256Msg1 => "First step of the SHA-256 message schedule.",
            ShaOp::Sha256Msg2 => "Final step of the SHA-256 message schedule.",
        }
        .into(),
        SOp::Gf(GfOp::Mul) => "Multiply bytes in GF(2^8) modulo the AES polynomial.".into(),
        SOp::Gf(GfOp::Affine(false)) => "Affine transform of each byte by an 8x8 bit matrix, XOR the immediate.".into(),
        SOp::Gf(GfOp::Affine(true)) => "Affine transform of each byte's GF(2^8) inverse, XOR the immediate.".into(),
        SOp::Scatter(index, elem) => format!(
            "Store {elem}-byte elements to base + index*scale for each {index}-byte index selected by the `&k` mask."
        ),
        SOp::MaskMov(ty) => format!(
            "Load or store the {} whose mask lane has its sign bit set; other lanes load as zero.",
            lanes(ty)
        ),
    }
}
