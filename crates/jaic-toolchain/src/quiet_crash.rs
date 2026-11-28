//! Running a program that is expected to crash without the OS writing a crash report for it.
//!
//! On macOS every crash makes `ReportCrash` symbolicate and save a report, seconds of CPU per
//! crash; tests that crash on purpose (failed checks, sanitizer reports) add up. Crash reporting is
//! per-task there: the kernel sends a crashing task's exception to the task's exception ports, and
//! `ReportCrash` is only the default receiver. [`quiet`] clears those ports for the child alone, so
//! the crash is still a signal in the child's exit status and nothing else on the machine changes.
//! Other systems need nothing, and [`quiet`] leaves the command as it is there.
use std::process::Command;

/// Makes `command`'s child crash without a crash report (macOS; elsewhere it changes nothing).
///
/// ```
/// let mut command = std::process::Command::new("true");
/// jaic_toolchain::quiet_crash::quiet(&mut command);
/// assert!(command.status().unwrap().success());
/// ```
pub fn quiet(command: &mut Command) -> &mut Command {
    imp::quiet(command)
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod imp {
    use std::io;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    /// `EXC_MASK_ALL`, `EXC_MASK_CRASH`, `EXC_MASK_RESOURCE`, `EXC_MASK_GUARD` and
    /// `EXC_MASK_CORPSE_NOTIFY` (`<mach/exception_types.h>`): the exceptions that end in a report.
    const CRASH_EXCEPTIONS: u32 = 0x7e | 0x400 | 0x800 | 0x1000 | 0x2000;

    /// `EXCEPTION_DEFAULT | MACH_EXCEPTION_CODES`.
    const BEHAVIOR: i32 = 1 | i32::MIN;

    /// `MACH_PORT_NULL`: no handler.
    const NO_PORT: u32 = 0;

    /// With no port the flavor is unused, and a nonzero one is rejected on arm64.
    const NO_FLAVOR: i32 = 0;

    unsafe extern "C" {
        fn mach_task_self() -> u32;

        fn task_set_exception_ports(
            task: u32,
            mask: u32,
            port: u32,
            behavior: i32,
            flavor: i32,
        ) -> i32;
    }

    pub fn quiet(command: &mut Command) -> &mut Command {
        // SAFETY: runs in the forked child before `exec`, and only makes one Mach call that
        // allocates nothing. Exception ports survive `exec`.
        unsafe {
            command.pre_exec(|| {
                let status = task_set_exception_ports(
                    mach_task_self(),
                    CRASH_EXCEPTIONS,
                    NO_PORT,
                    BEHAVIOR,
                    NO_FLAVOR,
                );

                if status == 0 {
                    Ok(())
                } else {
                    // `kern_return_t`, not an errno; the spawn error names the call.
                    Err(io::Error::other(format!(
                        "task_set_exception_ports failed: kern_return_t {status}"
                    )))
                }
            })
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::process::Command;

    pub fn quiet(command: &mut Command) -> &mut Command {
        command
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::process::ExitStatusExt;
    use std::process::Command;

    use super::quiet;

    #[test]
    fn a_crashing_child_still_dies_of_its_signal() {
        let status = quiet(Command::new("sh").args(["-c", "kill -SEGV $$"]))
            .status()
            .unwrap();

        assert_eq!(status.signal(), Some(11));
    }

    #[test]
    fn a_normal_child_runs_and_exits_as_usual() {
        let output = quiet(Command::new("sh").args(["-c", "echo hi; exit 3"]))
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(3));
        assert_eq!(output.stdout, b"hi\n");
    }
}
