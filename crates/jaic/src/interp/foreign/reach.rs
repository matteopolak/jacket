//! Which procedures of a target's C library compile-time code may call on a host whose OS
//! differs, and how: natively, by another name, answered in the target's ABI, or not at all.
use super::super::native;
#[cfg(unix)]
use super::super::posix;
use crate::ir::{self, Program};

/// C library procedures whose arguments, results and effects mean the same in every C library
/// the compiler meets (Linux's, macOS's and the Windows C runtime): memory, strings, numbers
/// of types they all size alike, the environment and process exit. Their failures show only
/// through `errno`, which each library numbers its own way.
const SAME_IN_EVERY_C_LIBRARY: &[&str] = &[
    "malloc", "calloc", "realloc", "free", "memcpy", "memmove", "memset", "memcmp", "memchr",
    "strlen", "strcmp", "strncmp", "strchr", "strrchr", "strstr", "strcpy", "strncpy", "strtod",
    "strtof", "strtoll", "strtoull", "atof", "atoi", "atoll", "getenv", "exit", "_exit", "abort",
    "abs", "sqrt", "sqrtf", "sin", "sinf", "cos", "cosf", "tan", "tanf", "atan2", "atan2f", "pow",
    "powf", "floor", "ceil", "fmod", "log", "exp",
];

/// Procedures that mean the same in the Linux and macOS C libraries but not in the Windows C
/// runtime, whose `long` has 32 bits, whose streams translate line ends and which has no
/// descriptor or path calls of these names: `long` numbers, descriptor reads and writes,
/// `FILE*` streams (opaque handles, standard mode strings, `SEEK_*` 0 to 2, 64-bit `off_t`) and
/// path calls that take only paths, `mkdir`'s permission bits or `access`'s `F_OK` family.
const SAME_IN_POSIX_C_LIBRARIES: &[&str] = &[
    "strtol",
    "strtoul",
    "atol",
    "labs",
    "posix_memalign",
    "aligned_alloc",
    "fopen",
    "fclose",
    "fread",
    "fwrite",
    "fflush",
    "fseeko",
    "ftello",
    "fseek",
    "ftell",
    "ferror",
    "feof",
    "realpath",
    "getcwd",
    "chdir",
    "access",
    "mkdir",
    "rmdir",
    "unlink",
    "rename",
    "link",
    "read",
    "write",
    "close",
    "lseek",
    "isatty",
];

/// Procedures one POSIX C library names differently from the other: glibc's 64-bit file
/// offset names (every host's `off_t` has 64 bits) and Darwin's `realpath`.
const POSIX_RENAMED: &[(&str, &str)] = &[
    ("fopen64", "fopen"),
    ("fseeko64", "fseeko"),
    ("ftello64", "ftello"),
    ("lseek64", "lseek"),
    ("realpath$DARWIN_EXTSN", "realpath"),
];

/// How compile-time code reaches a procedure of its target's C library, when the target's OS
/// is not the host's (`reach`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reach {
    /// Natively: the procedure means the same in both libraries.
    Native,
    /// Natively, by the host library's name for the same procedure.
    Renamed(&'static str),
    /// By `posix::native`, in the target's ABI over the host's files.
    #[cfg(unix)]
    Answered(posix::native::PosixFn),
    /// Not at all: the host's library does not provide it the same way.
    Refused,
}

/// The C libraries of the target of `program` and of the host, when the target's OS is not
/// the host's and both are POSIX ones: then `errno` and the answered calls translate.
#[cfg(unix)]
pub(super) fn other_posix_library(program: &Program) -> Option<posix::Libraries> {
    let (os, cpu) = (program.target_os?, program.target_cpu?);
    let target = posix::PosixAbi::of(os, cpu)?;
    let host = posix::PosixAbi::host()?;
    (os != crate::sema::compiler_host().0).then_some(posix::Libraries {
        target,
        host,
    })
}

/// How compile-time code of `program` reaches `foreign` on this host: `None` when it is not a
/// procedure of the C library, or the target's OS is the host's, so its call is native.
/// Other libraries keep their native lookup, which names a library the host lacks. Windows
/// targets reach their system API through `windows_native` instead.
pub(super) fn reach(program: &Program, foreign: &ir::Foreign) -> Option<Reach> {
    use crate::target::TargetOs;
    let library = &program.libraries[foreign.library?];
    let target = program.target_os?;
    let host = crate::sema::compiler_host().0;
    if !native::is_c_library(&library.name, library.system) || target == host {
        return None;
    }
    let symbol = foreign.symbol.as_str();
    if SAME_IN_EVERY_C_LIBRARY.contains(&symbol) {
        return Some(Reach::Native);
    }
    let posix = |os| matches!(os, TargetOs::Linux | TargetOs::MacOS);
    if !(posix(target) && posix(host)) {
        return Some(Reach::Refused);
    }
    if SAME_IN_POSIX_C_LIBRARIES.contains(&symbol) {
        return Some(Reach::Native);
    }
    if let Some(&(_, name)) = POSIX_RENAMED.iter().find(|(name, _)| *name == symbol) {
        return Some(Reach::Renamed(name));
    }
    #[cfg(unix)]
    if let Some(fun) = posix::native::PosixFn::from_name(symbol) {
        return Some(Reach::Answered(fun));
    }
    Some(Reach::Refused)
}
