//! Every foreign symbol the sandbox answers, in one registry.
//!
//! A symbol belongs to one operating system API ([`Api`]): the C library, Win32 (answered so
//! the editor can check a metaprogram written for Windows) or the Objective-C runtime. The
//! sandbox answers all of them whatever the host is, so `SandboxFn::lookup` is the one place
//! that maps a name to what implements it; the areas match on enum members, never on text.

use crate::macros::raw_enum;

raw_enum! {
    /// A C library procedure the sandbox implements itself.
    pub(in crate::interp) enum LibcFn by name {
        Write = "write",
        WasmWriteString = "wasm_write_string",
        Malloc = "malloc",
        Calloc = "calloc",
        Realloc = "realloc",
        Free = "free",
        Memcpy = "memcpy",
        Memmove = "memmove",
        Memset = "memset",
        Memcmp = "memcmp",
        Strlen = "strlen",
        Strcmp = "strcmp",
        Strncmp = "strncmp",
        Exit = "exit",
        ImmediateExit = "_exit",
        Abort = "abort",
        PthreadMutexattrInit = "pthread_mutexattr_init",
        PthreadMutexattrDestroy = "pthread_mutexattr_destroy",
        PthreadMutexattrSettype = "pthread_mutexattr_settype",
        PthreadAttrInit = "pthread_attr_init",
        PthreadAttrDestroy = "pthread_attr_destroy",
        PthreadAttrSetstacksize = "pthread_attr_setstacksize",
        PthreadMutexLock = "pthread_mutex_lock",
        PthreadMutexUnlock = "pthread_mutex_unlock",
        PthreadMutexInit = "pthread_mutex_init",
        PthreadMutexDestroy = "pthread_mutex_destroy",
        Isatty = "isatty",
        Getenv = "getenv",
        JaiHostProvides = "jai_host_provides",
        Strerror = "strerror",
        WasmDebugBreak = "wasm_debug_break",
        Nanosleep = "nanosleep",
        Usleep = "usleep",
        Sleep = "sleep",
        SchedYield = "sched_yield",
        Sysconf = "sysconf",
        Getpid = "getpid",
        Getentropy = "getentropy",
        ClockGettime = "clock_gettime",
        Gettimeofday = "gettimeofday",
        Time = "time",
        GmtimeR = "gmtime_r",
        LocaltimeR = "localtime_r",
        Atof = "atof",
        Strtod = "strtod",
        Strtof = "strtof",
        Setlocale = "setlocale",
        Atoi = "atoi",
        Atol = "atol",
        Atoll = "atoll",
        Strtol = "strtol",
        Strtoll = "strtoll",
        Strtoul = "strtoul",
        Strtoull = "strtoull",
        ErrnoLocation = "__errno_location",
        DarwinError = "__error",
        Getcwd = "getcwd",
        Chdir = "chdir",
        Realpath = "realpath",
        RealpathDarwin = "realpath$DARWIN_EXTSN",
        Access = "access",
        Chmod = "chmod",
        Fchmod = "fchmod",
        Stat = "stat",
        Lstat = "lstat",
        Fstat = "fstat",
        Open = "open",
        Open64 = "open64",
        Close = "close",
        Read = "read",
        Lseek = "lseek",
        Lseek64 = "lseek64",
        Fopen = "fopen",
        Fopen64 = "fopen64",
        Fclose = "fclose",
        Fflush = "fflush",
        Fread = "fread",
        Fwrite = "fwrite",
        Fseek = "fseek",
        Fseeko = "fseeko",
        Fseeko64 = "fseeko64",
        Ftell = "ftell",
        Ftello = "ftello",
        Ftello64 = "ftello64",
        Rewind = "rewind",
        Feof = "feof",
        Ferror = "ferror",
        Fileno = "fileno",
        Unlink = "unlink",
        Remove = "remove",
        Rename = "rename",
        Link = "link",
        Mkdir = "mkdir",
        Rmdir = "rmdir",
        Opendir = "opendir",
        Readdir = "readdir",
        Readdir64 = "readdir64",
        Closedir = "closedir",
        Pipe = "pipe",
        Pipe2 = "pipe2",
        Socketpair = "socketpair",
        Fork = "fork",
        Vfork = "vfork",
        Execv = "execv",
        Execve = "execve",
        Execvp = "execvp",
        PosixSpawn = "posix_spawn",
        PosixSpawnp = "posix_spawnp",
        ShmOpen = "shm_open",
        ShmUnlink = "shm_unlink",
        Fgets = "fgets",
        Getline = "getline",
        Getdelim = "getdelim",
        Fgetc = "fgetc",
        Getc = "getc",
        FgetcUnlocked = "fgetc_unlocked",
        GetcUnlocked = "getc_unlocked",
        Getchar = "getchar",
        GetcharUnlocked = "getchar_unlocked",
        Ungetc = "ungetc",
    }
}

raw_enum! {
    /// A Win32 procedure the sandbox implements itself.
    pub(in crate::interp) enum WinFn by name {
        GetLastError = "GetLastError",
        SetLastError = "SetLastError",
        MultiByteToWideChar = "MultiByteToWideChar",
        WideCharToMultiByte = "WideCharToMultiByte",
        GetSystemTimeAsFileTime = "GetSystemTimeAsFileTime",
        GetSystemTimePreciseAsFileTime = "GetSystemTimePreciseAsFileTime",
        QueryPerformanceCounter = "QueryPerformanceCounter",
        QueryPerformanceFrequency = "QueryPerformanceFrequency",
        GetCurrentDirectoryW = "GetCurrentDirectoryW",
        SetCurrentDirectoryW = "SetCurrentDirectoryW",
        GetFullPathNameW = "GetFullPathNameW",
        GetFileAttributesW = "GetFileAttributesW",
        CreateDirectoryW = "CreateDirectoryW",
        RemoveDirectoryW = "RemoveDirectoryW",
        DeleteFileW = "DeleteFileW",
        MoveFileExW = "MoveFileExW",
        MoveFileW = "MoveFileW",
        CreateFileW = "CreateFileW",
        GetStdHandle = "GetStdHandle",
        CloseHandle = "CloseHandle",
        ReadFile = "ReadFile",
        WriteFile = "WriteFile",
        GetFileSizeEx = "GetFileSizeEx",
        SetFilePointerEx = "SetFilePointerEx",
        FindFirstFileW = "FindFirstFileW",
        FindFirstFileExW = "FindFirstFileExW",
        FindNextFileW = "FindNextFileW",
        FindClose = "FindClose",
        CreatePipe = "CreatePipe",
        CreateProcessW = "CreateProcessW",
        CreateProcessA = "CreateProcessA",
        CreateFileMappingW = "CreateFileMappingW",
        CreateFileMappingA = "CreateFileMappingA",
        OpenFileMappingW = "OpenFileMappingW",
        OpenFileMappingA = "OpenFileMappingA",
    }
}

raw_enum! {
    /// An Objective-C runtime procedure the sandbox implements itself.
    pub(in crate::interp) enum ObjcFn by name {
        SelRegisterName = "sel_registerName",
        SelGetUid = "sel_getUid",
        SelGetName = "sel_getName",
        ClassGetName = "class_getName",
        ObjectGetClassName = "object_getClassName",
        ObjcGetClass = "objc_getClass",
        ObjcLookUpClass = "objc_lookUpClass",
        ObjcGetRequiredClass = "objc_getRequiredClass",
        ObjcGetMetaClass = "objc_getMetaClass",
        ClassGetInstanceSize = "class_getInstanceSize",
        ClassGetSuperclass = "class_getSuperclass",
        MsgSend = "objc_msgSend",
        MsgSendStret = "objc_msgSend_stret",
        MsgSendFpret = "objc_msgSend_fpret",
        MsgSendFpret2 = "objc_msgSend_fpret2",
        MsgSendSuper = "objc_msgSendSuper",
        MsgSendSuperStret = "objc_msgSendSuper_stret",
        MsgSendSuper2 = "objc_msgSendSuper2",
        Dlopen = "dlopen",
        Dlclose = "dlclose",
    }
}

/// A symbol the sandbox answers, with the API it comes from: the C library, Win32 (answered so
/// the editor can check a metaprogram written for Windows on any host) or the Objective-C
/// runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::interp) enum SandboxFn {
    Libc(LibcFn),
    Windows(WinFn),
    Objc(ObjcFn),
}

impl SandboxFn {
    /// The implementation of the foreign symbol `name`.
    pub(in crate::interp) fn lookup(name: &str) -> Option<Self> {
        WinFn::from_name(name)
            .map(Self::Windows)
            .or_else(|| ObjcFn::from_name(name).map(Self::Objc))
            .or_else(|| LibcFn::from_name(name).map(Self::Libc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_symbol_is_claimed_by_two_apis() {
        let mut seen = crate::fxhash::HashSet::default();
        let names = LibcFn::ALL
            .iter()
            .map(|f| f.name())
            .chain(WinFn::ALL.iter().map(|f| f.name()))
            .chain(ObjcFn::ALL.iter().map(|f| f.name()));
        for name in names {
            assert!(seen.insert(name), "`{name}` is registered twice");
            assert!(SandboxFn::lookup(name).is_some());
        }
    }
}
