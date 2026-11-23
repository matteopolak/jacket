//! The call plan of a foreign signature: its C ABI classification, made once.
#![cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use crate::abi::{self, Arch, Passing, Piece, PieceTy};
use crate::ir::Sig;
use crate::layout::DataLayout;

use super::SRET_WORDS;

/// How a foreign procedure's signature maps onto the host C ABI, worked out once (`Plan::new`)
/// and reused by every call: what is left per call is marshalling the arguments and the call.
pub struct Plan {
    pub(super) arch: Option<Arch>,
    pub(super) host: DataLayout,
    pub(super) ret_pieces: Option<Vec<Piece>>,
    /// The result is an aggregate with a hidden result pointer.
    pub(super) sret: bool,
    /// A result too big for `Sret`: the pointer is passed explicitly.
    pub(super) oversize: bool,
    /// Windows on arm64: the callee takes the result's address in `x0`.
    pub(super) result_in_x0: bool,
    /// Windows on arm64: a variadic procedure takes every argument in general registers.
    pub(super) general_only: bool,
    /// The result needs the assembly call of `wide.rs`.
    pub(super) wide_ret: bool,
    /// Every argument and the result is a scalar, so a call needs only registers (`scalar_call`).
    pub(super) scalar: bool,
    pub(super) params: Vec<Option<ArgPlan>>,
    /// Whether the callee must run on the main thread (macOS), once known.
    #[cfg(target_os = "macos")]
    pub(super) main_thread: std::sync::OnceLock<bool>,
}

/// How one aggregate argument is passed.
pub(super) struct ArgPlan {
    pub(super) regular: Passing,
    /// The classification for a variadic call that takes everything in general registers.
    pub(super) vararg: Option<Passing>,
}

impl Plan {
    pub fn new(sig: &Sig) -> Plan {
        let arch = Arch::host();
        let host = DataLayout::host();
        let cabi = sig.c_abi.as_deref();
        let ret_layout = cabi.and_then(|c| c.ret.as_ref());
        let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
        let (mut ret_pieces, mut sret, mut oversize, mut result_in_x0) =
            (None, false, false, false);
        let mut general_only = false;
        let mut params = Vec::new();
        if let Some(arch) = arch {
            ret_pieces = ret_layout
                .filter(|_| !forced_sret)
                .and_then(|l| abi::classify_ret(arch, l, &host));
            sret = ret_layout.is_some() && ret_pieces.is_none();
            oversize = sret
                && !(arch == Arch::Win64Arm && forced_sret)
                && ret_layout.is_some_and(|l| l.size as usize > SRET_WORDS * 8);
            general_only = arch == Arch::Win64Arm && sig.c_varargs;
            result_in_x0 = arch == Arch::Win64Arm && forced_sret && ret_layout.is_some();
            params = cabi
                .map(|c| {
                    c.params
                        .iter()
                        .map(|l| {
                            l.as_ref().map(|l| ArgPlan {
                                regular: abi::classify_arg(arch, l, &host),
                                vararg: general_only.then(|| abi::classify_vararg(arch, l, &host)),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
        }
        let wide_ret = ret_pieces.as_ref().is_some_and(|p| {
            p.iter()
                .any(|p| matches!(p.ty, PieceTy::X87 | PieceTy::F128))
        });
        let scalar = matches!(arch, Some(Arch::X86_64 | Arch::Aarch64))
            && ret_layout.is_none()
            && params.iter().all(Option::is_none)
            && !(sig.c_varargs && super::VARARGS_ON_STACK);
        Plan {
            arch,
            host,
            ret_pieces,
            sret,
            oversize,
            result_in_x0,
            general_only,
            wide_ret,
            scalar,
            params,
            #[cfg(target_os = "macos")]
            main_thread: std::sync::OnceLock::new(),
        }
    }
}
