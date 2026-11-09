//! The system a file says it is written for: `// test: only(.WINDOWS)` at the top of a test.
//!
//! A program that needs another system than the one being linted for (a `#assert OS == .WINDOWS`
//! in a Windows-only test) cannot compile on the host, so it is compiled for the system its
//! `only(...)` note names, the way `jacket test` runs it there.
use jaic::target::{TargetCpu, TargetOs, cpu_name, os_name, parse_os};
use jaic::test_notes::{Target, TargetFilter, header_notes};

/// The OS and CPU to compile the program in `source` for, when its `only(...)` notes rule out
/// `host_os` and `host_cpu`: the nearest system that passes them (the host's OS before others,
/// the host's CPU before others). `None` for both leaves the compile on the host, which is also
/// the answer when the notes name no system the compiler can target.
pub(super) fn marked_target(
    source: &str,
    host_os: TargetOs,
    host_cpu: TargetCpu,
) -> (Option<TargetOs>, Option<TargetCpu>) {
    let only: Vec<TargetFilter> = header_notes(source)
        .iter()
        .filter_map(|note| {
            let (call, args) = note.split_once('(')?;
            let args = args.strip_suffix(')')?;
            if call == "only" {
                TargetFilter::parse(call, args).ok()
            } else {
                None
            }
        })
        .collect();
    // Linting has no run mode, so a note that asks for one passes for either.
    let passes = |os: TargetOs, cpu: TargetCpu| {
        ["interp", "native"].iter().any(|mode| {
            let target = Target {
                os: os_name(os).into(),
                cpu: cpu_name(cpu).into(),
                interp: *mode == "interp",
            };
            only.iter().all(|filter| filter.allows(&target, mode))
        })
    };
    if only.is_empty() || passes(host_os, host_cpu) {
        return (None, None);
    }
    let oses = std::iter::once(host_os).chain(
        jaic::target::OS_NAMES
            .iter()
            .filter_map(|name| parse_os(name))
            .filter(|os| *os != host_os),
    );
    for os in oses {
        let cpus: &[TargetCpu] = if os == TargetOs::Wasm {
            &[TargetCpu::Wasm]
        } else if host_cpu == TargetCpu::Arm64 {
            &[TargetCpu::Arm64, TargetCpu::X64]
        } else {
            &[TargetCpu::X64, TargetCpu::Arm64]
        };
        if let Some(cpu) = cpus.iter().copied().find(|cpu| passes(os, *cpu)) {
            return (Some(os), Some(cpu));
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked(header: &str) -> (Option<TargetOs>, Option<TargetCpu>) {
        marked_target(
            &format!("{header}\nmain :: () {{}}\n"),
            TargetOs::MacOS,
            TargetCpu::Arm64,
        )
    }

    #[test]
    fn a_host_that_passes_stays_the_target() {
        assert_eq!(marked("// test: only(.MACOS)"), (None, None));
        assert_eq!(marked("// test: only(.LINUX, .MACOS) serial"), (None, None));
        assert_eq!(marked("// test: timeout(3)"), (None, None));
        assert_eq!(marked("main :: () {}"), (None, None));
    }

    #[test]
    fn the_nearest_passing_system_is_chosen() {
        assert_eq!(
            marked("// test: only(.WINDOWS)"),
            (Some(TargetOs::Windows), Some(TargetCpu::Arm64))
        );
        assert_eq!(
            marked("// test: only(.X64)"),
            (Some(TargetOs::MacOS), Some(TargetCpu::X64))
        );
        assert_eq!(
            marked("// test: only(.LINUX, cpu = .X64)"),
            (Some(TargetOs::Linux), Some(TargetCpu::X64))
        );
        assert_eq!(
            marked("// test: only(.WASM)"),
            (Some(TargetOs::Wasm), Some(TargetCpu::Wasm))
        );
    }

    #[test]
    fn a_system_the_compiler_cannot_target_leaves_the_host() {
        assert_eq!(marked("// test: only(.ANDROID)"), (None, None));
        assert_eq!(marked("// test: only(.WINDOWS) only(.WASM)"), (None, None));
    }

    #[test]
    fn a_run_mode_does_not_move_the_target() {
        assert_eq!(marked("// test: only(.INTERP)"), (None, None));
    }
}
