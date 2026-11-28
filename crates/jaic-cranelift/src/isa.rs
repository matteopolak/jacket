//! The Cranelift target for a build: triple, calling convention and code generation flags.
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::settings::{self, Configurable};
use jaic::abi::Arch;
use jaic::target::{Target, TargetCpu, TargetOs};
use jaic_native::Options;

use super::{R, unsupported};

/// The calling-convention family of `target`, which must be one this backend compiles for:
/// x86-64 or `AArch64`, on Linux or macOS.
pub(crate) fn arch(target: Target) -> R<Arch> {
    match (target.cpu, target.os) {
        (TargetCpu::X64 | TargetCpu::Arm64, TargetOs::Linux | TargetOs::MacOS) => target
            .arch()
            .ok_or_else(|| unsupported(format!("the target `{target}`"))),
        _ => Err(unsupported(format!(
            "the target `{target}` (only x86-64 and arm64 Linux and macOS)"
        ))),
    }
}

/// The triple Cranelift is given. On macOS it names the deployment target, which the object's
/// build version records, as LLVM's does (`jaic_native::macos_deployment_target`).
fn triple(target: Target, options: &Options) -> R<String> {
    let arch = match target.cpu {
        TargetCpu::X64 => "x86_64",
        TargetCpu::Arm64 => "aarch64",
        _ => return Err(unsupported(format!("the target `{target}`"))),
    };

    Ok(match target.os {
        TargetOs::MacOS => {
            let version =
                jaic_native::macos_deployment_target(options.codegen.macos_version.as_deref());
            format!("{arch}-apple-macosx{version}")
        }
        TargetOs::Linux => format!("{arch}-unknown-linux-gnu"),
        _ => return Err(unsupported(format!("the target `{target}`"))),
    })
}

/// The target ISA with the flags of an unoptimized build: no optimization, the single-pass
/// register allocator (half the compile time of the default one), position-independent code, frame records in every function (Apple's ABI needs them, and stack walks follow them),
/// and several scalar results returned however many registers they need.
pub(crate) fn target_isa(target: Target, options: &Options) -> R<OwnedTargetIsa> {
    arch(target)?;
    let triple = triple(target, options)?;
    let mut flags = settings::builder();

    for (name, value) in [
        ("opt_level", "none"),
        ("regalloc_algorithm", "single_pass"),
        ("is_pic", "true"),
        ("preserve_frame_pointers", "true"),
        (
            "enable_verifier",
            if cfg!(debug_assertions) {
                "true"
            } else {
                "false"
            },
        ),
        ("enable_multi_ret_implicit_sret", "true"),
        ("unwind_info", "false"),
        ("enable_probestack", "false"),
        // x86-64 C-variadic calls pass their callee in it (`Helper::VariadicCall`).
        (
            "enable_pinned_reg",
            if target.cpu == TargetCpu::X64 {
                "true"
            } else {
                "false"
            },
        ),
    ] {
        flags
            .set(name, value)
            .map_err(|e| format!("Cranelift setting `{name}`: {e}"))?;
    }

    let builder = cranelift_codegen::isa::lookup_by_name(&triple)
        .map_err(|e| format!("Cranelift has no backend for `{triple}`: {e}"))?;

    builder
        .finish(settings::Flags::new(flags))
        .map_err(|e| format!("Cranelift target `{triple}`: {e}").into())
}
