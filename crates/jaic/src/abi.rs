//! C ABI classification for aggregates passed or returned by value.
//!
//! Covers the 64-bit ABIs the compiler targets: AArch64 AAPCS64 (Apple
//! flavour), x86-64 System V and the Microsoft x64 convention (Windows). The IR
//! describes an aggregate only by its flattened scalar fields (`AggLayout`),
//! which is all these ABIs look at.
use crate::ir::{AggLayout, Ty};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arch {
    Aarch64,
    /// x86-64 System V (macOS, Linux).
    X86_64,
    /// x86-64 with the Microsoft x64 calling convention (Windows, MSVC and MinGW alike).
    Win64,
}

impl Arch {
    /// The architecture this compiler runs on (for the interpreter's native calls).
    pub fn host() -> Option<Arch> {
        if cfg!(target_arch = "aarch64") {
            Some(Arch::Aarch64)
        } else if cfg!(all(target_arch = "x86_64", windows)) {
            Some(Arch::Win64)
        } else if cfg!(target_arch = "x86_64") {
            Some(Arch::X86_64)
        } else {
            None
        }
    }

    pub fn from_triple(triple: &str) -> Option<Arch> {
        let cpu = triple.split('-').next()?;
        let windows = triple.contains("windows") || triple.contains("mingw");
        match cpu {
            "aarch64" | "arm64" if !windows => Some(Arch::Aarch64),
            "x86_64" | "amd64" if windows => Some(Arch::Win64),
            "x86_64" | "amd64" => Some(Arch::X86_64),
            _ => None,
        }
    }

    /// Whether the CPU is x86-64 (either calling convention).
    pub fn is_x86_64(self) -> bool {
        matches!(self, Arch::X86_64 | Arch::Win64)
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
    /// Caller makes a copy and passes its address (AArch64 large aggregates, and every
    /// Win64 aggregate that is not 1, 2, 4 or 8 bytes).
    Indirect,
}

/// Classify a by-value aggregate argument.
pub fn classify_arg(arch: Arch, layout: &AggLayout) -> Passing {
    match classify_registers(arch, layout) {
        Some(pieces) => Passing::Registers(pieces),
        None => match arch {
            Arch::X86_64 => Passing::ByVal,
            Arch::Aarch64 | Arch::Win64 => Passing::Indirect,
        },
    }
}

/// Classify an aggregate return value; `None` means returned through a hidden
/// `sret` pointer.
pub fn classify_ret(arch: Arch, layout: &AggLayout) -> Option<Vec<Piece>> {
    classify_registers(arch, layout)
}

fn classify_registers(arch: Arch, layout: &AggLayout) -> Option<Vec<Piece>> {
    if layout.size == 0 {
        return Some(Vec::new());
    }
    // Microsoft x64: an aggregate of exactly 1, 2, 4 or 8 bytes travels as an integer of that
    // size (in a general register even when its members are floats); anything else goes by
    // reference to a caller copy, and is returned through a hidden pointer.
    if arch == Arch::Win64 {
        return matches!(layout.size, 1 | 2 | 4 | 8).then(|| int_pieces(layout.size));
    }
    if layout.size > 16 && !(arch == Arch::Aarch64 && hfa(layout).is_some()) {
        return None;
    }
    match arch {
        Arch::Aarch64 => {
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
        Arch::Win64 => unreachable!("handled above"),
        Arch::X86_64 => {
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

fn int_pieces(size: u64) -> Vec<Piece> {
    (0..size.div_ceil(8))
        .map(|k| Piece {
            offset: k * 8,
            ty: PieceTy::I64,
        })
        .collect()
}

/// If the aggregate is a homogeneous float aggregate of 1..=4 members,
/// returns the member type.
fn hfa(layout: &AggLayout) -> Option<PieceTy> {
    let first = layout.fields.first()?.1;
    if layout.fields.len() > 4 || !first.is_float() {
        return None;
    }
    if layout.fields.iter().any(|&(_, t)| t != first) {
        return None;
    }
    let stride = first.size();
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
    let ty = if first == Ty::F32 {
        PieceTy::F32
    } else {
        PieceTy::F64
    };
    Some(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(size: u64, fields: &[(u64, Ty)]) -> AggLayout {
        AggLayout {
            size,
            align: 8,
            fields: fields.to_vec(),
        }
    }

    #[test]
    fn aarch64_small_int_struct_uses_x_registers() {
        let l = layout(12, &[(0, Ty::I64), (8, Ty::I32)]);
        assert_eq!(classify_ret(Arch::Aarch64, &l).unwrap().len(), 2);
    }

    #[test]
    fn aarch64_hfa_uses_float_registers() {
        let l = layout(
            16,
            &[(0, Ty::F32), (4, Ty::F32), (8, Ty::F32), (12, Ty::F32)],
        );
        let p = classify_ret(Arch::Aarch64, &l).unwrap();
        assert!(p.iter().all(|p| p.ty == PieceTy::F32) && p.len() == 4);
    }

    #[test]
    fn aarch64_large_aggregate_is_indirect() {
        let l = layout(24, &[(0, Ty::I64), (8, Ty::I64), (16, Ty::I64)]);
        assert_eq!(classify_arg(Arch::Aarch64, &l), Passing::Indirect);
        assert!(classify_ret(Arch::Aarch64, &l).is_none());
    }

    #[test]
    fn sysv_mixed_eightbytes() {
        let l = layout(16, &[(0, Ty::I64), (8, Ty::F64)]);
        let p = classify_ret(Arch::X86_64, &l).unwrap();
        assert_eq!(p[0].ty, PieceTy::I64);
        assert_eq!(p[1].ty, PieceTy::F64);
        let v = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        assert_eq!(
            classify_ret(Arch::X86_64, &v).unwrap()[0].ty,
            PieceTy::V2F32
        );
    }

    #[test]
    fn win64_power_of_two_sizes_use_one_integer_register() {
        let floats = layout(8, &[(0, Ty::F32), (4, Ty::F32)]);
        let p = classify_ret(Arch::Win64, &floats).unwrap();
        assert_eq!(
            p,
            vec![Piece {
                offset: 0,
                ty: PieceTy::I64
            }]
        );
        let small = layout(4, &[(0, Ty::I16), (2, Ty::I16)]);
        assert!(matches!(classify_arg(Arch::Win64, &small), Passing::Registers(p) if p.len() == 1));
    }

    #[test]
    fn win64_other_sizes_go_by_reference() {
        let odd = layout(12, &[(0, Ty::I32), (4, Ty::I32), (8, Ty::I32)]);
        assert_eq!(classify_arg(Arch::Win64, &odd), Passing::Indirect);
        assert!(classify_ret(Arch::Win64, &odd).is_none());
        let pair = layout(16, &[(0, Ty::F64), (8, Ty::F64)]);
        assert_eq!(classify_arg(Arch::Win64, &pair), Passing::Indirect);
        let three = layout(3, &[(0, Ty::I8), (1, Ty::I8), (2, Ty::I8)]);
        assert_eq!(classify_arg(Arch::Win64, &three), Passing::Indirect);
    }

    #[test]
    fn windows_triples_select_win64() {
        assert_eq!(
            Arch::from_triple("x86_64-pc-windows-msvc"),
            Some(Arch::Win64)
        );
        assert_eq!(
            Arch::from_triple("x86_64-w64-windows-gnu"),
            Some(Arch::Win64)
        );
        assert_eq!(Arch::from_triple("x86_64-w64-mingw32"), Some(Arch::Win64));
        assert_eq!(
            Arch::from_triple("x86_64-unknown-linux-gnu"),
            Some(Arch::X86_64)
        );
    }

    #[test]
    fn sysv_large_aggregate_is_byval() {
        let l = layout(24, &[(0, Ty::I64), (8, Ty::I64), (16, Ty::I64)]);
        assert_eq!(classify_arg(Arch::X86_64, &l), Passing::ByVal);
    }
}
