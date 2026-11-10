//! The system a file says it is written for: `// test: only(.WINDOWS)` at the top of a test.
//!
//! A program that needs another system than the one being linted for (a `#assert OS == .WINDOWS`
//! in a Windows-only test) cannot compile on the host, so it is compiled for the system its
//! `only(...)`, `skip_on(...)`, `mode =` and `interp_only`/`native_only` notes allow, the way
//! `jacket test` picks where to run it.
use jaic::target::{TargetCpu, TargetOs, cpu_name, os_name, parse_os};
use jaic::test_notes::TargetNotes;

/// Where the notes of a file say it is compiled.
#[derive(Debug, PartialEq)]
pub(super) enum Marked {
    /// The host passes the notes, or there are none.
    Host,
    /// The nearest system that passes them.
    Other(TargetOs, TargetCpu),
    /// No system the compiler can target passes them (`only(.ANDROID)`): `jacket test` runs the
    /// file nowhere either.
    Nowhere,
}

/// Where to compile a program given the notes in its header: the host's OS before others, the
/// host's CPU before others.
pub(super) fn marked_target(notes: &TargetNotes, host_os: TargetOs, host_cpu: TargetCpu) -> Marked {
    if !notes.restricts() {
        return Marked::Host;
    }
    // Linting has no run mode, so a file passes when it would run in either.
    let passes = |os: TargetOs, cpu: TargetCpu| notes.runs_on(os_name(os), cpu_name(cpu));
    if passes(host_os, host_cpu) {
        return Marked::Host;
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
            return Marked::Other(os, cpu);
        }
    }
    Marked::Nowhere
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked(header: &str) -> Marked {
        let source = format!("{header}\nmain :: () {{}}\n");
        marked_target(
            &jaic::test_notes::header_target_notes(&source).0,
            TargetOs::MacOS,
            TargetCpu::Arm64,
        )
    }

    #[test]
    fn a_host_that_passes_stays_the_target() {
        assert_eq!(marked("// test: only(.MACOS)"), Marked::Host);
        assert_eq!(marked("// test: only(.LINUX, .MACOS) serial"), Marked::Host);
        assert_eq!(marked("// test: timeout(3)"), Marked::Host);
        assert_eq!(marked("main :: () {}"), Marked::Host);
    }

    #[test]
    fn the_nearest_passing_system_is_chosen() {
        assert_eq!(
            marked("// test: only(.WINDOWS)"),
            Marked::Other(TargetOs::Windows, TargetCpu::Arm64)
        );
        assert_eq!(
            marked("// test: only(.X64)"),
            Marked::Other(TargetOs::MacOS, TargetCpu::X64)
        );
        assert_eq!(
            marked("// test: only(.LINUX, cpu = .X64)"),
            Marked::Other(TargetOs::Linux, TargetCpu::X64)
        );
        assert_eq!(
            marked("// test: only(.WASM)"),
            Marked::Other(TargetOs::Wasm, TargetCpu::Wasm)
        );
    }

    #[test]
    fn a_system_the_compiler_cannot_target_is_nowhere() {
        assert_eq!(marked("// test: only(.ANDROID)"), Marked::Nowhere);
        assert_eq!(
            marked("// test: only(.WINDOWS) only(.WASM)"),
            Marked::Nowhere
        );
    }

    #[test]
    fn skip_on_moves_the_target_off_the_skipped_system() {
        assert_eq!(
            marked("// test: skip_on(.MACOS)"),
            Marked::Other(TargetOs::Linux, TargetCpu::Arm64)
        );
        assert_eq!(marked("// test: skip_on(.WASM)"), Marked::Host);
        assert_eq!(
            marked("// test: skip_on(.ARM64)"),
            Marked::Other(TargetOs::MacOS, TargetCpu::X64)
        );
        assert_eq!(
            marked("// test: skip_on(.MACOS) skip_on(.LINUX, .WINDOWS)"),
            Marked::Other(TargetOs::Wasm, TargetCpu::Wasm)
        );
    }

    #[test]
    fn a_mode_narrows_a_note_without_moving_the_target() {
        assert_eq!(
            marked("// test: skip_on(.MACOS, mode = .NATIVE)"),
            Marked::Host
        );
        assert_eq!(
            marked("// test: only(.MACOS, mode = .NATIVE)"),
            Marked::Host
        );
        assert_eq!(
            marked("// test: skip_on(mode = .INTERP) native_only"),
            Marked::Host
        );
        assert_eq!(marked("// test: interp_only native_only"), Marked::Nowhere);
        assert_eq!(
            marked("// test: skip_on(.MACOS, mode = .NATIVE) skip_on(.MACOS, mode = .INTERP)"),
            Marked::Other(TargetOs::Linux, TargetCpu::Arm64)
        );
    }

    #[test]
    fn a_run_mode_does_not_move_the_target() {
        assert_eq!(marked("// test: only(.INTERP)"), Marked::Host);
    }
}
