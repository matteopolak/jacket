//! The C library ABI of a POSIX target: the numbers and records its `open`, `stat`, `readdir`
//! and `errno` use, which differ between Linux and macOS (and, for some Linux flags and system
//! calls, between CPUs).
//!
//! The sandbox's libc answers a program in its target's ABI, and `native` answers compile-time
//! code built for another POSIX OS than the host's over the host's files in the same ABI. Both
//! work with neutral values in between: an [`Open`] for open flags, Linux numbers for errors.
//! The tables are the stdlib's generated POSIX declarations' values; a test compiles the
//! declarations for every target and checks each one.

#[cfg(unix)]
pub(super) mod native;
pub(super) mod records;

use crate::target::TargetCpu;
#[cfg(unix)]
use crate::target::TargetOs;

/// Whose C library a POSIX program calls: macOS's, or Linux's on one CPU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PosixAbi {
    darwin: bool,
    /// Linux's open flags, system call numbers and `struct stat` differ by CPU. `Wasm` and the
    /// sandbox's default are Linux x86-64's, which the stdlib's wasm records share.
    cpu: TargetCpu,
}

/// The two C libraries a call passes between.
#[cfg(unix)]
#[derive(Clone, Copy)]
pub(super) struct Libraries {
    pub target: PosixAbi,
    pub host: PosixAbi,
}

#[cfg(unix)]
impl Libraries {
    /// The program's number for the host's error `code`.
    pub(super) fn to_target(self, code: u64) -> u64 {
        self.target.errno(self.host.linux_errno(code))
    }

    /// The host's number for the program's error `code`.
    pub(super) fn to_host(self, code: u64) -> u64 {
        self.host.errno(self.target.linux_errno(code))
    }
}

/// What a program asked `open` for, whichever OS numbers the flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Open {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub exclusive: bool,
    pub truncate: bool,
    pub append: bool,
    pub nonblock: bool,
    pub directory: bool,
    pub no_follow: bool,
    pub close_on_exec: bool,
    pub no_tty: bool,
    pub sync: bool,
}

/// One OS's (and CPU's) numbers for the open flags `Open` names.
struct OpenBits {
    create: u64,
    exclusive: u64,
    no_tty: u64,
    truncate: u64,
    append: u64,
    nonblock: u64,
    sync: u64,
    directory: u64,
    no_follow: u64,
    close_on_exec: u64,
    /// Set by a 32-bit `open64`; means nothing to a 64-bit host.
    large_file: u64,
}

const O_WRONLY: u64 = 1;
const O_RDWR: u64 = 2;
const O_ACCMODE: u64 = 3;

const DARWIN_OPEN: OpenBits = OpenBits {
    create: 0x200,
    exclusive: 0x800,
    no_tty: 0x20000,
    truncate: 0x400,
    append: 0x8,
    nonblock: 0x4,
    sync: 0x80,
    directory: 0x0010_0000,
    no_follow: 0x100,
    close_on_exec: 0x0100_0000,
    large_file: 0,
};

/// Linux's flags, with the CPU's own `O_DIRECTORY`, `O_NOFOLLOW` and `O_LARGEFILE`.
const fn linux_open(directory: u64, no_follow: u64, large_file: u64) -> OpenBits {
    OpenBits {
        create: 0x40,
        exclusive: 0x80,
        no_tty: 0x100,
        truncate: 0x200,
        append: 0x400,
        nonblock: 0x800,
        sync: 0x0010_1000,
        directory,
        no_follow,
        close_on_exec: 0x80000,
        large_file,
    }
}

/// A stat-family call a Linux program makes through `syscall(number, ...)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StatSyscall {
    /// `stat(path, buf)`.
    Stat,
    /// `lstat(path, buf)`.
    Lstat,
    /// `fstat(fd, buf)`.
    Fstat,
    /// `fstatat(dirfd, path, buf, flags)`.
    Fstatat,
}

/// Error names with their Linux and macOS numbers: every error both declare.
const ERRORS: &[(&str, u64, u64)] = &[
    ("EPERM", 1, 1),
    ("ENOENT", 2, 2),
    ("ESRCH", 3, 3),
    ("EINTR", 4, 4),
    ("EIO", 5, 5),
    ("ENXIO", 6, 6),
    ("E2BIG", 7, 7),
    ("ENOEXEC", 8, 8),
    ("EBADF", 9, 9),
    ("ECHILD", 10, 10),
    ("EAGAIN", 11, 35),
    ("ENOMEM", 12, 12),
    ("EACCES", 13, 13),
    ("EFAULT", 14, 14),
    ("ENOTBLK", 15, 15),
    ("EBUSY", 16, 16),
    ("EEXIST", 17, 17),
    ("EXDEV", 18, 18),
    ("ENODEV", 19, 19),
    ("ENOTDIR", 20, 20),
    ("EISDIR", 21, 21),
    ("EINVAL", 22, 22),
    ("ENFILE", 23, 23),
    ("EMFILE", 24, 24),
    ("ENOTTY", 25, 25),
    ("ETXTBSY", 26, 26),
    ("EFBIG", 27, 27),
    ("ENOSPC", 28, 28),
    ("ESPIPE", 29, 29),
    ("EROFS", 30, 30),
    ("EMLINK", 31, 31),
    ("EPIPE", 32, 32),
    ("EDOM", 33, 33),
    ("ERANGE", 34, 34),
    ("EDEADLK", 35, 11),
    ("ENAMETOOLONG", 36, 63),
    ("ENOLCK", 37, 77),
    ("ENOSYS", 38, 78),
    ("ENOTEMPTY", 39, 66),
    ("ELOOP", 40, 62),
    ("ENOMSG", 42, 91),
    ("EIDRM", 43, 90),
    ("ENOSTR", 60, 99),
    ("ENODATA", 61, 96),
    ("ETIME", 62, 101),
    ("ENOSR", 63, 98),
    ("EREMOTE", 66, 71),
    ("ENOLINK", 67, 97),
    ("EPROTO", 71, 100),
    ("EMULTIHOP", 72, 95),
    ("EBADMSG", 74, 94),
    ("EOVERFLOW", 75, 84),
    ("EILSEQ", 84, 92),
    ("EUSERS", 87, 68),
    ("ENOTSOCK", 88, 38),
    ("EDESTADDRREQ", 89, 39),
    ("EMSGSIZE", 90, 40),
    ("EPROTOTYPE", 91, 41),
    ("ENOPROTOOPT", 92, 42),
    ("EPROTONOSUPPORT", 93, 43),
    ("ESOCKTNOSUPPORT", 94, 44),
    ("EOPNOTSUPP", 95, 102),
    ("EPFNOSUPPORT", 96, 46),
    ("EAFNOSUPPORT", 97, 47),
    ("EADDRINUSE", 98, 48),
    ("EADDRNOTAVAIL", 99, 49),
    ("ENETDOWN", 100, 50),
    ("ENETUNREACH", 101, 51),
    ("ENETRESET", 102, 52),
    ("ECONNABORTED", 103, 53),
    ("ECONNRESET", 104, 54),
    ("ENOBUFS", 105, 55),
    ("EISCONN", 106, 56),
    ("ENOTCONN", 107, 57),
    ("ESHUTDOWN", 108, 58),
    ("ETOOMANYREFS", 109, 59),
    ("ETIMEDOUT", 110, 60),
    ("ECONNREFUSED", 111, 61),
    ("EHOSTDOWN", 112, 64),
    ("EHOSTUNREACH", 113, 65),
    ("EALREADY", 114, 37),
    ("EINPROGRESS", 115, 36),
    ("ESTALE", 116, 70),
    ("EDQUOT", 122, 69),
    ("ECANCELED", 125, 89),
    ("EOWNERDEAD", 130, 105),
    ("ENOTRECOVERABLE", 131, 104),
];

/// What an error only one of the two declares becomes on the other.
const EIO: u64 = 5;

impl PosixAbi {
    /// The ABI of a program built for `os` on `cpu`; `None` when `os` is not a POSIX OS whose
    /// C library is answered here.
    #[cfg(unix)]
    pub(super) fn of(os: TargetOs, cpu: TargetCpu) -> Option<PosixAbi> {
        match os {
            TargetOs::MacOS => Some(PosixAbi {
                darwin: true,
                cpu,
            }),
            TargetOs::Linux => Some(PosixAbi {
                darwin: false,
                cpu,
            }),
            _ => None,
        }
    }

    /// macOS's ABI when `darwin`, else Linux's on `cpu` (x86-64's when it has none, as the
    /// sandbox's wasm records are).
    pub(super) fn new(darwin: bool, cpu: Option<TargetCpu>) -> PosixAbi {
        PosixAbi {
            darwin,
            cpu: cpu.unwrap_or(TargetCpu::X64),
        }
    }

    /// The ABI of the C library the compiler itself runs on, if it is a POSIX one.
    #[cfg(unix)]
    pub(super) fn host() -> Option<PosixAbi> {
        let (os, cpu) = crate::sema::compiler_host();
        PosixAbi::of(os, cpu)
    }

    pub(super) fn darwin(self) -> bool {
        self.darwin
    }

    pub(super) fn cpu(self) -> TargetCpu {
        self.cpu
    }

    fn open_bits(self) -> OpenBits {
        if self.darwin {
            return DARWIN_OPEN;
        }

        match self.cpu {
            TargetCpu::X86 => linux_open(0x10000, 0x20000, 0x8000),
            TargetCpu::Arm => linux_open(0x4000, 0x8000, 0x20000),
            TargetCpu::Arm64 | TargetCpu::PowerPc64 => linux_open(0x4000, 0x8000, 0),
            TargetCpu::X64 | TargetCpu::S390x | TargetCpu::Wasm | TargetCpu::Avr => {
                linux_open(0x10000, 0x20000, 0)
            }
        }
    }

    /// What the open flags `flags` ask for, and the bits of them this ABI declares no flag
    /// `Open` names for.
    pub(super) fn decode_open(self, flags: u64) -> (Open, u64) {
        let bits = self.open_bits();
        let mode = flags & O_ACCMODE;
        let mut rest = flags & !O_ACCMODE & !bits.large_file;
        let mut has = |bit: u64| {
            rest &= !bit;
            bit != 0 && flags & bit == bit
        };
        let open = Open {
            read: mode != O_WRONLY,
            write: mode == O_WRONLY || mode == O_RDWR,
            create: has(bits.create),
            exclusive: has(bits.exclusive),
            truncate: has(bits.truncate),
            append: has(bits.append),
            nonblock: has(bits.nonblock),
            directory: has(bits.directory),
            no_follow: has(bits.no_follow),
            close_on_exec: has(bits.close_on_exec),
            no_tty: has(bits.no_tty),
            sync: has(bits.sync),
        };

        (open, rest)
    }

    /// This ABI's open flags for `open`.
    #[cfg(any(unix, test))]
    pub(super) fn encode_open(self, open: Open) -> u64 {
        let bits = self.open_bits();
        let mode = match (open.read, open.write) {
            (true, true) => O_RDWR,
            (false, true) => O_WRONLY,
            _ => 0,
        };
        let flag = |on: bool, bit: u64| {
            if on {
                bit
            } else {
                0
            }
        };

        mode | flag(open.create, bits.create)
            | flag(open.exclusive, bits.exclusive)
            | flag(open.truncate, bits.truncate)
            | flag(open.append, bits.append)
            | flag(open.nonblock, bits.nonblock)
            | flag(open.directory, bits.directory)
            | flag(open.no_follow, bits.no_follow)
            | flag(open.close_on_exec, bits.close_on_exec)
            | flag(open.no_tty, bits.no_tty)
            | flag(open.sync, bits.sync)
    }

    /// This ABI's number for the error Linux numbers `linux`.
    pub(super) fn errno(self, linux: u64) -> u64 {
        if !self.darwin {
            return linux;
        }

        ERRORS
            .iter()
            .find(|&&(_, number, _)| number == linux)
            .map_or(EIO, |&(_, _, darwin)| darwin)
    }

    /// The Linux number of this ABI's error `code`.
    #[cfg(any(unix, test))]
    pub(super) fn linux_errno(self, code: u64) -> u64 {
        if !self.darwin {
            return code;
        }

        ERRORS
            .iter()
            .find(|&&(_, _, darwin)| darwin == code)
            .map_or(EIO, |&(_, linux, _)| linux)
    }

    /// `AT_FDCWD`: the directory argument of an `*at` call that means the working directory.
    pub(super) fn at_working_directory(self) -> i64 {
        if self.darwin {
            -2
        } else {
            -100
        }
    }

    /// `AT_SYMLINK_NOFOLLOW`.
    pub(super) fn at_no_follow(self) -> u64 {
        if self.darwin {
            0x20
        } else {
            0x100
        }
    }

    /// The stat-family call a Linux program's `syscall(number, ...)` makes, if it is one. The
    /// stdlib reaches `struct stat` this way on x86-64 and arm64 (`POSIX/wrappers.jai`).
    pub(super) fn stat_syscall(self, number: u64) -> Option<StatSyscall> {
        if self.darwin {
            return None;
        }

        match (self.cpu, number) {
            (TargetCpu::X64, 4) => Some(StatSyscall::Stat),
            (TargetCpu::X64, 5) => Some(StatSyscall::Fstat),
            (TargetCpu::X64, 6) => Some(StatSyscall::Lstat),
            (TargetCpu::X64, 262) | (TargetCpu::Arm64, 79) => Some(StatSyscall::Fstatat),
            (TargetCpu::Arm64, 80) => Some(StatSyscall::Fstat),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
