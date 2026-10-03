# POSIX


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `OS_Error_Code :: #type,isa s32`
- `errno :: inline () -> OS_Error_Code #no_context`
- `set_errno :: inline (value: OS_Error_Code) #no_context`
- `size_t :: u64`
- `ssize_t :: s64`
- `off_t :: s64`
- [OS == .MACOS] `SYSCALL_MACH_BASE :: 0x1000000`
- [OS == .MACOS] `SYSCALL_UNIX_BASE :: 0x2000000`
- [OS == .ANDROID] `MAP_FAILED :: cast,trunc(*void) -1`
- `IFTODT :: inline (mode: mode_t) -> mode_t`
- `DTTOIF :: inline (dirtype: mode_t) -> mode_t`
- `S_ISDIR :: inline (mode: mode_t) -> bool`
- `S_ISCHR :: inline (mode: mode_t) -> bool`
- `S_ISBLK :: inline (mode: mode_t) -> bool`
- `S_ISREG :: inline (mode: mode_t) -> bool`
- `S_ISFIFO :: inline (mode: mode_t) -> bool`
- `S_ISLNK :: inline (mode: mode_t) -> bool`
- `S_ISSOCK :: inline (mode: mode_t) -> bool`
- [OS == .LINUX || OS == .ANDROID] `WEXITSTATUS :: inline (status: s32) -> s32`
- [OS == .LINUX || OS == .ANDROID] `WTERMSIG :: inline (status: s32) -> s32`
- [OS == .LINUX || OS == .ANDROID] `WSTOPSIG :: inline (status: s32) -> s32`
- [OS == .LINUX || OS == .ANDROID] `WIFEXITED :: inline (status: s32) -> bool`
- [OS == .LINUX || OS == .ANDROID] `WIFSIGNALED :: inline (status: s32) -> bool`
- [OS == .LINUX || OS == .ANDROID] `WIFSTOPPED :: inline (status: s32) -> bool`
- [OS == .LINUX || OS == .ANDROID] `WIFCONTINUED :: inline (status: s32) -> bool`
- [OS == .LINUX || OS == .ANDROID] `WCOREDUMP :: inline (status: s32) -> s32`
- [OS == .MACOS] `WEXITSTATUS :: inline (status: s32) -> s32`
- [OS == .MACOS] `WTERMSIG :: inline (status: s32) -> s32`
- [OS == .MACOS] `WSTOPSIG :: inline (status: s32) -> s32`
- [OS == .MACOS] `WIFEXITED :: inline (status: s32) -> bool`
- [OS == .MACOS] `WIFSIGNALED :: inline (status: s32) -> bool`
- [OS == .MACOS] `WIFSTOPPED :: inline (status: s32) -> bool`
- [OS == .MACOS] `WIFCONTINUED :: inline (status: s32) -> bool`
- [OS == .MACOS] `WCOREDUMP :: inline (status: s32) -> s32`
- `strsignal :: (sig: s32) -> *u8 #foreign libc`
- [OS == .LINUX] `RTLD_LAZY :: 0x00001`
- [OS == .LINUX] `RTLD_NOW :: 0x00002`
- [OS == .LINUX] `RTLD_BINDING_MASK :: 0x3`
- [OS == .LINUX] `RTLD_NOLOAD :: 0x00004`
- [OS == .LINUX] `RTLD_DEEPBIND :: 0x00008`
- [OS == .LINUX] `RTLD_GLOBAL :: 0x00100`
- [OS == .LINUX] `RTLD_LOCAL :: 0`
- [OS == .LINUX] `RTLD_NODELETE :: 0x01000`
- [OS == .LINUX] `RTLD_NEXT :: cast,trunc(*void) -1`
- [OS == .LINUX] `RTLD_DEFAULT :: cast(*void) 0`
- [OS == .LINUX] `LM_ID_BASE :: 0`
- [OS == .LINUX] `LM_ID_NEWLM :: -1`
- [OS == .LINUX] `__S32_TYPE :: s32`
- [OS == .LINUX] `__SLONG32_TYPE :: s32`
- [OS == .LINUX] `_DIRENT_MATCHES_DIRENT64 :: 1`
- [OS == .LINUX] `EPERM :: 1`
- [OS == .LINUX] `ENOENT :: 2`
- [OS == .LINUX] `ESRCH :: 3`
- [OS == .LINUX] `EINTR :: 4`
- [OS == .LINUX] `EIO :: 5`
- [OS == .LINUX] `ENXIO :: 6`
- [OS == .LINUX] `E2BIG :: 7`
- [OS == .LINUX] `ENOEXEC :: 8`
- [OS == .LINUX] `EBADF :: 9`
- [OS == .LINUX] `ECHILD :: 10`
- [OS == .LINUX] `EAGAIN :: 11`
- [OS == .LINUX] `ENOMEM :: 12`
- [OS == .LINUX] `EACCES :: 13`
- [OS == .LINUX] `EFAULT :: 14`
- [OS == .LINUX] `ENOTBLK :: 15`
- [OS == .LINUX] `EBUSY :: 16`
- [OS == .LINUX] `EEXIST :: 17`
- [OS == .LINUX] `EXDEV :: 18`
- [OS == .LINUX] `ENODEV :: 19`
- [OS == .LINUX] `ENOTDIR :: 20`
- [OS == .LINUX] `EISDIR :: 21`
- [OS == .LINUX] `EINVAL :: 22`
- [OS == .LINUX] `ENFILE :: 23`
- [OS == .LINUX] `EMFILE :: 24`
- [OS == .LINUX] `ENOTTY :: 25`
- [OS == .LINUX] `ETXTBSY :: 26`
- [OS == .LINUX] `EFBIG :: 27`
- [OS == .LINUX] `ENOSPC :: 28`
- [OS == .LINUX] `ESPIPE :: 29`
- [OS == .LINUX] `EROFS :: 30`
- [OS == .LINUX] `EMLINK :: 31`
- [OS == .LINUX] `EPIPE :: 32`
- [OS == .LINUX] `EDOM :: 33`
- [OS == .LINUX] `ERANGE :: 34`
- [OS == .LINUX] `EDEADLK :: 35`
- [OS == .LINUX] `ENAMETOOLONG :: 36`
- [OS == .LINUX] `ENOLCK :: 37`
- [OS == .LINUX] `ENOSYS :: 38`
- [OS == .LINUX] `ENOTEMPTY :: 39`
- [OS == .LINUX] `ELOOP :: 40`
- [OS == .LINUX] `EWOULDBLOCK :: EAGAIN`
- [OS == .LINUX] `ENOMSG :: 42`
- [OS == .LINUX] `EIDRM :: 43`
- [OS == .LINUX] `ECHRNG :: 44`
- [OS == .LINUX] `EL2NSYNC :: 45`
- [OS == .LINUX] `EL3HLT :: 46`
- [OS == .LINUX] `EL3RST :: 47`
- [OS == .LINUX] `ELNRNG :: 48`
- [OS == .LINUX] `EUNATCH :: 49`
- [OS == .LINUX] `ENOCSI :: 50`
- [OS == .LINUX] `EL2HLT :: 51`
- [OS == .LINUX] `EBADE :: 52`
- [OS == .LINUX] `EBADR :: 53`
- [OS == .LINUX] `EXFULL :: 54`
- [OS == .LINUX] `ENOANO :: 55`
- [OS == .LINUX] `EBADRQC :: 56`
- [OS == .LINUX] `EBADSLT :: 57`
- [OS == .LINUX] `EDEADLOCK :: EDEADLK`
- [OS == .LINUX] `EBFONT :: 59`
- [OS == .LINUX] `ENOSTR :: 60`
- [OS == .LINUX] `ENODATA :: 61`
- [OS == .LINUX] `ETIME :: 62`
- [OS == .LINUX] `ENOSR :: 63`
- [OS == .LINUX] `ENONET :: 64`
- [OS == .LINUX] `ENOPKG :: 65`
- [OS == .LINUX] `EREMOTE :: 66`
- [OS == .LINUX] `ENOLINK :: 67`
- [OS == .LINUX] `EADV :: 68`
- [OS == .LINUX] `ESRMNT :: 69`
- [OS == .LINUX] `ECOMM :: 70`
- [OS == .LINUX] `EPROTO :: 71`
- [OS == .LINUX] `EMULTIHOP :: 72`
- [OS == .LINUX] `EDOTDOT :: 73`
- [OS == .LINUX] `EBADMSG :: 74`
- [OS == .LINUX] `EOVERFLOW :: 75`
- [OS == .LINUX] `ENOTUNIQ :: 76`
- [OS == .LINUX] `EBADFD :: 77`
- [OS == .LINUX] `EREMCHG :: 78`
- [OS == .LINUX] `ELIBACC :: 79`
- [OS == .LINUX] `ELIBBAD :: 80`
- [OS == .LINUX] `ELIBSCN :: 81`
- [OS == .LINUX] `ELIBMAX :: 82`
- [OS == .LINUX] `ELIBEXEC :: 83`
- [OS == .LINUX] `EILSEQ :: 84`
- [OS == .LINUX] `ERESTART :: 85`
- [OS == .LINUX] `ESTRPIPE :: 86`
- [OS == .LINUX] `EUSERS :: 87`
- [OS == .LINUX] `ENOTSOCK :: 88`
- [OS == .LINUX] `EDESTADDRREQ :: 89`
- [OS == .LINUX] `EMSGSIZE :: 90`
- [OS == .LINUX] `EPROTOTYPE :: 91`
- [OS == .LINUX] `ENOPROTOOPT :: 92`
- [OS == .LINUX] `EPROTONOSUPPORT :: 93`
- [OS == .LINUX] `ESOCKTNOSUPPORT :: 94`
- [OS == .LINUX] `EOPNOTSUPP :: 95`
- [OS == .LINUX] `EPFNOSUPPORT :: 96`
- [OS == .LINUX] `EAFNOSUPPORT :: 97`
- [OS == .LINUX] `EADDRINUSE :: 98`
- [OS == .LINUX] `EADDRNOTAVAIL :: 99`
- [OS == .LINUX] `ENETDOWN :: 100`
- [OS == .LINUX] `ENETUNREACH :: 101`
- [OS == .LINUX] `ENETRESET :: 102`
- [OS == .LINUX] `ECONNABORTED :: 103`
- [OS == .LINUX] `ECONNRESET :: 104`
- [OS == .LINUX] `ENOBUFS :: 105`
- [OS == .LINUX] `EISCONN :: 106`
- [OS == .LINUX] `ENOTCONN :: 107`
- [OS == .LINUX] `ESHUTDOWN :: 108`
- [OS == .LINUX] `ETOOMANYREFS :: 109`
- [OS == .LINUX] `ETIMEDOUT :: 110`
- [OS == .LINUX] `ECONNREFUSED :: 111`
- [OS == .LINUX] `EHOSTDOWN :: 112`
- [OS == .LINUX] `EHOSTUNREACH :: 113`
- [OS == .LINUX] `EALREADY :: 114`
- [OS == .LINUX] `EINPROGRESS :: 115`
- [OS == .LINUX] `ESTALE :: 116`
- [OS == .LINUX] `EUCLEAN :: 117`
- [OS == .LINUX] `ENOTNAM :: 118`
- [OS == .LINUX] `ENAVAIL :: 119`
- [OS == .LINUX] `EISNAM :: 120`
- [OS == .LINUX] `EREMOTEIO :: 121`
- [OS == .LINUX] `EDQUOT :: 122`
- [OS == .LINUX] `ENOMEDIUM :: 123`
- [OS == .LINUX] `EMEDIUMTYPE :: 124`
- [OS == .LINUX] `ECANCELED :: 125`
- [OS == .LINUX] `ENOKEY :: 126`
- [OS == .LINUX] `EKEYEXPIRED :: 127`
- [OS == .LINUX] `EKEYREVOKED :: 128`
- [OS == .LINUX] `EKEYREJECTED :: 129`
- [OS == .LINUX] `EOWNERDEAD :: 130`
- [OS == .LINUX] `ENOTRECOVERABLE :: 131`
- [OS == .LINUX] `ERFKILL :: 132`
- [OS == .LINUX] `EHWPOISON :: 133`
- [OS == .LINUX] `ENOTSUP :: EOPNOTSUPP`
- [OS == .LINUX] `FNM_PATHNAME :: 1 << 0`
- [OS == .LINUX] `FNM_NOESCAPE :: 1 << 1`
- [OS == .LINUX] `FNM_PERIOD :: 1 << 2`
- [OS == .LINUX] `FNM_FILE_NAME :: FNM_PATHNAME`
- [OS == .LINUX] `FNM_LEADING_DIR :: 1 << 3`
- [OS == .LINUX] `FNM_CASEFOLD :: 1 << 4`
- [OS == .LINUX] `FNM_EXTMATCH :: 1 << 5`
- [OS == .LINUX] `FNM_NOMATCH :: 1`
- [OS == .LINUX] `FNM_NOSYS :: -1`
- [OS == .LINUX] `NSS_BUFLEN_PASSWD :: 1024`
- [OS == .LINUX] `SIG_ERR :: cast(#type (unknown0: s32) -> void #c_call) -1`
- [OS == .LINUX] `SIG_DFL :: cast(#type (unknown0: s32) -> void #c_call) 0`
- [OS == .LINUX] `SIG_IGN :: cast(#type (unknown0: s32) -> void #c_call) 1`
- [OS == .LINUX] `SIG_HOLD :: cast(#type (unknown0: s32) -> void #c_call) 2`
- [OS == .LINUX] `SIGINT :: 2`
- [OS == .LINUX] `SIGILL :: 4`
- [OS == .LINUX] `SIGABRT :: 6`
- [OS == .LINUX] `SIGFPE :: 8`
- [OS == .LINUX] `SIGSEGV :: 11`
- [OS == .LINUX] `SIGTERM :: 15`
- [OS == .LINUX] `SIGHUP :: 1`
- [OS == .LINUX] `SIGQUIT :: 3`
- [OS == .LINUX] `SIGTRAP :: 5`
- [OS == .LINUX] `SIGKILL :: 9`
- [OS == .LINUX] `SIGPWR :: 30`
- [OS == .LINUX] `SIGPOLL :: 29`
- [OS == .LINUX] `SIGPIPE :: 13`
- [OS == .LINUX] `SIGALRM :: 14`
- [OS == .LINUX] `SIGTSTP :: 20`
- [OS == .LINUX] `SIGCONT :: 18`
- [OS == .LINUX] `SIGSTOP :: 19`
- [OS == .LINUX] `SIGCHLD :: 17`
- [OS == .LINUX] `SIGUSR2 :: 12`
- [OS == .LINUX] `SIGTTIN :: 21`
- [OS == .LINUX] `SIGTTOU :: 22`
- [OS == .LINUX] `SIGURG :: 23`
- [OS == .LINUX] `SIGXCPU :: 24`
- [OS == .LINUX] `SIGXFSZ :: 25`
- [OS == .LINUX] `SIGVTALRM :: 26`
- [OS == .LINUX] `SIGPROF :: 27`
- [OS == .LINUX] `SIGBUS :: 7`
- [OS == .LINUX] `SIGUSR1 :: 10`
- [OS == .LINUX] `SIGWINCH :: 28`
- [OS == .LINUX] `SIGIO :: SIGPOLL`
- [OS == .LINUX] `SIGIOT :: SIGABRT`
- [OS == .LINUX] `SIGCLD :: SIGCHLD`
- [OS == .LINUX] `__SIGRTMIN :: 32`
- [OS == .LINUX] `SIGSYS :: 31`
- [OS == .LINUX] `_NSIG :: __SIGRTMAX + 1`
- [OS == .LINUX] `SIGSTKFLT :: 16`
- [OS == .LINUX] `__SIGRTMAX :: 64`
- [OS == .LINUX] `_STRUCT_TIMESPEC :: 1`
- [OS == .LINUX] `__SIGEV_MAX_SIZE :: 64`
- [OS == .LINUX] `NSIG :: _NSIG`
- [OS == .LINUX] `SA_NOCLDSTOP :: 1`
- [OS == .LINUX] `SA_NOCLDWAIT :: 2`
- [OS == .LINUX] `SA_SIGINFO :: 4`
- [OS == .LINUX] `SA_ONSTACK :: 0x08000000`
- [OS == .LINUX] `SA_RESTART :: 0x10000000`
- [OS == .LINUX] `SA_NODEFER :: 0x40000000`
- [OS == .LINUX] `SA_RESETHAND :: 0x80000000`
- [OS == .LINUX] `SA_INTERRUPT :: 0x20000000`
- [OS == .LINUX] `SA_NOMASK :: SA_NODEFER`
- [OS == .LINUX] `SA_ONESHOT :: SA_RESETHAND`
- [OS == .LINUX] `SA_STACK :: SA_ONSTACK`
- [OS == .LINUX] `SIG_BLOCK :: 0`
- [OS == .LINUX] `SIG_UNBLOCK :: 1`
- [OS == .LINUX] `SIG_SETMASK :: 2`
- [OS == .LINUX] `__NGREG :: 23`
- [OS == .LINUX] `NGREG :: __NGREG`
- [OS == .LINUX] `IPC_CREAT :: 0x200`
- [OS == .LINUX] `IPC_EXCL :: 0x400`
- [OS == .LINUX] `IPC_NOWAIT :: 0x800`
- [OS == .LINUX] `IPC_RMID :: 0`
- [OS == .LINUX] `IPC_SET :: 1`
- [OS == .LINUX] `IPC_STAT :: 2`
- [OS == .LINUX] `IPC_INFO :: 3`
- [OS == .LINUX] `IPC_PRIVATE :: cast(s32) 0`
- [OS == .LINUX] `MAP_32BIT :: 0x40`
- [OS == .LINUX] `MAP_GROWSDOWN :: 0x00100`
- [OS == .LINUX] `MAP_DENYWRITE :: 0x00800`
- [OS == .LINUX] `MAP_EXECUTABLE :: 0x01000`
- [OS == .LINUX] `MAP_LOCKED :: 0x02000`
- [OS == .LINUX] `MAP_NORESERVE :: 0x04000`
- [OS == .LINUX] `MAP_POPULATE :: 0x08000`
- [OS == .LINUX] `MAP_NONBLOCK :: 0x10000`
- [OS == .LINUX] `MAP_STACK :: 0x20000`
- [OS == .LINUX] `MAP_HUGETLB :: 0x40000`
- [OS == .LINUX] `MAP_SYNC :: 0x80000`
- [OS == .LINUX] `MAP_FIXED_NOREPLACE :: 0x100000`
- [OS == .LINUX] `PROT_READ :: 0x1`
- [OS == .LINUX] `PROT_WRITE :: 0x2`
- [OS == .LINUX] `PROT_EXEC :: 0x4`
- [OS == .LINUX] `PROT_NONE :: 0x0`
- [OS == .LINUX] `PROT_GROWSDOWN :: 0x01000000`
- [OS == .LINUX] `PROT_GROWSUP :: 0x02000000`
- [OS == .LINUX] `MAP_SHARED :: 0x01`
- [OS == .LINUX] `MAP_PRIVATE :: 0x02`
- [OS == .LINUX] `MAP_SHARED_VALIDATE :: 0x03`
- [OS == .LINUX] `MAP_TYPE :: 0x0f`
- [OS == .LINUX] `MAP_FIXED :: 0x10`
- [OS == .LINUX] `MAP_FILE :: 0`
- [OS == .LINUX] `MAP_ANONYMOUS :: 0x20`
- [OS == .LINUX] `MAP_ANON :: MAP_ANONYMOUS`
- [OS == .LINUX] `MAP_HUGE_SHIFT :: 26`
- [OS == .LINUX] `MAP_HUGE_MASK :: 0x3f`
- [OS == .LINUX] `MS_ASYNC :: 1`
- [OS == .LINUX] `MS_SYNC :: 4`
- [OS == .LINUX] `MS_INVALIDATE :: 2`
- [OS == .LINUX] `MADV_NORMAL :: 0`
- [OS == .LINUX] `MADV_RANDOM :: 1`
- [OS == .LINUX] `MADV_SEQUENTIAL :: 2`
- [OS == .LINUX] `MADV_WILLNEED :: 3`
- [OS == .LINUX] `MADV_DONTNEED :: 4`
- [OS == .LINUX] `MADV_FREE :: 8`
- [OS == .LINUX] `MADV_REMOVE :: 9`
- [OS == .LINUX] `MADV_DONTFORK :: 10`
- [OS == .LINUX] `MADV_DOFORK :: 11`
- [OS == .LINUX] `MADV_MERGEABLE :: 12`
- [OS == .LINUX] `MADV_UNMERGEABLE :: 13`
- [OS == .LINUX] `MADV_HUGEPAGE :: 14`
- [OS == .LINUX] `MADV_NOHUGEPAGE :: 15`
- [OS == .LINUX] `MADV_DONTDUMP :: 16`
- [OS == .LINUX] `MADV_DODUMP :: 17`
- [OS == .LINUX] `MADV_WIPEONFORK :: 18`
- [OS == .LINUX] `MADV_KEEPONFORK :: 19`
- [OS == .LINUX] `MADV_COLD :: 20`
- [OS == .LINUX] `MADV_PAGEOUT :: 21`
- [OS == .LINUX] `MADV_HWPOISON :: 100`
- [OS == .LINUX] `MCL_CURRENT :: 1`
- [OS == .LINUX] `MCL_FUTURE :: 2`
- [OS == .LINUX] `MCL_ONFAULT :: 4`
- [OS == .LINUX] `MAP_FAILED :: cast,trunc(*void) -1`
- [OS == .LINUX] `POLLIN :: 0x001`
- [OS == .LINUX] `POLLPRI :: 0x002`
- [OS == .LINUX] `POLLOUT :: 0x004`
- [OS == .LINUX] `POLLRDNORM :: 0x040`
- [OS == .LINUX] `POLLRDBAND :: 0x080`
- [OS == .LINUX] `POLLWRNORM :: 0x100`
- [OS == .LINUX] `POLLWRBAND :: 0x200`
- [OS == .LINUX] `POLLMSG :: 0x400`
- [OS == .LINUX] `POLLREMOVE :: 0x1000`
- [OS == .LINUX] `POLLRDHUP :: 0x2000`
- [OS == .LINUX] `POLLERR :: 0x008`
- [OS == .LINUX] `POLLHUP :: 0x010`
- [OS == .LINUX] `POLLNVAL :: 0x020`
- [OS == .LINUX] `SHM_R :: 0x100`
- [OS == .LINUX] `SHM_W :: 0x80`
- [OS == .LINUX] `SHM_RDONLY :: 0x1000`
- [OS == .LINUX] `SHM_RND :: 0x2000`
- [OS == .LINUX] `SHM_REMAP :: 0x4000`
- [OS == .LINUX] `SHM_EXEC :: 0x8000`
- [OS == .LINUX] `SHM_LOCK :: 11`
- [OS == .LINUX] `SHM_UNLOCK :: 12`
- [OS == .LINUX] `SHM_STAT :: 13`
- [OS == .LINUX] `SHM_INFO :: 14`
- [OS == .LINUX] `SHM_STAT_ANY :: 15`
- [OS == .LINUX] `SHM_DEST :: 0x200`
- [OS == .LINUX] `SHM_LOCKED :: 0x400`
- [OS == .LINUX] `SHM_HUGETLB :: 0x800`
- [OS == .LINUX] `SHM_NORESERVE :: 0x1000`
- [OS == .LINUX] `__BIT_TYPES_DEFINED__ :: 1`
- [OS == .LINUX] `WNOHANG :: 1`
- [OS == .LINUX] `WUNTRACED :: 2`
- [OS == .LINUX] `WSTOPPED :: 2`
- [OS == .LINUX] `WEXITED :: 4`
- [OS == .LINUX] `WCONTINUED :: 8`
- [OS == .LINUX] `WNOWAIT :: 0x01000000`
- [OS == .LINUX] `__WNOTHREAD :: 0x20000000`
- [OS == .LINUX] `__WALL :: 0x40000000`
- [OS == .LINUX] `__WCLONE :: 0x80000000`
- [OS == .LINUX] `__ENUM_IDTYPE_T :: 1`
- [OS == .LINUX] `WAIT_ANY :: -1`
- [OS == .LINUX] `WAIT_MYPGRP :: 0`
- [OS == .LINUX] `CLOCKS_PER_SEC :: cast(s64) 1000000`
- [OS == .LINUX] `clockid_t :: enum u32`
- [OS == .LINUX] `TIMER_ABSTIME :: 1`
- [OS == .LINUX] `TIME_UTC :: 1`
- [OS == .LINUX] `__POSIX2_THIS_VERSION :: 200809`
- [OS == .LINUX] `_XOPEN_VERSION :: 700`
- [OS == .LINUX] `_XOPEN_XCU_VERSION :: 4`
- [OS == .LINUX] `_XOPEN_XPG2 :: 1`
- [OS == .LINUX] `_XOPEN_XPG3 :: 1`
- [OS == .LINUX] `_XOPEN_XPG4 :: 1`
- [OS == .LINUX] `_XOPEN_UNIX :: 1`
- [OS == .LINUX] `_XOPEN_ENH_I18N :: 1`
- [OS == .LINUX] `_XOPEN_LEGACY :: 1`
- [OS == .LINUX] `STDIN_FILENO :: 0`
- [OS == .LINUX] `STDOUT_FILENO :: 1`
- [OS == .LINUX] `STDERR_FILENO :: 2`
- [OS == .LINUX] `L_SET :: SEEK_SET`
- [OS == .LINUX] `L_INCR :: SEEK_CUR`
- [OS == .LINUX] `L_XTND :: SEEK_END`
- [OS == .LINUX] `_dl_mcount_wrapper_check :: (__selfpc: *void) -> void #foreign libc`
- [OS == .LINUX] `Lmid_t :: s64`
- [OS == .LINUX] `dlopen :: (__file: *u8, __mode: s32) -> *void #foreign libdl`
- [OS == .LINUX] `dlclose :: (__handle: *void) -> s32 #foreign libdl`
- [OS == .LINUX] `dlsym :: (__handle: *void, __name: *u8) -> *void #foreign libdl`
- [OS == .LINUX] `dlmopen :: (__nsid: Lmid_t, __file: *u8, __mode: s32) -> *void #foreign libdl`
- [OS == .LINUX] `dlvsym :: (__handle: *void, __name: *u8, __version: *u8) -> *void #foreign libdl`
- [OS == .LINUX] `dlerror :: () -> *u8 #foreign libdl`
- [OS == .LINUX] `Dl_info :: struct`
- [OS == .LINUX] `dladdr :: (__address: *void, __info: *Dl_info) -> s32 #foreign libdl`
- [OS == .LINUX] `dladdr1 :: (__address: *void, __info: *Dl_info, __extra_info: **void, __flags: s32) -> s32 #foreign libdl`
- [OS == .LINUX] `RTLD_DL :: enum u32`
- [OS == .LINUX] `dlinfo :: (__handle: *void, __request: s32, __arg: *void) -> s32 #foreign libdl`
- [OS == .LINUX] `RTLD_DI :: enum u32`
- [OS == .LINUX] `Dl_serpath :: struct`
- [OS == .LINUX] `Dl_serinfo :: struct`
- [OS == .LINUX] `__fsid_t :: struct`
- [OS == .LINUX] `ino64_t :: u64`
- [OS == .LINUX] `dirent :: struct`
- [OS == .LINUX] `dirent64 :: struct`
- [OS == .LINUX] `DT :: enum u8`
- [OS == .LINUX] `__dirstream :: struct`
- [OS == .LINUX] `DIR :: __dirstream`
- [OS == .LINUX] `opendir :: (__name: *u8) -> *DIR #foreign libc`
- [OS == .LINUX] `fdopendir :: (__fd: s32) -> *DIR #foreign libc`
- [OS == .LINUX] `closedir :: (__dirp: *DIR) -> s32 #foreign libc`
- [OS == .LINUX] `readdir :: (__dirp: *DIR) -> *dirent #foreign libc`
- [OS == .LINUX] `readdir64 :: (__dirp: *DIR) -> *dirent64 #foreign libc`
- [OS == .LINUX] `readdir_r :: (__dirp: *DIR, __entry: *dirent, __result: **dirent) -> s32 #foreign libc`
- [OS == .LINUX] `readdir64_r :: (__dirp: *DIR, __entry: *dirent64, __result: **dirent64) -> s32 #foreign libc`
- [OS == .LINUX] `rewinddir :: (__dirp: *DIR) -> void #foreign libc`
- [OS == .LINUX] `seekdir :: (__dirp: *DIR, __pos: s64) -> void #foreign libc`
- [OS == .LINUX] `telldir :: (__dirp: *DIR) -> s64 #foreign libc`
- [OS == .LINUX] `dirfd :: (__dirp: *DIR) -> s32 #foreign libc`
- [OS == .LINUX] `scandir :: (__dir: *u8, __namelist: ***dirent, __selector: #type (unknown0: *dirent) -> s32 #c_call, __cmp: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .LINUX] `scandir64 :: (__dir: *u8, __namelist: ***dirent64, __selector: #type (unknown0: *dirent64) -> s32 #c_call, __cmp: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .LINUX] `scandirat :: (__dfd: s32, __dir: *u8, __namelist: ***dirent, __selector: #type (unknown0: *dirent) -> s32 #c_call, __cmp: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .LINUX] `scandirat64 :: (__dfd: s32, __dir: *u8, __namelist: ***dirent64, __selector: #type (unknown0: *dirent64) -> s32 #c_call, __cmp: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .LINUX] `alphasort :: (__e1: **dirent, __e2: **dirent) -> s32 #foreign libc`
- [OS == .LINUX] `alphasort64 :: (__e1: **dirent64, __e2: **dirent64) -> s32 #foreign libc`
- [OS == .LINUX] `getdirentries :: (__fd: s32, __buf: *u8, __nbytes: u64, __basep: *s64) -> s64 #foreign libc`
- [OS == .LINUX] `getdirentries64 :: (__fd: s32, __buf: *u8, __nbytes: u64, __basep: *s64) -> s64 #foreign libc`
- [OS == .LINUX] `versionsort :: (__e1: **dirent, __e2: **dirent) -> s32 #foreign libc`
- [OS == .LINUX] `versionsort64 :: (__e1: **dirent64, __e2: **dirent64) -> s32 #foreign libc`
- [OS == .LINUX] `__errno_location :: () -> *OS_Error_Code #foreign libc`
- [OS == .LINUX] `program_invocation_name: *u8 #elsewhere libc`
- [OS == .LINUX] `program_invocation_short_name: *u8 #elsewhere libc`
- [OS == .LINUX] `backtrace :: (__array: **void, __size: s32) -> s32 #foreign libc`
- [OS == .LINUX] `backtrace_symbols :: (__array: **void, __size: s32) -> **u8 #foreign libc`
- [OS == .LINUX] `backtrace_symbols_fd :: (__array: **void, __size: s32, __fd: s32) -> void #foreign libc`
- [OS == .LINUX] `fnmatch :: (__pattern: *u8, __name: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `passwd :: struct`
- [OS == .LINUX] `setpwent :: () -> void #foreign libc`
- [OS == .LINUX] `endpwent :: () -> void #foreign libc`
- [OS == .LINUX] `getpwent :: () -> *passwd #foreign libc`
- [OS == .LINUX] `fgetpwent :: (__stream: *FILE) -> *passwd #foreign libc`
- [OS == .LINUX] `putpwent :: (__p: *passwd, __f: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `getpwuid :: (__uid: u32) -> *passwd #foreign libc`
- [OS == .LINUX] `getpwnam :: (__name: *u8) -> *passwd #foreign libc`
- [OS == .LINUX] `getpwent_r :: (__resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .LINUX] `getpwuid_r :: (__uid: u32, __resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .LINUX] `getpwnam_r :: (__name: *u8, __resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .LINUX] `fgetpwent_r :: (__stream: *FILE, __resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .LINUX] `getpw :: (__uid: u32, __buffer: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `sigset_t :: __sigset_t`
- [OS == .LINUX] `timespec :: struct`
- [OS == .LINUX] `sigval :: struct`
- [OS == .LINUX] `siginfo_t :: struct`
- [OS == .LINUX] `sigevent :: struct`
- [OS == .LINUX] `sigevent_t :: sigevent`
- [OS == .LINUX] `__sysv_signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc`
- [OS == .LINUX] `sysv_signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc`
- [OS == .LINUX] `signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc`
- [OS == .LINUX] `kill :: (__pid: s32, __sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `killpg :: (__pgrp: s32, __sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `raise :: (__sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `ssignal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc`
- [OS == .LINUX] `gsignal :: (__sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `psignal :: (__sig: s32, __s: *u8) -> void #foreign libc`
- [OS == .LINUX] `psiginfo :: (__pinfo: *siginfo_t, __s: *u8) -> void #foreign libc`
- [OS == .LINUX] `sigpause :: (__sig: s32) -> s32 #foreign libc "__xpg_sigpause"`
- [OS == .LINUX] `sigblock :: (__mask: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigsetmask :: (__mask: s32) -> s32 #foreign libc`
- [OS == .LINUX] `siggetmask :: () -> s32 #foreign libc`
- [OS == .LINUX] `sighandler_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .LINUX] `sig_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .LINUX] `sigemptyset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigfillset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigaddset :: (__set: *sigset_t, __signo: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigdelset :: (__set: *sigset_t, __signo: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigismember :: (__set: *sigset_t, __signo: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigisemptyset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigandset :: (__set: *sigset_t, __left: *sigset_t, __right: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigorset :: (__set: *sigset_t, __left: *sigset_t, __right: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigaction_t :: struct`
- [OS == .LINUX] `sigprocmask :: (__how: s32, __set: *sigset_t, __oset: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigsuspend :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigaction :: (__sig: s32, __act: *sigaction_t, __oact: *sigaction_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigpending :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigwait :: (__set: *sigset_t, __sig: *s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigwaitinfo :: (__set: *sigset_t, __info: *siginfo_t) -> s32 #foreign libc`
- [OS == .LINUX] `sigtimedwait :: (__set: *sigset_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .LINUX] `sigqueue :: (__pid: s32, __sig: s32, __val: sigval) -> s32 #foreign libc`
- [OS == .LINUX] `_sys_siglist: [65] *u8 #elsewhere libc`
- [OS == .LINUX] `sys_siglist: [65] *u8 #elsewhere libc`
- [OS == .LINUX] `_fpxreg :: struct`
- [OS == .LINUX] `_xmmreg :: struct`
- [OS == .LINUX] `_fpstate :: struct`
- [OS == .LINUX] `sigcontext :: struct`
- [OS == .LINUX] `sigreturn :: (__scp: *sigcontext) -> s32 #foreign libc`
- [OS == .LINUX] `stack_t :: struct`
- [OS == .LINUX] `greg_t :: s64`
- [OS == .LINUX] `gregset_t :: [23] greg_t`
- [OS == .LINUX] `REG :: enum u32`
- [OS == .LINUX] `_libc_fpxreg :: struct`
- [OS == .LINUX] `_libc_xmmreg :: struct`
- [OS == .LINUX] `_libc_fpstate :: struct`
- [OS == .LINUX] `fpregset_t :: *_libc_fpstate`
- [OS == .LINUX] `mcontext_t :: struct`
- [OS == .LINUX] `ucontext_t :: struct`
- [OS == .LINUX] `siginterrupt :: (__sig: s32, __interrupt: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigaltstack :: (__ss: *stack_t, __oss: *stack_t) -> s32 #foreign libc`
- [OS == .LINUX] `sighold :: (__sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigrelse :: (__sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigignore :: (__sig: s32) -> s32 #foreign libc`
- [OS == .LINUX] `sigset :: (__sig: s32, __disp: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc`
- [OS == .LINUX] `__libc_current_sigrtmin :: () -> s32 #foreign libc`
- [OS == .LINUX] `__libc_current_sigrtmax :: () -> s32 #foreign libc`
- [OS == .LINUX] `ipc_perm :: struct`
- [OS == .LINUX] `key_t :: s32`
- [OS == .LINUX] `ftok :: (__pathname: *u8, __proj_id: s32) -> key_t #foreign libc`
- [OS == .LINUX] `mmap :: (__addr: *void, __len: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc`
- [OS == .LINUX] `mmap64 :: (__addr: *void, __len: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc`
- [OS == .LINUX] `munmap :: (__addr: *void, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `mprotect :: (__addr: *void, __len: u64, __prot: s32) -> s32 #foreign libc`
- [OS == .LINUX] `msync :: (__addr: *void, __len: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `madvise :: (__addr: *void, __len: u64, __advice: s32) -> s32 #foreign libc`
- [OS == .LINUX] `posix_madvise :: (__addr: *void, __len: u64, __advice: s32) -> s32 #foreign libc`
- [OS == .LINUX] `mlock :: (__addr: *void, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `munlock :: (__addr: *void, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `mlockall :: (__flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `munlockall :: () -> s32 #foreign libc`
- [OS == .LINUX] `mincore :: (__start: *void, __len: u64, __vec: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `mremap :: (__addr: *void, __old_len: u64, __new_len: u64, __flags: s32, __args: ..Any) -> *void #foreign libc`
- [OS == .LINUX] `remap_file_pages :: (__start: *void, __size: u64, __prot: s32, __pgoff: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `shm_open :: (__name: *u8, __oflag: s32, __mode: mode_t) -> s32 #foreign librt`
- [OS == .LINUX] `shm_unlink :: (__name: *u8) -> s32 #foreign librt`
- [OS == .LINUX] `nfds_t :: u64`
- [OS == .LINUX] `pollfd :: struct`
- [OS == .LINUX] `poll :: (__fds: *pollfd, __nfds: nfds_t, __timeout: s32) -> s32 #foreign libc`
- [OS == .LINUX] `ppoll :: (__fds: *pollfd, __nfds: nfds_t, __timeout: *timespec, __ss: *sigset_t) -> s32 #foreign libc`
- [OS == .LINUX] `shmatt_t :: u64`
- [OS == .LINUX] `shmid_ds :: struct`
- [OS == .LINUX] `shminfo :: struct`
- [OS == .LINUX] `shm_info :: struct`
- [OS == .LINUX] `time_t :: s64`
- [OS == .LINUX] `shmctl :: (__shmid: s32, __cmd: s32, __buf: *shmid_ds) -> s32 #foreign libc`
- [OS == .LINUX] `shmget :: (__key: key_t, __size: u64, __shmflg: s32) -> s32 #foreign libc`
- [OS == .LINUX] `shmat :: (__shmid: s32, __shmaddr: *void, __shmflg: s32) -> *void #foreign libc`
- [OS == .LINUX] `shmdt :: (__shmaddr: *void) -> s32 #foreign libc`
- [OS == .LINUX] `timeval :: struct`
- [OS == .LINUX] `suseconds_t :: s64`
- [OS == .LINUX] `timezone :: struct`
- [OS == .LINUX] `gettimeofday :: (__tv: *timeval, __tz: *void) -> s32 #foreign libc`
- [OS == .LINUX] `settimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc`
- [OS == .LINUX] `adjtime :: (__delta: *timeval, __olddelta: *timeval) -> s32 #foreign libc`
- [OS == .LINUX] `__itimer_which :: enum u32`
- [OS == .LINUX] `itimerval :: struct`
- [OS == .LINUX] `getitimer :: (__which: __itimer_which, __value: *itimerval) -> s32 #foreign libc`
- [OS == .LINUX] `setitimer :: (__which: __itimer_which, __new: *itimerval, __old: *itimerval) -> s32 #foreign libc`
- [OS == .LINUX] `utimes :: (__file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc`
- [OS == .LINUX] `lutimes :: (__file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc`
- [OS == .LINUX] `futimes :: (__fd: s32, __tvp: *[2] timeval) -> s32 #foreign libc`
- [OS == .LINUX] `futimesat :: (__fd: s32, __file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc`
- [OS == .LINUX] `fsid_t :: __fsid_t`
- [OS == .LINUX] `clock_t :: s64`
- [OS == .LINUX] `timer_t :: *void`
- [OS == .LINUX] `useconds_t :: u32`
- [OS == .LINUX] `register_t :: s64`
- [OS == .LINUX] `fsblkcnt_t :: u64`
- [OS == .LINUX] `fsfilcnt_t :: u64`
- [OS == .LINUX] `blkcnt64_t :: s64`
- [OS == .LINUX] `fsblkcnt64_t :: u64`
- [OS == .LINUX] `fsfilcnt64_t :: u64`
- [OS == .LINUX] `utsname :: struct`
- [OS == .LINUX] `uname :: (__name: *utsname) -> s32 #foreign libc`
- [OS == .LINUX] `idtype_t :: enum u32`
- [OS == .LINUX] `wait :: (__stat_loc: *s32) -> s32 #foreign libc`
- [OS == .LINUX] `waitpid :: (__pid: s32, __stat_loc: *s32, __options: s32) -> s32 #foreign libc`
- [OS == .LINUX] `waitid :: (__idtype: idtype_t, __id: u32, __infop: *siginfo_t, __options: s32) -> s32 #foreign libc`
- [OS == .LINUX] `wait3 :: (__stat_loc: *s32, __options: s32, __usage: *rusage) -> s32 #foreign libc`
- [OS == .LINUX] `wait4 :: (__pid: s32, __stat_loc: *s32, __options: s32, __usage: *rusage) -> s32 #foreign libc`
- [OS == .LINUX] `timex :: struct`
- [OS == .LINUX] `clock_adjtime :: (__clock_id: s32, __utx: *timex) -> s32 #foreign libc`
- [OS == .LINUX] `tm :: struct`
- [OS == .LINUX] `itimerspec :: struct`
- [OS == .LINUX] `__locale_struct :: struct`
- [OS == .LINUX] `locale_t :: *__locale_struct`
- [OS == .LINUX] `clock :: () -> clock_t #foreign libc`
- [OS == .LINUX] `time :: (__timer: *time_t) -> time_t #foreign libc`
- [OS == .LINUX] `difftime :: (__time1: time_t, __time0: time_t) -> float64 #foreign libc`
- [OS == .LINUX] `mktime :: (__tp: *tm) -> time_t #foreign libc`
- [OS == .LINUX] `strftime :: (__s: *u8, __maxsize: u64, __format: *u8, __tp: *tm) -> u64 #foreign libc`
- [OS == .LINUX] `strptime :: (__s: *u8, __fmt: *u8, __tp: *tm) -> *u8 #foreign libc`
- [OS == .LINUX] `strftime_l :: (__s: *u8, __maxsize: u64, __format: *u8, __tp: *tm, __loc: locale_t) -> u64 #foreign libc`
- [OS == .LINUX] `strptime_l :: (__s: *u8, __fmt: *u8, __tp: *tm, __loc: locale_t) -> *u8 #foreign libc`
- [OS == .LINUX] `gmtime :: (__timer: *time_t) -> *tm #foreign libc`
- [OS == .LINUX] `localtime :: (__timer: *time_t) -> *tm #foreign libc`
- [OS == .LINUX] `gmtime_r :: (__timer: *time_t, __tp: *tm) -> *tm #foreign libc`
- [OS == .LINUX] `localtime_r :: (__timer: *time_t, __tp: *tm) -> *tm #foreign libc`
- [OS == .LINUX] `asctime :: (__tp: *tm) -> *u8 #foreign libc`
- [OS == .LINUX] `ctime :: (__timer: *time_t) -> *u8 #foreign libc`
- [OS == .LINUX] `asctime_r :: (__tp: *tm, __buf: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `ctime_r :: (__timer: *time_t, __buf: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `__tzname: [2] *u8 #elsewhere libc`
- [OS == .LINUX] `__daylight: s32 #elsewhere libc`
- [OS == .LINUX] `__timezone: s64 #elsewhere libc`
- [OS == .LINUX] `tzname: [2] *u8 #elsewhere libc`
- [OS == .LINUX] `tzset :: () -> void #foreign libc`
- [OS == .LINUX] `daylight: s32 #elsewhere libc`
- [OS == .LINUX] `timegm :: (__tp: *tm) -> time_t #foreign libc`
- [OS == .LINUX] `timelocal :: (__tp: *tm) -> time_t #foreign libc`
- [OS == .LINUX] `dysize :: (__year: s32) -> s32 #foreign libc`
- [OS == .LINUX] `nanosleep :: (__requested_time: *timespec, __remaining: *timespec) -> s32 #foreign libc`
- [OS == .LINUX] `clock_getres :: (__clock_id: clockid_t, __res: *timespec) -> s32 #foreign libc`
- [OS == .LINUX] `clock_gettime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc`
- [OS == .LINUX] `clock_settime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc`
- [OS == .LINUX] `clock_nanosleep :: (__clock_id: clockid_t, __flags: s32, __req: *timespec, __rem: *timespec) -> s32 #foreign libc`
- [OS == .LINUX] `clock_getcpuclockid :: (__pid: pid_t, __clock_id: *clockid_t) -> s32 #foreign libc`
- [OS == .LINUX] `timer_create :: (__clock_id: clockid_t, __evp: *sigevent, __timerid: *timer_t) -> s32 #foreign librt`
- [OS == .LINUX] `timer_delete :: (__timerid: timer_t) -> s32 #foreign librt`
- [OS == .LINUX] `timer_settime :: (__timerid: timer_t, __flags: s32, __value: *itimerspec, __ovalue: *itimerspec) -> s32 #foreign librt`
- [OS == .LINUX] `timer_gettime :: (__timerid: timer_t, __value: *itimerspec) -> s32 #foreign librt`
- [OS == .LINUX] `timer_getoverrun :: (__timerid: timer_t) -> s32 #foreign librt`
- [OS == .LINUX] `timespec_get :: (__ts: *timespec, __base: s32) -> s32 #foreign libc`
- [OS == .LINUX] `getdate_err: s32 #elsewhere libc`
- [OS == .LINUX] `getdate :: (__string: *u8) -> *tm #foreign libc`
- [OS == .LINUX] `getdate_r :: (__string: *u8, __resbufp: *tm) -> s32 #foreign libc`
- [OS == .LINUX] `access :: (__name: *u8, __type: s32) -> s32 #foreign libc`
- [OS == .LINUX] `euidaccess :: (__name: *u8, __type: s32) -> s32 #foreign libc`
- [OS == .LINUX] `eaccess :: (__name: *u8, __type: s32) -> s32 #foreign libc`
- [OS == .LINUX] `faccessat :: (__fd: s32, __file: *u8, __type: s32, __flag: s32) -> s32 #foreign libc`
- [OS == .LINUX] `lseek :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc`
- [OS == .LINUX] `lseek64 :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc`
- [OS == .LINUX] `close :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `read :: (__fd: s32, __buf: *void, __nbytes: u64) -> s64 #foreign libc`
- [OS == .LINUX] `write :: (__fd: s32, __buf: *void, __n: u64) -> s64 #foreign libc`
- [OS == .LINUX] `pread :: (__fd: s32, __buf: *void, __nbytes: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `pwrite :: (__fd: s32, __buf: *void, __n: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `pread64 :: (__fd: s32, __buf: *void, __nbytes: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `pwrite64 :: (__fd: s32, __buf: *void, __n: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `pipe :: (__pipedes: *[2] s32) -> s32 #foreign libc`
- [OS == .LINUX] `pipe2 :: (__pipedes: *[2] s32, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `alarm :: (__seconds: u32) -> u32 #foreign libc`
- [OS == .LINUX] `sleep :: (__seconds: u32) -> u32 #foreign libc`
- [OS == .LINUX] `ualarm :: (__value: u32, __interval: u32) -> u32 #foreign libc`
- [OS == .LINUX] `usleep :: (__useconds: u32) -> s32 #foreign libc`
- [OS == .LINUX] `chown :: (__file: *u8, __owner: u32, __group: u32) -> s32 #foreign libc`
- [OS == .LINUX] `fchown :: (__fd: s32, __owner: u32, __group: u32) -> s32 #foreign libc`
- [OS == .LINUX] `lchown :: (__file: *u8, __owner: u32, __group: u32) -> s32 #foreign libc`
- [OS == .LINUX] `fchownat :: (__fd: s32, __file: *u8, __owner: u32, __group: u32, __flag: s32) -> s32 #foreign libc`
- [OS == .LINUX] `chdir :: (__path: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `fchdir :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `getcwd :: (__buf: *u8, __size: u64) -> *u8 #foreign libc`
- [OS == .LINUX] `get_current_dir_name :: () -> *u8 #foreign libc`
- [OS == .LINUX] `getwd :: (__buf: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `dup :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `dup2 :: (__fd: s32, __fd2: s32) -> s32 #foreign libc`
- [OS == .LINUX] `dup3 :: (__fd: s32, __fd2: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `__environ: **u8 #elsewhere libc`
- [OS == .LINUX] `environ: **u8 #elsewhere libc`
- [OS == .LINUX] `execve :: (__path: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .LINUX] `fexecve :: (__fd: s32, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .LINUX] `execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .LINUX] `execle :: (__path: *u8, __arg: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `execl :: (__path: *u8, __arg: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .LINUX] `execlp :: (__file: *u8, __arg: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `execvpe :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .LINUX] `nice :: (__inc: s32) -> s32 #foreign libc`
- [OS == .LINUX] `_exit :: (__status: s32) -> void #foreign libc`
- [OS == .LINUX] `PC :: enum u32`
- [OS == .LINUX] `using _SC_definitions :: enum s32`
- [OS == .LINUX] `CS :: enum u32`
- [OS == .LINUX] `pathconf :: (__path: *u8, __name: PC) -> s64 #foreign libc`
- [OS == .LINUX] `fpathconf :: (__fd: s32, __name: PC) -> s64 #foreign libc`
- [OS == .LINUX] `sysconf :: (name: _SC_definitions) -> s64 #foreign libc`
- [OS == .LINUX] `confstr :: (__name: s32, __buf: *u8, __len: u64) -> u64 #foreign libc`
- [OS == .LINUX] `getpid :: () -> s32 #foreign libc`
- [OS == .LINUX] `getppid :: () -> s32 #foreign libc`
- [OS == .LINUX] `getpgrp :: () -> s32 #foreign libc`
- [OS == .LINUX] `__getpgid :: (__pid: s32) -> s32 #foreign libc`
- [OS == .LINUX] `getpgid :: (__pid: s32) -> s32 #foreign libc`
- [OS == .LINUX] `setpgid :: (__pid: s32, __pgid: s32) -> s32 #foreign libc`
- [OS == .LINUX] `setpgrp :: () -> s32 #foreign libc`
- [OS == .LINUX] `setsid :: () -> s32 #foreign libc`
- [OS == .LINUX] `getsid :: (__pid: s32) -> s32 #foreign libc`
- [OS == .LINUX] `getuid :: () -> u32 #foreign libc`
- [OS == .LINUX] `geteuid :: () -> u32 #foreign libc`
- [OS == .LINUX] `getgid :: () -> u32 #foreign libc`
- [OS == .LINUX] `getegid :: () -> u32 #foreign libc`
- [OS == .LINUX] `getgroups :: (__size: s32, __list: *u32) -> s32 #foreign libc`
- [OS == .LINUX] `group_member :: (__gid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `setuid :: (__uid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `setreuid :: (__ruid: u32, __euid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `seteuid :: (__uid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `setgid :: (__gid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `setregid :: (__rgid: u32, __egid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `setegid :: (__gid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `getresuid :: (__ruid: *u32, __euid: *u32, __suid: *u32) -> s32 #foreign libc`
- [OS == .LINUX] `getresgid :: (__rgid: *u32, __egid: *u32, __sgid: *u32) -> s32 #foreign libc`
- [OS == .LINUX] `setresuid :: (__ruid: u32, __euid: u32, __suid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `setresgid :: (__rgid: u32, __egid: u32, __sgid: u32) -> s32 #foreign libc`
- [OS == .LINUX] `fork :: () -> s32 #foreign libc`
- [OS == .LINUX] `vfork :: () -> s32 #foreign libc`
- [OS == .LINUX] `ttyname :: (__fd: s32) -> *u8 #foreign libc`
- [OS == .LINUX] `ttyname_r :: (__fd: s32, __buf: *u8, __buflen: u64) -> s32 #foreign libc`
- [OS == .LINUX] `isatty :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `ttyslot :: () -> s32 #foreign libc`
- [OS == .LINUX] `link :: (__from: *u8, __to: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `linkat :: (__fromfd: s32, __from: *u8, __tofd: s32, __to: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `symlink :: (__from: *u8, __to: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `readlink :: (__path: *u8, __buf: *u8, __len: u64) -> s64 #foreign libc`
- [OS == .LINUX] `symlinkat :: (__from: *u8, __tofd: s32, __to: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `readlinkat :: (__fd: s32, __path: *u8, __buf: *u8, __len: u64) -> s64 #foreign libc`
- [OS == .LINUX] `unlink :: (__name: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `unlinkat :: (__fd: s32, __name: *u8, __flag: s32) -> s32 #foreign libc`
- [OS == .LINUX] `rmdir :: (__path: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `tcgetpgrp :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `tcsetpgrp :: (__fd: s32, __pgrp_id: s32) -> s32 #foreign libc`
- [OS == .LINUX] `getlogin :: () -> *u8 #foreign libc`
- [OS == .LINUX] `getlogin_r :: (__name: *u8, __name_len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `setlogin :: (__name: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `gethostname :: (__name: *u8, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `sethostname :: (__name: *u8, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `sethostid :: (__id: s64) -> s32 #foreign libc`
- [OS == .LINUX] `getdomainname :: (__name: *u8, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `setdomainname :: (__name: *u8, __len: u64) -> s32 #foreign libc`
- [OS == .LINUX] `vhangup :: () -> s32 #foreign libc`
- [OS == .LINUX] `revoke :: (__file: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `profil :: (__sample_buffer: *u16, __size: u64, __offset: u64, __scale: u32) -> s32 #foreign libc`
- [OS == .LINUX] `acct :: (__name: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `getusershell :: () -> *u8 #foreign libc`
- [OS == .LINUX] `endusershell :: () -> void #foreign libc`
- [OS == .LINUX] `setusershell :: () -> void #foreign libc`
- [OS == .LINUX] `daemon :: (__nochdir: s32, __noclose: s32) -> s32 #foreign libc`
- [OS == .LINUX] `chroot :: (__path: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `getpass :: (__prompt: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `fsync :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `syncfs :: (__fd: s32) -> s32 #foreign libc`
- [OS == .LINUX] `gethostid :: () -> s64 #foreign libc`
- [OS == .LINUX] `sync :: () -> void #foreign libc`
- [OS == .LINUX] `getpagesize :: () -> s32 #foreign libc`
- [OS == .LINUX] `getdtablesize :: () -> s32 #foreign libc`
- [OS == .LINUX] `truncate :: (__file: *u8, __length: s64) -> s32 #foreign libc`
- [OS == .LINUX] `truncate64 :: (__file: *u8, __length: s64) -> s32 #foreign libc`
- [OS == .LINUX] `ftruncate :: (__fd: s32, __length: s64) -> s32 #foreign libc`
- [OS == .LINUX] `ftruncate64 :: (__fd: s32, __length: s64) -> s32 #foreign libc`
- [OS == .LINUX] `brk :: (__addr: *void) -> s32 #foreign libc`
- [OS == .LINUX] `sbrk :: (__delta: s64) -> *void #foreign libc`
- [OS == .LINUX] `copy_file_range :: (__infd: s32, __pinoff: *s64, __outfd: s32, __poutoff: *s64, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .LINUX] `fdatasync :: (__fildes: s32) -> s32 #foreign libc`
- [OS == .LINUX] `swab :: (__from: *void, __to: *void, __n: s64) -> void #foreign libc`
- [OS == .LINUX] `getentropy :: (__buffer: *void, __length: u64) -> s32 #foreign libc`
- [OS == .LINUX] `iovec :: struct`
- [OS == .LINUX] `readv :: (__fd: s32, __iovec: *iovec, __count: s32) -> s64 #foreign libc`
- [OS == .LINUX] `writev :: (__fd: s32, __iovec: *iovec, __count: s32) -> s64 #foreign libc`
- [OS == .LINUX] `preadv :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `pwritev :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `preadv64 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `pwritev64 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc`
- [OS == .LINUX] `preadv2 :: (__fp: s32, __iovec: *iovec, __count: s32, __offset: s64, ___flags: s32) -> s64 #foreign libc`
- [OS == .LINUX] `pwritev2 :: (__fd: s32, __iodev: *iovec, __count: s32, __offset: s64, __flags: s32) -> s64 #foreign libc`
- [OS == .LINUX] `preadv64v2 :: (__fp: s32, __iovec: *iovec, __count: s32, __offset: s64, ___flags: s32) -> s64 #foreign libc`
- [OS == .LINUX] `pwritev64v2 :: (__fd: s32, __iodev: *iovec, __count: s32, __offset: s64, __flags: s32) -> s64 #foreign libc`
- [OS == .LINUX] `_GNU_SOURCE :: 1`
- [OS == .LINUX] `_IOFBF :: 0`
- [OS == .LINUX] `_IOLBF :: 1`
- [OS == .LINUX] `_IONBF :: 2`
- [OS == .LINUX] `BUFSIZ :: 8192`
- [OS == .LINUX] `EOF :: -1`
- [OS == .LINUX] `SEEK_SET :: 0`
- [OS == .LINUX] `SEEK_CUR :: 1`
- [OS == .LINUX] `SEEK_END :: 2`
- [OS == .LINUX] `SEEK_DATA :: 3`
- [OS == .LINUX] `SEEK_HOLE :: 4`
- [OS == .LINUX] `P_tmpdir :: "/tmp"`
- [OS == .LINUX] `RENAME_NOREPLACE :: 1 << 0`
- [OS == .LINUX] `RENAME_EXCHANGE :: 1 << 1`
- [OS == .LINUX] `RENAME_WHITEOUT :: 1 << 2`
- [OS == .LINUX] `__O_LARGEFILE :: 0`
- [OS == .LINUX] `F_GETLK64 :: 5`
- [OS == .LINUX] `F_SETLK64 :: 6`
- [OS == .LINUX] `F_SETLKW64 :: 7`
- [OS == .LINUX] `O_ACCMODE :: 0x3`
- [OS == .LINUX] `O_RDONLY :: 0x0`
- [OS == .LINUX] `O_WRONLY :: 0x1`
- [OS == .LINUX] `O_RDWR :: 0x2`
- [OS == .LINUX] `O_CREAT :: 0x40`
- [OS == .LINUX] `O_EXCL :: 0x80`
- [OS == .LINUX] `O_NOCTTY :: 0x100`
- [OS == .LINUX] `O_TRUNC :: 0x200`
- [OS == .LINUX] `O_APPEND :: 0x400`
- [OS == .LINUX] `O_NONBLOCK :: 0x800`
- [OS == .LINUX] `O_NDELAY :: O_NONBLOCK`
- [OS == .LINUX] `O_SYNC :: 0x101000`
- [OS == .LINUX] `O_FSYNC :: O_SYNC`
- [OS == .LINUX] `O_ASYNC :: 0x2000`
- [OS == .LINUX] `__O_DIRECTORY :: 0x10000`
- [OS == .LINUX] `__O_NOFOLLOW :: 0x20000`
- [OS == .LINUX] `__O_CLOEXEC :: 0x80000`
- [OS == .LINUX] `__O_DIRECT :: 0x4000`
- [OS == .LINUX] `__O_NOATIME :: 0x40000`
- [OS == .LINUX] `__O_PATH :: 0x200000`
- [OS == .LINUX] `__O_DSYNC :: 0x1000`
- [OS == .LINUX] `__O_TMPFILE :: 0x400000 | __O_DIRECTORY`
- [OS == .LINUX] `F_GETLK :: 5`
- [OS == .LINUX] `F_SETLK :: 6`
- [OS == .LINUX] `F_SETLKW :: 7`
- [OS == .LINUX] `F_OFD_GETLK :: 36`
- [OS == .LINUX] `F_OFD_SETLK :: 37`
- [OS == .LINUX] `F_OFD_SETLKW :: 38`
- [OS == .LINUX] `O_LARGEFILE :: __O_LARGEFILE`
- [OS == .LINUX] `O_DIRECTORY :: __O_DIRECTORY`
- [OS == .LINUX] `O_NOFOLLOW :: __O_NOFOLLOW`
- [OS == .LINUX] `O_CLOEXEC :: __O_CLOEXEC`
- [OS == .LINUX] `O_DIRECT :: __O_DIRECT`
- [OS == .LINUX] `O_NOATIME :: __O_NOATIME`
- [OS == .LINUX] `O_PATH :: __O_PATH`
- [OS == .LINUX] `O_TMPFILE :: __O_TMPFILE`
- [OS == .LINUX] `O_DSYNC :: __O_DSYNC`
- [OS == .LINUX] `O_RSYNC :: O_SYNC`
- [OS == .LINUX] `F_DUPFD :: 0`
- [OS == .LINUX] `F_GETFD :: 1`
- [OS == .LINUX] `F_SETFD :: 2`
- [OS == .LINUX] `F_GETFL :: 3`
- [OS == .LINUX] `F_SETFL :: 4`
- [OS == .LINUX] `__F_SETOWN :: 8`
- [OS == .LINUX] `__F_GETOWN :: 9`
- [OS == .LINUX] `F_SETOWN :: __F_SETOWN`
- [OS == .LINUX] `F_GETOWN :: __F_GETOWN`
- [OS == .LINUX] `__F_SETSIG :: 10`
- [OS == .LINUX] `__F_GETSIG :: 11`
- [OS == .LINUX] `__F_SETOWN_EX :: 15`
- [OS == .LINUX] `__F_GETOWN_EX :: 16`
- [OS == .LINUX] `F_SETSIG :: __F_SETSIG`
- [OS == .LINUX] `F_GETSIG :: __F_GETSIG`
- [OS == .LINUX] `F_SETOWN_EX :: __F_SETOWN_EX`
- [OS == .LINUX] `F_GETOWN_EX :: __F_GETOWN_EX`
- [OS == .LINUX] `F_SETLEASE :: 1024`
- [OS == .LINUX] `F_GETLEASE :: 1025`
- [OS == .LINUX] `F_NOTIFY :: 1026`
- [OS == .LINUX] `F_SETPIPE_SZ :: 1031`
- [OS == .LINUX] `F_GETPIPE_SZ :: 1032`
- [OS == .LINUX] `F_ADD_SEALS :: 1033`
- [OS == .LINUX] `F_GET_SEALS :: 1034`
- [OS == .LINUX] `F_GET_RW_HINT :: 1035`
- [OS == .LINUX] `F_SET_RW_HINT :: 1036`
- [OS == .LINUX] `F_GET_FILE_RW_HINT :: 1037`
- [OS == .LINUX] `F_SET_FILE_RW_HINT :: 1038`
- [OS == .LINUX] `F_DUPFD_CLOEXEC :: 1030`
- [OS == .LINUX] `FD_CLOEXEC :: 1`
- [OS == .LINUX] `F_RDLCK :: 0`
- [OS == .LINUX] `F_WRLCK :: 1`
- [OS == .LINUX] `F_UNLCK :: 2`
- [OS == .LINUX] `F_EXLCK :: 4`
- [OS == .LINUX] `F_SHLCK :: 8`
- [OS == .LINUX] `LOCK_SH :: 1`
- [OS == .LINUX] `LOCK_EX :: 2`
- [OS == .LINUX] `LOCK_NB :: 4`
- [OS == .LINUX] `LOCK_UN :: 8`
- [OS == .LINUX] `LOCK_MAND :: 32`
- [OS == .LINUX] `LOCK_READ :: 64`
- [OS == .LINUX] `LOCK_WRITE :: 128`
- [OS == .LINUX] `LOCK_RW :: 192`
- [OS == .LINUX] `DN_ACCESS :: 0x00000001`
- [OS == .LINUX] `DN_MODIFY :: 0x00000002`
- [OS == .LINUX] `DN_CREATE :: 0x00000004`
- [OS == .LINUX] `DN_DELETE :: 0x00000008`
- [OS == .LINUX] `DN_RENAME :: 0x00000010`
- [OS == .LINUX] `DN_ATTRIB :: 0x00000020`
- [OS == .LINUX] `DN_MULTISHOT :: 0x80000000`
- [OS == .LINUX] `F_SEAL_SEAL :: 0x0001`
- [OS == .LINUX] `F_SEAL_SHRINK :: 0x0002`
- [OS == .LINUX] `F_SEAL_GROW :: 0x0004`
- [OS == .LINUX] `F_SEAL_WRITE :: 0x0008`
- [OS == .LINUX] `F_SEAL_FUTURE_WRITE :: 0x0010`
- [OS == .LINUX] `RWF_WRITE_LIFE_NOT_SET :: 0`
- [OS == .LINUX] `RWH_WRITE_LIFE_NONE :: 1`
- [OS == .LINUX] `RWH_WRITE_LIFE_SHORT :: 2`
- [OS == .LINUX] `RWH_WRITE_LIFE_MEDIUM :: 3`
- [OS == .LINUX] `RWH_WRITE_LIFE_LONG :: 4`
- [OS == .LINUX] `RWH_WRITE_LIFE_EXTREME :: 5`
- [OS == .LINUX] `FAPPEND :: O_APPEND`
- [OS == .LINUX] `FFSYNC :: O_FSYNC`
- [OS == .LINUX] `FASYNC :: O_ASYNC`
- [OS == .LINUX] `FNONBLOCK :: O_NONBLOCK`
- [OS == .LINUX] `FNDELAY :: O_NDELAY`
- [OS == .LINUX] `__POSIX_FADV_DONTNEED :: 4`
- [OS == .LINUX] `__POSIX_FADV_NOREUSE :: 5`
- [OS == .LINUX] `POSIX_FADV_NORMAL :: 0`
- [OS == .LINUX] `POSIX_FADV_RANDOM :: 1`
- [OS == .LINUX] `POSIX_FADV_SEQUENTIAL :: 2`
- [OS == .LINUX] `POSIX_FADV_WILLNEED :: 3`
- [OS == .LINUX] `POSIX_FADV_DONTNEED :: __POSIX_FADV_DONTNEED`
- [OS == .LINUX] `POSIX_FADV_NOREUSE :: __POSIX_FADV_NOREUSE`
- [OS == .LINUX] `SYNC_FILE_RANGE_WAIT_BEFORE :: 1`
- [OS == .LINUX] `SYNC_FILE_RANGE_WRITE :: 2`
- [OS == .LINUX] `SYNC_FILE_RANGE_WAIT_AFTER :: 4`
- [OS == .LINUX] `SYNC_FILE_RANGE_WRITE_AND_WAIT :: SYNC_FILE_RANGE_WRITE | SYNC_FILE_RANGE_WAIT_BEFORE | SYNC_FILE_RANGE_WAIT_AFTER`
- [OS == .LINUX] `SPLICE_F_MOVE :: 1`
- [OS == .LINUX] `SPLICE_F_NONBLOCK :: 2`
- [OS == .LINUX] `SPLICE_F_MORE :: 4`
- [OS == .LINUX] `SPLICE_F_GIFT :: 8`
- [OS == .LINUX] `MAX_HANDLE_SZ :: 128`
- [OS == .LINUX] `AT_FDCWD :: -100`
- [OS == .LINUX] `AT_SYMLINK_NOFOLLOW :: 0x100`
- [OS == .LINUX] `AT_REMOVEDIR :: 0x200`
- [OS == .LINUX] `AT_SYMLINK_FOLLOW :: 0x400`
- [OS == .LINUX] `AT_NO_AUTOMOUNT :: 0x800`
- [OS == .LINUX] `AT_EMPTY_PATH :: 0x1000`
- [OS == .LINUX] `AT_STATX_SYNC_TYPE :: 0x6000`
- [OS == .LINUX] `AT_STATX_SYNC_AS_STAT :: 0x0000`
- [OS == .LINUX] `AT_STATX_FORCE_SYNC :: 0x2000`
- [OS == .LINUX] `AT_STATX_DONT_SYNC :: 0x4000`
- [OS == .LINUX] `AT_RECURSIVE :: 0x8000`
- [OS == .LINUX] `AT_EACCESS :: 0x200`
- [OS == .LINUX] `_STAT_VER_KERNEL :: 0`
- [OS == .LINUX] `_STAT_VER_LINUX :: 1`
- [OS == .LINUX] `_MKNOD_VER_LINUX :: 0`
- [OS == .LINUX] `_STAT_VER :: _STAT_VER_LINUX`
- [OS == .LINUX] `__S_IFMT :: 0xf000`
- [OS == .LINUX] `__S_IFDIR :: 0x4000`
- [OS == .LINUX] `__S_IFCHR :: 0x2000`
- [OS == .LINUX] `__S_IFBLK :: 0x6000`
- [OS == .LINUX] `__S_IFREG :: 0x8000`
- [OS == .LINUX] `__S_IFIFO :: 0x1000`
- [OS == .LINUX] `__S_IFLNK :: 0xa000`
- [OS == .LINUX] `__S_IFSOCK :: 0xc000`
- [OS == .LINUX] `__S_ISUID :: 0x800`
- [OS == .LINUX] `__S_ISGID :: 0x400`
- [OS == .LINUX] `__S_ISVTX :: 0x200`
- [OS == .LINUX] `__S_IREAD :: 0x100`
- [OS == .LINUX] `__S_IWRITE :: 0x80`
- [OS == .LINUX] `__S_IEXEC :: 0x40`
- [OS == .LINUX] `UTIME_NOW :: (1 << 30) - 1`
- [OS == .LINUX] `UTIME_OMIT :: (1 << 30) - 2`
- [OS == .LINUX] `S_IFMT :: __S_IFMT`
- [OS == .LINUX] `S_IFDIR :: __S_IFDIR`
- [OS == .LINUX] `S_IFCHR :: __S_IFCHR`
- [OS == .LINUX] `S_IFBLK :: __S_IFBLK`
- [OS == .LINUX] `S_IFREG :: __S_IFREG`
- [OS == .LINUX] `S_IFIFO :: __S_IFIFO`
- [OS == .LINUX] `S_IFLNK :: __S_IFLNK`
- [OS == .LINUX] `S_IFSOCK :: __S_IFSOCK`
- [OS == .LINUX] `S_ISUID :: __S_ISUID`
- [OS == .LINUX] `S_ISGID :: __S_ISGID`
- [OS == .LINUX] `S_ISVTX :: __S_ISVTX`
- [OS == .LINUX] `S_IRUSR :: __S_IREAD`
- [OS == .LINUX] `S_IWUSR :: __S_IWRITE`
- [OS == .LINUX] `S_IXUSR :: __S_IEXEC`
- [OS == .LINUX] `S_IRWXU :: __S_IREAD|__S_IWRITE|__S_IEXEC`
- [OS == .LINUX] `S_IRGRP :: S_IRUSR >> 3`
- [OS == .LINUX] `S_IWGRP :: S_IWUSR >> 3`
- [OS == .LINUX] `S_IXGRP :: S_IXUSR >> 3`
- [OS == .LINUX] `S_IRWXG :: S_IRWXU >> 3`
- [OS == .LINUX] `S_IROTH :: S_IRGRP >> 3`
- [OS == .LINUX] `S_IWOTH :: S_IWGRP >> 3`
- [OS == .LINUX] `S_IXOTH :: S_IXGRP >> 3`
- [OS == .LINUX] `S_IRWXO :: S_IRWXG >> 3`
- [OS == .LINUX] `R_OK :: 4`
- [OS == .LINUX] `W_OK :: 2`
- [OS == .LINUX] `X_OK :: 1`
- [OS == .LINUX] `F_OK :: 0`
- [OS == .LINUX] `F_ULOCK :: 0`
- [OS == .LINUX] `F_LOCK :: 1`
- [OS == .LINUX] `F_TLOCK :: 2`
- [OS == .LINUX] `F_TEST :: 3`
- [OS == .LINUX] `S_IREAD :: S_IRUSR`
- [OS == .LINUX] `S_IWRITE :: S_IWUSR`
- [OS == .LINUX] `S_IEXEC :: S_IXUSR`
- [OS == .LINUX] `ACCESSPERMS :: S_IRWXU|S_IRWXG|S_IRWXO`
- [OS == .LINUX] `ALLPERMS :: S_ISUID|S_ISGID|S_ISVTX|S_IRWXU|S_IRWXG|S_IRWXO`
- [OS == .LINUX] `DEFFILEMODE :: S_IRUSR|S_IWUSR|S_IRGRP|S_IWGRP|S_IROTH|S_IWOTH`
- [OS == .LINUX] `S_BLKSIZE :: 512`
- [OS == .LINUX] `_MKNOD_VER :: 0`
- [OS == .LINUX] `STATX_TYPE :: 0x00000001`
- [OS == .LINUX] `STATX_MODE :: 0x00000002`
- [OS == .LINUX] `STATX_NLINK :: 0x00000004`
- [OS == .LINUX] `STATX_UID :: 0x00000008`
- [OS == .LINUX] `STATX_GID :: 0x00000010`
- [OS == .LINUX] `STATX_ATIME :: 0x00000020`
- [OS == .LINUX] `STATX_MTIME :: 0x00000040`
- [OS == .LINUX] `STATX_CTIME :: 0x00000080`
- [OS == .LINUX] `STATX_INO :: 0x00000100`
- [OS == .LINUX] `STATX_SIZE :: 0x00000200`
- [OS == .LINUX] `STATX_BLOCKS :: 0x00000400`
- [OS == .LINUX] `STATX_BASIC_STATS :: 0x000007ff`
- [OS == .LINUX] `STATX_BTIME :: 0x00000800`
- [OS == .LINUX] `STATX_ALL :: 0x00000fff`
- [OS == .LINUX] `STATX__RESERVED :: 0x80000000`
- [OS == .LINUX] `STATX_ATTR_COMPRESSED :: 0x00000004`
- [OS == .LINUX] `STATX_ATTR_IMMUTABLE :: 0x00000010`
- [OS == .LINUX] `STATX_ATTR_APPEND :: 0x00000020`
- [OS == .LINUX] `STATX_ATTR_NODUMP :: 0x00000040`
- [OS == .LINUX] `STATX_ATTR_ENCRYPTED :: 0x00000800`
- [OS == .LINUX] `STATX_ATTR_AUTOMOUNT :: 0x00001000`
- [OS == .LINUX] `TCGETS :: 0x5401`
- [OS == .LINUX] `TCSETS :: 0x5402`
- [OS == .LINUX] `TCSETSW :: 0x5403`
- [OS == .LINUX] `TCSETSF :: 0x5404`
- [OS == .LINUX] `TCGETA :: 0x5405`
- [OS == .LINUX] `TCSETA :: 0x5406`
- [OS == .LINUX] `TCSETAW :: 0x5407`
- [OS == .LINUX] `TCSETAF :: 0x5408`
- [OS == .LINUX] `TCSBRK :: 0x5409`
- [OS == .LINUX] `TCXONC :: 0x540A`
- [OS == .LINUX] `TCFLSH :: 0x540B`
- [OS == .LINUX] `TIOCEXCL :: 0x540C`
- [OS == .LINUX] `TIOCNXCL :: 0x540D`
- [OS == .LINUX] `TIOCSCTTY :: 0x540E`
- [OS == .LINUX] `TIOCGPGRP :: 0x540F`
- [OS == .LINUX] `TIOCSPGRP :: 0x5410`
- [OS == .LINUX] `TIOCOUTQ :: 0x5411`
- [OS == .LINUX] `TIOCSTI :: 0x5412`
- [OS == .LINUX] `TIOCGWINSZ :: 0x5413`
- [OS == .LINUX] `TIOCSWINSZ :: 0x5414`
- [OS == .LINUX] `TIOCMGET :: 0x5415`
- [OS == .LINUX] `TIOCMBIS :: 0x5416`
- [OS == .LINUX] `TIOCMBIC :: 0x5417`
- [OS == .LINUX] `TIOCMSET :: 0x5418`
- [OS == .LINUX] `TIOCGSOFTCAR :: 0x5419`
- [OS == .LINUX] `TIOCSSOFTCAR :: 0x541A`
- [OS == .LINUX] `FIONREAD :: 0x541B`
- [OS == .LINUX] `TIOCINQ :: FIONREAD`
- [OS == .LINUX] `TIOCLINUX :: 0x541C`
- [OS == .LINUX] `TIOCCONS :: 0x541D`
- [OS == .LINUX] `TIOCGSERIAL :: 0x541E`
- [OS == .LINUX] `TIOCSSERIAL :: 0x541F`
- [OS == .LINUX] `TIOCPKT :: 0x5420`
- [OS == .LINUX] `FIONBIO :: 0x5421`
- [OS == .LINUX] `TIOCNOTTY :: 0x5422`
- [OS == .LINUX] `TIOCSETD :: 0x5423`
- [OS == .LINUX] `TIOCGETD :: 0x5424`
- [OS == .LINUX] `TCSBRKP :: 0x5425`
- [OS == .LINUX] `TIOCSBRK :: 0x5427`
- [OS == .LINUX] `TIOCCBRK :: 0x5428`
- [OS == .LINUX] `TIOCGSID :: 0x5429`
- [OS == .LINUX] `TIOCGRS485 :: 0x542E`
- [OS == .LINUX] `TIOCSRS485 :: 0x542F`
- [OS == .LINUX] `TCGETX :: 0x5432`
- [OS == .LINUX] `TCSETX :: 0x5433`
- [OS == .LINUX] `TCSETXF :: 0x5434`
- [OS == .LINUX] `TCSETXW :: 0x5435`
- [OS == .LINUX] `TIOCVHANGUP :: 0x5437`
- [OS == .LINUX] `FIONCLEX :: 0x5450`
- [OS == .LINUX] `FIOCLEX :: 0x5451`
- [OS == .LINUX] `FIOASYNC :: 0x5452`
- [OS == .LINUX] `TIOCSERCONFIG :: 0x5453`
- [OS == .LINUX] `TIOCSERGWILD :: 0x5454`
- [OS == .LINUX] `TIOCSERSWILD :: 0x5455`
- [OS == .LINUX] `TIOCGLCKTRMIOS :: 0x5456`
- [OS == .LINUX] `TIOCSLCKTRMIOS :: 0x5457`
- [OS == .LINUX] `TIOCSERGSTRUCT :: 0x5458`
- [OS == .LINUX] `TIOCSERGETLSR :: 0x5459`
- [OS == .LINUX] `TIOCSERGETMULTI :: 0x545A`
- [OS == .LINUX] `TIOCSERSETMULTI :: 0x545B`
- [OS == .LINUX] `TIOCMIWAIT :: 0x545C`
- [OS == .LINUX] `TIOCGICOUNT :: 0x545D`
- [OS == .LINUX] `FIOQSIZE :: 0x5460`
- [OS == .LINUX] `TIOCPKT_DATA :: 0`
- [OS == .LINUX] `TIOCPKT_FLUSHREAD :: 1`
- [OS == .LINUX] `TIOCPKT_FLUSHWRITE :: 2`
- [OS == .LINUX] `TIOCPKT_STOP :: 4`
- [OS == .LINUX] `TIOCPKT_START :: 8`
- [OS == .LINUX] `TIOCPKT_NOSTOP :: 16`
- [OS == .LINUX] `TIOCPKT_DOSTOP :: 32`
- [OS == .LINUX] `TIOCPKT_IOCTL :: 64`
- [OS == .LINUX] `TIOCSER_TEMT :: 0x01`
- [OS == .LINUX] `SIOCADDRT :: 0x890B`
- [OS == .LINUX] `SIOCDELRT :: 0x890C`
- [OS == .LINUX] `SIOCRTMSG :: 0x890D`
- [OS == .LINUX] `SIOCGIFNAME :: 0x8910`
- [OS == .LINUX] `SIOCSIFLINK :: 0x8911`
- [OS == .LINUX] `SIOCGIFCONF :: 0x8912`
- [OS == .LINUX] `SIOCGIFFLAGS :: 0x8913`
- [OS == .LINUX] `SIOCSIFFLAGS :: 0x8914`
- [OS == .LINUX] `SIOCGIFADDR :: 0x8915`
- [OS == .LINUX] `SIOCSIFADDR :: 0x8916`
- [OS == .LINUX] `SIOCGIFDSTADDR :: 0x8917`
- [OS == .LINUX] `SIOCSIFDSTADDR :: 0x8918`
- [OS == .LINUX] `SIOCGIFBRDADDR :: 0x8919`
- [OS == .LINUX] `SIOCSIFBRDADDR :: 0x891a`
- [OS == .LINUX] `SIOCGIFNETMASK :: 0x891b`
- [OS == .LINUX] `SIOCSIFNETMASK :: 0x891c`
- [OS == .LINUX] `SIOCGIFMETRIC :: 0x891d`
- [OS == .LINUX] `SIOCSIFMETRIC :: 0x891e`
- [OS == .LINUX] `SIOCGIFMEM :: 0x891f`
- [OS == .LINUX] `SIOCSIFMEM :: 0x8920`
- [OS == .LINUX] `SIOCGIFMTU :: 0x8921`
- [OS == .LINUX] `SIOCSIFMTU :: 0x8922`
- [OS == .LINUX] `SIOCSIFNAME :: 0x8923`
- [OS == .LINUX] `SIOCSIFHWADDR :: 0x8924`
- [OS == .LINUX] `SIOCGIFENCAP :: 0x8925`
- [OS == .LINUX] `SIOCSIFENCAP :: 0x8926`
- [OS == .LINUX] `SIOCGIFHWADDR :: 0x8927`
- [OS == .LINUX] `SIOCGIFSLAVE :: 0x8929`
- [OS == .LINUX] `SIOCSIFSLAVE :: 0x8930`
- [OS == .LINUX] `SIOCADDMULTI :: 0x8931`
- [OS == .LINUX] `SIOCDELMULTI :: 0x8932`
- [OS == .LINUX] `SIOCGIFINDEX :: 0x8933`
- [OS == .LINUX] `SIOGIFINDEX :: SIOCGIFINDEX`
- [OS == .LINUX] `SIOCSIFPFLAGS :: 0x8934`
- [OS == .LINUX] `SIOCGIFPFLAGS :: 0x8935`
- [OS == .LINUX] `SIOCDIFADDR :: 0x8936`
- [OS == .LINUX] `SIOCSIFHWBROADCAST :: 0x8937`
- [OS == .LINUX] `SIOCGIFCOUNT :: 0x8938`
- [OS == .LINUX] `SIOCGIFBR :: 0x8940`
- [OS == .LINUX] `SIOCSIFBR :: 0x8941`
- [OS == .LINUX] `SIOCGIFTXQLEN :: 0x8942`
- [OS == .LINUX] `SIOCSIFTXQLEN :: 0x8943`
- [OS == .LINUX] `SIOCDARP :: 0x8953`
- [OS == .LINUX] `SIOCGARP :: 0x8954`
- [OS == .LINUX] `SIOCSARP :: 0x8955`
- [OS == .LINUX] `SIOCDRARP :: 0x8960`
- [OS == .LINUX] `SIOCGRARP :: 0x8961`
- [OS == .LINUX] `SIOCSRARP :: 0x8962`
- [OS == .LINUX] `SIOCGIFMAP :: 0x8970`
- [OS == .LINUX] `SIOCSIFMAP :: 0x8971`
- [OS == .LINUX] `SIOCADDDLCI :: 0x8980`
- [OS == .LINUX] `SIOCDELDLCI :: 0x8981`
- [OS == .LINUX] `SIOCDEVPRIVATE :: 0x89F0`
- [OS == .LINUX] `SIOCPROTOPRIVATE :: 0x89E0`
- [OS == .LINUX] `__mbstate_t :: struct`
- [OS == .LINUX] `_G_fpos_t :: struct`
- [OS == .LINUX] `_G_fpos64_t :: struct`
- [OS == .LINUX] `cookie_read_function_t :: #type (__cookie: *void, __buf: *u8, __nbytes: u64) -> s64 #c_call`
- [OS == .LINUX] `cookie_write_function_t :: #type (__cookie: *void, __buf: *u8, __nbytes: u64) -> s64 #c_call`
- [OS == .LINUX] `cookie_seek_function_t :: #type (__cookie: *void, __pos: *s64, __w: s32) -> s32 #c_call`
- [OS == .LINUX] `cookie_close_function_t :: #type (__cookie: *void) -> s32 #c_call`
- [OS == .LINUX] `_IO_cookie_io_functions_t :: struct`
- [OS == .LINUX] `cookie_io_functions_t :: _IO_cookie_io_functions_t`
- [OS == .LINUX] `fpos_t :: _G_fpos_t`
- [OS == .LINUX] `fpos64_t :: _G_fpos64_t`
- [OS == .LINUX] `stdin: *FILE #elsewhere libc`
- [OS == .LINUX] `stdout: *FILE #elsewhere libc`
- [OS == .LINUX] `stderr: *FILE #elsewhere libc`
- [OS == .LINUX] `_remove :: (__filename: *u8) -> s32 #foreign libc "remove"`
- [OS == .LINUX] `rename :: (__old: *u8, __new: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `renameat :: (__oldfd: s32, __old: *u8, __newfd: s32, __new: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `renameat2 :: (__oldfd: s32, __old: *u8, __newfd: s32, __new: *u8, __flags: u32) -> s32 #foreign libc`
- [OS == .LINUX] `tmpfile :: () -> *FILE #foreign libc`
- [OS == .LINUX] `tmpfile64 :: () -> *FILE #foreign libc`
- [OS == .LINUX] `tmpnam :: (__s: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `tmpnam_r :: (__s: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `tempnam :: (__dir: *u8, __pfx: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `fclose :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fflush :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fflush_unlocked :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fcloseall :: () -> s32 #foreign libc`
- [OS == .LINUX] `fopen :: (__filename: *u8, __modes: *u8) -> *FILE #foreign libc`
- [OS == .LINUX] `freopen :: (__filename: *u8, __modes: *u8, __stream: *FILE) -> *FILE #foreign libc`
- [OS == .LINUX] `fopen64 :: (__filename: *u8, __modes: *u8) -> *FILE #foreign libc`
- [OS == .LINUX] `freopen64 :: (__filename: *u8, __modes: *u8, __stream: *FILE) -> *FILE #foreign libc`
- [OS == .LINUX] `fdopen :: (__fd: s32, __modes: *u8) -> *FILE #foreign libc`
- [OS == .LINUX] `fopencookie :: (__magic_cookie: *void, __modes: *u8, __io_funcs: cookie_io_functions_t) -> *FILE #foreign libc`
- [OS == .LINUX] `fmemopen :: (__s: *void, __len: u64, __modes: *u8) -> *FILE #foreign libc`
- [OS == .LINUX] `open_memstream :: (__bufloc: **u8, __sizeloc: *u64) -> *FILE #foreign libc`
- [OS == .LINUX] `setbuf :: (__stream: *FILE, __buf: *u8) -> void #foreign libc`
- [OS == .LINUX] `setvbuf :: (__stream: *FILE, __buf: *u8, __modes: s32, __n: u64) -> s32 #foreign libc`
- [OS == .LINUX] `setbuffer :: (__stream: *FILE, __buf: *u8, __size: u64) -> void #foreign libc`
- [OS == .LINUX] `setlinebuf :: (__stream: *FILE) -> void #foreign libc`
- [OS == .LINUX] `fprintf :: (__stream: *FILE, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `printf :: (__format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `sprintf :: (__s: *u8, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `snprintf :: (__s: *u8, __maxlen: u64, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `__asprintf :: (__ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `asprintf :: (__ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `dprintf :: (__fd: s32, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `fscanf :: (__stream: *FILE, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `scanf :: (__format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `sscanf :: (__s: *u8, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `fgetc :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `getc :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `getchar :: () -> s32 #foreign libc`
- [OS == .LINUX] `getc_unlocked :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `getchar_unlocked :: () -> s32 #foreign libc`
- [OS == .LINUX] `fgetc_unlocked :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fputc :: (__c: s32, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `putc :: (__c: s32, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `putchar :: (__c: s32) -> s32 #foreign libc`
- [OS == .LINUX] `fputc_unlocked :: (__c: s32, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `putc_unlocked :: (__c: s32, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `putchar_unlocked :: (__c: s32) -> s32 #foreign libc`
- [OS == .LINUX] `getw :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `putw :: (__w: s32, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fgets :: (__s: *u8, __n: s32, __stream: *FILE) -> *u8 #foreign libc`
- [OS == .LINUX] `fgets_unlocked :: (__s: *u8, __n: s32, __stream: *FILE) -> *u8 #foreign libc`
- [OS == .LINUX] `__getdelim :: (__lineptr: **u8, __n: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc`
- [OS == .LINUX] `getdelim :: (__lineptr: **u8, __n: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc`
- [OS == .LINUX] `getline :: (__lineptr: **u8, __n: *u64, __stream: *FILE) -> s64 #foreign libc`
- [OS == .LINUX] `fputs :: (__s: *u8, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `puts :: (__s: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `ungetc :: (__c: s32, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fread :: (__ptr: *void, __size: u64, __n: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .LINUX] `fwrite :: (__ptr: *void, __size: u64, __n: u64, __s: *FILE) -> u64 #foreign libc`
- [OS == .LINUX] `fputs_unlocked :: (__s: *u8, __stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fread_unlocked :: (__ptr: *void, __size: u64, __n: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .LINUX] `fwrite_unlocked :: (__ptr: *void, __size: u64, __n: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .LINUX] `fseek :: (__stream: *FILE, __off: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .LINUX] `ftell :: (__stream: *FILE) -> s64 #foreign libc`
- [OS == .LINUX] `rewind :: (__stream: *FILE) -> void #foreign libc`
- [OS == .LINUX] `fseeko :: (__stream: *FILE, __off: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .LINUX] `ftello :: (__stream: *FILE) -> s64 #foreign libc`
- [OS == .LINUX] `fgetpos :: (__stream: *FILE, __pos: *fpos_t) -> s32 #foreign libc`
- [OS == .LINUX] `fsetpos :: (__stream: *FILE, __pos: *fpos_t) -> s32 #foreign libc`
- [OS == .LINUX] `fseeko64 :: (__stream: *FILE, __off: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .LINUX] `ftello64 :: (__stream: *FILE) -> s64 #foreign libc`
- [OS == .LINUX] `fgetpos64 :: (__stream: *FILE, __pos: *fpos64_t) -> s32 #foreign libc`
- [OS == .LINUX] `fsetpos64 :: (__stream: *FILE, __pos: *fpos64_t) -> s32 #foreign libc`
- [OS == .LINUX] `clearerr :: (__stream: *FILE) -> void #foreign libc`
- [OS == .LINUX] `feof :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `ferror :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `clearerr_unlocked :: (__stream: *FILE) -> void #foreign libc`
- [OS == .LINUX] `feof_unlocked :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `ferror_unlocked :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `perror :: (__s: *u8) -> void #foreign libc`
- [OS == .LINUX] `fileno :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `fileno_unlocked :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `popen :: (__command: *u8, __modes: *u8) -> *FILE #foreign libc`
- [OS == .LINUX] `pclose :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `ctermid :: (__s: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `cuserid :: (__s: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `obstack :: struct`
- [OS == .LINUX] `obstack_printf :: (__obstack: *obstack, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `flockfile :: (__stream: *FILE) -> void #foreign libc`
- [OS == .LINUX] `ftrylockfile :: (__stream: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `funlockfile :: (__stream: *FILE) -> void #foreign libc`
- [OS == .LINUX] `__uflow :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .LINUX] `__overflow :: (unknown0: *FILE, unknown1: s32) -> s32 #foreign libc`
- [OS == .LINUX] `flock :: struct`
- [OS == .LINUX] `flock64 :: struct`
- [OS == .LINUX] `__pid_type :: enum u32`
- [OS == .LINUX] `f_owner_ex :: struct`
- [OS == .LINUX] `file_handle :: struct`
- [OS == .LINUX] `readahead :: (__fd: s32, __offset: s64, __count: u64) -> s64 #foreign libc`
- [OS == .LINUX] `sync_file_range :: (__fd: s32, __offset: s64, __count: s64, __flags: u32) -> s32 #foreign libc`
- [OS == .LINUX] `vmsplice :: (__fdout: s32, __iov: *iovec, __count: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .LINUX] `splice :: (__fdin: s32, __offin: *s64, __fdout: s32, __offout: *s64, __len: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .LINUX] `tee :: (__fdin: s32, __fdout: s32, __len: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .LINUX] `fallocate :: (__fd: s32, __mode: s32, __offset: s64, __len: s64) -> s32 #foreign libc`
- [OS == .LINUX] `fallocate64 :: (__fd: s32, __mode: s32, __offset: s64, __len: s64) -> s32 #foreign libc`
- [OS == .LINUX] `name_to_handle_at :: (__dfd: s32, __name: *u8, __handle: *file_handle, __mnt_id: *s32, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `open_by_handle_at :: (__mountdirfd: s32, __handle: *file_handle, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `mode_t :: u32`
- [OS == .LINUX] `pid_t :: s32`
- [OS == .LINUX] `stat_t :: struct`
- [OS == .LINUX] `stat64_t :: struct`
- [OS == .LINUX] `fcntl :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `fcntl64 :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `open :: (__file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `open64 :: (__file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `openat :: (__fd: s32, __file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `openat64 :: (__fd: s32, __file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `creat :: (__file: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .LINUX] `creat64 :: (__file: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .LINUX] `lockf :: (__fd: s32, __cmd: s32, __len: s64) -> s32 #foreign libc`
- [OS == .LINUX] `lockf64 :: (__fd: s32, __cmd: s32, __len: s64) -> s32 #foreign libc`
- [OS == .LINUX] `posix_fadvise :: (__fd: s32, __offset: s64, __len: s64, __advise: s32) -> s32 #foreign libc`
- [OS == .LINUX] `posix_fadvise64 :: (__fd: s32, __offset: s64, __len: s64, __advise: s32) -> s32 #foreign libc`
- [OS == .LINUX] `posix_fallocate :: (__fd: s32, __offset: s64, __len: s64) -> s32 #foreign libc`
- [OS == .LINUX] `posix_fallocate64 :: (__fd: s32, __offset: s64, __len: s64) -> s32 #foreign libc`
- [OS == .LINUX] `dev_t :: u64`
- [OS == .LINUX] `gid_t :: u32`
- [OS == .LINUX] `ino_t :: u64`
- [OS == .LINUX] `nlink_t :: u64`
- [OS == .LINUX] `uid_t :: u32`
- [OS == .LINUX] `blkcnt_t :: s64`
- [OS == .LINUX] `blksize_t :: s64`
- [OS == .LINUX] `chmod :: (__file: *u8, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `lchmod :: (__file: *u8, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `fchmod :: (__fd: s32, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `fchmodat :: (__fd: s32, __file: *u8, __mode: u32, __flag: s32) -> s32 #foreign libc`
- [OS == .LINUX] `umask :: (__mask: u32) -> u32 #foreign libc`
- [OS == .LINUX] `mkdir :: (__path: *u8, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `mkdirat :: (__fd: s32, __path: *u8, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `mkfifo :: (__path: *u8, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `mkfifoat :: (__fd: s32, __path: *u8, __mode: u32) -> s32 #foreign libc`
- [OS == .LINUX] `utimensat :: (__fd: s32, __path: *u8, __times: *[2] timespec, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc`
- [OS == .LINUX] `__fxstat :: (__ver: s32, __fildes: s32, __stat_buf: *stat_t) -> s32 #foreign libc`
- [OS == .LINUX] `__xstat :: (__ver: s32, __filename: *u8, __stat_buf: *stat_t) -> s32 #foreign libc`
- [OS == .LINUX] `__lxstat :: (__ver: s32, __filename: *u8, __stat_buf: *stat_t) -> s32 #foreign libc`
- [OS == .LINUX] `__fxstatat :: (__ver: s32, __fildes: s32, __filename: *u8, __stat_buf: *stat_t, __flag: s32) -> s32 #foreign libc`
- [OS == .LINUX] `__fxstat64 :: (__ver: s32, __fildes: s32, __stat_buf: *stat64_t) -> s32 #foreign libc`
- [OS == .LINUX] `__xstat64 :: (__ver: s32, __filename: *u8, __stat_buf: *stat64_t) -> s32 #foreign libc`
- [OS == .LINUX] `__lxstat64 :: (__ver: s32, __filename: *u8, __stat_buf: *stat64_t) -> s32 #foreign libc`
- [OS == .LINUX] `__fxstatat64 :: (__ver: s32, __fildes: s32, __filename: *u8, __stat_buf: *stat64_t, __flag: s32) -> s32 #foreign libc`
- [OS == .LINUX] `__xmknod :: (__ver: s32, __path: *u8, __mode: u32, __dev: *u64) -> s32 #foreign libc`
- [OS == .LINUX] `__xmknodat :: (__ver: s32, __fd: s32, __path: *u8, __mode: u32, __dev: *u64) -> s32 #foreign libc`
- [OS == .LINUX] `statx_timestamp :: struct`
- [OS == .LINUX] `statx_t :: struct`
- [OS == .LINUX] `ioctl :: (__fd: s32, __request: u64, __args: ..Any) -> s32 #foreign libc`
- [OS == .LINUX] `aiocb :: struct`
- [OS == .LINUX] `aiocb64 :: struct`
- [OS == .LINUX] `aioinit :: struct`
- [OS == .LINUX] `AIO :: enum u32`
- [OS == .LINUX] `LIO :: enum u32`
- [OS == .LINUX] `LIO_1 :: enum u32`
- [OS == .LINUX] `aio_init :: (__init: *aioinit) -> void #foreign librt`
- [OS == .LINUX] `aio_read :: (__aiocbp: *aiocb) -> s32 #foreign librt`
- [OS == .LINUX] `aio_write :: (__aiocbp: *aiocb) -> s32 #foreign librt`
- [OS == .LINUX] `lio_listio :: (__mode: s32, __list: **aiocb, __nent: s32, __sig: *sigevent) -> s32 #foreign librt`
- [OS == .LINUX] `aio_error :: (__aiocbp: *aiocb) -> s32 #foreign librt`
- [OS == .LINUX] `aio_return :: (__aiocbp: *aiocb) -> s64 #foreign librt`
- [OS == .LINUX] `aio_cancel :: (__fildes: s32, __aiocbp: *aiocb) -> s32 #foreign librt`
- [OS == .LINUX] `aio_suspend :: (__list: **aiocb, __nent: s32, __timeout: *timespec) -> s32 #foreign librt`
- [OS == .LINUX] `aio_fsync :: (__operation: s32, __aiocbp: *aiocb) -> s32 #foreign librt`
- [OS == .LINUX] `aio_read64 :: (__aiocbp: *aiocb64) -> s32 #foreign librt`
- [OS == .LINUX] `aio_write64 :: (__aiocbp: *aiocb64) -> s32 #foreign librt`
- [OS == .LINUX] `lio_listio64 :: (__mode: s32, __list: **aiocb64, __nent: s32, __sig: *sigevent) -> s32 #foreign librt`
- [OS == .LINUX] `aio_error64 :: (__aiocbp: *aiocb64) -> s32 #foreign librt`
- [OS == .LINUX] `aio_return64 :: (__aiocbp: *aiocb64) -> s64 #foreign librt`
- [OS == .LINUX] `aio_cancel64 :: (__fildes: s32, __aiocbp: *aiocb64) -> s32 #foreign librt`
- [OS == .LINUX] `aio_suspend64 :: (__list: **aiocb64, __nent: s32, __timeout: *timespec) -> s32 #foreign librt`
- [OS == .LINUX] `aio_fsync64 :: (__operation: s32, __aiocbp: *aiocb64) -> s32 #foreign librt`
- [OS == .LINUX] `EFD :: enum u32`
- [OS == .LINUX] `eventfd_t :: u64`
- [OS == .LINUX] `eventfd :: (__count: u32, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `eventfd_read :: (__fd: s32, __value: *eventfd_t) -> s32 #foreign libc`
- [OS == .LINUX] `eventfd_write :: (__fd: s32, __value: eventfd_t) -> s32 #foreign libc`
- [OS == .LINUX] `TFD :: enum u32`
- [OS == .LINUX] `TFD_TIMER :: enum u32`
- [OS == .LINUX] `timerfd_create :: (__clock_id: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `timerfd_settime :: (__ufd: s32, __flags: s32, __utmr: *itimerspec, __otmr: *itimerspec) -> s32 #foreign libc`
- [OS == .LINUX] `timerfd_gettime :: (__ufd: s32, __otmr: *itimerspec) -> s32 #foreign libc`
- [OS == .LINUX] `SFD :: enum u32`
- [OS == .LINUX] `signalfd_siginfo :: struct`
- [OS == .LINUX] `signalfd :: (__fd: s32, __mask: *sigset_t, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `FILE :: _IO_FILE`
- [OS == .LINUX] `_IO_FILE :: struct`
- [OS == .LINUX] `stat :: (path: *u8, buf: *stat_t) -> s64`
- [OS == .LINUX] `fstat :: (fd: s32, buf: *stat_t) -> s64`
- [OS == .LINUX] `lstat :: (path: *u8, buf: *stat_t) -> s64`
- [OS == .LINUX] `getenv :: (__name: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `setenv :: (__name: *u8, __value: *u8, __replace: s32) -> s32 #foreign libc`
- [OS == .LINUX] `unsetenv :: (__name: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `mktemp :: (__template: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `mkstemp :: (__template: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `mkstemps :: (__template: *u8, __suffixlen: s32) -> s32 #foreign libc`
- [OS == .LINUX] `mkdtemp :: (__template: *u8) -> *u8 #foreign libc`
- [OS == .LINUX] `mkostemp :: (__template: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `mkostemps :: (__template: *u8, __suffixlen: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .LINUX] `system :: (__command: *u8) -> s32 #foreign libc`
- [OS == .LINUX] `realpath :: (__name: *u8, __resolved: *u8) -> *u8 #foreign libc`
- [OS == .LINUX && CPU == .X64] `SYS__sysctl :: 156`
- [OS == .LINUX && CPU == .X64] `SYS_accept :: 43`
- [OS == .LINUX && CPU == .X64] `SYS_accept4 :: 288`
- [OS == .LINUX && CPU == .X64] `SYS_access :: 21`
- [OS == .LINUX && CPU == .X64] `SYS_acct :: 163`
- [OS == .LINUX && CPU == .X64] `SYS_add_key :: 248`
- [OS == .LINUX && CPU == .X64] `SYS_adjtimex :: 159`
- [OS == .LINUX && CPU == .X64] `SYS_afs_syscall :: 183`
- [OS == .LINUX && CPU == .X64] `SYS_alarm :: 37`
- [OS == .LINUX && CPU == .X64] `SYS_arch_prctl :: 158`
- [OS == .LINUX && CPU == .X64] `SYS_bind :: 49`
- [OS == .LINUX && CPU == .X64] `SYS_bpf :: 321`
- [OS == .LINUX && CPU == .X64] `SYS_brk :: 12`
- [OS == .LINUX && CPU == .X64] `SYS_capget :: 125`
- [OS == .LINUX && CPU == .X64] `SYS_capset :: 126`
- [OS == .LINUX && CPU == .X64] `SYS_chdir :: 80`
- [OS == .LINUX && CPU == .X64] `SYS_chmod :: 90`
- [OS == .LINUX && CPU == .X64] `SYS_chown :: 92`
- [OS == .LINUX && CPU == .X64] `SYS_chroot :: 161`
- [OS == .LINUX && CPU == .X64] `SYS_clock_adjtime :: 305`
- [OS == .LINUX && CPU == .X64] `SYS_clock_getres :: 229`
- [OS == .LINUX && CPU == .X64] `SYS_clock_gettime :: 228`
- [OS == .LINUX && CPU == .X64] `SYS_clock_nanosleep :: 230`
- [OS == .LINUX && CPU == .X64] `SYS_clock_settime :: 227`
- [OS == .LINUX && CPU == .X64] `SYS_clone :: 56`
- [OS == .LINUX && CPU == .X64] `SYS_clone3 :: 435`
- [OS == .LINUX && CPU == .X64] `SYS_close :: 3`
- [OS == .LINUX && CPU == .X64] `SYS_connect :: 42`
- [OS == .LINUX && CPU == .X64] `SYS_copy_file_range :: 326`
- [OS == .LINUX && CPU == .X64] `SYS_creat :: 85`
- [OS == .LINUX && CPU == .X64] `SYS_create_module :: 174`
- [OS == .LINUX && CPU == .X64] `SYS_delete_module :: 176`
- [OS == .LINUX && CPU == .X64] `SYS_dup :: 32`
- [OS == .LINUX && CPU == .X64] `SYS_dup2 :: 33`
- [OS == .LINUX && CPU == .X64] `SYS_dup3 :: 292`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_create :: 213`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_create1 :: 291`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_ctl :: 233`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_ctl_old :: 214`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_pwait :: 281`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_wait :: 232`
- [OS == .LINUX && CPU == .X64] `SYS_epoll_wait_old :: 215`
- [OS == .LINUX && CPU == .X64] `SYS_eventfd :: 284`
- [OS == .LINUX && CPU == .X64] `SYS_eventfd2 :: 290`
- [OS == .LINUX && CPU == .X64] `SYS_execve :: 59`
- [OS == .LINUX && CPU == .X64] `SYS_execveat :: 322`
- [OS == .LINUX && CPU == .X64] `SYS_exit :: 60`
- [OS == .LINUX && CPU == .X64] `SYS_exit_group :: 231`
- [OS == .LINUX && CPU == .X64] `SYS_faccessat :: 269`
- [OS == .LINUX && CPU == .X64] `SYS_fadvise64 :: 221`
- [OS == .LINUX && CPU == .X64] `SYS_fallocate :: 285`
- [OS == .LINUX && CPU == .X64] `SYS_fanotify_init :: 300`
- [OS == .LINUX && CPU == .X64] `SYS_fanotify_mark :: 301`
- [OS == .LINUX && CPU == .X64] `SYS_fchdir :: 81`
- [OS == .LINUX && CPU == .X64] `SYS_fchmod :: 91`
- [OS == .LINUX && CPU == .X64] `SYS_fchmodat :: 268`
- [OS == .LINUX && CPU == .X64] `SYS_fchown :: 93`
- [OS == .LINUX && CPU == .X64] `SYS_fchownat :: 260`
- [OS == .LINUX && CPU == .X64] `SYS_fcntl :: 72`
- [OS == .LINUX && CPU == .X64] `SYS_fdatasync :: 75`
- [OS == .LINUX && CPU == .X64] `SYS_fgetxattr :: 193`
- [OS == .LINUX && CPU == .X64] `SYS_finit_module :: 313`
- [OS == .LINUX && CPU == .X64] `SYS_flistxattr :: 196`
- [OS == .LINUX && CPU == .X64] `SYS_flock :: 73`
- [OS == .LINUX && CPU == .X64] `SYS_fork :: 57`
- [OS == .LINUX && CPU == .X64] `SYS_fremovexattr :: 199`
- [OS == .LINUX && CPU == .X64] `SYS_fsconfig :: 431`
- [OS == .LINUX && CPU == .X64] `SYS_fsetxattr :: 190`
- [OS == .LINUX && CPU == .X64] `SYS_fsmount :: 432`
- [OS == .LINUX && CPU == .X64] `SYS_fsopen :: 430`
- [OS == .LINUX && CPU == .X64] `SYS_fspick :: 433`
- [OS == .LINUX && CPU == .X64] `SYS_fstat :: 5`
- [OS == .LINUX && CPU == .X64] `SYS_fstatfs :: 138`
- [OS == .LINUX && CPU == .X64] `SYS_fsync :: 74`
- [OS == .LINUX && CPU == .X64] `SYS_ftruncate :: 77`
- [OS == .LINUX && CPU == .X64] `SYS_futex :: 202`
- [OS == .LINUX && CPU == .X64] `SYS_futimesat :: 261`
- [OS == .LINUX && CPU == .X64] `SYS_get_kernel_syms :: 177`
- [OS == .LINUX && CPU == .X64] `SYS_get_mempolicy :: 239`
- [OS == .LINUX && CPU == .X64] `SYS_get_robust_list :: 274`
- [OS == .LINUX && CPU == .X64] `SYS_get_thread_area :: 211`
- [OS == .LINUX && CPU == .X64] `SYS_getcpu :: 309`
- [OS == .LINUX && CPU == .X64] `SYS_getcwd :: 79`
- [OS == .LINUX && CPU == .X64] `SYS_getdents :: 78`
- [OS == .LINUX && CPU == .X64] `SYS_getdents64 :: 217`
- [OS == .LINUX && CPU == .X64] `SYS_getegid :: 108`
- [OS == .LINUX && CPU == .X64] `SYS_geteuid :: 107`
- [OS == .LINUX && CPU == .X64] `SYS_getgid :: 104`
- [OS == .LINUX && CPU == .X64] `SYS_getgroups :: 115`
- [OS == .LINUX && CPU == .X64] `SYS_getitimer :: 36`
- [OS == .LINUX && CPU == .X64] `SYS_getpeername :: 52`
- [OS == .LINUX && CPU == .X64] `SYS_getpgid :: 121`
- [OS == .LINUX && CPU == .X64] `SYS_getpgrp :: 111`
- [OS == .LINUX && CPU == .X64] `SYS_getpid :: 39`
- [OS == .LINUX && CPU == .X64] `SYS_getpmsg :: 181`
- [OS == .LINUX && CPU == .X64] `SYS_getppid :: 110`
- [OS == .LINUX && CPU == .X64] `SYS_getpriority :: 140`
- [OS == .LINUX && CPU == .X64] `SYS_getrandom :: 318`
- [OS == .LINUX && CPU == .X64] `SYS_getresgid :: 120`
- [OS == .LINUX && CPU == .X64] `SYS_getresuid :: 118`
- [OS == .LINUX && CPU == .X64] `SYS_getrlimit :: 97`
- [OS == .LINUX && CPU == .X64] `SYS_getrusage :: 98`
- [OS == .LINUX && CPU == .X64] `SYS_getsid :: 124`
- [OS == .LINUX && CPU == .X64] `SYS_getsockname :: 51`
- [OS == .LINUX && CPU == .X64] `SYS_getsockopt :: 55`
- [OS == .LINUX && CPU == .X64] `SYS_gettid :: 186`
- [OS == .LINUX && CPU == .X64] `SYS_gettimeofday :: 96`
- [OS == .LINUX && CPU == .X64] `SYS_getuid :: 102`
- [OS == .LINUX && CPU == .X64] `SYS_getxattr :: 191`
- [OS == .LINUX && CPU == .X64] `SYS_init_module :: 175`
- [OS == .LINUX && CPU == .X64] `SYS_inotify_add_watch :: 254`
- [OS == .LINUX && CPU == .X64] `SYS_inotify_init :: 253`
- [OS == .LINUX && CPU == .X64] `SYS_inotify_init1 :: 294`
- [OS == .LINUX && CPU == .X64] `SYS_inotify_rm_watch :: 255`
- [OS == .LINUX && CPU == .X64] `SYS_io_cancel :: 210`
- [OS == .LINUX && CPU == .X64] `SYS_io_destroy :: 207`
- [OS == .LINUX && CPU == .X64] `SYS_io_getevents :: 208`
- [OS == .LINUX && CPU == .X64] `SYS_io_pgetevents :: 333`
- [OS == .LINUX && CPU == .X64] `SYS_io_setup :: 206`
- [OS == .LINUX && CPU == .X64] `SYS_io_submit :: 209`
- [OS == .LINUX && CPU == .X64] `SYS_io_uring_enter :: 426`
- [OS == .LINUX && CPU == .X64] `SYS_io_uring_register :: 427`
- [OS == .LINUX && CPU == .X64] `SYS_io_uring_setup :: 425`
- [OS == .LINUX && CPU == .X64] `SYS_ioctl :: 16`
- [OS == .LINUX && CPU == .X64] `SYS_ioperm :: 173`
- [OS == .LINUX && CPU == .X64] `SYS_iopl :: 172`
- [OS == .LINUX && CPU == .X64] `SYS_ioprio_get :: 252`
- [OS == .LINUX && CPU == .X64] `SYS_ioprio_set :: 251`
- [OS == .LINUX && CPU == .X64] `SYS_kcmp :: 312`
- [OS == .LINUX && CPU == .X64] `SYS_kexec_file_load :: 320`
- [OS == .LINUX && CPU == .X64] `SYS_kexec_load :: 246`
- [OS == .LINUX && CPU == .X64] `SYS_keyctl :: 250`
- [OS == .LINUX && CPU == .X64] `SYS_kill :: 62`
- [OS == .LINUX && CPU == .X64] `SYS_lchown :: 94`
- [OS == .LINUX && CPU == .X64] `SYS_lgetxattr :: 192`
- [OS == .LINUX && CPU == .X64] `SYS_link :: 86`
- [OS == .LINUX && CPU == .X64] `SYS_linkat :: 265`
- [OS == .LINUX && CPU == .X64] `SYS_listen :: 50`
- [OS == .LINUX && CPU == .X64] `SYS_listxattr :: 194`
- [OS == .LINUX && CPU == .X64] `SYS_llistxattr :: 195`
- [OS == .LINUX && CPU == .X64] `SYS_lookup_dcookie :: 212`
- [OS == .LINUX && CPU == .X64] `SYS_lremovexattr :: 198`
- [OS == .LINUX && CPU == .X64] `SYS_lseek :: 8`
- [OS == .LINUX && CPU == .X64] `SYS_lsetxattr :: 189`
- [OS == .LINUX && CPU == .X64] `SYS_lstat :: 6`
- [OS == .LINUX && CPU == .X64] `SYS_madvise :: 28`
- [OS == .LINUX && CPU == .X64] `SYS_mbind :: 237`
- [OS == .LINUX && CPU == .X64] `SYS_membarrier :: 324`
- [OS == .LINUX && CPU == .X64] `SYS_memfd_create :: 319`
- [OS == .LINUX && CPU == .X64] `SYS_migrate_pages :: 256`
- [OS == .LINUX && CPU == .X64] `SYS_mincore :: 27`
- [OS == .LINUX && CPU == .X64] `SYS_mkdir :: 83`
- [OS == .LINUX && CPU == .X64] `SYS_mkdirat :: 258`
- [OS == .LINUX && CPU == .X64] `SYS_mknod :: 133`
- [OS == .LINUX && CPU == .X64] `SYS_mknodat :: 259`
- [OS == .LINUX && CPU == .X64] `SYS_mlock :: 149`
- [OS == .LINUX && CPU == .X64] `SYS_mlock2 :: 325`
- [OS == .LINUX && CPU == .X64] `SYS_mlockall :: 151`
- [OS == .LINUX && CPU == .X64] `SYS_mmap :: 9`
- [OS == .LINUX && CPU == .X64] `SYS_modify_ldt :: 154`
- [OS == .LINUX && CPU == .X64] `SYS_mount :: 165`
- [OS == .LINUX && CPU == .X64] `SYS_move_mount :: 429`
- [OS == .LINUX && CPU == .X64] `SYS_move_pages :: 279`
- [OS == .LINUX && CPU == .X64] `SYS_mprotect :: 10`
- [OS == .LINUX && CPU == .X64] `SYS_mq_getsetattr :: 245`
- [OS == .LINUX && CPU == .X64] `SYS_mq_notify :: 244`
- [OS == .LINUX && CPU == .X64] `SYS_mq_open :: 240`
- [OS == .LINUX && CPU == .X64] `SYS_mq_timedreceive :: 243`
- [OS == .LINUX && CPU == .X64] `SYS_mq_timedsend :: 242`
- [OS == .LINUX && CPU == .X64] `SYS_mq_unlink :: 241`
- [OS == .LINUX && CPU == .X64] `SYS_mremap :: 25`
- [OS == .LINUX && CPU == .X64] `SYS_msgctl :: 71`
- [OS == .LINUX && CPU == .X64] `SYS_msgget :: 68`
- [OS == .LINUX && CPU == .X64] `SYS_msgrcv :: 70`
- [OS == .LINUX && CPU == .X64] `SYS_msgsnd :: 69`
- [OS == .LINUX && CPU == .X64] `SYS_msync :: 26`
- [OS == .LINUX && CPU == .X64] `SYS_munlock :: 150`
- [OS == .LINUX && CPU == .X64] `SYS_munlockall :: 152`
- [OS == .LINUX && CPU == .X64] `SYS_munmap :: 11`
- [OS == .LINUX && CPU == .X64] `SYS_name_to_handle_at :: 303`
- [OS == .LINUX && CPU == .X64] `SYS_nanosleep :: 35`
- [OS == .LINUX && CPU == .X64] `SYS_newfstatat :: 262`
- [OS == .LINUX && CPU == .X64] `SYS_nfsservctl :: 180`
- [OS == .LINUX && CPU == .X64] `SYS_open :: 2`
- [OS == .LINUX && CPU == .X64] `SYS_open_by_handle_at :: 304`
- [OS == .LINUX && CPU == .X64] `SYS_open_tree :: 428`
- [OS == .LINUX && CPU == .X64] `SYS_openat :: 257`
- [OS == .LINUX && CPU == .X64] `SYS_pause :: 34`
- [OS == .LINUX && CPU == .X64] `SYS_perf_event_open :: 298`
- [OS == .LINUX && CPU == .X64] `SYS_personality :: 135`
- [OS == .LINUX && CPU == .X64] `SYS_pidfd_open :: 434`
- [OS == .LINUX && CPU == .X64] `SYS_pidfd_send_signal :: 424`
- [OS == .LINUX && CPU == .X64] `SYS_pipe :: 22`
- [OS == .LINUX && CPU == .X64] `SYS_pipe2 :: 293`
- [OS == .LINUX && CPU == .X64] `SYS_pivot_root :: 155`
- [OS == .LINUX && CPU == .X64] `SYS_pkey_alloc :: 330`
- [OS == .LINUX && CPU == .X64] `SYS_pkey_free :: 331`
- [OS == .LINUX && CPU == .X64] `SYS_pkey_mprotect :: 329`
- [OS == .LINUX && CPU == .X64] `SYS_poll :: 7`
- [OS == .LINUX && CPU == .X64] `SYS_ppoll :: 271`
- [OS == .LINUX && CPU == .X64] `SYS_prctl :: 157`
- [OS == .LINUX && CPU == .X64] `SYS_pread64 :: 17`
- [OS == .LINUX && CPU == .X64] `SYS_preadv :: 295`
- [OS == .LINUX && CPU == .X64] `SYS_preadv2 :: 327`
- [OS == .LINUX && CPU == .X64] `SYS_prlimit64 :: 302`
- [OS == .LINUX && CPU == .X64] `SYS_process_vm_readv :: 310`
- [OS == .LINUX && CPU == .X64] `SYS_process_vm_writev :: 311`
- [OS == .LINUX && CPU == .X64] `SYS_pselect6 :: 270`
- [OS == .LINUX && CPU == .X64] `SYS_ptrace :: 101`
- [OS == .LINUX && CPU == .X64] `SYS_putpmsg :: 182`
- [OS == .LINUX && CPU == .X64] `SYS_pwrite64 :: 18`
- [OS == .LINUX && CPU == .X64] `SYS_pwritev :: 296`
- [OS == .LINUX && CPU == .X64] `SYS_pwritev2 :: 328`
- [OS == .LINUX && CPU == .X64] `SYS_query_module :: 178`
- [OS == .LINUX && CPU == .X64] `SYS_quotactl :: 179`
- [OS == .LINUX && CPU == .X64] `SYS_read :: 0`
- [OS == .LINUX && CPU == .X64] `SYS_readahead :: 187`
- [OS == .LINUX && CPU == .X64] `SYS_readlink :: 89`
- [OS == .LINUX && CPU == .X64] `SYS_readlinkat :: 267`
- [OS == .LINUX && CPU == .X64] `SYS_readv :: 19`
- [OS == .LINUX && CPU == .X64] `SYS_reboot :: 169`
- [OS == .LINUX && CPU == .X64] `SYS_recvfrom :: 45`
- [OS == .LINUX && CPU == .X64] `SYS_recvmmsg :: 299`
- [OS == .LINUX && CPU == .X64] `SYS_recvmsg :: 47`
- [OS == .LINUX && CPU == .X64] `SYS_remap_file_pages :: 216`
- [OS == .LINUX && CPU == .X64] `SYS_removexattr :: 197`
- [OS == .LINUX && CPU == .X64] `SYS_rename :: 82`
- [OS == .LINUX && CPU == .X64] `SYS_renameat :: 264`
- [OS == .LINUX && CPU == .X64] `SYS_renameat2 :: 316`
- [OS == .LINUX && CPU == .X64] `SYS_request_key :: 249`
- [OS == .LINUX && CPU == .X64] `SYS_restart_syscall :: 219`
- [OS == .LINUX && CPU == .X64] `SYS_rmdir :: 84`
- [OS == .LINUX && CPU == .X64] `SYS_rseq :: 334`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigaction :: 13`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigpending :: 127`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigprocmask :: 14`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigqueueinfo :: 129`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigreturn :: 15`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigsuspend :: 130`
- [OS == .LINUX && CPU == .X64] `SYS_rt_sigtimedwait :: 128`
- [OS == .LINUX && CPU == .X64] `SYS_rt_tgsigqueueinfo :: 297`
- [OS == .LINUX && CPU == .X64] `SYS_sched_get_priority_max :: 146`
- [OS == .LINUX && CPU == .X64] `SYS_sched_get_priority_min :: 147`
- [OS == .LINUX && CPU == .X64] `SYS_sched_getaffinity :: 204`
- [OS == .LINUX && CPU == .X64] `SYS_sched_getattr :: 315`
- [OS == .LINUX && CPU == .X64] `SYS_sched_getparam :: 143`
- [OS == .LINUX && CPU == .X64] `SYS_sched_getscheduler :: 145`
- [OS == .LINUX && CPU == .X64] `SYS_sched_rr_get_interval :: 148`
- [OS == .LINUX && CPU == .X64] `SYS_sched_setaffinity :: 203`
- [OS == .LINUX && CPU == .X64] `SYS_sched_setattr :: 314`
- [OS == .LINUX && CPU == .X64] `SYS_sched_setparam :: 142`
- [OS == .LINUX && CPU == .X64] `SYS_sched_setscheduler :: 144`
- [OS == .LINUX && CPU == .X64] `SYS_sched_yield :: 24`
- [OS == .LINUX && CPU == .X64] `SYS_seccomp :: 317`
- [OS == .LINUX && CPU == .X64] `SYS_security :: 185`
- [OS == .LINUX && CPU == .X64] `SYS_select :: 23`
- [OS == .LINUX && CPU == .X64] `SYS_semctl :: 66`
- [OS == .LINUX && CPU == .X64] `SYS_semget :: 64`
- [OS == .LINUX && CPU == .X64] `SYS_semop :: 65`
- [OS == .LINUX && CPU == .X64] `SYS_semtimedop :: 220`
- [OS == .LINUX && CPU == .X64] `SYS_sendfile :: 40`
- [OS == .LINUX && CPU == .X64] `SYS_sendmmsg :: 307`
- [OS == .LINUX && CPU == .X64] `SYS_sendmsg :: 46`
- [OS == .LINUX && CPU == .X64] `SYS_sendto :: 44`
- [OS == .LINUX && CPU == .X64] `SYS_set_mempolicy :: 238`
- [OS == .LINUX && CPU == .X64] `SYS_set_robust_list :: 273`
- [OS == .LINUX && CPU == .X64] `SYS_set_thread_area :: 205`
- [OS == .LINUX && CPU == .X64] `SYS_set_tid_address :: 218`
- [OS == .LINUX && CPU == .X64] `SYS_setdomainname :: 171`
- [OS == .LINUX && CPU == .X64] `SYS_setfsgid :: 123`
- [OS == .LINUX && CPU == .X64] `SYS_setfsuid :: 122`
- [OS == .LINUX && CPU == .X64] `SYS_setgid :: 106`
- [OS == .LINUX && CPU == .X64] `SYS_setgroups :: 116`
- [OS == .LINUX && CPU == .X64] `SYS_sethostname :: 170`
- [OS == .LINUX && CPU == .X64] `SYS_setitimer :: 38`
- [OS == .LINUX && CPU == .X64] `SYS_setns :: 308`
- [OS == .LINUX && CPU == .X64] `SYS_setpgid :: 109`
- [OS == .LINUX && CPU == .X64] `SYS_setpriority :: 141`
- [OS == .LINUX && CPU == .X64] `SYS_setregid :: 114`
- [OS == .LINUX && CPU == .X64] `SYS_setresgid :: 119`
- [OS == .LINUX && CPU == .X64] `SYS_setresuid :: 117`
- [OS == .LINUX && CPU == .X64] `SYS_setreuid :: 113`
- [OS == .LINUX && CPU == .X64] `SYS_setrlimit :: 160`
- [OS == .LINUX && CPU == .X64] `SYS_setsid :: 112`
- [OS == .LINUX && CPU == .X64] `SYS_setsockopt :: 54`
- [OS == .LINUX && CPU == .X64] `SYS_settimeofday :: 164`
- [OS == .LINUX && CPU == .X64] `SYS_setuid :: 105`
- [OS == .LINUX && CPU == .X64] `SYS_setxattr :: 188`
- [OS == .LINUX && CPU == .X64] `SYS_shmat :: 30`
- [OS == .LINUX && CPU == .X64] `SYS_shmctl :: 31`
- [OS == .LINUX && CPU == .X64] `SYS_shmdt :: 67`
- [OS == .LINUX && CPU == .X64] `SYS_shmget :: 29`
- [OS == .LINUX && CPU == .X64] `SYS_shutdown :: 48`
- [OS == .LINUX && CPU == .X64] `SYS_sigaltstack :: 131`
- [OS == .LINUX && CPU == .X64] `SYS_signalfd :: 282`
- [OS == .LINUX && CPU == .X64] `SYS_signalfd4 :: 289`
- [OS == .LINUX && CPU == .X64] `SYS_socket :: 41`
- [OS == .LINUX && CPU == .X64] `SYS_socketpair :: 53`
- [OS == .LINUX && CPU == .X64] `SYS_splice :: 275`
- [OS == .LINUX && CPU == .X64] `SYS_stat :: 4`
- [OS == .LINUX && CPU == .X64] `SYS_statfs :: 137`
- [OS == .LINUX && CPU == .X64] `SYS_statx :: 332`
- [OS == .LINUX && CPU == .X64] `SYS_swapoff :: 168`
- [OS == .LINUX && CPU == .X64] `SYS_swapon :: 167`
- [OS == .LINUX && CPU == .X64] `SYS_symlink :: 88`
- [OS == .LINUX && CPU == .X64] `SYS_symlinkat :: 266`
- [OS == .LINUX && CPU == .X64] `SYS_sync :: 162`
- [OS == .LINUX && CPU == .X64] `SYS_sync_file_range :: 277`
- [OS == .LINUX && CPU == .X64] `SYS_syncfs :: 306`
- [OS == .LINUX && CPU == .X64] `SYS_sysfs :: 139`
- [OS == .LINUX && CPU == .X64] `SYS_sysinfo :: 99`
- [OS == .LINUX && CPU == .X64] `SYS_syslog :: 103`
- [OS == .LINUX && CPU == .X64] `SYS_tee :: 276`
- [OS == .LINUX && CPU == .X64] `SYS_tgkill :: 234`
- [OS == .LINUX && CPU == .X64] `SYS_time :: 201`
- [OS == .LINUX && CPU == .X64] `SYS_timer_create :: 222`
- [OS == .LINUX && CPU == .X64] `SYS_timer_delete :: 226`
- [OS == .LINUX && CPU == .X64] `SYS_timer_getoverrun :: 225`
- [OS == .LINUX && CPU == .X64] `SYS_timer_gettime :: 224`
- [OS == .LINUX && CPU == .X64] `SYS_timer_settime :: 223`
- [OS == .LINUX && CPU == .X64] `SYS_timerfd_create :: 283`
- [OS == .LINUX && CPU == .X64] `SYS_timerfd_gettime :: 287`
- [OS == .LINUX && CPU == .X64] `SYS_timerfd_settime :: 286`
- [OS == .LINUX && CPU == .X64] `SYS_times :: 100`
- [OS == .LINUX && CPU == .X64] `SYS_tkill :: 200`
- [OS == .LINUX && CPU == .X64] `SYS_truncate :: 76`
- [OS == .LINUX && CPU == .X64] `SYS_tuxcall :: 184`
- [OS == .LINUX && CPU == .X64] `SYS_umask :: 95`
- [OS == .LINUX && CPU == .X64] `SYS_umount2 :: 166`
- [OS == .LINUX && CPU == .X64] `SYS_uname :: 63`
- [OS == .LINUX && CPU == .X64] `SYS_unlink :: 87`
- [OS == .LINUX && CPU == .X64] `SYS_unlinkat :: 263`
- [OS == .LINUX && CPU == .X64] `SYS_unshare :: 272`
- [OS == .LINUX && CPU == .X64] `SYS_uselib :: 134`
- [OS == .LINUX && CPU == .X64] `SYS_userfaultfd :: 323`
- [OS == .LINUX && CPU == .X64] `SYS_ustat :: 136`
- [OS == .LINUX && CPU == .X64] `SYS_utime :: 132`
- [OS == .LINUX && CPU == .X64] `SYS_utimensat :: 280`
- [OS == .LINUX && CPU == .X64] `SYS_utimes :: 235`
- [OS == .LINUX && CPU == .X64] `SYS_vfork :: 58`
- [OS == .LINUX && CPU == .X64] `SYS_vhangup :: 153`
- [OS == .LINUX && CPU == .X64] `SYS_vmsplice :: 278`
- [OS == .LINUX && CPU == .X64] `SYS_vserver :: 236`
- [OS == .LINUX && CPU == .X64] `SYS_wait4 :: 61`
- [OS == .LINUX && CPU == .X64] `SYS_waitid :: 247`
- [OS == .LINUX && CPU == .X64] `SYS_write :: 1`
- [OS == .LINUX && CPU == .X64] `SYS_writev :: 20`
- [OS == .LINUX] `syscall :: (__sysno: s64, __args: ..Any) -> s64 #foreign libc`
- [OS == .LINUX] `SCHED_OTHER :: 0`
- [OS == .LINUX] `SCHED_FIFO :: 1`
- [OS == .LINUX] `SCHED_RR :: 2`
- [OS == .LINUX] `SCHED_BATCH :: 3`
- [OS == .LINUX] `SCHED_ISO :: 4`
- [OS == .LINUX] `SCHED_IDLE :: 5`
- [OS == .LINUX] `SCHED_DEADLINE :: 6`
- [OS == .LINUX] `SCHED_RESET_ON_FORK :: 0x40000000`
- [OS == .LINUX] `CSIGNAL :: 0x000000ff`
- [OS == .LINUX] `CLONE_VM :: 0x00000100`
- [OS == .LINUX] `CLONE_FS :: 0x00000200`
- [OS == .LINUX] `CLONE_FILES :: 0x00000400`
- [OS == .LINUX] `CLONE_SIGHAND :: 0x00000800`
- [OS == .LINUX] `CLONE_PIDFD :: 0x00001000`
- [OS == .LINUX] `CLONE_PTRACE :: 0x00002000`
- [OS == .LINUX] `CLONE_VFORK :: 0x00004000`
- [OS == .LINUX] `CLONE_PARENT :: 0x00008000`
- [OS == .LINUX] `CLONE_THREAD :: 0x00010000`
- [OS == .LINUX] `CLONE_NEWNS :: 0x00020000`
- [OS == .LINUX] `CLONE_SYSVSEM :: 0x00040000`
- [OS == .LINUX] `CLONE_SETTLS :: 0x00080000`
- [OS == .LINUX] `CLONE_PARENT_SETTID :: 0x00100000`
- [OS == .LINUX] `CLONE_CHILD_CLEARTID :: 0x00200000`
- [OS == .LINUX] `CLONE_DETACHED :: 0x00400000`
- [OS == .LINUX] `CLONE_UNTRACED :: 0x00800000`
- [OS == .LINUX] `CLONE_CHILD_SETTID :: 0x01000000`
- [OS == .LINUX] `CLONE_NEWCGROUP :: 0x02000000`
- [OS == .LINUX] `CLONE_NEWUTS :: 0x04000000`
- [OS == .LINUX] `CLONE_NEWIPC :: 0x08000000`
- [OS == .LINUX] `CLONE_NEWUSER :: 0x10000000`
- [OS == .LINUX] `CLONE_NEWPID :: 0x20000000`
- [OS == .LINUX] `CLONE_NEWNET :: 0x40000000`
- [OS == .LINUX] `CLONE_IO :: 0x80000000`
- [OS == .LINUX] `_BITS_TYPES_STRUCT_SCHED_PARAM :: 1`
- [OS == .LINUX] `__CPU_SETSIZE :: 1024`
- [OS == .LINUX] `CPU_SETSIZE :: __CPU_SETSIZE`
- [OS == .LINUX] `__PTHREAD_MUTEX_HAVE_PREV :: 1`
- [OS == .LINUX] `__have_pthread_attr_t :: 1`
- [OS == .LINUX] `PTHREAD_CANCELED :: cast,trunc(*void) -1`
- [OS == .LINUX] `PTHREAD_ONCE_INIT :: 0`
- [OS == .LINUX] `PTHREAD_BARRIER_SERIAL_THREAD :: -1`
- [OS == .LINUX] `sched_param :: struct`
- [OS == .LINUX] `clone :: (__fn: #type (__arg: *void) -> s32 #c_call, __child_stack: *void, __flags: s32, __arg: *void, __args: ..Any) -> s32 #foreign libpthread`
- [OS == .LINUX] `unshare :: (__flags: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_getcpu :: () -> s32 #foreign libpthread`
- [OS == .LINUX] `getcpu :: (unknown0: *u32, unknown1: *u32) -> s32 #foreign libpthread`
- [OS == .LINUX] `setns :: (__fd: s32, __nstype: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `__cpu_mask :: u64`
- [OS == .LINUX] `cpu_set_t :: struct`
- [OS == .LINUX] `__sched_cpucount :: (__setsize: u64, __setp: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `__sched_cpualloc :: (__count: u64) -> *cpu_set_t #foreign libpthread`
- [OS == .LINUX] `__sched_cpufree :: (__set: *cpu_set_t) -> void #foreign libpthread`
- [OS == .LINUX] `sched_setparam :: (__pid: s32, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_getparam :: (__pid: s32, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_setscheduler :: (__pid: s32, __policy: s32, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_getscheduler :: (__pid: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_yield :: () -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_get_priority_max :: (__algorithm: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_get_priority_min :: (__algorithm: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_rr_get_interval :: (__pid: s32, __t: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_setaffinity :: (__pid: s32, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `sched_getaffinity :: (__pid: s32, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `__pthread_internal_list :: struct`
- [OS == .LINUX] `__pthread_internal_slist :: struct`
- [OS == .LINUX] `__pthread_mutex_s :: struct`
- [OS == .LINUX] `__pthread_rwlock_arch_t :: struct`
- [OS == .LINUX] `__pthread_cond_s :: struct`
- [OS == .LINUX] `pthread_t :: u64`
- [OS == .LINUX] `pthread_mutexattr_t :: struct`
- [OS == .LINUX] `pthread_condattr_t :: struct`
- [OS == .LINUX] `pthread_key_t :: u32`
- [OS == .LINUX] `pthread_once_t :: s32`
- [OS == .LINUX] `pthread_attr_t :: struct`
- [OS == .LINUX] `pthread_mutex_t :: struct`
- [OS == .LINUX] `pthread_cond_t :: struct`
- [OS == .LINUX] `pthread_rwlock_t :: struct`
- [OS == .LINUX] `pthread_rwlockattr_t :: struct`
- [OS == .LINUX] `pthread_spinlock_t :: s32`
- [OS == .LINUX] `pthread_barrier_t :: struct`
- [OS == .LINUX] `pthread_barrierattr_t :: struct`
- [OS == .LINUX] `__jmp_buf :: [8] s64`
- [OS == .LINUX] `PTHREAD_CREATE :: enum u32`
- [OS == .LINUX] `PTHREAD_MUTEX :: enum u32`
- [OS == .LINUX] `PTHREAD_MUTEX_1 :: enum u32`
- [OS == .LINUX] `PTHREAD_PRIO :: enum u32`
- [OS == .LINUX] `PTHREAD_RWLOCK :: enum u32`
- [OS == .LINUX] `PTHREAD :: enum u32`
- [OS == .LINUX] `PTHREAD_SCOPE :: enum u32`
- [OS == .LINUX] `PTHREAD_PROCESS :: enum u32`
- [OS == .LINUX] `_pthread_cleanup_buffer :: struct`
- [OS == .LINUX] `PTHREAD_CANCEL :: enum u32`
- [OS == .LINUX] `PTHREAD_CANCEL_1 :: enum u32`
- [OS == .LINUX] `pthread_create :: (__newthread: *pthread_t, __attr: *pthread_attr_t, __start_routine: #type (unknown0: *void) -> *void #c_call, __arg: *void) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_exit :: (__retval: *void) -> void #foreign libpthread`
- [OS == .LINUX] `pthread_join :: (__th: pthread_t, __thread_return: **void) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_tryjoin_np :: (__th: pthread_t, __thread_return: **void) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_timedjoin_np :: (__th: pthread_t, __thread_return: **void, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_clockjoin_np :: (__th: pthread_t, __thread_return: **void, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_detach :: (__th: pthread_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_self :: () -> pthread_t #foreign libpthread`
- [OS == .LINUX] `pthread_equal :: (__thread1: pthread_t, __thread2: pthread_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_init :: (__attr: *pthread_attr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_destroy :: (__attr: *pthread_attr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getdetachstate :: (__attr: *pthread_attr_t, __detachstate: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setdetachstate :: (__attr: *pthread_attr_t, __detachstate: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getguardsize :: (__attr: *pthread_attr_t, __guardsize: *u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setguardsize :: (__attr: *pthread_attr_t, __guardsize: u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getschedpolicy :: (__attr: *pthread_attr_t, __policy: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setschedpolicy :: (__attr: *pthread_attr_t, __policy: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getinheritsched :: (__attr: *pthread_attr_t, __inherit: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setinheritsched :: (__attr: *pthread_attr_t, __inherit: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getscope :: (__attr: *pthread_attr_t, __scope: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setscope :: (__attr: *pthread_attr_t, __scope: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getstackaddr :: (__attr: *pthread_attr_t, __stackaddr: **void) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setstackaddr :: (__attr: *pthread_attr_t, __stackaddr: *void) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getstacksize :: (__attr: *pthread_attr_t, __stacksize: *u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setstacksize :: (__attr: *pthread_attr_t, __stacksize: u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getstack :: (__attr: *pthread_attr_t, __stackaddr: **void, __stacksize: *u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setstack :: (__attr: *pthread_attr_t, __stackaddr: *void, __stacksize: u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_setaffinity_np :: (__attr: *pthread_attr_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_attr_getaffinity_np :: (__attr: *pthread_attr_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getattr_default_np :: (__attr: *pthread_attr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setattr_default_np :: (__attr: *pthread_attr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getattr_np :: (__th: pthread_t, __attr: *pthread_attr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setschedparam :: (__target_thread: pthread_t, __policy: s32, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getschedparam :: (__target_thread: pthread_t, __policy: *s32, __param: *sched_param) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setschedprio :: (__target_thread: pthread_t, __prio: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getname_np :: (__target_thread: pthread_t, __buf: *u8, __buflen: u64) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setname_np :: (__target_thread: pthread_t, __name: *u8) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getconcurrency :: () -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setconcurrency :: (__level: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setaffinity_np :: (__th: pthread_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getaffinity_np :: (__th: pthread_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_once :: (__once_control: *pthread_once_t, __init_routine: #type () -> void #c_call) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setcancelstate :: (__state: s32, __oldstate: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_setcanceltype :: (__type: s32, __oldtype: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cancel :: (__th: pthread_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_testcancel :: () -> void #foreign libpthread`
- [OS == .LINUX] `__pthread_unwind_buf_t :: struct`
- [OS == .LINUX] `__pthread_cleanup_frame :: struct`
- [OS == .LINUX] `__pthread_register_cancel :: (__buf: *__pthread_unwind_buf_t) -> void #foreign libpthread`
- [OS == .LINUX] `__pthread_unregister_cancel :: (__buf: *__pthread_unwind_buf_t) -> void #foreign libpthread`
- [OS == .LINUX] `__pthread_register_cancel_defer :: (__buf: *__pthread_unwind_buf_t) -> void #foreign libpthread`
- [OS == .LINUX] `__pthread_unregister_cancel_restore :: (__buf: *__pthread_unwind_buf_t) -> void #foreign libpthread`
- [OS == .LINUX] `__pthread_unwind_next :: (__buf: *__pthread_unwind_buf_t) -> void #foreign libpthread`
- [OS == .LINUX] `__sigsetjmp :: (__env: *__jmp_buf_tag, __savemask: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_init :: (__mutex: *pthread_mutex_t, __mutexattr: *pthread_mutexattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_destroy :: (__mutex: *pthread_mutex_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_trylock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_lock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_timedlock :: (__mutex: *pthread_mutex_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_clocklock :: (__mutex: *pthread_mutex_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_unlock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_getprioceiling :: (__mutex: *pthread_mutex_t, __prioceiling: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_setprioceiling :: (__mutex: *pthread_mutex_t, __prioceiling: s32, __old_ceiling: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutex_consistent :: (__mutex: *pthread_mutex_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_init :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_destroy :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_getpshared :: (__attr: *pthread_mutexattr_t, __pshared: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_setpshared :: (__attr: *pthread_mutexattr_t, __pshared: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_gettype :: (__attr: *pthread_mutexattr_t, __kind: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_settype :: (__attr: *pthread_mutexattr_t, __kind: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_getprotocol :: (__attr: *pthread_mutexattr_t, __protocol: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_setprotocol :: (__attr: *pthread_mutexattr_t, __protocol: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_getprioceiling :: (__attr: *pthread_mutexattr_t, __prioceiling: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_setprioceiling :: (__attr: *pthread_mutexattr_t, __prioceiling: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_getrobust :: (__attr: *pthread_mutexattr_t, __robustness: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_mutexattr_setrobust :: (__attr: *pthread_mutexattr_t, __robustness: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_init :: (__rwlock: *pthread_rwlock_t, __attr: *pthread_rwlockattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_destroy :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_rdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_tryrdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_timedrdlock :: (__rwlock: *pthread_rwlock_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_clockrdlock :: (__rwlock: *pthread_rwlock_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_wrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_trywrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_timedwrlock :: (__rwlock: *pthread_rwlock_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_clockwrlock :: (__rwlock: *pthread_rwlock_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlock_unlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlockattr_init :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlockattr_destroy :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlockattr_getpshared :: (__attr: *pthread_rwlockattr_t, __pshared: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlockattr_setpshared :: (__attr: *pthread_rwlockattr_t, __pshared: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlockattr_getkind_np :: (__attr: *pthread_rwlockattr_t, __pref: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_rwlockattr_setkind_np :: (__attr: *pthread_rwlockattr_t, __pref: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_init :: (__cond: *pthread_cond_t, __cond_attr: *pthread_condattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_destroy :: (__cond: *pthread_cond_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_signal :: (__cond: *pthread_cond_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_broadcast :: (__cond: *pthread_cond_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_wait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_timedwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_cond_clockwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __clock_id: s32, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_condattr_init :: (__attr: *pthread_condattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_condattr_destroy :: (__attr: *pthread_condattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_condattr_getpshared :: (__attr: *pthread_condattr_t, __pshared: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_condattr_setpshared :: (__attr: *pthread_condattr_t, __pshared: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_condattr_getclock :: (__attr: *pthread_condattr_t, __clock_id: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_condattr_setclock :: (__attr: *pthread_condattr_t, __clock_id: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_spin_init :: (__lock: *pthread_spinlock_t, __pshared: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_spin_destroy :: (__lock: *pthread_spinlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_spin_lock :: (__lock: *pthread_spinlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_spin_trylock :: (__lock: *pthread_spinlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_spin_unlock :: (__lock: *pthread_spinlock_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrier_init :: (__barrier: *pthread_barrier_t, __attr: *pthread_barrierattr_t, __count: u32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrier_destroy :: (__barrier: *pthread_barrier_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrier_wait :: (__barrier: *pthread_barrier_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrierattr_init :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrierattr_destroy :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrierattr_getpshared :: (__attr: *pthread_barrierattr_t, __pshared: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_barrierattr_setpshared :: (__attr: *pthread_barrierattr_t, __pshared: s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_key_create :: (__key: *pthread_key_t, __destr_function: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_key_delete :: (__key: pthread_key_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getspecific :: (__key: pthread_key_t) -> *void #foreign libpthread`
- [OS == .LINUX] `pthread_setspecific :: (__key: pthread_key_t, __pointer: *void) -> s32 #foreign libpthread`
- [OS == .LINUX] `pthread_getcpuclockid :: (__thread_id: pthread_t, __clock_id: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `__sigset_t :: struct`
- [OS == .LINUX] `sem_t :: struct`
- [OS == .LINUX] `sem_init :: (__sem: *sem_t, __pshared: s32, __value: u32) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_destroy :: (__sem: *sem_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_open :: (__name: *u8, __oflag: s32, __args: ..Any) -> *sem_t #foreign libpthread`
- [OS == .LINUX] `sem_close :: (__sem: *sem_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_unlink :: (__name: *u8) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_wait :: (__sem: *sem_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_timedwait :: (__sem: *sem_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_clockwait :: (__sem: *sem_t, clock: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_trywait :: (__sem: *sem_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_post :: (__sem: *sem_t) -> s32 #foreign libpthread`
- [OS == .LINUX] `sem_getvalue :: (__sem: *sem_t, __sval: *s32) -> s32 #foreign libpthread`
- [OS == .LINUX] `__jmp_buf_tag :: struct`
- [OS == .LINUX] `jmp_buf :: [1] __jmp_buf_tag`
- [OS == .LINUX] `setjmp :: (__env: *jmp_buf) -> s32 #foreign libpthread`
- [OS == .LINUX] `_setjmp :: (__env: *[1] __jmp_buf_tag) -> s32 #foreign libpthread`
- [OS == .LINUX] `longjmp :: (__env: *[1] __jmp_buf_tag, __val: s32) -> void #foreign libpthread`
- [OS == .LINUX] `_longjmp :: (__env: *[1] __jmp_buf_tag, __val: s32) -> void #foreign libpthread`
- [OS == .LINUX] `sigjmp_buf :: [1] __jmp_buf_tag`
- [OS == .LINUX] `siglongjmp :: (__env: *sigjmp_buf, __val: s32) -> void #foreign libpthread`
- [OS == .LINUX] `RLIM_INFINITY :: cast,trunc(u64) -1`
- [OS == .LINUX] `RLIM64_INFINITY :: 0xffffffffffffffff`
- [OS == .LINUX] `RLIM_SAVED_MAX :: RLIM_INFINITY`
- [OS == .LINUX] `RLIM_SAVED_CUR :: RLIM_INFINITY`
- [OS == .LINUX] `__rusage_defined :: 1`
- [OS == .LINUX] `PRIO_MIN :: -20`
- [OS == .LINUX] `PRIO_MAX :: 20`
- [OS == .LINUX] `RLIMIT_RESOURCE :: enum u32`
- [OS == .LINUX] `rlim_t :: u64`
- [OS == .LINUX] `rlim64_t :: u64`
- [OS == .LINUX] `rlimit :: struct`
- [OS == .LINUX] `rlimit64 :: struct`
- [OS == .LINUX] `RUSAGE :: enum s32`
- [OS == .LINUX] `rusage :: struct`
- [OS == .LINUX] `PRIO :: enum u32`
- [OS == .LINUX] `prlimit :: (__pid: s32, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit, __old_limit: *rlimit) -> s32 #foreign libc`
- [OS == .LINUX] `prlimit64 :: (__pid: s32, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit64, __old_limit: *rlimit64) -> s32 #foreign libc`
- [OS == .LINUX] `id_t :: u32`
- [OS == .LINUX] `getrlimit :: (__resource: RLIMIT_RESOURCE, __rlimits: *rlimit) -> s32 #foreign libc`
- [OS == .LINUX] `getrlimit64 :: (__resource: RLIMIT_RESOURCE, __rlimits: *rlimit64) -> s32 #foreign libc`
- [OS == .LINUX] `setrlimit :: (__resource: RLIMIT_RESOURCE, __rlimits: *rlimit) -> s32 #foreign libc`
- [OS == .LINUX] `setrlimit64 :: (__resource: RLIMIT_RESOURCE, __rlimits: *rlimit64) -> s32 #foreign libc`
- [OS == .LINUX] `getrusage :: (__who: RUSAGE, __usage: *rusage) -> s32 #foreign libc`
- [OS == .LINUX] `getpriority :: (__which: PRIO, __who: id_t) -> s32 #foreign libc`
- [OS == .LINUX] `setpriority :: (__which: PRIO, __who: id_t, __prio: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `RTLD_LAZY :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `RTLD_NOW :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `RTLD_LOCAL :: 0x4`
- [OS == .MACOS && CPU == .ARM64] `RTLD_GLOBAL :: 0x8`
- [OS == .MACOS && CPU == .ARM64] `RTLD_NOLOAD :: 0x10`
- [OS == .MACOS && CPU == .ARM64] `RTLD_NODELETE :: 0x80`
- [OS == .MACOS && CPU == .ARM64] `RTLD_FIRST :: 0x100`
- [OS == .MACOS && CPU == .ARM64] `RTLD_NEXT :: cast,trunc(*void) -1`
- [OS == .MACOS && CPU == .ARM64] `RTLD_DEFAULT :: cast,trunc(*void) -2`
- [OS == .MACOS && CPU == .ARM64] `RTLD_SELF :: cast,trunc(*void) -3`
- [OS == .MACOS && CPU == .ARM64] `RTLD_MAIN_ONLY :: cast,trunc(*void) -5`
- [OS == .MACOS && CPU == .ARM64] `__DARWIN_MAXNAMLEN :: 255`
- [OS == .MACOS && CPU == .ARM64] `__DARWIN_MAXPATHLEN :: 1024`
- [OS == .MACOS && CPU == .ARM64] `MAXNAMLEN :: __DARWIN_MAXNAMLEN`
- [OS == .MACOS && CPU == .ARM64] `DT :: enum u8`
- [OS == .MACOS && CPU == .ARM64] `DIRBLKSIZ :: 1024`
- [OS == .MACOS && CPU == .ARM64] `DTF_HIDEW :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `DTF_NODUP :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `DTF_REWIND :: 0x0004`
- [OS == .MACOS && CPU == .ARM64] `__DTF_READALL :: 0x0008`
- [OS == .MACOS && CPU == .ARM64] `__DTF_SKIPREAD :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `__DTF_ATEND :: 0x0020`
- [OS == .MACOS && CPU == .ARM64] `EPERM :: 1`
- [OS == .MACOS && CPU == .ARM64] `ENOENT :: 2`
- [OS == .MACOS && CPU == .ARM64] `ESRCH :: 3`
- [OS == .MACOS && CPU == .ARM64] `EINTR :: 4`
- [OS == .MACOS && CPU == .ARM64] `EIO :: 5`
- [OS == .MACOS && CPU == .ARM64] `ENXIO :: 6`
- [OS == .MACOS && CPU == .ARM64] `E2BIG :: 7`
- [OS == .MACOS && CPU == .ARM64] `ENOEXEC :: 8`
- [OS == .MACOS && CPU == .ARM64] `EBADF :: 9`
- [OS == .MACOS && CPU == .ARM64] `ECHILD :: 10`
- [OS == .MACOS && CPU == .ARM64] `EDEADLK :: 11`
- [OS == .MACOS && CPU == .ARM64] `ENOMEM :: 12`
- [OS == .MACOS && CPU == .ARM64] `EACCES :: 13`
- [OS == .MACOS && CPU == .ARM64] `EFAULT :: 14`
- [OS == .MACOS && CPU == .ARM64] `ENOTBLK :: 15`
- [OS == .MACOS && CPU == .ARM64] `EBUSY :: 16`
- [OS == .MACOS && CPU == .ARM64] `EEXIST :: 17`
- [OS == .MACOS && CPU == .ARM64] `EXDEV :: 18`
- [OS == .MACOS && CPU == .ARM64] `ENODEV :: 19`
- [OS == .MACOS && CPU == .ARM64] `ENOTDIR :: 20`
- [OS == .MACOS && CPU == .ARM64] `EISDIR :: 21`
- [OS == .MACOS && CPU == .ARM64] `EINVAL :: 22`
- [OS == .MACOS && CPU == .ARM64] `ENFILE :: 23`
- [OS == .MACOS && CPU == .ARM64] `EMFILE :: 24`
- [OS == .MACOS && CPU == .ARM64] `ENOTTY :: 25`
- [OS == .MACOS && CPU == .ARM64] `ETXTBSY :: 26`
- [OS == .MACOS && CPU == .ARM64] `EFBIG :: 27`
- [OS == .MACOS && CPU == .ARM64] `ENOSPC :: 28`
- [OS == .MACOS && CPU == .ARM64] `ESPIPE :: 29`
- [OS == .MACOS && CPU == .ARM64] `EROFS :: 30`
- [OS == .MACOS && CPU == .ARM64] `EMLINK :: 31`
- [OS == .MACOS && CPU == .ARM64] `EPIPE :: 32`
- [OS == .MACOS && CPU == .ARM64] `EDOM :: 33`
- [OS == .MACOS && CPU == .ARM64] `ERANGE :: 34`
- [OS == .MACOS && CPU == .ARM64] `EAGAIN :: 35`
- [OS == .MACOS && CPU == .ARM64] `EWOULDBLOCK :: EAGAIN`
- [OS == .MACOS && CPU == .ARM64] `EINPROGRESS :: 36`
- [OS == .MACOS && CPU == .ARM64] `EALREADY :: 37`
- [OS == .MACOS && CPU == .ARM64] `ENOTSOCK :: 38`
- [OS == .MACOS && CPU == .ARM64] `EDESTADDRREQ :: 39`
- [OS == .MACOS && CPU == .ARM64] `EMSGSIZE :: 40`
- [OS == .MACOS && CPU == .ARM64] `EPROTOTYPE :: 41`
- [OS == .MACOS && CPU == .ARM64] `ENOPROTOOPT :: 42`
- [OS == .MACOS && CPU == .ARM64] `EPROTONOSUPPORT :: 43`
- [OS == .MACOS && CPU == .ARM64] `ESOCKTNOSUPPORT :: 44`
- [OS == .MACOS && CPU == .ARM64] `ENOTSUP :: 45`
- [OS == .MACOS && CPU == .ARM64] `EPFNOSUPPORT :: 46`
- [OS == .MACOS && CPU == .ARM64] `EAFNOSUPPORT :: 47`
- [OS == .MACOS && CPU == .ARM64] `EADDRINUSE :: 48`
- [OS == .MACOS && CPU == .ARM64] `EADDRNOTAVAIL :: 49`
- [OS == .MACOS && CPU == .ARM64] `ENETDOWN :: 50`
- [OS == .MACOS && CPU == .ARM64] `ENETUNREACH :: 51`
- [OS == .MACOS && CPU == .ARM64] `ENETRESET :: 52`
- [OS == .MACOS && CPU == .ARM64] `ECONNABORTED :: 53`
- [OS == .MACOS && CPU == .ARM64] `ECONNRESET :: 54`
- [OS == .MACOS && CPU == .ARM64] `ENOBUFS :: 55`
- [OS == .MACOS && CPU == .ARM64] `EISCONN :: 56`
- [OS == .MACOS && CPU == .ARM64] `ENOTCONN :: 57`
- [OS == .MACOS && CPU == .ARM64] `ESHUTDOWN :: 58`
- [OS == .MACOS && CPU == .ARM64] `ETOOMANYREFS :: 59`
- [OS == .MACOS && CPU == .ARM64] `ETIMEDOUT :: 60`
- [OS == .MACOS && CPU == .ARM64] `ECONNREFUSED :: 61`
- [OS == .MACOS && CPU == .ARM64] `ELOOP :: 62`
- [OS == .MACOS && CPU == .ARM64] `ENAMETOOLONG :: 63`
- [OS == .MACOS && CPU == .ARM64] `EHOSTDOWN :: 64`
- [OS == .MACOS && CPU == .ARM64] `EHOSTUNREACH :: 65`
- [OS == .MACOS && CPU == .ARM64] `ENOTEMPTY :: 66`
- [OS == .MACOS && CPU == .ARM64] `EPROCLIM :: 67`
- [OS == .MACOS && CPU == .ARM64] `EUSERS :: 68`
- [OS == .MACOS && CPU == .ARM64] `EDQUOT :: 69`
- [OS == .MACOS && CPU == .ARM64] `ESTALE :: 70`
- [OS == .MACOS && CPU == .ARM64] `EREMOTE :: 71`
- [OS == .MACOS && CPU == .ARM64] `EBADRPC :: 72`
- [OS == .MACOS && CPU == .ARM64] `ERPCMISMATCH :: 73`
- [OS == .MACOS && CPU == .ARM64] `EPROGUNAVAIL :: 74`
- [OS == .MACOS && CPU == .ARM64] `EPROGMISMATCH :: 75`
- [OS == .MACOS && CPU == .ARM64] `EPROCUNAVAIL :: 76`
- [OS == .MACOS && CPU == .ARM64] `ENOLCK :: 77`
- [OS == .MACOS && CPU == .ARM64] `ENOSYS :: 78`
- [OS == .MACOS && CPU == .ARM64] `EFTYPE :: 79`
- [OS == .MACOS && CPU == .ARM64] `EAUTH :: 80`
- [OS == .MACOS && CPU == .ARM64] `ENEEDAUTH :: 81`
- [OS == .MACOS && CPU == .ARM64] `EPWROFF :: 82`
- [OS == .MACOS && CPU == .ARM64] `EDEVERR :: 83`
- [OS == .MACOS && CPU == .ARM64] `EOVERFLOW :: 84`
- [OS == .MACOS && CPU == .ARM64] `EBADEXEC :: 85`
- [OS == .MACOS && CPU == .ARM64] `EBADARCH :: 86`
- [OS == .MACOS && CPU == .ARM64] `ESHLIBVERS :: 87`
- [OS == .MACOS && CPU == .ARM64] `EBADMACHO :: 88`
- [OS == .MACOS && CPU == .ARM64] `ECANCELED :: 89`
- [OS == .MACOS && CPU == .ARM64] `EIDRM :: 90`
- [OS == .MACOS && CPU == .ARM64] `ENOMSG :: 91`
- [OS == .MACOS && CPU == .ARM64] `EILSEQ :: 92`
- [OS == .MACOS && CPU == .ARM64] `ENOATTR :: 93`
- [OS == .MACOS && CPU == .ARM64] `EBADMSG :: 94`
- [OS == .MACOS && CPU == .ARM64] `EMULTIHOP :: 95`
- [OS == .MACOS && CPU == .ARM64] `ENODATA :: 96`
- [OS == .MACOS && CPU == .ARM64] `ENOLINK :: 97`
- [OS == .MACOS && CPU == .ARM64] `ENOSR :: 98`
- [OS == .MACOS && CPU == .ARM64] `ENOSTR :: 99`
- [OS == .MACOS && CPU == .ARM64] `EPROTO :: 100`
- [OS == .MACOS && CPU == .ARM64] `ETIME :: 101`
- [OS == .MACOS && CPU == .ARM64] `EOPNOTSUPP :: 102`
- [OS == .MACOS && CPU == .ARM64] `ENOPOLICY :: 103`
- [OS == .MACOS && CPU == .ARM64] `ENOTRECOVERABLE :: 104`
- [OS == .MACOS && CPU == .ARM64] `EOWNERDEAD :: 105`
- [OS == .MACOS && CPU == .ARM64] `EQFULL :: 106`
- [OS == .MACOS && CPU == .ARM64] `ELAST :: 106`
- [OS == .MACOS && CPU == .ARM64] `FNM_NOMATCH :: 1`
- [OS == .MACOS && CPU == .ARM64] `FNM_NOESCAPE :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `FNM_PATHNAME :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `FNM_PERIOD :: 0x04`
- [OS == .MACOS && CPU == .ARM64] `FNM_NOSYS :: -1`
- [OS == .MACOS && CPU == .ARM64] `FNM_LEADING_DIR :: 0x08`
- [OS == .MACOS && CPU == .ARM64] `FNM_CASEFOLD :: 0x10`
- [OS == .MACOS && CPU == .ARM64] `FNM_IGNORECASE :: FNM_CASEFOLD`
- [OS == .MACOS && CPU == .ARM64] `FNM_FILE_NAME :: FNM_PATHNAME`
- [OS == .MACOS && CPU == .ARM64] `_PATH_PWD :: "/etc"`
- [OS == .MACOS && CPU == .ARM64] `_PATH_PASSWD :: "/etc/passwd"`
- [OS == .MACOS && CPU == .ARM64] `_PASSWD :: "passwd"`
- [OS == .MACOS && CPU == .ARM64] `_PATH_MASTERPASSWD :: "/etc/master.passwd"`
- [OS == .MACOS && CPU == .ARM64] `_PATH_MASTERPASSWD_LOCK :: "/etc/ptmp"`
- [OS == .MACOS && CPU == .ARM64] `_MASTERPASSWD :: "master.passwd"`
- [OS == .MACOS && CPU == .ARM64] `_PATH_MP_DB :: "/etc/pwd.db"`
- [OS == .MACOS && CPU == .ARM64] `_MP_DB :: "pwd.db"`
- [OS == .MACOS && CPU == .ARM64] `_PATH_SMP_DB :: "/etc/spwd.db"`
- [OS == .MACOS && CPU == .ARM64] `_SMP_DB :: "spwd.db"`
- [OS == .MACOS && CPU == .ARM64] `_PATH_PWD_MKDB :: "/usr/sbin/pwd_mkdb"`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_LEN :: 128`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_NOUID :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_NOGID :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_NOCHG :: 0x04`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_NOEXP :: 0x08`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_WARNDAYS :: 14`
- [OS == .MACOS && CPU == .ARM64] `_PASSWORD_CHGNOW :: -1`
- [OS == .MACOS && CPU == .ARM64] `__DARWIN_NSIG :: 32`
- [OS == .MACOS && CPU == .ARM64] `NSIG :: __DARWIN_NSIG`
- [OS == .MACOS && CPU == .ARM64] `_ARM_SIGNAL_ :: 1`
- [OS == .MACOS && CPU == .ARM64] `SIGHUP :: 1`
- [OS == .MACOS && CPU == .ARM64] `SIGINT :: 2`
- [OS == .MACOS && CPU == .ARM64] `SIGQUIT :: 3`
- [OS == .MACOS && CPU == .ARM64] `SIGILL :: 4`
- [OS == .MACOS && CPU == .ARM64] `SIGTRAP :: 5`
- [OS == .MACOS && CPU == .ARM64] `SIGABRT :: 6`
- [OS == .MACOS && CPU == .ARM64] `SIGIOT :: SIGABRT`
- [OS == .MACOS && CPU == .ARM64] `SIGEMT :: 7`
- [OS == .MACOS && CPU == .ARM64] `SIGFPE :: 8`
- [OS == .MACOS && CPU == .ARM64] `SIGKILL :: 9`
- [OS == .MACOS && CPU == .ARM64] `SIGBUS :: 10`
- [OS == .MACOS && CPU == .ARM64] `SIGSEGV :: 11`
- [OS == .MACOS && CPU == .ARM64] `SIGSYS :: 12`
- [OS == .MACOS && CPU == .ARM64] `SIGPIPE :: 13`
- [OS == .MACOS && CPU == .ARM64] `SIGALRM :: 14`
- [OS == .MACOS && CPU == .ARM64] `SIGTERM :: 15`
- [OS == .MACOS && CPU == .ARM64] `SIGURG :: 16`
- [OS == .MACOS && CPU == .ARM64] `SIGSTOP :: 17`
- [OS == .MACOS && CPU == .ARM64] `SIGTSTP :: 18`
- [OS == .MACOS && CPU == .ARM64] `SIGCONT :: 19`
- [OS == .MACOS && CPU == .ARM64] `SIGCHLD :: 20`
- [OS == .MACOS && CPU == .ARM64] `SIGTTIN :: 21`
- [OS == .MACOS && CPU == .ARM64] `SIGTTOU :: 22`
- [OS == .MACOS && CPU == .ARM64] `SIGIO :: 23`
- [OS == .MACOS && CPU == .ARM64] `SIGXCPU :: 24`
- [OS == .MACOS && CPU == .ARM64] `SIGXFSZ :: 25`
- [OS == .MACOS && CPU == .ARM64] `SIGVTALRM :: 26`
- [OS == .MACOS && CPU == .ARM64] `SIGPROF :: 27`
- [OS == .MACOS && CPU == .ARM64] `SIGWINCH :: 28`
- [OS == .MACOS && CPU == .ARM64] `SIGINFO :: 29`
- [OS == .MACOS && CPU == .ARM64] `SIGUSR1 :: 30`
- [OS == .MACOS && CPU == .ARM64] `SIGUSR2 :: 31`
- [OS == .MACOS && CPU == .ARM64] `SIG_DFL :: cast(*void) 0`
- [OS == .MACOS && CPU == .ARM64] `SIG_IGN :: cast(*void) 1`
- [OS == .MACOS && CPU == .ARM64] `SIG_HOLD :: cast(*void) 5`
- [OS == .MACOS && CPU == .ARM64] `SIG_ERR :: cast(*void) -1`
- [OS == .MACOS && CPU == .ARM64] `__DARWIN_OPAQUE_ARM_THREAD_STATE64 :: 0`
- [OS == .MACOS && CPU == .ARM64] `SIGEV_NONE :: 0`
- [OS == .MACOS && CPU == .ARM64] `SIGEV_SIGNAL :: 1`
- [OS == .MACOS && CPU == .ARM64] `SIGEV_THREAD :: 3`
- [OS == .MACOS && CPU == .ARM64] `ILL_NOOP :: 0`
- [OS == .MACOS && CPU == .ARM64] `ILL_ILLOPC :: 1`
- [OS == .MACOS && CPU == .ARM64] `ILL_ILLTRP :: 2`
- [OS == .MACOS && CPU == .ARM64] `ILL_PRVOPC :: 3`
- [OS == .MACOS && CPU == .ARM64] `ILL_ILLOPN :: 4`
- [OS == .MACOS && CPU == .ARM64] `ILL_ILLADR :: 5`
- [OS == .MACOS && CPU == .ARM64] `ILL_PRVREG :: 6`
- [OS == .MACOS && CPU == .ARM64] `ILL_COPROC :: 7`
- [OS == .MACOS && CPU == .ARM64] `ILL_BADSTK :: 8`
- [OS == .MACOS && CPU == .ARM64] `FPE_NOOP :: 0`
- [OS == .MACOS && CPU == .ARM64] `FPE_FLTDIV :: 1`
- [OS == .MACOS && CPU == .ARM64] `FPE_FLTOVF :: 2`
- [OS == .MACOS && CPU == .ARM64] `FPE_FLTUND :: 3`
- [OS == .MACOS && CPU == .ARM64] `FPE_FLTRES :: 4`
- [OS == .MACOS && CPU == .ARM64] `FPE_FLTINV :: 5`
- [OS == .MACOS && CPU == .ARM64] `FPE_FLTSUB :: 6`
- [OS == .MACOS && CPU == .ARM64] `FPE_INTDIV :: 7`
- [OS == .MACOS && CPU == .ARM64] `FPE_INTOVF :: 8`
- [OS == .MACOS && CPU == .ARM64] `SEGV_NOOP :: 0`
- [OS == .MACOS && CPU == .ARM64] `SEGV_MAPERR :: 1`
- [OS == .MACOS && CPU == .ARM64] `SEGV_ACCERR :: 2`
- [OS == .MACOS && CPU == .ARM64] `BUS_NOOP :: 0`
- [OS == .MACOS && CPU == .ARM64] `BUS_ADRALN :: 1`
- [OS == .MACOS && CPU == .ARM64] `BUS_ADRERR :: 2`
- [OS == .MACOS && CPU == .ARM64] `BUS_OBJERR :: 3`
- [OS == .MACOS && CPU == .ARM64] `TRAP_BRKPT :: 1`
- [OS == .MACOS && CPU == .ARM64] `TRAP_TRACE :: 2`
- [OS == .MACOS && CPU == .ARM64] `CLD_NOOP :: 0`
- [OS == .MACOS && CPU == .ARM64] `CLD_EXITED :: 1`
- [OS == .MACOS && CPU == .ARM64] `CLD_KILLED :: 2`
- [OS == .MACOS && CPU == .ARM64] `CLD_DUMPED :: 3`
- [OS == .MACOS && CPU == .ARM64] `CLD_TRAPPED :: 4`
- [OS == .MACOS && CPU == .ARM64] `CLD_STOPPED :: 5`
- [OS == .MACOS && CPU == .ARM64] `CLD_CONTINUED :: 6`
- [OS == .MACOS && CPU == .ARM64] `POLL_IN :: 1`
- [OS == .MACOS && CPU == .ARM64] `POLL_OUT :: 2`
- [OS == .MACOS && CPU == .ARM64] `POLL_MSG :: 3`
- [OS == .MACOS && CPU == .ARM64] `POLL_ERR :: 4`
- [OS == .MACOS && CPU == .ARM64] `POLL_PRI :: 5`
- [OS == .MACOS && CPU == .ARM64] `POLL_HUP :: 6`
- [OS == .MACOS && CPU == .ARM64] `SA_ONSTACK :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `SA_RESTART :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `SA_RESETHAND :: 0x0004`
- [OS == .MACOS && CPU == .ARM64] `SA_NOCLDSTOP :: 0x0008`
- [OS == .MACOS && CPU == .ARM64] `SA_NODEFER :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `SA_NOCLDWAIT :: 0x0020`
- [OS == .MACOS && CPU == .ARM64] `SA_SIGINFO :: 0x0040`
- [OS == .MACOS && CPU == .ARM64] `SA_USERTRAMP :: 0x0100`
- [OS == .MACOS && CPU == .ARM64] `SA_64REGSET :: 0x0200`
- [OS == .MACOS && CPU == .ARM64] `SA_USERSPACE_MASK :: SA_ONSTACK | SA_RESTART | SA_RESETHAND | SA_NOCLDSTOP | SA_NODEFER | SA_NOCLDWAIT | SA_SIGINFO`
- [OS == .MACOS && CPU == .ARM64] `SIG_BLOCK :: 1`
- [OS == .MACOS && CPU == .ARM64] `SIG_UNBLOCK :: 2`
- [OS == .MACOS && CPU == .ARM64] `SIG_SETMASK :: 3`
- [OS == .MACOS && CPU == .ARM64] `SI_USER :: 0x10001`
- [OS == .MACOS && CPU == .ARM64] `SI_QUEUE :: 0x10002`
- [OS == .MACOS && CPU == .ARM64] `SI_TIMER :: 0x10003`
- [OS == .MACOS && CPU == .ARM64] `SI_ASYNCIO :: 0x10004`
- [OS == .MACOS && CPU == .ARM64] `SI_MESGQ :: 0x10005`
- [OS == .MACOS && CPU == .ARM64] `SS_ONSTACK :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `SS_DISABLE :: 0x0004`
- [OS == .MACOS && CPU == .ARM64] `MINSIGSTKSZ :: 32768`
- [OS == .MACOS && CPU == .ARM64] `SIGSTKSZ :: 131072`
- [OS == .MACOS && CPU == .ARM64] `SV_ONSTACK :: SA_ONSTACK`
- [OS == .MACOS && CPU == .ARM64] `SV_INTERRUPT :: SA_RESTART`
- [OS == .MACOS && CPU == .ARM64] `SV_RESETHAND :: SA_RESETHAND`
- [OS == .MACOS && CPU == .ARM64] `SV_NODEFER :: SA_NODEFER`
- [OS == .MACOS && CPU == .ARM64] `SV_NOCLDSTOP :: SA_NOCLDSTOP`
- [OS == .MACOS && CPU == .ARM64] `SV_SIGINFO :: SA_SIGINFO`
- [OS == .MACOS && CPU == .ARM64] `BADSIG :: SIG_ERR`
- [OS == .MACOS && CPU == .ARM64] `__ipc_perm_new :: ipc_perm`
- [OS == .MACOS && CPU == .ARM64] `IPC_CREAT :: 0x200`
- [OS == .MACOS && CPU == .ARM64] `IPC_EXCL :: 0x400`
- [OS == .MACOS && CPU == .ARM64] `IPC_NOWAIT :: 0x800`
- [OS == .MACOS && CPU == .ARM64] `IPC_PRIVATE :: cast(key_t) 0`
- [OS == .MACOS && CPU == .ARM64] `IPC_RMID :: 0`
- [OS == .MACOS && CPU == .ARM64] `IPC_SET :: 1`
- [OS == .MACOS && CPU == .ARM64] `IPC_STAT :: 2`
- [OS == .MACOS && CPU == .ARM64] `IPC_R :: 0x100`
- [OS == .MACOS && CPU == .ARM64] `IPC_W :: 0x80`
- [OS == .MACOS && CPU == .ARM64] `IPC_M :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `PROT_NONE :: 0x00`
- [OS == .MACOS && CPU == .ARM64] `PROT_READ :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `PROT_WRITE :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `PROT_EXEC :: 0x04`
- [OS == .MACOS && CPU == .ARM64] `MAP_SHARED :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `MAP_PRIVATE :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `MAP_COPY :: MAP_PRIVATE`
- [OS == .MACOS && CPU == .ARM64] `MAP_FIXED :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `MAP_RENAME :: 0x0020`
- [OS == .MACOS && CPU == .ARM64] `MAP_NORESERVE :: 0x0040`
- [OS == .MACOS && CPU == .ARM64] `MAP_RESERVED0080 :: 0x0080`
- [OS == .MACOS && CPU == .ARM64] `MAP_NOEXTEND :: 0x0100`
- [OS == .MACOS && CPU == .ARM64] `MAP_HASSEMAPHORE :: 0x0200`
- [OS == .MACOS && CPU == .ARM64] `MAP_NOCACHE :: 0x0400`
- [OS == .MACOS && CPU == .ARM64] `MAP_JIT :: 0x0800`
- [OS == .MACOS && CPU == .ARM64] `MAP_FILE :: 0x0000`
- [OS == .MACOS && CPU == .ARM64] `MAP_ANON :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `MAP_ANONYMOUS :: MAP_ANON`
- [OS == .MACOS && CPU == .ARM64] `MAP_RESILIENT_CODESIGN :: 0x2000`
- [OS == .MACOS && CPU == .ARM64] `MAP_RESILIENT_MEDIA :: 0x4000`
- [OS == .MACOS && CPU == .ARM64] `MAP_32BIT :: 0x8000`
- [OS == .MACOS && CPU == .ARM64] `MAP_TRANSLATED_ALLOW_EXECUTE :: 0x20000`
- [OS == .MACOS && CPU == .ARM64] `MAP_UNIX03 :: 0x40000`
- [OS == .MACOS && CPU == .ARM64] `MCL_CURRENT :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `MCL_FUTURE :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `MAP_FAILED :: cast,trunc(*void) -1`
- [OS == .MACOS && CPU == .ARM64] `MS_ASYNC :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `MS_INVALIDATE :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `MS_SYNC :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `MS_KILLPAGES :: 0x0004`
- [OS == .MACOS && CPU == .ARM64] `MS_DEACTIVATE :: 0x0008`
- [OS == .MACOS && CPU == .ARM64] `MADV_NORMAL :: 0`
- [OS == .MACOS && CPU == .ARM64] `MADV_RANDOM :: 1`
- [OS == .MACOS && CPU == .ARM64] `MADV_SEQUENTIAL :: 2`
- [OS == .MACOS && CPU == .ARM64] `MADV_WILLNEED :: 3`
- [OS == .MACOS && CPU == .ARM64] `MADV_DONTNEED :: 4`
- [OS == .MACOS && CPU == .ARM64] `MADV_FREE :: 5`
- [OS == .MACOS && CPU == .ARM64] `MADV_ZERO_WIRED_PAGES :: 6`
- [OS == .MACOS && CPU == .ARM64] `MADV_FREE_REUSABLE :: 7`
- [OS == .MACOS && CPU == .ARM64] `MADV_FREE_REUSE :: 8`
- [OS == .MACOS && CPU == .ARM64] `MADV_CAN_REUSE :: 9`
- [OS == .MACOS && CPU == .ARM64] `MADV_PAGEOUT :: 10`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_INCORE :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_REFERENCED :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_MODIFIED :: 0x4`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_REFERENCED_OTHER :: 0x8`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_MODIFIED_OTHER :: 0x10`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_PAGED_OUT :: 0x20`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_COPIED :: 0x40`
- [OS == .MACOS && CPU == .ARM64] `MINCORE_ANONYMOUS :: 0x80`
- [OS == .MACOS && CPU == .ARM64] `POLLIN :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `POLLPRI :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `POLLOUT :: 0x0004`
- [OS == .MACOS && CPU == .ARM64] `POLLRDNORM :: 0x0040`
- [OS == .MACOS && CPU == .ARM64] `POLLWRNORM :: POLLOUT`
- [OS == .MACOS && CPU == .ARM64] `POLLRDBAND :: 0x0080`
- [OS == .MACOS && CPU == .ARM64] `POLLWRBAND :: 0x0100`
- [OS == .MACOS && CPU == .ARM64] `POLLEXTEND :: 0x0200`
- [OS == .MACOS && CPU == .ARM64] `POLLATTRIB :: 0x0400`
- [OS == .MACOS && CPU == .ARM64] `POLLNLINK :: 0x0800`
- [OS == .MACOS && CPU == .ARM64] `POLLWRITE :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `POLLERR :: 0x0008`
- [OS == .MACOS && CPU == .ARM64] `POLLHUP :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `POLLNVAL :: 0x0020`
- [OS == .MACOS && CPU == .ARM64] `POLLSTANDARD :: POLLIN|POLLPRI|POLLOUT|POLLRDNORM|POLLRDBAND| POLLWRBAND|POLLERR|POLLHUP|POLLNVAL`
- [OS == .MACOS && CPU == .ARM64] `SHM_RDONLY :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `SHM_RND :: 0x2000`
- [OS == .MACOS && CPU == .ARM64] `SHMLBA :: 16*1024`
- [OS == .MACOS && CPU == .ARM64] `SHM_R :: IPC_R`
- [OS == .MACOS && CPU == .ARM64] `SHM_W :: IPC_W`
- [OS == .MACOS && CPU == .ARM64] `shmid_ds :: __shmid_ds_new`
- [OS == .MACOS && CPU == .ARM64] `ITIMER_REAL :: 0`
- [OS == .MACOS && CPU == .ARM64] `ITIMER_VIRTUAL :: 1`
- [OS == .MACOS && CPU == .ARM64] `ITIMER_PROF :: 2`
- [OS == .MACOS && CPU == .ARM64] `DST_NONE :: 0`
- [OS == .MACOS && CPU == .ARM64] `DST_USA :: 1`
- [OS == .MACOS && CPU == .ARM64] `DST_AUST :: 2`
- [OS == .MACOS && CPU == .ARM64] `DST_WET :: 3`
- [OS == .MACOS && CPU == .ARM64] `DST_MET :: 4`
- [OS == .MACOS && CPU == .ARM64] `DST_EET :: 5`
- [OS == .MACOS && CPU == .ARM64] `DST_CAN :: 6`
- [OS == .MACOS && CPU == .ARM64] `CLOCKS_PER_SEC :: 1000000`
- [OS == .MACOS && CPU == .ARM64] `TIME_UTC :: 1`
- [OS == .MACOS && CPU == .ARM64] `_SYS_NAMELEN :: 256`
- [OS == .MACOS && CPU == .ARM64] `WNOHANG :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `WUNTRACED :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `WCOREFLAG :: 0x80`
- [OS == .MACOS && CPU == .ARM64] `_WSTOPPED :: 0x7f`
- [OS == .MACOS && CPU == .ARM64] `WEXITED :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `WSTOPPED :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `WCONTINUED :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `WNOWAIT :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `WAIT_ANY :: -1`
- [OS == .MACOS && CPU == .ARM64] `WAIT_MYPGRP :: 0`
- [OS == .MACOS && CPU == .ARM64] `F_OK :: 0`
- [OS == .MACOS && CPU == .ARM64] `X_OK :: 1<<0`
- [OS == .MACOS && CPU == .ARM64] `W_OK :: 1<<1`
- [OS == .MACOS && CPU == .ARM64] `R_OK :: 1<<2`
- [OS == .MACOS && CPU == .ARM64] `_READ_OK :: 1<<9`
- [OS == .MACOS && CPU == .ARM64] `_WRITE_OK :: 1<<10`
- [OS == .MACOS && CPU == .ARM64] `_EXECUTE_OK :: 1<<11`
- [OS == .MACOS && CPU == .ARM64] `_DELETE_OK :: 1<<12`
- [OS == .MACOS && CPU == .ARM64] `_APPEND_OK :: 1<<13`
- [OS == .MACOS && CPU == .ARM64] `_RMFILE_OK :: 1<<14`
- [OS == .MACOS && CPU == .ARM64] `_RATTR_OK :: 1<<15`
- [OS == .MACOS && CPU == .ARM64] `_WATTR_OK :: 1<<16`
- [OS == .MACOS && CPU == .ARM64] `_REXT_OK :: 1<<17`
- [OS == .MACOS && CPU == .ARM64] `_WEXT_OK :: 1<<18`
- [OS == .MACOS && CPU == .ARM64] `_RPERM_OK :: 1<<19`
- [OS == .MACOS && CPU == .ARM64] `_WPERM_OK :: 1<<20`
- [OS == .MACOS && CPU == .ARM64] `_CHOWN_OK :: 1<<21`
- [OS == .MACOS && CPU == .ARM64] `_ACCESS_EXTENDED_MASK :: _READ_OK | _WRITE_OK | _EXECUTE_OK | _DELETE_OK | _APPEND_OK | _RMFILE_OK | _REXT_OK | _WEXT_OK | _RATTR_OK | _WATTR_OK | _RPERM_OK | _WPERM_OK | _CHOWN_OK`
- [OS == .MACOS && CPU == .ARM64] `ACCESSX_MAX_DESCRIPTORS :: 100`
- [OS == .MACOS && CPU == .ARM64] `ACCESSX_MAX_TABLESIZE :: 16 * 1024`
- [OS == .MACOS && CPU == .ARM64] `PC :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `_CS_PATH :: 1`
- [OS == .MACOS && CPU == .ARM64] `STDIN_FILENO :: 0`
- [OS == .MACOS && CPU == .ARM64] `STDOUT_FILENO :: 1`
- [OS == .MACOS && CPU == .ARM64] `STDERR_FILENO :: 2`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_VERSION :: 600`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_XCU_VERSION :: 4`
- [OS == .MACOS && CPU == .ARM64] `__ILP32_OFF32 :: -1`
- [OS == .MACOS && CPU == .ARM64] `__ILP32_OFFBIG :: -1`
- [OS == .MACOS && CPU == .ARM64] `__LP64_OFF64 :: 1`
- [OS == .MACOS && CPU == .ARM64] `__LPBIG_OFFBIG :: 1`
- [OS == .MACOS && CPU == .ARM64] `_V6_ILP32_OFF32 :: __ILP32_OFF32`
- [OS == .MACOS && CPU == .ARM64] `_V6_ILP32_OFFBIG :: __ILP32_OFFBIG`
- [OS == .MACOS && CPU == .ARM64] `_V6_LP64_OFF64 :: __LP64_OFF64`
- [OS == .MACOS && CPU == .ARM64] `_V6_LPBIG_OFFBIG :: __LPBIG_OFFBIG`
- [OS == .MACOS && CPU == .ARM64] `_XBS5_ILP32_OFF32 :: __ILP32_OFF32`
- [OS == .MACOS && CPU == .ARM64] `_XBS5_ILP32_OFFBIG :: __ILP32_OFFBIG`
- [OS == .MACOS && CPU == .ARM64] `_XBS5_LP64_OFF64 :: __LP64_OFF64`
- [OS == .MACOS && CPU == .ARM64] `_XBS5_LPBIG_OFFBIG :: __LPBIG_OFFBIG`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_CRYPT :: 1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_ENH_I18N :: 1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_LEGACY :: -1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_REALTIME :: -1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_REALTIME_THREADS :: -1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_SHM :: 1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_STREAMS :: -1`
- [OS == .MACOS && CPU == .ARM64] `_XOPEN_UNIX :: 1`
- [OS == .MACOS && CPU == .ARM64] `using _SC_definitions :: enum s32`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_ILP32_OFF32_CFLAGS :: 2`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_ILP32_OFF32_LDFLAGS :: 3`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_ILP32_OFF32_LIBS :: 4`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_ILP32_OFFBIG_CFLAGS :: 5`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_ILP32_OFFBIG_LDFLAGS :: 6`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_ILP32_OFFBIG_LIBS :: 7`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_LP64_OFF64_CFLAGS :: 8`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_LP64_OFF64_LDFLAGS :: 9`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_LP64_OFF64_LIBS :: 10`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_LPBIG_OFFBIG_CFLAGS :: 11`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_LPBIG_OFFBIG_LDFLAGS :: 12`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_LPBIG_OFFBIG_LIBS :: 13`
- [OS == .MACOS && CPU == .ARM64] `_CS_POSIX_V6_WIDTH_RESTRICTED_ENVS :: 14`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFF32_CFLAGS :: 20`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFF32_LDFLAGS :: 21`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFF32_LIBS :: 22`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFF32_LINTFLAGS :: 23`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFFBIG_CFLAGS :: 24`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFFBIG_LDFLAGS :: 25`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFFBIG_LIBS :: 26`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_ILP32_OFFBIG_LINTFLAGS :: 27`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LP64_OFF64_CFLAGS :: 28`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LP64_OFF64_LDFLAGS :: 29`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LP64_OFF64_LIBS :: 30`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LP64_OFF64_LINTFLAGS :: 31`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LPBIG_OFFBIG_CFLAGS :: 32`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LPBIG_OFFBIG_LDFLAGS :: 33`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LPBIG_OFFBIG_LIBS :: 34`
- [OS == .MACOS && CPU == .ARM64] `_CS_XBS5_LPBIG_OFFBIG_LINTFLAGS :: 35`
- [OS == .MACOS && CPU == .ARM64] `_CS_DARWIN_USER_DIR :: 65536`
- [OS == .MACOS && CPU == .ARM64] `_CS_DARWIN_USER_TEMP_DIR :: 65537`
- [OS == .MACOS && CPU == .ARM64] `_CS_DARWIN_USER_CACHE_DIR :: 65538`
- [OS == .MACOS && CPU == .ARM64] `F_ULOCK :: 0`
- [OS == .MACOS && CPU == .ARM64] `F_LOCK :: 1`
- [OS == .MACOS && CPU == .ARM64] `F_TLOCK :: 2`
- [OS == .MACOS && CPU == .ARM64] `F_TEST :: 3`
- [OS == .MACOS && CPU == .ARM64] `SYNC_VOLUME_FULLSYNC :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `SYNC_VOLUME_WAIT :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `FSOPT_NOFOLLOW :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `FSOPT_NOINMEMUPDATE :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `FSOPT_REPORT_FULLSIZE :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `FSOPT_PACK_INVAL_ATTRS :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `FSOPT_ATTR_CMN_EXTENDED :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `FSOPT_RETURN_REALDEV :: 0x00000200`
- [OS == .MACOS && CPU == .ARM64] `SEARCHFS_MAX_SEARCHPARMS :: 4096`
- [OS == .MACOS && CPU == .ARM64] `ATTR_BIT_MAP_COUNT :: 5`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAPABILITIES_FORMAT :: 0`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAPABILITIES_INTERFACES :: 1`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAPABILITIES_RESERVED1 :: 2`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAPABILITIES_RESERVED2 :: 3`
- [OS == .MACOS && CPU == .ARM64] `ATTR_MAX_BUFFER :: 8192`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_PERSISTENTOBJECTIDS :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_SYMBOLICLINKS :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_HARDLINKS :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_JOURNAL :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_JOURNAL_ACTIVE :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_NO_ROOT_TIMES :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_SPARSE_FILES :: 0x00000040`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_ZERO_RUNS :: 0x00000080`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_CASE_SENSITIVE :: 0x00000100`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_CASE_PRESERVING :: 0x00000200`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_FAST_STATFS :: 0x00000400`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_2TB_FILESIZE :: 0x00000800`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_OPENDENYMODES :: 0x00001000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_HIDDEN_FILES :: 0x00002000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_PATH_FROM_ID :: 0x00004000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_NO_VOLUME_SIZES :: 0x00008000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_DECMPFS_COMPRESSION :: 0x00010000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_64BIT_OBJECT_IDS :: 0x00020000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_DIR_HARDLINKS :: 0x00040000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_DOCUMENT_ID :: 0x00080000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_WRITE_GENERATION_COUNT :: 0x00100000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_NO_IMMUTABLE_FILES :: 0x00200000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_NO_PERMISSIONS :: 0x00400000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_SHARED_SPACE :: 0x00800000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_VOL_GROUPS :: 0x01000000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_FMT_SEALED :: 0x02000000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_SEARCHFS :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_ATTRLIST :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_NFSEXPORT :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_READDIRATTR :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_EXCHANGEDATA :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_COPYFILE :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_ALLOCATE :: 0x00000040`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_VOL_RENAME :: 0x00000080`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_ADVLOCK :: 0x00000100`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_FLOCK :: 0x00000200`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_EXTENDED_SECURITY :: 0x00000400`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_USERACCESS :: 0x00000800`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_MANLOCK :: 0x00001000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_NAMEDSTREAMS :: 0x00002000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_EXTENDED_ATTR :: 0x00004000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_CLONE :: 0x00010000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_SNAPSHOT :: 0x00020000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_RENAME_SWAP :: 0x00040000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_RENAME_EXCL :: 0x00080000`
- [OS == .MACOS && CPU == .ARM64] `VOL_CAP_INT_RENAME_OPENFAIL :: 0x00100000`
- [OS == .MACOS && CPU == .ARM64] `ATTR_CMN :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `ATTR_VOL :: enum`
- [OS == .MACOS && CPU == .ARM64] `ATTR_DIR :: enum`
- [OS == .MACOS && CPU == .ARM64] `DIR_MNTSTATUS_MNTPOINT :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `DIR_MNTSTATUS_TRIGGER :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `ATTR_FILE :: enum`
- [OS == .MACOS && CPU == .ARM64] `ATTR_FORK :: enum`
- [OS == .MACOS && CPU == .ARM64] `ATTR_BULK_REQUIRED :: xx ATTR_CMN.NAME | xx ATTR_CMN.RETURNED_ATTRS`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_START :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_MATCHPARTIALNAMES :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_MATCHDIRS :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_MATCHFILES :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_SKIPLINKS :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_SKIPINVISIBLE :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_SKIPPACKAGES :: 0x00000040`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_SKIPINAPPROPRIATE :: 0x00000080`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_NEGATEPARAMS :: 0x80000000`
- [OS == .MACOS && CPU == .ARM64] `SRCHFS_VALIDOPTIONSMASK :: 0x800000FF`
- [OS == .MACOS && CPU == .ARM64] `FST_EOF :: -1`
- [OS == .MACOS && CPU == .ARM64] `HAVE_VT_LOCKERFS :: 1`
- [OS == .MACOS && CPU == .ARM64] `VNODE_READ :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `VNODE_WRITE :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `VNODE_BLOCKMAP_NO_TRACK :: 0x04`
- [OS == .MACOS && CPU == .ARM64] `PREALLOCATE :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `ALLOCATECONTIG :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `ALLOCATEALL :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `FREEREMAINDER :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `ALLOCATEFROMPEOF :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `ALLOCATEFROMVOL :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `VNOVAL :: -1`
- [OS == .MACOS && CPU == .ARM64] `dl_info :: struct`
- [OS == .MACOS && CPU == .ARM64] `Dl_info :: dl_info`
- [OS == .MACOS && CPU == .ARM64] `dladdr :: (unknown0: *void, unknown1: *Dl_info) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dlclose :: (__handle: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dlerror :: () -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dlopen :: (__path: *u8, __mode: s32) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dlsym :: (__handle: *void, __symbol: *u8) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dlopen_preflight :: (__path: *u8) -> bool #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ino_t :: u64`
- [OS == .MACOS && CPU == .ARM64] `dirent :: struct`
- [OS == .MACOS && CPU == .ARM64] `_telldir :: struct`
- [OS == .MACOS && CPU == .ARM64] `DIR :: struct`
- [OS == .MACOS && CPU == .ARM64] `closedir :: (unknown0: *DIR) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `opendir :: (unknown0: *u8) -> *DIR #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `readdir :: (unknown0: *DIR) -> *dirent #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `readdir_r :: (unknown0: *DIR, unknown1: *dirent, unknown2: **dirent) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rewinddir :: (unknown0: *DIR) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `seekdir :: (unknown0: *DIR, unknown1: s64) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `telldir :: (unknown0: *DIR) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fdopendir :: (unknown0: s32) -> *DIR #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `alphasort :: (unknown0: **dirent, unknown1: **dirent) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dirfd :: (dirp: *DIR) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `scandir :: (unknown0: *u8, unknown1: ***dirent, unknown2: #type (unknown0: *dirent) -> s32 #c_call, unknown3: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `__opendir2 :: (unknown0: *u8, unknown1: s32) -> *DIR #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `__errno_location :: () -> *OS_Error_Code #foreign libc "__error"`
- [OS == .MACOS && CPU == .ARM64] `uuid_t :: [16] u8`
- [OS == .MACOS && CPU == .ARM64] `backtrace :: (unknown0: **void, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `backtrace_from_fp :: (startfp: *void, array: **void, size: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `backtrace_symbols :: (unknown0: **void, unknown1: s32) -> **u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `backtrace_symbols_fd :: (unknown0: **void, unknown1: s32, unknown2: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `image_offset :: struct`
- [OS == .MACOS && CPU == .ARM64] `backtrace_image_offsets :: (array: **void, image_offsets: *image_offset, size: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fnmatch :: (unknown0: *u8, unknown1: *u8, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gid_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `uid_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `passwd :: struct`
- [OS == .MACOS && CPU == .ARM64] `getpwuid :: (unknown0: uid_t) -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpwnam :: (unknown0: *u8) -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpwuid_r :: (unknown0: uid_t, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpwnam_r :: (unknown0: *u8, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpwent :: () -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setpwent :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `endpwent :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setpassent :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `user_from_uid :: (unknown0: uid_t, unknown1: s32) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpwuuid :: (unknown0: *uuid_t) -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpwuuid_r :: (unknown0: *uuid_t, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sig_atomic_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_exception_state :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_exception_state64 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_thread_state :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_thread_state64 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_vfp_state :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_neon_state64 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_neon_state :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_amx_state_v1 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__arm_pagein_state :: struct`
- [OS == .MACOS && CPU == .ARM64] `__arm_legacy_debug_state :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_debug_state32 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_debug_state64 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_arm_cpmu_state64 :: struct`
- [OS == .MACOS && CPU == .ARM64] `__darwin_mcontext64 :: struct`
- [OS == .MACOS && CPU == .ARM64] `stack_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `ucontext_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `pid_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `sigset_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `sigval :: struct`
- [OS == .MACOS && CPU == .ARM64] `sigevent :: struct`
- [OS == .MACOS && CPU == .ARM64] `__siginfo :: struct`
- [OS == .MACOS && CPU == .ARM64] `siginfo_t :: __siginfo`
- [OS == .MACOS && CPU == .ARM64] `__sigaction_u :: struct`
- [OS == .MACOS && CPU == .ARM64] `__sigaction :: struct`
- [OS == .MACOS && CPU == .ARM64] `sigaction_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `sig_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .MACOS && CPU == .ARM64] `sigvec :: struct`
- [OS == .MACOS && CPU == .ARM64] `signal :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `raise :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `bsd_signal :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `kill :: (unknown0: pid_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `killpg :: (unknown0: pid_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigaction :: (unknown0: s32, unknown1: *sigaction_t, unknown2: *sigaction_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigaddset :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigaltstack :: (unknown0: *stack_t, unknown1: *stack_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigdelset :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigemptyset :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigfillset :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sighold :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigignore :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `siginterrupt :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigismember :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigpause :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigpending :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigprocmask :: (unknown0: s32, unknown1: *sigset_t, unknown2: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigrelse :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigset :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigsuspend :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigwait :: (unknown0: *sigset_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `psignal :: (unknown0: u32, unknown1: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigblock :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigsetmask :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigvec_ :: (unknown0: s32, unknown1: *sigvec, unknown2: *sigvec) -> s32 #foreign libc "sigvec"`
- [OS == .MACOS && CPU == .ARM64] `mode_t :: u16`
- [OS == .MACOS && CPU == .ARM64] `key_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `ipc_perm :: struct`
- [OS == .MACOS && CPU == .ARM64] `ftok :: (unknown0: *u8, unknown1: s32) -> key_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mlockall :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `munlockall :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mlock :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mmap :: (unknown0: *void, unknown1: u64, unknown2: s32, unknown3: s32, unknown4: s32, unknown5: s64) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mprotect :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `msync :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `munlock :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `munmap :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `shm_open :: (unknown0: *u8, unknown1: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `shm_unlink :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `posix_madvise :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `madvise :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mincore :: (unknown0: *void, unknown1: u64, unknown2: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `minherit :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pollfd :: struct`
- [OS == .MACOS && CPU == .ARM64] `nfds_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `poll :: (unknown0: *pollfd, unknown1: nfds_t, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `time_t :: s64`
- [OS == .MACOS && CPU == .ARM64] `shmatt_t :: u16`
- [OS == .MACOS && CPU == .ARM64] `__shmid_ds_new :: struct`
- [OS == .MACOS && CPU == .ARM64] `shmsys :: (unknown0: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `shmat :: (unknown0: s32, unknown1: *void, unknown2: s32) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `shmctl :: (unknown0: s32, unknown1: s32, unknown2: *__shmid_ds_new) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `shmdt :: (unknown0: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `shmget :: (unknown0: key_t, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `timespec :: struct`
- [OS == .MACOS && CPU == .ARM64] `timeval :: struct`
- [OS == .MACOS && CPU == .ARM64] `suseconds_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `itimerval :: struct`
- [OS == .MACOS && CPU == .ARM64] `timezone :: struct`
- [OS == .MACOS && CPU == .ARM64] `clockinfo :: struct`
- [OS == .MACOS && CPU == .ARM64] `clock_t :: u64`
- [OS == .MACOS && CPU == .ARM64] `tm :: struct`
- [OS == .MACOS && CPU == .ARM64] `tzname: **u8 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `getdate_err: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `daylight: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `asctime :: (unknown0: *tm) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `clock :: () -> clock_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ctime :: (unknown0: *time_t) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `difftime :: (unknown0: time_t, unknown1: time_t) -> float64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getdate :: (unknown0: *u8) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gmtime :: (unknown0: *time_t) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `localtime :: (unknown0: *time_t) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mktime :: (unknown0: *tm) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `strftime :: (unknown0: *u8, unknown1: u64, unknown2: *u8, unknown3: *tm) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `strptime :: (unknown0: *u8, unknown1: *u8, unknown2: *tm) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `time :: (unknown0: *time_t) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tzset :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `asctime_r :: (unknown0: *tm, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ctime_r :: (unknown0: *time_t, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gmtime_r :: (unknown0: *time_t, unknown1: *tm) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `localtime_r :: (unknown0: *time_t, unknown1: *tm) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `posix2time :: (unknown0: time_t) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tzsetwall :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `time2posix :: (unknown0: time_t) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `timelocal :: (unknown0: *tm) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `timegm :: (unknown0: *tm) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `nanosleep :: (__rqtp: *timespec, __rmtp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `clockid_t :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `clock_getres :: (__clock_id: clockid_t, __res: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `clock_gettime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `clock_gettime_nsec_np :: (__clock_id: clockid_t) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `clock_settime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `timespec_get :: (ts: *timespec, base: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `adjtime :: (unknown0: *timeval, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `futimes :: (unknown0: s32, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lutimes :: (unknown0: *u8, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `settimeofday :: (unknown0: *timeval, unknown1: *timezone) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getitimer :: (unknown0: s32, unknown1: *itimerval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gettimeofday :: (unknown0: *timeval, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setitimer :: (unknown0: s32, unknown1: *itimerval, unknown2: *itimerval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `utimes :: (unknown0: *u8, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dev_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `fixpt_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `blkcnt_t :: s64`
- [OS == .MACOS && CPU == .ARM64] `blksize_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `nlink_t :: u16`
- [OS == .MACOS && CPU == .ARM64] `segsz_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `useconds_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `utsname :: struct`
- [OS == .MACOS && CPU == .ARM64] `uname :: (unknown0: *utsname) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `idtype_t :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `wait_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `set_w_T_w_Termsig :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .ARM64] `get_w_T_w_Termsig :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .ARM64] `set_w_T_w_Coredump :: (fields: *wait_t, value: bool)`
- [OS == .MACOS && CPU == .ARM64] `get_w_T_w_Coredump :: (fields: wait_t) -> bool`
- [OS == .MACOS && CPU == .ARM64] `set_w_T_w_Retcode :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .ARM64] `get_w_T_w_Retcode :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .ARM64] `set_w_T_w_Filler :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .ARM64] `get_w_T_w_Filler :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .ARM64] `set_w_S_w_Stopval :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .ARM64] `get_w_S_w_Stopval :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .ARM64] `set_w_S_w_Stopsig :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .ARM64] `get_w_S_w_Stopsig :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .ARM64] `set_w_S_w_Filler :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .ARM64] `get_w_S_w_Filler :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .ARM64] `wait :: (unknown0: *s32) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `waitpid :: (unknown0: pid_t, unknown1: *s32, unknown2: s32) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `waitid :: (unknown0: idtype_t, unknown1: id_t, unknown2: *siginfo_t, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `wait3 :: (unknown0: *s32, unknown1: s32, unknown2: *rusage) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `wait4 :: (unknown0: pid_t, unknown1: *s32, unknown2: s32, unknown3: *rusage) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `accessx_descriptor :: struct`
- [OS == .MACOS && CPU == .ARM64] `getattrlistbulk :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getattrlistat :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: *void, unknown4: u64, unknown5: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setattrlistat :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: *void, unknown4: u64, unknown5: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `faccessat :: (unknown0: s32, unknown1: *u8, unknown2: s32, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchownat :: (unknown0: s32, unknown1: *u8, unknown2: uid_t, unknown3: gid_t, unknown4: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `linkat :: (unknown0: s32, unknown1: *u8, unknown2: s32, unknown3: *u8, unknown4: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `readlinkat :: (unknown0: s32, unknown1: *u8, unknown2: *u8, unknown3: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `symlinkat :: (unknown0: *u8, unknown1: s32, unknown2: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `unlinkat :: (unknown0: s32, unknown1: *u8, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `_exit :: (unknown0: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `access :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `alarm :: (unknown0: u32) -> u32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `chdir :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `chown :: (unknown0: *u8, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `close :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dup :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dup2 :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execl :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execle :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execlp :: (__file: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execve :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fork :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fpathconf :: (unknown0: s32, __name: PC) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getcwd :: (unknown0: *u8, unknown1: u64) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getegid :: () -> gid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `geteuid :: () -> uid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getgid :: () -> gid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getgroups :: (unknown0: s32, unknown1: *gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getlogin :: () -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpgrp :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpid :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getppid :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getuid :: () -> uid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `isatty :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `link :: (unknown0: *u8, unknown1: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lseek :: (unknown0: s32, unknown1: s64, unknown2: s32) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pathconf :: (unknown0: *u8, __name: PC) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pause :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pipe :: (unknown0: *[2] s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `read :: (unknown0: s32, unknown1: *void, unknown2: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rmdir :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setgid :: (unknown0: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setpgid :: (unknown0: pid_t, unknown1: pid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setsid :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setuid :: (unknown0: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sleep :: (unknown0: u32) -> u32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sysconf :: (name: _SC_definitions) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tcgetpgrp :: (unknown0: s32) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tcsetpgrp :: (unknown0: s32, unknown1: pid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ttyname :: (unknown0: s32) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ttyname_r :: (unknown0: s32, unknown1: *u8, unknown2: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `unlink :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `write :: (__fd: s32, __buf: *void, __nbyte: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `confstr :: (unknown0: s32, unknown1: *u8, unknown2: u64) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getopt :: (unknown0: s32, unknown1: **u8, unknown2: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `optarg: *u8 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `optind: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `opterr: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `optopt: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `brk :: (unknown0: *void) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `chroot :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `crypt :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `encrypt :: (unknown0: *u8, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchdir :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gethostid :: () -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpgid :: (unknown0: pid_t) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getsid :: (unknown0: pid_t) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getdtablesize :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpagesize :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpass :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getwd :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lchown :: (unknown0: *u8, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lockf :: (unknown0: s32, unknown1: s32, unknown2: s64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `nice :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pread :: (__fd: s32, __buf: *void, __nbyte: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pwrite :: (__fd: s32, __buf: *void, __nbyte: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sbrk :: (unknown0: s32) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setpgrp :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setregid :: (unknown0: gid_t, unknown1: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setreuid :: (unknown0: uid_t, unknown1: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `swab :: (unknown0: *void, unknown1: *void, unknown2: s64) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sync :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `truncate :: (unknown0: *u8, unknown1: s64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ualarm :: (unknown0: useconds_t, unknown1: useconds_t) -> useconds_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `usleep :: (unknown0: useconds_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `vfork :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fsync :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ftruncate :: (unknown0: s32, unknown1: s64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getlogin_r :: (unknown0: *u8, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchown :: (unknown0: s32, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gethostname :: (unknown0: *u8, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `readlink :: (unknown0: *u8, unknown1: *u8, unknown2: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setegid :: (unknown0: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `seteuid :: (unknown0: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `symlink :: (unknown0: *u8, unknown1: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `_Exit :: (unknown0: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `acct :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `endusershell :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `execvP :: (__file: *u8, __searchpath: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fflagstostr :: (unknown0: u64) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getdomainname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getgrouplist :: (unknown0: *u8, unknown1: s32, unknown2: *s32, unknown3: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getmode :: (unknown0: *void, unknown1: mode_t) -> mode_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getpeereid :: (unknown0: s32, unknown1: *uid_t, unknown2: *gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getsgroups_np :: (unknown0: *s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getusershell :: () -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getwgroups_np :: (unknown0: *s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `initgroups :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `issetugid :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `nfssvc :: (unknown0: s32, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `reboot :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `revoke :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rcmd :: (unknown0: **u8, unknown1: s32, unknown2: *u8, unknown3: *u8, unknown4: *u8, unknown5: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rcmd_af :: (unknown0: **u8, unknown1: s32, unknown2: *u8, unknown3: *u8, unknown4: *u8, unknown5: *s32, unknown6: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rresvport :: (unknown0: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rresvport_af :: (unknown0: *s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `iruserok :: (unknown0: u64, unknown1: s32, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `iruserok_sa :: (unknown0: *void, unknown1: s32, unknown2: s32, unknown3: *u8, unknown4: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ruserok :: (unknown0: *u8, unknown1: s32, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setdomainname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setgroups :: (unknown0: s32, unknown1: *gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sethostid :: (unknown0: s64) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sethostname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setkey :: (unknown0: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setlogin :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setmode :: (unknown0: *u8) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setrgid :: (unknown0: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setruid :: (unknown0: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setsgroups_np :: (unknown0: s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setusershell :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setwgroups_np :: (unknown0: s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `strtofflags :: (unknown0: **u8, unknown1: *u64, unknown2: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `swapon :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ttyslot :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `undelete :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `valloc :: (unknown0: u64) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `suboptarg: *u8 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `getsubopt :: (unknown0: **u8, unknown1: **u8, unknown2: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fgetattrlist :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fsetattrlist :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getattrlist :: (unknown0: *u8, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setattrlist :: (unknown0: *u8, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `exchangedata :: (unknown0: *u8, unknown1: *u8, unknown2: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getdirentriesattr :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: *u32, unknown5: *u32, unknown6: *u32, unknown7: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `searchfs :: (unknown0: *u8, unknown1: *fssearchblock, unknown2: *u64, unknown3: u32, unknown4: u32, unknown5: *searchstate) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fsctl :: (unknown0: *u8, unknown1: u64, unknown2: *void, unknown3: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ffsctl :: (unknown0: s32, unknown1: u64, unknown2: *void, unknown3: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fsync_volume_np :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sync_volume_np :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `optreset: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `mach_port_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `text_encoding_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `fsobj_type_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `fsobj_tag_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `fsfile_type_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `fsvolid_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `attrgroup_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `attrlist :: struct`
- [OS == .MACOS && CPU == .ARM64] `attribute_set :: struct`
- [OS == .MACOS && CPU == .ARM64] `attribute_set_t :: attribute_set`
- [OS == .MACOS && CPU == .ARM64] `attrreference :: struct`
- [OS == .MACOS && CPU == .ARM64] `attrreference_t :: attrreference`
- [OS == .MACOS && CPU == .ARM64] `diskextent :: struct`
- [OS == .MACOS && CPU == .ARM64] `extentrecord :: [8] diskextent`
- [OS == .MACOS && CPU == .ARM64] `vol_capabilities_set_t :: [4] u32`
- [OS == .MACOS && CPU == .ARM64] `vol_capabilities_attr :: struct`
- [OS == .MACOS && CPU == .ARM64] `vol_capabilities_attr_t :: vol_capabilities_attr`
- [OS == .MACOS && CPU == .ARM64] `vol_attributes_attr :: struct`
- [OS == .MACOS && CPU == .ARM64] `vol_attributes_attr_t :: vol_attributes_attr`
- [OS == .MACOS && CPU == .ARM64] `fssearchblock :: struct`
- [OS == .MACOS && CPU == .ARM64] `searchstate :: struct`
- [OS == .MACOS && CPU == .ARM64] `iovec :: struct`
- [OS == .MACOS && CPU == .ARM64] `vtype :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `vtagtype :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `__SLBF :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `__SNBF :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `__SRD :: 0x0004`
- [OS == .MACOS && CPU == .ARM64] `__SWR :: 0x0008`
- [OS == .MACOS && CPU == .ARM64] `__SRW :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `__SEOF :: 0x0020`
- [OS == .MACOS && CPU == .ARM64] `__SERR :: 0x0040`
- [OS == .MACOS && CPU == .ARM64] `__SMBF :: 0x0080`
- [OS == .MACOS && CPU == .ARM64] `__SAPP :: 0x0100`
- [OS == .MACOS && CPU == .ARM64] `__SSTR :: 0x0200`
- [OS == .MACOS && CPU == .ARM64] `__SOPT :: 0x0400`
- [OS == .MACOS && CPU == .ARM64] `__SNPT :: 0x0800`
- [OS == .MACOS && CPU == .ARM64] `__SOFF :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `__SMOD :: 0x2000`
- [OS == .MACOS && CPU == .ARM64] `__SALC :: 0x4000`
- [OS == .MACOS && CPU == .ARM64] `__SIGN :: 0x8000`
- [OS == .MACOS && CPU == .ARM64] `_IOFBF :: 0`
- [OS == .MACOS && CPU == .ARM64] `_IOLBF :: 1`
- [OS == .MACOS && CPU == .ARM64] `_IONBF :: 2`
- [OS == .MACOS && CPU == .ARM64] `BUFSIZ :: 1024`
- [OS == .MACOS && CPU == .ARM64] `EOF :: -1`
- [OS == .MACOS && CPU == .ARM64] `FOPEN_MAX :: 20`
- [OS == .MACOS && CPU == .ARM64] `FILENAME_MAX :: 1024`
- [OS == .MACOS && CPU == .ARM64] `P_tmpdir :: "/var/tmp/"`
- [OS == .MACOS && CPU == .ARM64] `L_tmpnam :: 1024`
- [OS == .MACOS && CPU == .ARM64] `TMP_MAX :: 308915776`
- [OS == .MACOS && CPU == .ARM64] `SEEK_SET :: 0`
- [OS == .MACOS && CPU == .ARM64] `SEEK_CUR :: 1`
- [OS == .MACOS && CPU == .ARM64] `SEEK_END :: 2`
- [OS == .MACOS && CPU == .ARM64] `L_ctermid :: 1024`
- [OS == .MACOS && CPU == .ARM64] `O_RDONLY :: 0x0000`
- [OS == .MACOS && CPU == .ARM64] `O_WRONLY :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `O_RDWR :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `O_ACCMODE :: 0x0003`
- [OS == .MACOS && CPU == .ARM64] `FREAD :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `FWRITE :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `O_NONBLOCK :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `O_APPEND :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `O_SYNC :: 0x0080`
- [OS == .MACOS && CPU == .ARM64] `O_SHLOCK :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `O_EXLOCK :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `O_ASYNC :: 0x00000040`
- [OS == .MACOS && CPU == .ARM64] `O_FSYNC :: O_SYNC`
- [OS == .MACOS && CPU == .ARM64] `O_NOFOLLOW :: 0x00000100`
- [OS == .MACOS && CPU == .ARM64] `O_CREAT :: 0x00000200`
- [OS == .MACOS && CPU == .ARM64] `O_TRUNC :: 0x00000400`
- [OS == .MACOS && CPU == .ARM64] `O_EXCL :: 0x00000800`
- [OS == .MACOS && CPU == .ARM64] `O_EVTONLY :: 0x00008000`
- [OS == .MACOS && CPU == .ARM64] `O_NOCTTY :: 0x00020000`
- [OS == .MACOS && CPU == .ARM64] `O_DIRECTORY :: 0x00100000`
- [OS == .MACOS && CPU == .ARM64] `O_SYMLINK :: 0x00200000`
- [OS == .MACOS && CPU == .ARM64] `O_CLOEXEC :: 0x01000000`
- [OS == .MACOS && CPU == .ARM64] `O_NOFOLLOW_ANY :: 0x20000000`
- [OS == .MACOS && CPU == .ARM64] `AT_FDCWD :: -2`
- [OS == .MACOS && CPU == .ARM64] `AT_EACCESS :: 0x0010`
- [OS == .MACOS && CPU == .ARM64] `AT_SYMLINK_NOFOLLOW :: 0x0020`
- [OS == .MACOS && CPU == .ARM64] `AT_SYMLINK_FOLLOW :: 0x0040`
- [OS == .MACOS && CPU == .ARM64] `AT_REMOVEDIR :: 0x0080`
- [OS == .MACOS && CPU == .ARM64] `AT_REALDEV :: 0x0200`
- [OS == .MACOS && CPU == .ARM64] `AT_FDONLY :: 0x0400`
- [OS == .MACOS && CPU == .ARM64] `O_DP_GETRAWENCRYPTED :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `O_DP_GETRAWUNENCRYPTED :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `FAPPEND :: O_APPEND`
- [OS == .MACOS && CPU == .ARM64] `FASYNC :: O_ASYNC`
- [OS == .MACOS && CPU == .ARM64] `FFSYNC :: O_FSYNC`
- [OS == .MACOS && CPU == .ARM64] `FNONBLOCK :: O_NONBLOCK`
- [OS == .MACOS && CPU == .ARM64] `FNDELAY :: O_NONBLOCK`
- [OS == .MACOS && CPU == .ARM64] `O_NDELAY :: O_NONBLOCK`
- [OS == .MACOS && CPU == .ARM64] `CPF_OVERWRITE :: 0x0001`
- [OS == .MACOS && CPU == .ARM64] `CPF_IGNORE_MODE :: 0x0002`
- [OS == .MACOS && CPU == .ARM64] `CPF_MASK :: CPF_OVERWRITE|CPF_IGNORE_MODE`
- [OS == .MACOS && CPU == .ARM64] `F_DUPFD :: 0`
- [OS == .MACOS && CPU == .ARM64] `F_GETFD :: 1`
- [OS == .MACOS && CPU == .ARM64] `F_SETFD :: 2`
- [OS == .MACOS && CPU == .ARM64] `F_GETFL :: 3`
- [OS == .MACOS && CPU == .ARM64] `F_SETFL :: 4`
- [OS == .MACOS && CPU == .ARM64] `F_GETOWN :: 5`
- [OS == .MACOS && CPU == .ARM64] `F_SETOWN :: 6`
- [OS == .MACOS && CPU == .ARM64] `F_GETLK :: 7`
- [OS == .MACOS && CPU == .ARM64] `F_SETLK :: 8`
- [OS == .MACOS && CPU == .ARM64] `F_SETLKW :: 9`
- [OS == .MACOS && CPU == .ARM64] `F_SETLKWTIMEOUT :: 10`
- [OS == .MACOS && CPU == .ARM64] `F_FLUSH_DATA :: 40`
- [OS == .MACOS && CPU == .ARM64] `F_CHKCLEAN :: 41`
- [OS == .MACOS && CPU == .ARM64] `F_PREALLOCATE :: 42`
- [OS == .MACOS && CPU == .ARM64] `F_SETSIZE :: 43`
- [OS == .MACOS && CPU == .ARM64] `F_RDADVISE :: 44`
- [OS == .MACOS && CPU == .ARM64] `F_RDAHEAD :: 45`
- [OS == .MACOS && CPU == .ARM64] `F_NOCACHE :: 48`
- [OS == .MACOS && CPU == .ARM64] `F_LOG2PHYS :: 49`
- [OS == .MACOS && CPU == .ARM64] `F_GETPATH :: 50`
- [OS == .MACOS && CPU == .ARM64] `F_FULLFSYNC :: 51`
- [OS == .MACOS && CPU == .ARM64] `F_PATHPKG_CHECK :: 52`
- [OS == .MACOS && CPU == .ARM64] `F_FREEZE_FS :: 53`
- [OS == .MACOS && CPU == .ARM64] `F_THAW_FS :: 54`
- [OS == .MACOS && CPU == .ARM64] `F_GLOBAL_NOCACHE :: 55`
- [OS == .MACOS && CPU == .ARM64] `F_ADDSIGS :: 59`
- [OS == .MACOS && CPU == .ARM64] `F_ADDFILESIGS :: 61`
- [OS == .MACOS && CPU == .ARM64] `F_NODIRECT :: 62`
- [OS == .MACOS && CPU == .ARM64] `F_GETPROTECTIONCLASS :: 63`
- [OS == .MACOS && CPU == .ARM64] `F_SETPROTECTIONCLASS :: 64`
- [OS == .MACOS && CPU == .ARM64] `F_LOG2PHYS_EXT :: 65`
- [OS == .MACOS && CPU == .ARM64] `F_GETLKPID :: 66`
- [OS == .MACOS && CPU == .ARM64] `F_SETBACKINGSTORE :: 70`
- [OS == .MACOS && CPU == .ARM64] `F_GETPATH_MTMINFO :: 71`
- [OS == .MACOS && CPU == .ARM64] `F_GETCODEDIR :: 72`
- [OS == .MACOS && CPU == .ARM64] `F_SETNOSIGPIPE :: 73`
- [OS == .MACOS && CPU == .ARM64] `F_GETNOSIGPIPE :: 74`
- [OS == .MACOS && CPU == .ARM64] `F_TRANSCODEKEY :: 75`
- [OS == .MACOS && CPU == .ARM64] `F_SINGLE_WRITER :: 76`
- [OS == .MACOS && CPU == .ARM64] `F_GETPROTECTIONLEVEL :: 77`
- [OS == .MACOS && CPU == .ARM64] `F_FINDSIGS :: 78`
- [OS == .MACOS && CPU == .ARM64] `F_ADDFILESIGS_FOR_DYLD_SIM :: 83`
- [OS == .MACOS && CPU == .ARM64] `F_BARRIERFSYNC :: 85`
- [OS == .MACOS && CPU == .ARM64] `F_ADDFILESIGS_RETURN :: 97`
- [OS == .MACOS && CPU == .ARM64] `F_CHECK_LV :: 98`
- [OS == .MACOS && CPU == .ARM64] `F_PUNCHHOLE :: 99`
- [OS == .MACOS && CPU == .ARM64] `F_TRIM_ACTIVE_FILE :: 100`
- [OS == .MACOS && CPU == .ARM64] `F_SPECULATIVE_READ :: 101`
- [OS == .MACOS && CPU == .ARM64] `F_GETPATH_NOFIRMLINK :: 102`
- [OS == .MACOS && CPU == .ARM64] `F_ADDFILESIGS_INFO :: 103`
- [OS == .MACOS && CPU == .ARM64] `F_ADDFILESUPPL :: 104`
- [OS == .MACOS && CPU == .ARM64] `F_GETSIGSINFO :: 105`
- [OS == .MACOS && CPU == .ARM64] `FCNTL_FS_SPECIFIC_BASE :: 0x00010000`
- [OS == .MACOS && CPU == .ARM64] `F_DUPFD_CLOEXEC :: 67`
- [OS == .MACOS && CPU == .ARM64] `FD_CLOEXEC :: 1`
- [OS == .MACOS && CPU == .ARM64] `F_RDLCK :: 1`
- [OS == .MACOS && CPU == .ARM64] `F_UNLCK :: 2`
- [OS == .MACOS && CPU == .ARM64] `F_WRLCK :: 3`
- [OS == .MACOS && CPU == .ARM64] `S_IFMT :: 0xf000`
- [OS == .MACOS && CPU == .ARM64] `S_IFIFO :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `S_IFCHR :: 0x2000`
- [OS == .MACOS && CPU == .ARM64] `S_IFDIR :: 0x4000`
- [OS == .MACOS && CPU == .ARM64] `S_IFBLK :: 0x6000`
- [OS == .MACOS && CPU == .ARM64] `S_IFREG :: 0x8000`
- [OS == .MACOS && CPU == .ARM64] `S_IFLNK :: 0xa000`
- [OS == .MACOS && CPU == .ARM64] `S_IFSOCK :: 0xc000`
- [OS == .MACOS && CPU == .ARM64] `S_IFWHT :: 0xe000`
- [OS == .MACOS && CPU == .ARM64] `S_IRWXU :: 0x1c0`
- [OS == .MACOS && CPU == .ARM64] `S_IRUSR :: 0x100`
- [OS == .MACOS && CPU == .ARM64] `S_IWUSR :: 0x80`
- [OS == .MACOS && CPU == .ARM64] `S_IXUSR :: 0x40`
- [OS == .MACOS && CPU == .ARM64] `S_IRWXG :: 0x38`
- [OS == .MACOS && CPU == .ARM64] `S_IRGRP :: 0x20`
- [OS == .MACOS && CPU == .ARM64] `S_IWGRP :: 0x10`
- [OS == .MACOS && CPU == .ARM64] `S_IXGRP :: 0x8`
- [OS == .MACOS && CPU == .ARM64] `S_IRWXO :: 0x7`
- [OS == .MACOS && CPU == .ARM64] `S_IROTH :: 0x4`
- [OS == .MACOS && CPU == .ARM64] `S_IWOTH :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `S_IXOTH :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `S_ISUID :: 0x800`
- [OS == .MACOS && CPU == .ARM64] `S_ISGID :: 0x400`
- [OS == .MACOS && CPU == .ARM64] `S_ISVTX :: 0x200`
- [OS == .MACOS && CPU == .ARM64] `S_ISTXT :: S_ISVTX`
- [OS == .MACOS && CPU == .ARM64] `S_IREAD :: S_IRUSR`
- [OS == .MACOS && CPU == .ARM64] `S_IWRITE :: S_IWUSR`
- [OS == .MACOS && CPU == .ARM64] `S_IEXEC :: S_IXUSR`
- [OS == .MACOS && CPU == .ARM64] `F_ALLOCATECONTIG :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `F_ALLOCATEALL :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `F_PEOFPOSMODE :: 3`
- [OS == .MACOS && CPU == .ARM64] `F_VOLPOSMODE :: 4`
- [OS == .MACOS && CPU == .ARM64] `USER_FSIGNATURES_CDHASH_LEN :: 20`
- [OS == .MACOS && CPU == .ARM64] `GETSIGSINFO_PLATFORM_BINARY :: 1`
- [OS == .MACOS && CPU == .ARM64] `LOCK_SH :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `LOCK_EX :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `LOCK_NB :: 0x04`
- [OS == .MACOS && CPU == .ARM64] `LOCK_UN :: 0x08`
- [OS == .MACOS && CPU == .ARM64] `O_POPUP :: 0x80000000`
- [OS == .MACOS && CPU == .ARM64] `O_ALERT :: 0x20000000`
- [OS == .MACOS && CPU == .ARM64] `_FILESEC_UNSET_PROPERTY :: cast(*void) 0`
- [OS == .MACOS && CPU == .ARM64] `_FILESEC_REMOVE_ACL :: cast(*void) 1`
- [OS == .MACOS && CPU == .ARM64] `ACCESSPERMS :: S_IRWXU|S_IRWXG|S_IRWXO`
- [OS == .MACOS && CPU == .ARM64] `ALLPERMS :: S_ISUID|S_ISGID|S_ISTXT|S_IRWXU|S_IRWXG|S_IRWXO`
- [OS == .MACOS && CPU == .ARM64] `DEFFILEMODE :: S_IRUSR|S_IWUSR|S_IRGRP|S_IWGRP|S_IROTH|S_IWOTH`
- [OS == .MACOS && CPU == .ARM64] `S_BLKSIZE :: 512`
- [OS == .MACOS && CPU == .ARM64] `UF_SETTABLE :: 0x0000ffff`
- [OS == .MACOS && CPU == .ARM64] `UF_NODUMP :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `UF_IMMUTABLE :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `UF_APPEND :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `UF_OPAQUE :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `UF_COMPRESSED :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `UF_TRACKED :: 0x00000040`
- [OS == .MACOS && CPU == .ARM64] `UF_DATAVAULT :: 0x00000080`
- [OS == .MACOS && CPU == .ARM64] `UF_HIDDEN :: 0x00008000`
- [OS == .MACOS && CPU == .ARM64] `SF_SUPPORTED :: 0x009f0000`
- [OS == .MACOS && CPU == .ARM64] `SF_SETTABLE :: 0x3fff0000`
- [OS == .MACOS && CPU == .ARM64] `SF_SYNTHETIC :: 0xc0000000`
- [OS == .MACOS && CPU == .ARM64] `SF_ARCHIVED :: 0x00010000`
- [OS == .MACOS && CPU == .ARM64] `SF_IMMUTABLE :: 0x00020000`
- [OS == .MACOS && CPU == .ARM64] `SF_APPEND :: 0x00040000`
- [OS == .MACOS && CPU == .ARM64] `SF_RESTRICTED :: 0x00080000`
- [OS == .MACOS && CPU == .ARM64] `SF_NOUNLINK :: 0x00100000`
- [OS == .MACOS && CPU == .ARM64] `SF_FIRMLINK :: 0x00800000`
- [OS == .MACOS && CPU == .ARM64] `SF_DATALESS :: 0x40000000`
- [OS == .MACOS && CPU == .ARM64] `EF_MAY_SHARE_BLOCKS :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `EF_NO_XATTRS :: 0x00000002`
- [OS == .MACOS && CPU == .ARM64] `EF_IS_SYNC_ROOT :: 0x00000004`
- [OS == .MACOS && CPU == .ARM64] `EF_IS_PURGEABLE :: 0x00000008`
- [OS == .MACOS && CPU == .ARM64] `EF_IS_SPARSE :: 0x00000010`
- [OS == .MACOS && CPU == .ARM64] `EF_IS_SYNTHETIC :: 0x00000020`
- [OS == .MACOS && CPU == .ARM64] `UTIME_NOW :: -1`
- [OS == .MACOS && CPU == .ARM64] `UTIME_OMIT :: -2`
- [OS == .MACOS && CPU == .ARM64] `AIO_ALLDONE :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `AIO_CANCELED :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `AIO_NOTCANCELED :: 0x4`
- [OS == .MACOS && CPU == .ARM64] `LIO_NOP :: 0x0`
- [OS == .MACOS && CPU == .ARM64] `LIO_READ :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `LIO_WRITE :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `LIO_NOWAIT :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `LIO_WAIT :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `AIO_LISTIO_MAX :: 16`
- [OS == .MACOS && CPU == .ARM64] `fpos_t :: s64`
- [OS == .MACOS && CPU == .ARM64] `stdin: *FILE #elsewhere libc "__stdinp"`
- [OS == .MACOS && CPU == .ARM64] `stdout: *FILE #elsewhere libc "__stdoutp"`
- [OS == .MACOS && CPU == .ARM64] `stderr: *FILE #elsewhere libc "__stderrp"`
- [OS == .MACOS && CPU == .ARM64] `clearerr :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fclose :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `feof :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ferror :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fflush :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fgetc :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fgetpos :: (unknown0: *FILE, unknown1: *fpos_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fgets :: (unknown0: *u8, unknown1: s32, unknown2: *FILE) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fopen :: (__filename: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fprintf :: (unknown0: *FILE, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fputc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fputs :: (unknown0: *u8, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fread :: (__ptr: *void, __size: u64, __nitems: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `freopen :: (unknown0: *u8, unknown1: *u8, unknown2: *FILE) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fscanf :: (unknown0: *FILE, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fseek :: (unknown0: *FILE, unknown1: s64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fsetpos :: (unknown0: *FILE, unknown1: *fpos_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ftell :: (unknown0: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fwrite :: (__ptr: *void, __size: u64, __nitems: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getc :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getchar :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `gets :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `perror :: (unknown0: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `printf :: (unknown0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `putc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `putchar :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `puts :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `_remove :: (unknown0: *u8) -> s32 #foreign libc "remove"`
- [OS == .MACOS && CPU == .ARM64] `rename :: (__old: *u8, __new: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `rewind :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `scanf :: (unknown0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setbuf :: (unknown0: *FILE, unknown1: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setvbuf :: (unknown0: *FILE, unknown1: *u8, unknown2: s32, unknown3: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sprintf :: (unknown0: *u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sscanf :: (unknown0: *u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tmpfile :: () -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tmpnam :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ungetc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fdopen :: (unknown0: s32, unknown1: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fileno :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pclose :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `popen :: (unknown0: *u8, unknown1: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `__srget :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `__swbuf :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `flockfile :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ftrylockfile :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `funlockfile :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getc_unlocked :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getchar_unlocked :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `putc_unlocked :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `putchar_unlocked :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getw :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `putw :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `tempnam :: (__dir: *u8, __prefix: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fseeko :: (__stream: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ftello :: (__stream: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `snprintf :: (__str: *u8, __size: u64, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `dprintf :: (unknown0: s32, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getdelim :: (__linep: **u8, __linecapp: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getline :: (__linep: **u8, __linecapp: *u64, __stream: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fmemopen :: (__buf: *void, __size: u64, __mode: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `open_memstream :: (__bufp: **u8, __sizep: *u64) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sys_nerr: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `sys_errlist: **u8 #elsewhere libc`
- [OS == .MACOS && CPU == .ARM64] `asprintf :: (unknown0: **u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ctermid_r :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fgetln :: (unknown0: *FILE, unknown1: *u64) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fmtcheck :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fpurge :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setbuffer :: (unknown0: *FILE, unknown1: *u8, unknown2: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setlinebuf :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `funopen :: (unknown0: *void, unknown1: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, unknown2: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, unknown3: #type (unknown0: *void, unknown1: fpos_t, unknown2: s32) -> fpos_t #c_call, unknown4: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `flock :: struct`
- [OS == .MACOS && CPU == .ARM64] `flocktimeout :: struct`
- [OS == .MACOS && CPU == .ARM64] `radvisory :: struct`
- [OS == .MACOS && CPU == .ARM64] `fsignatures :: struct`
- [OS == .MACOS && CPU == .ARM64] `fsignatures_t :: fsignatures`
- [OS == .MACOS && CPU == .ARM64] `fsupplement :: struct`
- [OS == .MACOS && CPU == .ARM64] `fsupplement_t :: fsupplement`
- [OS == .MACOS && CPU == .ARM64] `fchecklv :: struct`
- [OS == .MACOS && CPU == .ARM64] `fchecklv_t :: fchecklv`
- [OS == .MACOS && CPU == .ARM64] `fgetsigsinfo :: struct`
- [OS == .MACOS && CPU == .ARM64] `fgetsigsinfo_t :: fgetsigsinfo`
- [OS == .MACOS && CPU == .ARM64] `fstore :: struct`
- [OS == .MACOS && CPU == .ARM64] `fstore_t :: fstore`
- [OS == .MACOS && CPU == .ARM64] `fpunchhole :: struct`
- [OS == .MACOS && CPU == .ARM64] `fpunchhole_t :: fpunchhole`
- [OS == .MACOS && CPU == .ARM64] `ftrimactivefile :: struct`
- [OS == .MACOS && CPU == .ARM64] `ftrimactivefile_t :: ftrimactivefile`
- [OS == .MACOS && CPU == .ARM64] `fspecread :: struct`
- [OS == .MACOS && CPU == .ARM64] `fspecread_t :: fspecread`
- [OS == .MACOS && CPU == .ARM64] `fbootstraptransfer :: struct`
- [OS == .MACOS && CPU == .ARM64] `fbootstraptransfer_t :: fbootstraptransfer`
- [OS == .MACOS && CPU == .ARM64] `log2phys :: struct`
- [OS == .MACOS && CPU == .ARM64] `_filesec :: struct`
- [OS == .MACOS && CPU == .ARM64] `filesec_t :: *_filesec`
- [OS == .MACOS && CPU == .ARM64] `filesec_property_t :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `open :: (unknown0: *u8, unknown1: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `openat :: (unknown0: s32, unknown1: *u8, unknown2: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `creat :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fcntl :: (unknown0: s32, unknown1: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `openx_np :: (unknown0: *u8, unknown1: s32, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `open_dprotected_np :: (unknown0: *u8, unknown1: s32, unknown2: s32, unknown3: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `flock_ :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc "flock"`
- [OS == .MACOS && CPU == .ARM64] `filesec_init :: () -> filesec_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `filesec_dup :: (unknown0: filesec_t) -> filesec_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `filesec_free :: (unknown0: filesec_t) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `filesec_get_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `filesec_query_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `filesec_set_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `filesec_unset_property :: (unknown0: filesec_t, unknown1: filesec_property_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ostat :: struct`
- [OS == .MACOS && CPU == .ARM64] `stat_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `chmod :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchmod :: (unknown0: s32, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fstat :: (unknown0: s32, unknown1: *stat_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lstat :: (unknown0: *u8, unknown1: *stat_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkdir :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkfifo :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `stat :: (unknown0: *u8, unknown1: *stat_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mknod :: (unknown0: *u8, unknown1: mode_t, unknown2: dev_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `umask :: (unknown0: mode_t) -> mode_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchmodat :: (unknown0: s32, unknown1: *u8, unknown2: mode_t, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fstatat :: (unknown0: s32, unknown1: *u8, unknown2: *stat_t, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkdirat :: (unknown0: s32, unknown1: *u8, unknown2: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `utimensat :: (__fd: s32, __path: *u8, __times: *[2] timespec, __flag: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `chflags :: (unknown0: *u8, unknown1: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `chmodx_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchflags :: (unknown0: s32, unknown1: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fchmodx_np :: (unknown0: s32, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `fstatx_np :: (unknown0: s32, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lchflags :: (unknown0: *u8, unknown1: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lchmod :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lstatx_np :: (unknown0: *u8, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkdirx_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkfifox_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `statx_np :: (unknown0: *u8, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `umaskx_np :: (unknown0: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ttysize :: struct`
- [OS == .MACOS && CPU == .ARM64] `ioctl :: (unknown0: s32, unknown1: u64, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aiocb :: struct`
- [OS == .MACOS && CPU == .ARM64] `aio_cancel :: (fd: s32, aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aio_error :: (aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aio_fsync :: (op: s32, aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aio_read :: (aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aio_return :: (aiocbp: *aiocb) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aio_suspend :: (aiocblist: **aiocb, nent: s32, timeoutp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `aio_write :: (aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `lio_listio :: (mode: s32, aiocblist: **aiocb, nent: s32, sigp: *sigevent) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `FILE :: struct`
- [OS == .MACOS && CPU == .ARM64] `ftello64 :: ftello`
- [OS == .MACOS && CPU == .ARM64] `getenv :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `system :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mktemp :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkstemp :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `realpath :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc "realpath$DARWIN_EXTSN"`
- [OS == .MACOS && CPU == .ARM64] `setenv :: (__name: *u8, __value: *u8, __overwrite: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `unsetenv :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkdtemp :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkpath_np :: (path: *u8, omode: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkpathat_np :: (dfd: s32, path: *u8, omode: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkstemps :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkostemp :: (path: *u8, oflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkostemps :: (path: *u8, slen: s32, oflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkstemp_dprotected_np :: (path: *u8, dpclass: s32, dpflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkdtempat_np :: (dfd: s32, path: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkstempsat_np :: (dfd: s32, path: *u8, slen: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `mkostempsat_np :: (dfd: s32, path: *u8, slen: s32, oflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `SYS_syscall :: 0`
- [OS == .MACOS && CPU == .ARM64] `SYS_exit :: 1`
- [OS == .MACOS && CPU == .ARM64] `SYS_fork :: 2`
- [OS == .MACOS && CPU == .ARM64] `SYS_read :: 3`
- [OS == .MACOS && CPU == .ARM64] `SYS_write :: 4`
- [OS == .MACOS && CPU == .ARM64] `SYS_open :: 5`
- [OS == .MACOS && CPU == .ARM64] `SYS_close :: 6`
- [OS == .MACOS && CPU == .ARM64] `SYS_wait4 :: 7`
- [OS == .MACOS && CPU == .ARM64] `SYS_link :: 9`
- [OS == .MACOS && CPU == .ARM64] `SYS_unlink :: 10`
- [OS == .MACOS && CPU == .ARM64] `SYS_chdir :: 12`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchdir :: 13`
- [OS == .MACOS && CPU == .ARM64] `SYS_mknod :: 14`
- [OS == .MACOS && CPU == .ARM64] `SYS_chmod :: 15`
- [OS == .MACOS && CPU == .ARM64] `SYS_chown :: 16`
- [OS == .MACOS && CPU == .ARM64] `SYS_getfsstat :: 18`
- [OS == .MACOS && CPU == .ARM64] `SYS_getpid :: 20`
- [OS == .MACOS && CPU == .ARM64] `SYS_setuid :: 23`
- [OS == .MACOS && CPU == .ARM64] `SYS_getuid :: 24`
- [OS == .MACOS && CPU == .ARM64] `SYS_geteuid :: 25`
- [OS == .MACOS && CPU == .ARM64] `SYS_ptrace :: 26`
- [OS == .MACOS && CPU == .ARM64] `SYS_recvmsg :: 27`
- [OS == .MACOS && CPU == .ARM64] `SYS_sendmsg :: 28`
- [OS == .MACOS && CPU == .ARM64] `SYS_recvfrom :: 29`
- [OS == .MACOS && CPU == .ARM64] `SYS_accept :: 30`
- [OS == .MACOS && CPU == .ARM64] `SYS_getpeername :: 31`
- [OS == .MACOS && CPU == .ARM64] `SYS_getsockname :: 32`
- [OS == .MACOS && CPU == .ARM64] `SYS_access :: 33`
- [OS == .MACOS && CPU == .ARM64] `SYS_chflags :: 34`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchflags :: 35`
- [OS == .MACOS && CPU == .ARM64] `SYS_sync :: 36`
- [OS == .MACOS && CPU == .ARM64] `SYS_kill :: 37`
- [OS == .MACOS && CPU == .ARM64] `SYS_getppid :: 39`
- [OS == .MACOS && CPU == .ARM64] `SYS_dup :: 41`
- [OS == .MACOS && CPU == .ARM64] `SYS_pipe :: 42`
- [OS == .MACOS && CPU == .ARM64] `SYS_getegid :: 43`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigaction :: 46`
- [OS == .MACOS && CPU == .ARM64] `SYS_getgid :: 47`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigprocmask :: 48`
- [OS == .MACOS && CPU == .ARM64] `SYS_getlogin :: 49`
- [OS == .MACOS && CPU == .ARM64] `SYS_setlogin :: 50`
- [OS == .MACOS && CPU == .ARM64] `SYS_acct :: 51`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigpending :: 52`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigaltstack :: 53`
- [OS == .MACOS && CPU == .ARM64] `SYS_ioctl :: 54`
- [OS == .MACOS && CPU == .ARM64] `SYS_reboot :: 55`
- [OS == .MACOS && CPU == .ARM64] `SYS_revoke :: 56`
- [OS == .MACOS && CPU == .ARM64] `SYS_symlink :: 57`
- [OS == .MACOS && CPU == .ARM64] `SYS_readlink :: 58`
- [OS == .MACOS && CPU == .ARM64] `SYS_execve :: 59`
- [OS == .MACOS && CPU == .ARM64] `SYS_umask :: 60`
- [OS == .MACOS && CPU == .ARM64] `SYS_chroot :: 61`
- [OS == .MACOS && CPU == .ARM64] `SYS_msync :: 65`
- [OS == .MACOS && CPU == .ARM64] `SYS_vfork :: 66`
- [OS == .MACOS && CPU == .ARM64] `SYS_munmap :: 73`
- [OS == .MACOS && CPU == .ARM64] `SYS_mprotect :: 74`
- [OS == .MACOS && CPU == .ARM64] `SYS_madvise :: 75`
- [OS == .MACOS && CPU == .ARM64] `SYS_mincore :: 78`
- [OS == .MACOS && CPU == .ARM64] `SYS_getgroups :: 79`
- [OS == .MACOS && CPU == .ARM64] `SYS_setgroups :: 80`
- [OS == .MACOS && CPU == .ARM64] `SYS_getpgrp :: 81`
- [OS == .MACOS && CPU == .ARM64] `SYS_setpgid :: 82`
- [OS == .MACOS && CPU == .ARM64] `SYS_setitimer :: 83`
- [OS == .MACOS && CPU == .ARM64] `SYS_swapon :: 85`
- [OS == .MACOS && CPU == .ARM64] `SYS_getitimer :: 86`
- [OS == .MACOS && CPU == .ARM64] `SYS_getdtablesize :: 89`
- [OS == .MACOS && CPU == .ARM64] `SYS_dup2 :: 90`
- [OS == .MACOS && CPU == .ARM64] `SYS_fcntl :: 92`
- [OS == .MACOS && CPU == .ARM64] `SYS_select :: 93`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsync :: 95`
- [OS == .MACOS && CPU == .ARM64] `SYS_setpriority :: 96`
- [OS == .MACOS && CPU == .ARM64] `SYS_socket :: 97`
- [OS == .MACOS && CPU == .ARM64] `SYS_connect :: 98`
- [OS == .MACOS && CPU == .ARM64] `SYS_getpriority :: 100`
- [OS == .MACOS && CPU == .ARM64] `SYS_bind :: 104`
- [OS == .MACOS && CPU == .ARM64] `SYS_setsockopt :: 105`
- [OS == .MACOS && CPU == .ARM64] `SYS_listen :: 106`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigsuspend :: 111`
- [OS == .MACOS && CPU == .ARM64] `SYS_gettimeofday :: 116`
- [OS == .MACOS && CPU == .ARM64] `SYS_getrusage :: 117`
- [OS == .MACOS && CPU == .ARM64] `SYS_getsockopt :: 118`
- [OS == .MACOS && CPU == .ARM64] `SYS_readv :: 120`
- [OS == .MACOS && CPU == .ARM64] `SYS_writev :: 121`
- [OS == .MACOS && CPU == .ARM64] `SYS_settimeofday :: 122`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchown :: 123`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchmod :: 124`
- [OS == .MACOS && CPU == .ARM64] `SYS_setreuid :: 126`
- [OS == .MACOS && CPU == .ARM64] `SYS_setregid :: 127`
- [OS == .MACOS && CPU == .ARM64] `SYS_rename :: 128`
- [OS == .MACOS && CPU == .ARM64] `SYS_flock :: 131`
- [OS == .MACOS && CPU == .ARM64] `SYS_mkfifo :: 132`
- [OS == .MACOS && CPU == .ARM64] `SYS_sendto :: 133`
- [OS == .MACOS && CPU == .ARM64] `SYS_shutdown :: 134`
- [OS == .MACOS && CPU == .ARM64] `SYS_socketpair :: 135`
- [OS == .MACOS && CPU == .ARM64] `SYS_mkdir :: 136`
- [OS == .MACOS && CPU == .ARM64] `SYS_rmdir :: 137`
- [OS == .MACOS && CPU == .ARM64] `SYS_utimes :: 138`
- [OS == .MACOS && CPU == .ARM64] `SYS_futimes :: 139`
- [OS == .MACOS && CPU == .ARM64] `SYS_adjtime :: 140`
- [OS == .MACOS && CPU == .ARM64] `SYS_gethostuuid :: 142`
- [OS == .MACOS && CPU == .ARM64] `SYS_setsid :: 147`
- [OS == .MACOS && CPU == .ARM64] `SYS_getpgid :: 151`
- [OS == .MACOS && CPU == .ARM64] `SYS_setprivexec :: 152`
- [OS == .MACOS && CPU == .ARM64] `SYS_pread :: 153`
- [OS == .MACOS && CPU == .ARM64] `SYS_pwrite :: 154`
- [OS == .MACOS && CPU == .ARM64] `SYS_nfssvc :: 155`
- [OS == .MACOS && CPU == .ARM64] `SYS_statfs :: 157`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstatfs :: 158`
- [OS == .MACOS && CPU == .ARM64] `SYS_unmount :: 159`
- [OS == .MACOS && CPU == .ARM64] `SYS_getfh :: 161`
- [OS == .MACOS && CPU == .ARM64] `SYS_quotactl :: 165`
- [OS == .MACOS && CPU == .ARM64] `SYS_mount :: 167`
- [OS == .MACOS && CPU == .ARM64] `SYS_csops :: 169`
- [OS == .MACOS && CPU == .ARM64] `SYS_csops_audittoken :: 170`
- [OS == .MACOS && CPU == .ARM64] `SYS_waitid :: 173`
- [OS == .MACOS && CPU == .ARM64] `SYS_kdebug_typefilter :: 177`
- [OS == .MACOS && CPU == .ARM64] `SYS_kdebug_trace_string :: 178`
- [OS == .MACOS && CPU == .ARM64] `SYS_kdebug_trace64 :: 179`
- [OS == .MACOS && CPU == .ARM64] `SYS_kdebug_trace :: 180`
- [OS == .MACOS && CPU == .ARM64] `SYS_setgid :: 181`
- [OS == .MACOS && CPU == .ARM64] `SYS_setegid :: 182`
- [OS == .MACOS && CPU == .ARM64] `SYS_seteuid :: 183`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigreturn :: 184`
- [OS == .MACOS && CPU == .ARM64] `SYS_thread_selfcounts :: 186`
- [OS == .MACOS && CPU == .ARM64] `SYS_fdatasync :: 187`
- [OS == .MACOS && CPU == .ARM64] `SYS_stat :: 188`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstat :: 189`
- [OS == .MACOS && CPU == .ARM64] `SYS_lstat :: 190`
- [OS == .MACOS && CPU == .ARM64] `SYS_pathconf :: 191`
- [OS == .MACOS && CPU == .ARM64] `SYS_fpathconf :: 192`
- [OS == .MACOS && CPU == .ARM64] `SYS_getrlimit :: 194`
- [OS == .MACOS && CPU == .ARM64] `SYS_setrlimit :: 195`
- [OS == .MACOS && CPU == .ARM64] `SYS_getdirentries :: 196`
- [OS == .MACOS && CPU == .ARM64] `SYS_mmap :: 197`
- [OS == .MACOS && CPU == .ARM64] `SYS_lseek :: 199`
- [OS == .MACOS && CPU == .ARM64] `SYS_truncate :: 200`
- [OS == .MACOS && CPU == .ARM64] `SYS_ftruncate :: 201`
- [OS == .MACOS && CPU == .ARM64] `SYS_sysctl :: 202`
- [OS == .MACOS && CPU == .ARM64] `SYS_mlock :: 203`
- [OS == .MACOS && CPU == .ARM64] `SYS_munlock :: 204`
- [OS == .MACOS && CPU == .ARM64] `SYS_undelete :: 205`
- [OS == .MACOS && CPU == .ARM64] `SYS_open_dprotected_np :: 216`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsgetpath_ext :: 217`
- [OS == .MACOS && CPU == .ARM64] `SYS_getattrlist :: 220`
- [OS == .MACOS && CPU == .ARM64] `SYS_setattrlist :: 221`
- [OS == .MACOS && CPU == .ARM64] `SYS_getdirentriesattr :: 222`
- [OS == .MACOS && CPU == .ARM64] `SYS_exchangedata :: 223`
- [OS == .MACOS && CPU == .ARM64] `SYS_searchfs :: 225`
- [OS == .MACOS && CPU == .ARM64] `SYS_delete :: 226`
- [OS == .MACOS && CPU == .ARM64] `SYS_copyfile :: 227`
- [OS == .MACOS && CPU == .ARM64] `SYS_fgetattrlist :: 228`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsetattrlist :: 229`
- [OS == .MACOS && CPU == .ARM64] `SYS_poll :: 230`
- [OS == .MACOS && CPU == .ARM64] `SYS_getxattr :: 234`
- [OS == .MACOS && CPU == .ARM64] `SYS_fgetxattr :: 235`
- [OS == .MACOS && CPU == .ARM64] `SYS_setxattr :: 236`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsetxattr :: 237`
- [OS == .MACOS && CPU == .ARM64] `SYS_removexattr :: 238`
- [OS == .MACOS && CPU == .ARM64] `SYS_fremovexattr :: 239`
- [OS == .MACOS && CPU == .ARM64] `SYS_listxattr :: 240`
- [OS == .MACOS && CPU == .ARM64] `SYS_flistxattr :: 241`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsctl :: 242`
- [OS == .MACOS && CPU == .ARM64] `SYS_initgroups :: 243`
- [OS == .MACOS && CPU == .ARM64] `SYS_posix_spawn :: 244`
- [OS == .MACOS && CPU == .ARM64] `SYS_ffsctl :: 245`
- [OS == .MACOS && CPU == .ARM64] `SYS_nfsclnt :: 247`
- [OS == .MACOS && CPU == .ARM64] `SYS_fhopen :: 248`
- [OS == .MACOS && CPU == .ARM64] `SYS_minherit :: 250`
- [OS == .MACOS && CPU == .ARM64] `SYS_semsys :: 251`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgsys :: 252`
- [OS == .MACOS && CPU == .ARM64] `SYS_shmsys :: 253`
- [OS == .MACOS && CPU == .ARM64] `SYS_semctl :: 254`
- [OS == .MACOS && CPU == .ARM64] `SYS_semget :: 255`
- [OS == .MACOS && CPU == .ARM64] `SYS_semop :: 256`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgctl :: 258`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgget :: 259`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgsnd :: 260`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgrcv :: 261`
- [OS == .MACOS && CPU == .ARM64] `SYS_shmat :: 262`
- [OS == .MACOS && CPU == .ARM64] `SYS_shmctl :: 263`
- [OS == .MACOS && CPU == .ARM64] `SYS_shmdt :: 264`
- [OS == .MACOS && CPU == .ARM64] `SYS_shmget :: 265`
- [OS == .MACOS && CPU == .ARM64] `SYS_shm_open :: 266`
- [OS == .MACOS && CPU == .ARM64] `SYS_shm_unlink :: 267`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_open :: 268`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_close :: 269`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_unlink :: 270`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_wait :: 271`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_trywait :: 272`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_post :: 273`
- [OS == .MACOS && CPU == .ARM64] `SYS_sysctlbyname :: 274`
- [OS == .MACOS && CPU == .ARM64] `SYS_open_extended :: 277`
- [OS == .MACOS && CPU == .ARM64] `SYS_umask_extended :: 278`
- [OS == .MACOS && CPU == .ARM64] `SYS_stat_extended :: 279`
- [OS == .MACOS && CPU == .ARM64] `SYS_lstat_extended :: 280`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstat_extended :: 281`
- [OS == .MACOS && CPU == .ARM64] `SYS_chmod_extended :: 282`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchmod_extended :: 283`
- [OS == .MACOS && CPU == .ARM64] `SYS_access_extended :: 284`
- [OS == .MACOS && CPU == .ARM64] `SYS_settid :: 285`
- [OS == .MACOS && CPU == .ARM64] `SYS_gettid :: 286`
- [OS == .MACOS && CPU == .ARM64] `SYS_setsgroups :: 287`
- [OS == .MACOS && CPU == .ARM64] `SYS_getsgroups :: 288`
- [OS == .MACOS && CPU == .ARM64] `SYS_setwgroups :: 289`
- [OS == .MACOS && CPU == .ARM64] `SYS_getwgroups :: 290`
- [OS == .MACOS && CPU == .ARM64] `SYS_mkfifo_extended :: 291`
- [OS == .MACOS && CPU == .ARM64] `SYS_mkdir_extended :: 292`
- [OS == .MACOS && CPU == .ARM64] `SYS_identitysvc :: 293`
- [OS == .MACOS && CPU == .ARM64] `SYS_shared_region_check_np :: 294`
- [OS == .MACOS && CPU == .ARM64] `SYS_vm_pressure_monitor :: 296`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_longrdlock :: 297`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_yieldwrlock :: 298`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_downgrade :: 299`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_upgrade :: 300`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_mutexwait :: 301`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_mutexdrop :: 302`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_cvbroad :: 303`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_cvsignal :: 304`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_cvwait :: 305`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_rdlock :: 306`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_wrlock :: 307`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_unlock :: 308`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_rw_unlock2 :: 309`
- [OS == .MACOS && CPU == .ARM64] `SYS_getsid :: 310`
- [OS == .MACOS && CPU == .ARM64] `SYS_settid_with_pid :: 311`
- [OS == .MACOS && CPU == .ARM64] `SYS_psynch_cvclrprepost :: 312`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_fsync :: 313`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_return :: 314`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_suspend :: 315`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_cancel :: 316`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_error :: 317`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_read :: 318`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_write :: 319`
- [OS == .MACOS && CPU == .ARM64] `SYS_lio_listio :: 320`
- [OS == .MACOS && CPU == .ARM64] `SYS_iopolicysys :: 322`
- [OS == .MACOS && CPU == .ARM64] `SYS_process_policy :: 323`
- [OS == .MACOS && CPU == .ARM64] `SYS_mlockall :: 324`
- [OS == .MACOS && CPU == .ARM64] `SYS_munlockall :: 325`
- [OS == .MACOS && CPU == .ARM64] `SYS_issetugid :: 327`
- [OS == .MACOS && CPU == .ARM64] `SYS___pthread_kill :: 328`
- [OS == .MACOS && CPU == .ARM64] `SYS___pthread_sigmask :: 329`
- [OS == .MACOS && CPU == .ARM64] `SYS___sigwait :: 330`
- [OS == .MACOS && CPU == .ARM64] `SYS___disable_threadsignal :: 331`
- [OS == .MACOS && CPU == .ARM64] `SYS___pthread_markcancel :: 332`
- [OS == .MACOS && CPU == .ARM64] `SYS___pthread_canceled :: 333`
- [OS == .MACOS && CPU == .ARM64] `SYS___semwait_signal :: 334`
- [OS == .MACOS && CPU == .ARM64] `SYS_proc_info :: 336`
- [OS == .MACOS && CPU == .ARM64] `SYS_sendfile :: 337`
- [OS == .MACOS && CPU == .ARM64] `SYS_stat64 :: 338`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstat64 :: 339`
- [OS == .MACOS && CPU == .ARM64] `SYS_lstat64 :: 340`
- [OS == .MACOS && CPU == .ARM64] `SYS_stat64_extended :: 341`
- [OS == .MACOS && CPU == .ARM64] `SYS_lstat64_extended :: 342`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstat64_extended :: 343`
- [OS == .MACOS && CPU == .ARM64] `SYS_getdirentries64 :: 344`
- [OS == .MACOS && CPU == .ARM64] `SYS_statfs64 :: 345`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstatfs64 :: 346`
- [OS == .MACOS && CPU == .ARM64] `SYS_getfsstat64 :: 347`
- [OS == .MACOS && CPU == .ARM64] `SYS___pthread_chdir :: 348`
- [OS == .MACOS && CPU == .ARM64] `SYS___pthread_fchdir :: 349`
- [OS == .MACOS && CPU == .ARM64] `SYS_audit :: 350`
- [OS == .MACOS && CPU == .ARM64] `SYS_auditon :: 351`
- [OS == .MACOS && CPU == .ARM64] `SYS_getauid :: 353`
- [OS == .MACOS && CPU == .ARM64] `SYS_setauid :: 354`
- [OS == .MACOS && CPU == .ARM64] `SYS_getaudit_addr :: 357`
- [OS == .MACOS && CPU == .ARM64] `SYS_setaudit_addr :: 358`
- [OS == .MACOS && CPU == .ARM64] `SYS_auditctl :: 359`
- [OS == .MACOS && CPU == .ARM64] `SYS_bsdthread_create :: 360`
- [OS == .MACOS && CPU == .ARM64] `SYS_bsdthread_terminate :: 361`
- [OS == .MACOS && CPU == .ARM64] `SYS_kqueue :: 362`
- [OS == .MACOS && CPU == .ARM64] `SYS_kevent :: 363`
- [OS == .MACOS && CPU == .ARM64] `SYS_lchown :: 364`
- [OS == .MACOS && CPU == .ARM64] `SYS_bsdthread_register :: 366`
- [OS == .MACOS && CPU == .ARM64] `SYS_workq_open :: 367`
- [OS == .MACOS && CPU == .ARM64] `SYS_workq_kernreturn :: 368`
- [OS == .MACOS && CPU == .ARM64] `SYS_kevent64 :: 369`
- [OS == .MACOS && CPU == .ARM64] `SYS___old_semwait_signal :: 370`
- [OS == .MACOS && CPU == .ARM64] `SYS___old_semwait_signal_nocancel :: 371`
- [OS == .MACOS && CPU == .ARM64] `SYS_thread_selfid :: 372`
- [OS == .MACOS && CPU == .ARM64] `SYS_ledger :: 373`
- [OS == .MACOS && CPU == .ARM64] `SYS_kevent_qos :: 374`
- [OS == .MACOS && CPU == .ARM64] `SYS_kevent_id :: 375`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_execve :: 380`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_syscall :: 381`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_get_file :: 382`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_set_file :: 383`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_get_link :: 384`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_set_link :: 385`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_get_proc :: 386`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_set_proc :: 387`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_get_fd :: 388`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_set_fd :: 389`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_get_pid :: 390`
- [OS == .MACOS && CPU == .ARM64] `SYS_pselect :: 394`
- [OS == .MACOS && CPU == .ARM64] `SYS_pselect_nocancel :: 395`
- [OS == .MACOS && CPU == .ARM64] `SYS_read_nocancel :: 396`
- [OS == .MACOS && CPU == .ARM64] `SYS_write_nocancel :: 397`
- [OS == .MACOS && CPU == .ARM64] `SYS_open_nocancel :: 398`
- [OS == .MACOS && CPU == .ARM64] `SYS_close_nocancel :: 399`
- [OS == .MACOS && CPU == .ARM64] `SYS_wait4_nocancel :: 400`
- [OS == .MACOS && CPU == .ARM64] `SYS_recvmsg_nocancel :: 401`
- [OS == .MACOS && CPU == .ARM64] `SYS_sendmsg_nocancel :: 402`
- [OS == .MACOS && CPU == .ARM64] `SYS_recvfrom_nocancel :: 403`
- [OS == .MACOS && CPU == .ARM64] `SYS_accept_nocancel :: 404`
- [OS == .MACOS && CPU == .ARM64] `SYS_msync_nocancel :: 405`
- [OS == .MACOS && CPU == .ARM64] `SYS_fcntl_nocancel :: 406`
- [OS == .MACOS && CPU == .ARM64] `SYS_select_nocancel :: 407`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsync_nocancel :: 408`
- [OS == .MACOS && CPU == .ARM64] `SYS_connect_nocancel :: 409`
- [OS == .MACOS && CPU == .ARM64] `SYS_sigsuspend_nocancel :: 410`
- [OS == .MACOS && CPU == .ARM64] `SYS_readv_nocancel :: 411`
- [OS == .MACOS && CPU == .ARM64] `SYS_writev_nocancel :: 412`
- [OS == .MACOS && CPU == .ARM64] `SYS_sendto_nocancel :: 413`
- [OS == .MACOS && CPU == .ARM64] `SYS_pread_nocancel :: 414`
- [OS == .MACOS && CPU == .ARM64] `SYS_pwrite_nocancel :: 415`
- [OS == .MACOS && CPU == .ARM64] `SYS_waitid_nocancel :: 416`
- [OS == .MACOS && CPU == .ARM64] `SYS_poll_nocancel :: 417`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgsnd_nocancel :: 418`
- [OS == .MACOS && CPU == .ARM64] `SYS_msgrcv_nocancel :: 419`
- [OS == .MACOS && CPU == .ARM64] `SYS_sem_wait_nocancel :: 420`
- [OS == .MACOS && CPU == .ARM64] `SYS_aio_suspend_nocancel :: 421`
- [OS == .MACOS && CPU == .ARM64] `SYS___sigwait_nocancel :: 422`
- [OS == .MACOS && CPU == .ARM64] `SYS___semwait_signal_nocancel :: 423`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_mount :: 424`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_get_mount :: 425`
- [OS == .MACOS && CPU == .ARM64] `SYS___mac_getfsstat :: 426`
- [OS == .MACOS && CPU == .ARM64] `SYS_fsgetpath :: 427`
- [OS == .MACOS && CPU == .ARM64] `SYS_audit_session_self :: 428`
- [OS == .MACOS && CPU == .ARM64] `SYS_audit_session_join :: 429`
- [OS == .MACOS && CPU == .ARM64] `SYS_fileport_makeport :: 430`
- [OS == .MACOS && CPU == .ARM64] `SYS_fileport_makefd :: 431`
- [OS == .MACOS && CPU == .ARM64] `SYS_audit_session_port :: 432`
- [OS == .MACOS && CPU == .ARM64] `SYS_pid_suspend :: 433`
- [OS == .MACOS && CPU == .ARM64] `SYS_pid_resume :: 434`
- [OS == .MACOS && CPU == .ARM64] `SYS_pid_hibernate :: 435`
- [OS == .MACOS && CPU == .ARM64] `SYS_pid_shutdown_sockets :: 436`
- [OS == .MACOS && CPU == .ARM64] `SYS_shared_region_map_and_slide_np :: 438`
- [OS == .MACOS && CPU == .ARM64] `SYS_kas_info :: 439`
- [OS == .MACOS && CPU == .ARM64] `SYS_memorystatus_control :: 440`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_open_np :: 441`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_close_np :: 442`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_kqueue_np :: 443`
- [OS == .MACOS && CPU == .ARM64] `SYS_change_fdguard_np :: 444`
- [OS == .MACOS && CPU == .ARM64] `SYS_usrctl :: 445`
- [OS == .MACOS && CPU == .ARM64] `SYS_proc_rlimit_control :: 446`
- [OS == .MACOS && CPU == .ARM64] `SYS_connectx :: 447`
- [OS == .MACOS && CPU == .ARM64] `SYS_disconnectx :: 448`
- [OS == .MACOS && CPU == .ARM64] `SYS_peeloff :: 449`
- [OS == .MACOS && CPU == .ARM64] `SYS_socket_delegate :: 450`
- [OS == .MACOS && CPU == .ARM64] `SYS_telemetry :: 451`
- [OS == .MACOS && CPU == .ARM64] `SYS_proc_uuid_policy :: 452`
- [OS == .MACOS && CPU == .ARM64] `SYS_memorystatus_get_level :: 453`
- [OS == .MACOS && CPU == .ARM64] `SYS_system_override :: 454`
- [OS == .MACOS && CPU == .ARM64] `SYS_vfs_purge :: 455`
- [OS == .MACOS && CPU == .ARM64] `SYS_sfi_ctl :: 456`
- [OS == .MACOS && CPU == .ARM64] `SYS_sfi_pidctl :: 457`
- [OS == .MACOS && CPU == .ARM64] `SYS_coalition :: 458`
- [OS == .MACOS && CPU == .ARM64] `SYS_coalition_info :: 459`
- [OS == .MACOS && CPU == .ARM64] `SYS_necp_match_policy :: 460`
- [OS == .MACOS && CPU == .ARM64] `SYS_getattrlistbulk :: 461`
- [OS == .MACOS && CPU == .ARM64] `SYS_clonefileat :: 462`
- [OS == .MACOS && CPU == .ARM64] `SYS_openat :: 463`
- [OS == .MACOS && CPU == .ARM64] `SYS_openat_nocancel :: 464`
- [OS == .MACOS && CPU == .ARM64] `SYS_renameat :: 465`
- [OS == .MACOS && CPU == .ARM64] `SYS_faccessat :: 466`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchmodat :: 467`
- [OS == .MACOS && CPU == .ARM64] `SYS_fchownat :: 468`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstatat :: 469`
- [OS == .MACOS && CPU == .ARM64] `SYS_fstatat64 :: 470`
- [OS == .MACOS && CPU == .ARM64] `SYS_linkat :: 471`
- [OS == .MACOS && CPU == .ARM64] `SYS_unlinkat :: 472`
- [OS == .MACOS && CPU == .ARM64] `SYS_readlinkat :: 473`
- [OS == .MACOS && CPU == .ARM64] `SYS_symlinkat :: 474`
- [OS == .MACOS && CPU == .ARM64] `SYS_mkdirat :: 475`
- [OS == .MACOS && CPU == .ARM64] `SYS_getattrlistat :: 476`
- [OS == .MACOS && CPU == .ARM64] `SYS_proc_trace_log :: 477`
- [OS == .MACOS && CPU == .ARM64] `SYS_bsdthread_ctl :: 478`
- [OS == .MACOS && CPU == .ARM64] `SYS_openbyid_np :: 479`
- [OS == .MACOS && CPU == .ARM64] `SYS_recvmsg_x :: 480`
- [OS == .MACOS && CPU == .ARM64] `SYS_sendmsg_x :: 481`
- [OS == .MACOS && CPU == .ARM64] `SYS_thread_selfusage :: 482`
- [OS == .MACOS && CPU == .ARM64] `SYS_csrctl :: 483`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_open_dprotected_np :: 484`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_write_np :: 485`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_pwrite_np :: 486`
- [OS == .MACOS && CPU == .ARM64] `SYS_guarded_writev_np :: 487`
- [OS == .MACOS && CPU == .ARM64] `SYS_renameatx_np :: 488`
- [OS == .MACOS && CPU == .ARM64] `SYS_mremap_encrypted :: 489`
- [OS == .MACOS && CPU == .ARM64] `SYS_netagent_trigger :: 490`
- [OS == .MACOS && CPU == .ARM64] `SYS_stack_snapshot_with_config :: 491`
- [OS == .MACOS && CPU == .ARM64] `SYS_microstackshot :: 492`
- [OS == .MACOS && CPU == .ARM64] `SYS_grab_pgo_data :: 493`
- [OS == .MACOS && CPU == .ARM64] `SYS_persona :: 494`
- [OS == .MACOS && CPU == .ARM64] `SYS_mach_eventlink_signal :: 496`
- [OS == .MACOS && CPU == .ARM64] `SYS_mach_eventlink_wait_until :: 497`
- [OS == .MACOS && CPU == .ARM64] `SYS_mach_eventlink_signal_wait_until :: 498`
- [OS == .MACOS && CPU == .ARM64] `SYS_work_interval_ctl :: 499`
- [OS == .MACOS && CPU == .ARM64] `SYS_getentropy :: 500`
- [OS == .MACOS && CPU == .ARM64] `SYS_necp_open :: 501`
- [OS == .MACOS && CPU == .ARM64] `SYS_necp_client_action :: 502`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_open :: 503`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_register :: 504`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_deregister :: 505`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_create :: 506`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_destroy :: 507`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_get_opt :: 508`
- [OS == .MACOS && CPU == .ARM64] `SYS___nexus_set_opt :: 509`
- [OS == .MACOS && CPU == .ARM64] `SYS___channel_open :: 510`
- [OS == .MACOS && CPU == .ARM64] `SYS___channel_get_info :: 511`
- [OS == .MACOS && CPU == .ARM64] `SYS___channel_sync :: 512`
- [OS == .MACOS && CPU == .ARM64] `SYS___channel_get_opt :: 513`
- [OS == .MACOS && CPU == .ARM64] `SYS___channel_set_opt :: 514`
- [OS == .MACOS && CPU == .ARM64] `SYS_ulock_wait :: 515`
- [OS == .MACOS && CPU == .ARM64] `SYS_ulock_wake :: 516`
- [OS == .MACOS && CPU == .ARM64] `SYS_fclonefileat :: 517`
- [OS == .MACOS && CPU == .ARM64] `SYS_fs_snapshot :: 518`
- [OS == .MACOS && CPU == .ARM64] `SYS_terminate_with_payload :: 520`
- [OS == .MACOS && CPU == .ARM64] `SYS_abort_with_payload :: 521`
- [OS == .MACOS && CPU == .ARM64] `SYS_necp_session_open :: 522`
- [OS == .MACOS && CPU == .ARM64] `SYS_necp_session_action :: 523`
- [OS == .MACOS && CPU == .ARM64] `SYS_setattrlistat :: 524`
- [OS == .MACOS && CPU == .ARM64] `SYS_net_qos_guideline :: 525`
- [OS == .MACOS && CPU == .ARM64] `SYS_fmount :: 526`
- [OS == .MACOS && CPU == .ARM64] `SYS_ntp_adjtime :: 527`
- [OS == .MACOS && CPU == .ARM64] `SYS_ntp_gettime :: 528`
- [OS == .MACOS && CPU == .ARM64] `SYS_os_fault_with_payload :: 529`
- [OS == .MACOS && CPU == .ARM64] `SYS_kqueue_workloop_ctl :: 530`
- [OS == .MACOS && CPU == .ARM64] `SYS___mach_bridge_remote_time :: 531`
- [OS == .MACOS && CPU == .ARM64] `SYS_coalition_ledger :: 532`
- [OS == .MACOS && CPU == .ARM64] `SYS_log_data :: 533`
- [OS == .MACOS && CPU == .ARM64] `SYS_memorystatus_available_memory :: 534`
- [OS == .MACOS && CPU == .ARM64] `SYS_shared_region_map_and_slide_2_np :: 536`
- [OS == .MACOS && CPU == .ARM64] `SYS_pivot_root :: 537`
- [OS == .MACOS && CPU == .ARM64] `SYS_task_inspect_for_pid :: 538`
- [OS == .MACOS && CPU == .ARM64] `SYS_task_read_for_pid :: 539`
- [OS == .MACOS && CPU == .ARM64] `SYS_preadv :: 540`
- [OS == .MACOS && CPU == .ARM64] `SYS_pwritev :: 541`
- [OS == .MACOS && CPU == .ARM64] `SYS_preadv_nocancel :: 542`
- [OS == .MACOS && CPU == .ARM64] `SYS_pwritev_nocancel :: 543`
- [OS == .MACOS && CPU == .ARM64] `SYS_ulock_wait2 :: 544`
- [OS == .MACOS && CPU == .ARM64] `SYS_proc_info_extended_id :: 545`
- [OS == .MACOS && CPU == .ARM64] `SYS_MAXSYSCALL :: 546`
- [OS == .MACOS && CPU == .ARM64] `SYS_invalid :: 63`
- [OS == .MACOS && CPU == .ARM64] `syscall :: (unknown0: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_SIZE__ :: 8176`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_ATTR_SIZE__ :: 56`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_MUTEXATTR_SIZE__ :: 8`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_MUTEX_SIZE__ :: 56`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_CONDATTR_SIZE__ :: 8`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_COND_SIZE__ :: 40`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_ONCE_SIZE__ :: 8`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_RWLOCK_SIZE__ :: 192`
- [OS == .MACOS && CPU == .ARM64] `__PTHREAD_RWLOCKATTR_SIZE__ :: 16`
- [OS == .MACOS && CPU == .ARM64] `QOS_MIN_RELATIVE_PRIORITY :: -15`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CREATE_JOINABLE :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CREATE_DETACHED :: 2`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_INHERIT_SCHED :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_EXPLICIT_SCHED :: 2`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CANCEL_ENABLE :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CANCEL_DISABLE :: 0x00`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CANCEL_DEFERRED :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CANCEL_ASYNCHRONOUS :: 0x00`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_CANCELED :: cast(*void) 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_SCOPE_SYSTEM :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_SCOPE_PROCESS :: 2`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_PROCESS_SHARED :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_PROCESS_PRIVATE :: 2`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_PRIO_NONE :: 0`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_PRIO_INHERIT :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_PRIO_PROTECT :: 2`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_MUTEX_NORMAL :: 0`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_MUTEX_ERRORCHECK :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_MUTEX_RECURSIVE :: 2`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_MUTEX_DEFAULT :: PTHREAD_MUTEX_NORMAL`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_MUTEX_POLICY_FAIRSHARE_NP :: 1`
- [OS == .MACOS && CPU == .ARM64] `PTHREAD_MUTEX_POLICY_FIRSTFIT_NP :: 3`
- [OS == .MACOS && CPU == .ARM64] `_JBLEN :: (14 + 8 + 2) * 2`
- [OS == .MACOS && CPU == .ARM64] `_PTHREAD_MUTEX_POLICY_FAIRSHARE :: PTHREAD_MUTEX_POLICY_FAIRSHARE_NP`
- [OS == .MACOS && CPU == .ARM64] `_PTHREAD_MUTEX_POLICY_FIRSTFIT :: PTHREAD_MUTEX_POLICY_FIRSTFIT_NP`
- [OS == .MACOS && CPU == .ARM64] `__darwin_pthread_handler_rec :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_attr_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_cond_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_condattr_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_mutex_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_mutexattr_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_once_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_rwlock_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_rwlockattr_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `_opaque_pthread_t :: struct`
- [OS == .MACOS && CPU == .ARM64] `sched_param :: struct`
- [OS == .MACOS && CPU == .ARM64] `sched_yield :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sched_get_priority_min :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sched_get_priority_max :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_t :: _opaque_pthread_attr_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_t :: _opaque_pthread_cond_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_condattr_t :: _opaque_pthread_condattr_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_key_t :: u64`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_t :: _opaque_pthread_mutex_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_t :: _opaque_pthread_mutexattr_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_once_t :: _opaque_pthread_once_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_t :: _opaque_pthread_rwlock_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlockattr_t :: _opaque_pthread_rwlockattr_t`
- [OS == .MACOS && CPU == .ARM64] `pthread_t :: *_opaque_pthread_t`
- [OS == .MACOS && CPU == .ARM64] `QOS_CLASS :: enum u32`
- [OS == .MACOS && CPU == .ARM64] `qos_class_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `qos_class_self :: () -> qos_class_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `qos_class_main :: () -> qos_class_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_set_qos_class_np :: (__attr: *pthread_attr_t, __qos_class: qos_class_t, __relative_priority: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_get_qos_class_np :: (__attr: *pthread_attr_t, __qos_class: *qos_class_t, __relative_priority: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_set_qos_class_self_np :: (__qos_class: qos_class_t, __relative_priority: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_get_qos_class_np :: (__pthread: pthread_t, __qos_class: *qos_class_t, __relative_priority: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_override_s :: struct`
- [OS == .MACOS && CPU == .ARM64] `pthread_override_t :: *pthread_override_s`
- [OS == .MACOS && CPU == .ARM64] `pthread_override_qos_class_start_np :: (__pthread: pthread_t, __qos_class: qos_class_t, __relative_priority: s32) -> pthread_override_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_override_qos_class_end_np :: (__override: pthread_override_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_atfork :: (unknown0: #type () -> void #c_call, unknown1: #type () -> void #c_call, unknown2: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_destroy :: (unknown0: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getdetachstate :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getguardsize :: (unknown0: *pthread_attr_t, unknown1: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getinheritsched :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getschedparam :: (unknown0: *pthread_attr_t, unknown1: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getschedpolicy :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getscope :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getstack :: (unknown0: *pthread_attr_t, unknown1: **void, unknown2: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getstackaddr :: (unknown0: *pthread_attr_t, unknown1: **void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_getstacksize :: (unknown0: *pthread_attr_t, unknown1: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_init :: (unknown0: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setdetachstate :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setguardsize :: (unknown0: *pthread_attr_t, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setinheritsched :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setschedparam :: (unknown0: *pthread_attr_t, unknown1: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setschedpolicy :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setscope :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setstack :: (unknown0: *pthread_attr_t, unknown1: *void, unknown2: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setstackaddr :: (unknown0: *pthread_attr_t, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_attr_setstacksize :: (unknown0: *pthread_attr_t, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cancel :: (unknown0: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_broadcast :: (unknown0: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_destroy :: (unknown0: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_init :: (unknown0: *pthread_cond_t, unknown1: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_signal :: (unknown0: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_timedwait :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t, unknown2: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_wait :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_condattr_destroy :: (unknown0: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_condattr_init :: (unknown0: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_condattr_getpshared :: (unknown0: *pthread_condattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_condattr_setpshared :: (unknown0: *pthread_condattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_create :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_detach :: (unknown0: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_equal :: (unknown0: pthread_t, unknown1: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_exit :: (unknown0: *void) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_getconcurrency :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_getschedparam :: (unknown0: pthread_t, unknown1: *s32, unknown2: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_getspecific :: (unknown0: pthread_key_t) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_join :: (unknown0: pthread_t, unknown1: **void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_key_create :: (unknown0: *pthread_key_t, unknown1: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_key_delete :: (unknown0: pthread_key_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_destroy :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_getprioceiling :: (unknown0: *pthread_mutex_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_init :: (unknown0: *pthread_mutex_t, unknown1: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_lock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_setprioceiling :: (unknown0: *pthread_mutex_t, unknown1: s32, unknown2: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_trylock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutex_unlock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_destroy :: (unknown0: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_getprioceiling :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_getprotocol :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_getpshared :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_gettype :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_getpolicy_np :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_init :: (unknown0: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_setprioceiling :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_setprotocol :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_setpshared :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_settype :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mutexattr_setpolicy_np :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_once :: (unknown0: *pthread_once_t, unknown1: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_destroy :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_init :: (unknown0: *pthread_rwlock_t, unknown1: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_rdlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_tryrdlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_trywrlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_wrlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlock_unlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlockattr_destroy :: (unknown0: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlockattr_getpshared :: (unknown0: *pthread_rwlockattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlockattr_init :: (unknown0: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_rwlockattr_setpshared :: (unknown0: *pthread_rwlockattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_self :: () -> pthread_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_setcancelstate :: (unknown0: s32, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_setcanceltype :: (unknown0: s32, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_setconcurrency :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_setschedparam :: (unknown0: pthread_t, unknown1: s32, unknown2: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_setspecific :: (unknown0: pthread_key_t, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_testcancel :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_is_threaded_np :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_threadid_np :: (unknown0: pthread_t, unknown1: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_getname_np :: (unknown0: pthread_t, unknown1: *u8, unknown2: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_setname_np :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_main_np :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_mach_thread_np :: (unknown0: pthread_t) -> mach_port_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_get_stacksize_np :: (unknown0: pthread_t) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_get_stackaddr_np :: (unknown0: pthread_t) -> *void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_signal_thread_np :: (unknown0: *pthread_cond_t, unknown1: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cond_timedwait_relative_np :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t, unknown2: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_create_suspended_np :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_kill :: (unknown0: pthread_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_from_mach_thread_np :: (unknown0: mach_port_t) -> pthread_t #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_sigmask :: (unknown0: s32, unknown1: *sigset_t, unknown2: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_yield_np :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_jit_write_protect_np :: (enabled: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_jit_write_protect_supported_np :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_cpu_number_np :: (cpu_number_out: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sem_t :: s32`
- [OS == .MACOS && CPU == .ARM64] `jmp_buf :: [48] s32`
- [OS == .MACOS && CPU == .ARM64] `sigjmp_buf :: [49] s32`
- [OS == .MACOS && CPU == .ARM64] `setjmp :: (unknown0: *jmp_buf) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `longjmp :: (unknown0: *jmp_buf, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `_setjmp :: (unknown0: *jmp_buf) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `_longjmp :: (unknown0: *jmp_buf, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sigsetjmp :: (unknown0: *sigjmp_buf, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `siglongjmp :: (unknown0: *sigjmp_buf, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `_pthread_mutex_enable_legacy_mode :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `pthread_create_from_mach_thread :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `PRIO :: enum s32`
- [OS == .MACOS && CPU == .ARM64] `PRIO_MIN :: -20`
- [OS == .MACOS && CPU == .ARM64] `PRIO_MAX :: 20`
- [OS == .MACOS && CPU == .ARM64] `RUSAGE :: enum s32`
- [OS == .MACOS && CPU == .ARM64] `RUSAGE_INFO :: enum`
- [OS == .MACOS && CPU == .ARM64] `RU_PROC_RUNS_RESLIDE :: 0x00000001`
- [OS == .MACOS && CPU == .ARM64] `RLIM_INFINITY :: (cast(u64) 1 << 63) - 1`
- [OS == .MACOS && CPU == .ARM64] `RLIM_SAVED_MAX :: RLIM_INFINITY`
- [OS == .MACOS && CPU == .ARM64] `RLIM_SAVED_CUR :: RLIM_INFINITY`
- [OS == .MACOS && CPU == .ARM64] `RLIMIT_RESOURCE :: enum s32`
- [OS == .MACOS && CPU == .ARM64] `RLIM_NLIMITS :: 9`
- [OS == .MACOS && CPU == .ARM64] `_RLIMIT_POSIX_FLAG :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `RLIMIT_WAKEUPS_MONITOR :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `RLIMIT_CPU_USAGE_MONITOR :: 0x2`
- [OS == .MACOS && CPU == .ARM64] `RLIMIT_THREAD_CPULIMITS :: 0x3`
- [OS == .MACOS && CPU == .ARM64] `RLIMIT_FOOTPRINT_INTERVAL :: 0x4`
- [OS == .MACOS && CPU == .ARM64] `WAKEMON_ENABLE :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `WAKEMON_DISABLE :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `WAKEMON_GET_PARAMS :: 0x04`
- [OS == .MACOS && CPU == .ARM64] `WAKEMON_SET_DEFAULTS :: 0x08`
- [OS == .MACOS && CPU == .ARM64] `WAKEMON_MAKE_FATAL :: 0x10`
- [OS == .MACOS && CPU == .ARM64] `CPUMON_MAKE_FATAL :: 0x1000`
- [OS == .MACOS && CPU == .ARM64] `FOOTPRINT_INTERVAL_RESET :: 0x1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_TYPE_DISK :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_TYPE_VFS_ATIME_UPDATES :: 2`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES :: 3`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_TYPE_VFS_STATFS_NO_DATA_VOLUME :: 4`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_TYPE_VFS_TRIGGER_RESOLVE :: 5`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_TYPE_VFS_IGNORE_CONTENT_PROTECTION :: 6`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_SCOPE_PROCESS :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_SCOPE_THREAD :: 1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_SCOPE_DARWIN_BG :: 2`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_DEFAULT :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_IMPORTANT :: 1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_PASSIVE :: 2`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_THROTTLE :: 3`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_UTILITY :: 4`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_STANDARD :: 5`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_APPLICATION :: IOPOL_STANDARD`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_NORMAL :: IOPOL_IMPORTANT`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_ATIME_UPDATES_DEFAULT :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_ATIME_UPDATES_OFF :: 1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_MATERIALIZE_DATALESS_FILES_DEFAULT :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_MATERIALIZE_DATALESS_FILES_OFF :: 1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_MATERIALIZE_DATALESS_FILES_ON :: 2`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_VFS_STATFS_NO_DATA_VOLUME_DEFAULT :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_VFS_STATFS_FORCE_NO_DATA_VOLUME :: 1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_VFS_TRIGGER_RESOLVE_DEFAULT :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_VFS_TRIGGER_RESOLVE_OFF :: 1`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_VFS_CONTENT_PROTECTION_DEFAULT :: 0`
- [OS == .MACOS && CPU == .ARM64] `IOPOL_VFS_CONTENT_PROTECTION_IGNORE :: 1`
- [OS == .MACOS && CPU == .ARM64] `id_t :: u32`
- [OS == .MACOS && CPU == .ARM64] `rlim_t :: u64`
- [OS == .MACOS && CPU == .ARM64] `rusage :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_t :: *void`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_v0 :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_v1 :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_v2 :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_v3 :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_v4 :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_v5 :: struct`
- [OS == .MACOS && CPU == .ARM64] `rusage_info_current :: rusage_info_v5`
- [OS == .MACOS && CPU == .ARM64] `rlimit :: struct`
- [OS == .MACOS && CPU == .ARM64] `proc_rlimit_control_wakeupmon :: struct`
- [OS == .MACOS && CPU == .ARM64] `getpriority :: (unknown0: PRIO, unknown1: id_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getiopolicy_np :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getrlimit :: (unknown0: RLIMIT_RESOURCE, unknown1: *rlimit) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `getrusage :: (unknown0: RUSAGE, unknown1: *rusage) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setpriority :: (unknown0: PRIO, unknown1: id_t, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setiopolicy_np :: (unknown0: s32, unknown1: s32, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `setrlimit :: (unknown0: RLIMIT_RESOURCE, unknown1: *rlimit) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `ucred :: struct`
- [OS == .MACOS && CPU == .ARM64] `P_TRACED :: 0x00000800`
- [OS == .MACOS && CPU == .ARM64] `CTL_MAXNAME :: 12`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE :: 0xf`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE_NODE :: 1`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE_INT :: 2`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE_STRING :: 3`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE_QUAD :: 4`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE_OPAQUE :: 5`
- [OS == .MACOS && CPU == .ARM64] `CTLTYPE_STRUCT :: CTLTYPE_OPAQUE`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_RD :: 0x80000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_WR :: 0x40000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_RW :: CTLFLAG_RD|CTLFLAG_WR`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_NOLOCK :: 0x20000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_ANYBODY :: 0x10000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_SECURE :: 0x08000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_MASKED :: 0x04000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_NOAUTO :: 0x02000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_KERN :: 0x01000000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_LOCKED :: 0x00800000`
- [OS == .MACOS && CPU == .ARM64] `CTLFLAG_OID2 :: 0x00400000`
- [OS == .MACOS && CPU == .ARM64] `OID_AUTO :: -1`
- [OS == .MACOS && CPU == .ARM64] `OID_AUTO_START :: 100`
- [OS == .MACOS && CPU == .ARM64] `CTL_UNSPEC :: 0`
- [OS == .MACOS && CPU == .ARM64] `CTL_KERN :: 1`
- [OS == .MACOS && CPU == .ARM64] `CTL_VM :: 2`
- [OS == .MACOS && CPU == .ARM64] `CTL_VFS :: 3`
- [OS == .MACOS && CPU == .ARM64] `CTL_NET :: 4`
- [OS == .MACOS && CPU == .ARM64] `CTL_DEBUG :: 5`
- [OS == .MACOS && CPU == .ARM64] `CTL_HW :: 6`
- [OS == .MACOS && CPU == .ARM64] `CTL_MACHDEP :: 7`
- [OS == .MACOS && CPU == .ARM64] `CTL_USER :: 8`
- [OS == .MACOS && CPU == .ARM64] `CTL_MAXID :: 9`
- [OS == .MACOS && CPU == .ARM64] `KERN_OSTYPE :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_OSRELEASE :: 2`
- [OS == .MACOS && CPU == .ARM64] `KERN_OSREV :: 3`
- [OS == .MACOS && CPU == .ARM64] `KERN_VERSION :: 4`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXVNODES :: 5`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXPROC :: 6`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXFILES :: 7`
- [OS == .MACOS && CPU == .ARM64] `KERN_ARGMAX :: 8`
- [OS == .MACOS && CPU == .ARM64] `KERN_SECURELVL :: 9`
- [OS == .MACOS && CPU == .ARM64] `KERN_HOSTNAME :: 10`
- [OS == .MACOS && CPU == .ARM64] `KERN_HOSTID :: 11`
- [OS == .MACOS && CPU == .ARM64] `KERN_CLOCKRATE :: 12`
- [OS == .MACOS && CPU == .ARM64] `KERN_VNODE :: 13`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC :: 14`
- [OS == .MACOS && CPU == .ARM64] `KERN_FILE :: 15`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROF :: 16`
- [OS == .MACOS && CPU == .ARM64] `KERN_POSIX1 :: 17`
- [OS == .MACOS && CPU == .ARM64] `KERN_NGROUPS :: 18`
- [OS == .MACOS && CPU == .ARM64] `KERN_JOB_CONTROL :: 19`
- [OS == .MACOS && CPU == .ARM64] `KERN_SAVED_IDS :: 20`
- [OS == .MACOS && CPU == .ARM64] `KERN_BOOTTIME :: 21`
- [OS == .MACOS && CPU == .ARM64] `KERN_NISDOMAINNAME :: 22`
- [OS == .MACOS && CPU == .ARM64] `KERN_DOMAINNAME :: KERN_NISDOMAINNAME`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXPARTITIONS :: 23`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDEBUG :: 24`
- [OS == .MACOS && CPU == .ARM64] `KERN_UPDATEINTERVAL :: 25`
- [OS == .MACOS && CPU == .ARM64] `KERN_OSRELDATE :: 26`
- [OS == .MACOS && CPU == .ARM64] `KERN_NTP_PLL :: 27`
- [OS == .MACOS && CPU == .ARM64] `KERN_BOOTFILE :: 28`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXFILESPERPROC :: 29`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXPROCPERUID :: 30`
- [OS == .MACOS && CPU == .ARM64] `KERN_DUMPDEV :: 31`
- [OS == .MACOS && CPU == .ARM64] `KERN_IPC :: 32`
- [OS == .MACOS && CPU == .ARM64] `KERN_DUMMY :: 33`
- [OS == .MACOS && CPU == .ARM64] `KERN_PS_STRINGS :: 34`
- [OS == .MACOS && CPU == .ARM64] `KERN_USRSTACK32 :: 35`
- [OS == .MACOS && CPU == .ARM64] `KERN_LOGSIGEXIT :: 36`
- [OS == .MACOS && CPU == .ARM64] `KERN_SYMFILE :: 37`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROCARGS :: 38`
- [OS == .MACOS && CPU == .ARM64] `KERN_NETBOOT :: 40`
- [OS == .MACOS && CPU == .ARM64] `KERN_SYSV :: 42`
- [OS == .MACOS && CPU == .ARM64] `KERN_AFFINITY :: 43`
- [OS == .MACOS && CPU == .ARM64] `KERN_TRANSLATE :: 44`
- [OS == .MACOS && CPU == .ARM64] `KERN_CLASSIC :: KERN_TRANSLATE`
- [OS == .MACOS && CPU == .ARM64] `KERN_EXEC :: 45`
- [OS == .MACOS && CPU == .ARM64] `KERN_CLASSICHANDLER :: KERN_EXEC`
- [OS == .MACOS && CPU == .ARM64] `KERN_AIOMAX :: 46`
- [OS == .MACOS && CPU == .ARM64] `KERN_AIOPROCMAX :: 47`
- [OS == .MACOS && CPU == .ARM64] `KERN_AIOTHREADS :: 48`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROCARGS2 :: 49`
- [OS == .MACOS && CPU == .ARM64] `KERN_COREFILE :: 50`
- [OS == .MACOS && CPU == .ARM64] `KERN_COREDUMP :: 51`
- [OS == .MACOS && CPU == .ARM64] `KERN_SUGID_COREDUMP :: 52`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROCDELAYTERM :: 53`
- [OS == .MACOS && CPU == .ARM64] `KERN_SHREG_PRIVATIZABLE :: 54`
- [OS == .MACOS && CPU == .ARM64] `KERN_LOW_PRI_WINDOW :: 56`
- [OS == .MACOS && CPU == .ARM64] `KERN_LOW_PRI_DELAY :: 57`
- [OS == .MACOS && CPU == .ARM64] `KERN_POSIX :: 58`
- [OS == .MACOS && CPU == .ARM64] `KERN_USRSTACK64 :: 59`
- [OS == .MACOS && CPU == .ARM64] `KERN_NX_PROTECTION :: 60`
- [OS == .MACOS && CPU == .ARM64] `KERN_TFP :: 61`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROCNAME :: 62`
- [OS == .MACOS && CPU == .ARM64] `KERN_THALTSTACK :: 63`
- [OS == .MACOS && CPU == .ARM64] `KERN_SPECULATIVE_READS :: 64`
- [OS == .MACOS && CPU == .ARM64] `KERN_OSVERSION :: 65`
- [OS == .MACOS && CPU == .ARM64] `KERN_SAFEBOOT :: 66`
- [OS == .MACOS && CPU == .ARM64] `KERN_RAGEVNODE :: 68`
- [OS == .MACOS && CPU == .ARM64] `KERN_TTY :: 69`
- [OS == .MACOS && CPU == .ARM64] `KERN_CHECKOPENEVT :: 70`
- [OS == .MACOS && CPU == .ARM64] `KERN_THREADNAME :: 71`
- [OS == .MACOS && CPU == .ARM64] `KERN_MAXID :: 72`
- [OS == .MACOS && CPU == .ARM64] `KERN_USRSTACK :: KERN_USRSTACK64`
- [OS == .MACOS && CPU == .ARM64] `KERN_RAGE_PROC :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_RAGE_THREAD :: 2`
- [OS == .MACOS && CPU == .ARM64] `KERN_UNRAGE_PROC :: 3`
- [OS == .MACOS && CPU == .ARM64] `KERN_UNRAGE_THREAD :: 4`
- [OS == .MACOS && CPU == .ARM64] `KERN_OPENEVT_PROC :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_UNOPENEVT_PROC :: 2`
- [OS == .MACOS && CPU == .ARM64] `KERN_TFP_POLICY :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_TFP_POLICY_DENY :: 0`
- [OS == .MACOS && CPU == .ARM64] `KERN_TFP_POLICY_DEFAULT :: 2`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDEFLAGS :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDDFLAGS :: 2`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDENABLE :: 3`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDSETBUF :: 4`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDGETBUF :: 5`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDSETUP :: 6`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDREMOVE :: 7`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDSETREG :: 8`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDGETREG :: 9`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDREADTR :: 10`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDPIDTR :: 11`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDTHRMAP :: 12`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDPIDEX :: 14`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDSETRTCDEC :: 15`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDGETENTROPY :: 16`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDWRITETR :: 17`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDWRITEMAP :: 18`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDTEST :: 19`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDREADCURTHRMAP :: 21`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDSET_TYPEFILTER :: 22`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDBUFWAIT :: 23`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDCPUMAP :: 24`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDWRITEMAP_V3 :: 27`
- [OS == .MACOS && CPU == .ARM64] `KERN_KDWRITETR_V3 :: 28`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_ALL :: 0`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_PID :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_PGRP :: 2`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_SESSION :: 3`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_TTY :: 4`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_UID :: 5`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_RUID :: 6`
- [OS == .MACOS && CPU == .ARM64] `KERN_PROC_LCID :: 7`
- [OS == .MACOS && CPU == .ARM64] `KERN_VFSNSPACE_HANDLE_PROC :: 1`
- [OS == .MACOS && CPU == .ARM64] `KERN_VFSNSPACE_UNHANDLE_PROC :: 2`
- [OS == .MACOS && CPU == .ARM64] `WMESGLEN :: 7`
- [OS == .MACOS && CPU == .ARM64] `EPROC_CTTY :: 0x01`
- [OS == .MACOS && CPU == .ARM64] `EPROC_SLEADER :: 0x02`
- [OS == .MACOS && CPU == .ARM64] `COMAPT_MAXLOGNAME :: 12`
- [OS == .MACOS && CPU == .ARM64] `KIPC_MAXSOCKBUF :: 1`
- [OS == .MACOS && CPU == .ARM64] `KIPC_SOCKBUF_WASTE :: 2`
- [OS == .MACOS && CPU == .ARM64] `KIPC_SOMAXCONN :: 3`
- [OS == .MACOS && CPU == .ARM64] `KIPC_MAX_LINKHDR :: 4`
- [OS == .MACOS && CPU == .ARM64] `KIPC_MAX_PROTOHDR :: 5`
- [OS == .MACOS && CPU == .ARM64] `KIPC_MAX_HDR :: 6`
- [OS == .MACOS && CPU == .ARM64] `KIPC_MAX_DATALEN :: 7`
- [OS == .MACOS && CPU == .ARM64] `KIPC_MBSTAT :: 8`
- [OS == .MACOS && CPU == .ARM64] `KIPC_NMBCLUSTERS :: 9`
- [OS == .MACOS && CPU == .ARM64] `KIPC_SOQLIMITCOMPAT :: 10`
- [OS == .MACOS && CPU == .ARM64] `VM_METER :: 1`
- [OS == .MACOS && CPU == .ARM64] `VM_LOADAVG :: 2`
- [OS == .MACOS && CPU == .ARM64] `VM_MACHFACTOR :: 4`
- [OS == .MACOS && CPU == .ARM64] `VM_SWAPUSAGE :: 5`
- [OS == .MACOS && CPU == .ARM64] `VM_MAXID :: 6`
- [OS == .MACOS && CPU == .ARM64] `LSCALE :: 1000`
- [OS == .MACOS && CPU == .ARM64] `HW_MACHINE :: 1`
- [OS == .MACOS && CPU == .ARM64] `HW_MODEL :: 2`
- [OS == .MACOS && CPU == .ARM64] `HW_NCPU :: 3`
- [OS == .MACOS && CPU == .ARM64] `HW_BYTEORDER :: 4`
- [OS == .MACOS && CPU == .ARM64] `HW_PHYSMEM :: 5`
- [OS == .MACOS && CPU == .ARM64] `HW_USERMEM :: 6`
- [OS == .MACOS && CPU == .ARM64] `HW_PAGESIZE :: 7`
- [OS == .MACOS && CPU == .ARM64] `HW_DISKNAMES :: 8`
- [OS == .MACOS && CPU == .ARM64] `HW_DISKSTATS :: 9`
- [OS == .MACOS && CPU == .ARM64] `HW_EPOCH :: 10`
- [OS == .MACOS && CPU == .ARM64] `HW_FLOATINGPT :: 11`
- [OS == .MACOS && CPU == .ARM64] `HW_MACHINE_ARCH :: 12`
- [OS == .MACOS && CPU == .ARM64] `HW_VECTORUNIT :: 13`
- [OS == .MACOS && CPU == .ARM64] `HW_BUS_FREQ :: 14`
- [OS == .MACOS && CPU == .ARM64] `HW_CPU_FREQ :: 15`
- [OS == .MACOS && CPU == .ARM64] `HW_CACHELINE :: 16`
- [OS == .MACOS && CPU == .ARM64] `HW_L1ICACHESIZE :: 17`
- [OS == .MACOS && CPU == .ARM64] `HW_L1DCACHESIZE :: 18`
- [OS == .MACOS && CPU == .ARM64] `HW_L2SETTINGS :: 19`
- [OS == .MACOS && CPU == .ARM64] `HW_L2CACHESIZE :: 20`
- [OS == .MACOS && CPU == .ARM64] `HW_L3SETTINGS :: 21`
- [OS == .MACOS && CPU == .ARM64] `HW_L3CACHESIZE :: 22`
- [OS == .MACOS && CPU == .ARM64] `HW_TB_FREQ :: 23`
- [OS == .MACOS && CPU == .ARM64] `HW_MEMSIZE :: 24`
- [OS == .MACOS && CPU == .ARM64] `HW_AVAILCPU :: 25`
- [OS == .MACOS && CPU == .ARM64] `HW_TARGET :: 26`
- [OS == .MACOS && CPU == .ARM64] `HW_PRODUCT :: 27`
- [OS == .MACOS && CPU == .ARM64] `HW_MAXID :: 28`
- [OS == .MACOS && CPU == .ARM64] `USER_CS_PATH :: 1`
- [OS == .MACOS && CPU == .ARM64] `USER_BC_BASE_MAX :: 2`
- [OS == .MACOS && CPU == .ARM64] `USER_BC_DIM_MAX :: 3`
- [OS == .MACOS && CPU == .ARM64] `USER_BC_SCALE_MAX :: 4`
- [OS == .MACOS && CPU == .ARM64] `USER_BC_STRING_MAX :: 5`
- [OS == .MACOS && CPU == .ARM64] `USER_COLL_WEIGHTS_MAX :: 6`
- [OS == .MACOS && CPU == .ARM64] `USER_EXPR_NEST_MAX :: 7`
- [OS == .MACOS && CPU == .ARM64] `USER_LINE_MAX :: 8`
- [OS == .MACOS && CPU == .ARM64] `USER_RE_DUP_MAX :: 9`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_VERSION :: 10`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_C_BIND :: 11`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_C_DEV :: 12`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_CHAR_TERM :: 13`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_FORT_DEV :: 14`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_FORT_RUN :: 15`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_LOCALEDEF :: 16`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_SW_DEV :: 17`
- [OS == .MACOS && CPU == .ARM64] `USER_POSIX2_UPE :: 18`
- [OS == .MACOS && CPU == .ARM64] `USER_STREAM_MAX :: 19`
- [OS == .MACOS && CPU == .ARM64] `USER_TZNAME_MAX :: 20`
- [OS == .MACOS && CPU == .ARM64] `USER_MAXID :: 21`
- [OS == .MACOS && CPU == .ARM64] `CTL_DEBUG_NAME :: 0`
- [OS == .MACOS && CPU == .ARM64] `CTL_DEBUG_VALUE :: 1`
- [OS == .MACOS && CPU == .ARM64] `CTL_DEBUG_MAXID :: 20`
- [OS == .MACOS && CPU == .ARM64] `session :: struct`
- [OS == .MACOS && CPU == .ARM64] `pgrp :: struct`
- [OS == .MACOS && CPU == .ARM64] `proc :: struct`
- [OS == .MACOS && CPU == .ARM64] `extern_proc :: struct`
- [OS == .MACOS && CPU == .ARM64] `vmspace :: struct`
- [OS == .MACOS && CPU == .ARM64] `ctlname :: struct`
- [OS == .MACOS && CPU == .ARM64] `_pcred :: struct`
- [OS == .MACOS && CPU == .ARM64] `_ucred :: struct`
- [OS == .MACOS && CPU == .ARM64] `kinfo_proc :: struct`
- [OS == .MACOS && CPU == .ARM64] `xsw_usage :: struct`
- [OS == .MACOS && CPU == .ARM64] `loadavg :: struct`
- [OS == .MACOS && CPU == .ARM64] `sysctl :: (unknown0: *s32, unknown1: u32, unknown2: *void, unknown3: *u64, unknown4: *void, unknown5: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sysctlbyname :: (unknown0: *u8, unknown1: *void, unknown2: *u64, unknown3: *void, unknown4: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .ARM64] `sysctlnametomib :: (unknown0: *u8, unknown1: *s32, unknown2: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `RTLD_LAZY :: 0x1`
- [OS == .MACOS && CPU == .X64] `RTLD_NOW :: 0x2`
- [OS == .MACOS && CPU == .X64] `RTLD_LOCAL :: 0x4`
- [OS == .MACOS && CPU == .X64] `RTLD_GLOBAL :: 0x8`
- [OS == .MACOS && CPU == .X64] `RTLD_NOLOAD :: 0x10`
- [OS == .MACOS && CPU == .X64] `RTLD_NODELETE :: 0x80`
- [OS == .MACOS && CPU == .X64] `RTLD_FIRST :: 0x100`
- [OS == .MACOS && CPU == .X64] `RTLD_NEXT :: cast,trunc(*void) -1`
- [OS == .MACOS && CPU == .X64] `RTLD_DEFAULT :: cast,trunc(*void) -2`
- [OS == .MACOS && CPU == .X64] `RTLD_SELF :: cast,trunc(*void) -3`
- [OS == .MACOS && CPU == .X64] `RTLD_MAIN_ONLY :: cast,trunc(*void) -5`
- [OS == .MACOS && CPU == .X64] `__DARWIN_MAXNAMLEN :: 255`
- [OS == .MACOS && CPU == .X64] `__DARWIN_MAXPATHLEN :: 1024`
- [OS == .MACOS && CPU == .X64] `MAXNAMLEN :: __DARWIN_MAXNAMLEN`
- [OS == .MACOS && CPU == .X64] `DT :: enum u8`
- [OS == .MACOS && CPU == .X64] `DIRBLKSIZ :: 1024`
- [OS == .MACOS && CPU == .X64] `DTF_HIDEW :: 0x0001`
- [OS == .MACOS && CPU == .X64] `DTF_NODUP :: 0x0002`
- [OS == .MACOS && CPU == .X64] `DTF_REWIND :: 0x0004`
- [OS == .MACOS && CPU == .X64] `__DTF_READALL :: 0x0008`
- [OS == .MACOS && CPU == .X64] `__DTF_SKIPREAD :: 0x0010`
- [OS == .MACOS && CPU == .X64] `EPERM :: 1`
- [OS == .MACOS && CPU == .X64] `ENOENT :: 2`
- [OS == .MACOS && CPU == .X64] `ESRCH :: 3`
- [OS == .MACOS && CPU == .X64] `EINTR :: 4`
- [OS == .MACOS && CPU == .X64] `EIO :: 5`
- [OS == .MACOS && CPU == .X64] `ENXIO :: 6`
- [OS == .MACOS && CPU == .X64] `E2BIG :: 7`
- [OS == .MACOS && CPU == .X64] `ENOEXEC :: 8`
- [OS == .MACOS && CPU == .X64] `EBADF :: 9`
- [OS == .MACOS && CPU == .X64] `ECHILD :: 10`
- [OS == .MACOS && CPU == .X64] `EDEADLK :: 11`
- [OS == .MACOS && CPU == .X64] `ENOMEM :: 12`
- [OS == .MACOS && CPU == .X64] `EACCES :: 13`
- [OS == .MACOS && CPU == .X64] `EFAULT :: 14`
- [OS == .MACOS && CPU == .X64] `ENOTBLK :: 15`
- [OS == .MACOS && CPU == .X64] `EBUSY :: 16`
- [OS == .MACOS && CPU == .X64] `EEXIST :: 17`
- [OS == .MACOS && CPU == .X64] `EXDEV :: 18`
- [OS == .MACOS && CPU == .X64] `ENODEV :: 19`
- [OS == .MACOS && CPU == .X64] `ENOTDIR :: 20`
- [OS == .MACOS && CPU == .X64] `EISDIR :: 21`
- [OS == .MACOS && CPU == .X64] `EINVAL :: 22`
- [OS == .MACOS && CPU == .X64] `ENFILE :: 23`
- [OS == .MACOS && CPU == .X64] `EMFILE :: 24`
- [OS == .MACOS && CPU == .X64] `ENOTTY :: 25`
- [OS == .MACOS && CPU == .X64] `ETXTBSY :: 26`
- [OS == .MACOS && CPU == .X64] `EFBIG :: 27`
- [OS == .MACOS && CPU == .X64] `ENOSPC :: 28`
- [OS == .MACOS && CPU == .X64] `ESPIPE :: 29`
- [OS == .MACOS && CPU == .X64] `EROFS :: 30`
- [OS == .MACOS && CPU == .X64] `EMLINK :: 31`
- [OS == .MACOS && CPU == .X64] `EPIPE :: 32`
- [OS == .MACOS && CPU == .X64] `EDOM :: 33`
- [OS == .MACOS && CPU == .X64] `ERANGE :: 34`
- [OS == .MACOS && CPU == .X64] `EAGAIN :: 35`
- [OS == .MACOS && CPU == .X64] `EWOULDBLOCK :: EAGAIN`
- [OS == .MACOS && CPU == .X64] `EINPROGRESS :: 36`
- [OS == .MACOS && CPU == .X64] `EALREADY :: 37`
- [OS == .MACOS && CPU == .X64] `ENOTSOCK :: 38`
- [OS == .MACOS && CPU == .X64] `EDESTADDRREQ :: 39`
- [OS == .MACOS && CPU == .X64] `EMSGSIZE :: 40`
- [OS == .MACOS && CPU == .X64] `EPROTOTYPE :: 41`
- [OS == .MACOS && CPU == .X64] `ENOPROTOOPT :: 42`
- [OS == .MACOS && CPU == .X64] `EPROTONOSUPPORT :: 43`
- [OS == .MACOS && CPU == .X64] `ESOCKTNOSUPPORT :: 44`
- [OS == .MACOS && CPU == .X64] `ENOTSUP :: 45`
- [OS == .MACOS && CPU == .X64] `EPFNOSUPPORT :: 46`
- [OS == .MACOS && CPU == .X64] `EAFNOSUPPORT :: 47`
- [OS == .MACOS && CPU == .X64] `EADDRINUSE :: 48`
- [OS == .MACOS && CPU == .X64] `EADDRNOTAVAIL :: 49`
- [OS == .MACOS && CPU == .X64] `ENETDOWN :: 50`
- [OS == .MACOS && CPU == .X64] `ENETUNREACH :: 51`
- [OS == .MACOS && CPU == .X64] `ENETRESET :: 52`
- [OS == .MACOS && CPU == .X64] `ECONNABORTED :: 53`
- [OS == .MACOS && CPU == .X64] `ECONNRESET :: 54`
- [OS == .MACOS && CPU == .X64] `ENOBUFS :: 55`
- [OS == .MACOS && CPU == .X64] `EISCONN :: 56`
- [OS == .MACOS && CPU == .X64] `ENOTCONN :: 57`
- [OS == .MACOS && CPU == .X64] `ESHUTDOWN :: 58`
- [OS == .MACOS && CPU == .X64] `ETOOMANYREFS :: 59`
- [OS == .MACOS && CPU == .X64] `ETIMEDOUT :: 60`
- [OS == .MACOS && CPU == .X64] `ECONNREFUSED :: 61`
- [OS == .MACOS && CPU == .X64] `ELOOP :: 62`
- [OS == .MACOS && CPU == .X64] `ENAMETOOLONG :: 63`
- [OS == .MACOS && CPU == .X64] `EHOSTDOWN :: 64`
- [OS == .MACOS && CPU == .X64] `EHOSTUNREACH :: 65`
- [OS == .MACOS && CPU == .X64] `ENOTEMPTY :: 66`
- [OS == .MACOS && CPU == .X64] `EPROCLIM :: 67`
- [OS == .MACOS && CPU == .X64] `EUSERS :: 68`
- [OS == .MACOS && CPU == .X64] `EDQUOT :: 69`
- [OS == .MACOS && CPU == .X64] `ESTALE :: 70`
- [OS == .MACOS && CPU == .X64] `EREMOTE :: 71`
- [OS == .MACOS && CPU == .X64] `EBADRPC :: 72`
- [OS == .MACOS && CPU == .X64] `ERPCMISMATCH :: 73`
- [OS == .MACOS && CPU == .X64] `EPROGUNAVAIL :: 74`
- [OS == .MACOS && CPU == .X64] `EPROGMISMATCH :: 75`
- [OS == .MACOS && CPU == .X64] `EPROCUNAVAIL :: 76`
- [OS == .MACOS && CPU == .X64] `ENOLCK :: 77`
- [OS == .MACOS && CPU == .X64] `ENOSYS :: 78`
- [OS == .MACOS && CPU == .X64] `EFTYPE :: 79`
- [OS == .MACOS && CPU == .X64] `EAUTH :: 80`
- [OS == .MACOS && CPU == .X64] `ENEEDAUTH :: 81`
- [OS == .MACOS && CPU == .X64] `EPWROFF :: 82`
- [OS == .MACOS && CPU == .X64] `EDEVERR :: 83`
- [OS == .MACOS && CPU == .X64] `EOVERFLOW :: 84`
- [OS == .MACOS && CPU == .X64] `EBADEXEC :: 85`
- [OS == .MACOS && CPU == .X64] `EBADARCH :: 86`
- [OS == .MACOS && CPU == .X64] `ESHLIBVERS :: 87`
- [OS == .MACOS && CPU == .X64] `EBADMACHO :: 88`
- [OS == .MACOS && CPU == .X64] `ECANCELED :: 89`
- [OS == .MACOS && CPU == .X64] `EIDRM :: 90`
- [OS == .MACOS && CPU == .X64] `ENOMSG :: 91`
- [OS == .MACOS && CPU == .X64] `EILSEQ :: 92`
- [OS == .MACOS && CPU == .X64] `ENOATTR :: 93`
- [OS == .MACOS && CPU == .X64] `EBADMSG :: 94`
- [OS == .MACOS && CPU == .X64] `EMULTIHOP :: 95`
- [OS == .MACOS && CPU == .X64] `ENODATA :: 96`
- [OS == .MACOS && CPU == .X64] `ENOLINK :: 97`
- [OS == .MACOS && CPU == .X64] `ENOSR :: 98`
- [OS == .MACOS && CPU == .X64] `ENOSTR :: 99`
- [OS == .MACOS && CPU == .X64] `EPROTO :: 100`
- [OS == .MACOS && CPU == .X64] `ETIME :: 101`
- [OS == .MACOS && CPU == .X64] `EOPNOTSUPP :: 102`
- [OS == .MACOS && CPU == .X64] `ENOPOLICY :: 103`
- [OS == .MACOS && CPU == .X64] `ENOTRECOVERABLE :: 104`
- [OS == .MACOS && CPU == .X64] `EOWNERDEAD :: 105`
- [OS == .MACOS && CPU == .X64] `EQFULL :: 106`
- [OS == .MACOS && CPU == .X64] `ELAST :: 106`
- [OS == .MACOS && CPU == .X64] `FNM_NOMATCH :: 1`
- [OS == .MACOS && CPU == .X64] `FNM_NOESCAPE :: 0x01`
- [OS == .MACOS && CPU == .X64] `FNM_PATHNAME :: 0x02`
- [OS == .MACOS && CPU == .X64] `FNM_PERIOD :: 0x04`
- [OS == .MACOS && CPU == .X64] `FNM_NOSYS :: -1`
- [OS == .MACOS && CPU == .X64] `FNM_LEADING_DIR :: 0x08`
- [OS == .MACOS && CPU == .X64] `FNM_CASEFOLD :: 0x10`
- [OS == .MACOS && CPU == .X64] `FNM_IGNORECASE :: FNM_CASEFOLD`
- [OS == .MACOS && CPU == .X64] `FNM_FILE_NAME :: FNM_PATHNAME`
- [OS == .MACOS && CPU == .X64] `_PATH_PWD :: "/etc"`
- [OS == .MACOS && CPU == .X64] `_PATH_PASSWD :: "/etc/passwd"`
- [OS == .MACOS && CPU == .X64] `_PASSWD :: "passwd"`
- [OS == .MACOS && CPU == .X64] `_PATH_MASTERPASSWD :: "/etc/master.passwd"`
- [OS == .MACOS && CPU == .X64] `_PATH_MASTERPASSWD_LOCK :: "/etc/ptmp"`
- [OS == .MACOS && CPU == .X64] `_MASTERPASSWD :: "master.passwd"`
- [OS == .MACOS && CPU == .X64] `_PATH_MP_DB :: "/etc/pwd.db"`
- [OS == .MACOS && CPU == .X64] `_MP_DB :: "pwd.db"`
- [OS == .MACOS && CPU == .X64] `_PATH_SMP_DB :: "/etc/spwd.db"`
- [OS == .MACOS && CPU == .X64] `_SMP_DB :: "spwd.db"`
- [OS == .MACOS && CPU == .X64] `_PATH_PWD_MKDB :: "/usr/sbin/pwd_mkdb"`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_LEN :: 128`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_NOUID :: 0x01`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_NOGID :: 0x02`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_NOCHG :: 0x04`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_NOEXP :: 0x08`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_WARNDAYS :: 14`
- [OS == .MACOS && CPU == .X64] `_PASSWORD_CHGNOW :: -1`
- [OS == .MACOS && CPU == .X64] `__DARWIN_NSIG :: 32`
- [OS == .MACOS && CPU == .X64] `NSIG :: __DARWIN_NSIG`
- [OS == .MACOS && CPU == .X64] `SIGHUP :: 1`
- [OS == .MACOS && CPU == .X64] `SIGINT :: 2`
- [OS == .MACOS && CPU == .X64] `SIGQUIT :: 3`
- [OS == .MACOS && CPU == .X64] `SIGILL :: 4`
- [OS == .MACOS && CPU == .X64] `SIGTRAP :: 5`
- [OS == .MACOS && CPU == .X64] `SIGABRT :: 6`
- [OS == .MACOS && CPU == .X64] `SIGIOT :: SIGABRT`
- [OS == .MACOS && CPU == .X64] `SIGEMT :: 7`
- [OS == .MACOS && CPU == .X64] `SIGFPE :: 8`
- [OS == .MACOS && CPU == .X64] `SIGKILL :: 9`
- [OS == .MACOS && CPU == .X64] `SIGBUS :: 10`
- [OS == .MACOS && CPU == .X64] `SIGSEGV :: 11`
- [OS == .MACOS && CPU == .X64] `SIGSYS :: 12`
- [OS == .MACOS && CPU == .X64] `SIGPIPE :: 13`
- [OS == .MACOS && CPU == .X64] `SIGALRM :: 14`
- [OS == .MACOS && CPU == .X64] `SIGTERM :: 15`
- [OS == .MACOS && CPU == .X64] `SIGURG :: 16`
- [OS == .MACOS && CPU == .X64] `SIGSTOP :: 17`
- [OS == .MACOS && CPU == .X64] `SIGTSTP :: 18`
- [OS == .MACOS && CPU == .X64] `SIGCONT :: 19`
- [OS == .MACOS && CPU == .X64] `SIGCHLD :: 20`
- [OS == .MACOS && CPU == .X64] `SIGTTIN :: 21`
- [OS == .MACOS && CPU == .X64] `SIGTTOU :: 22`
- [OS == .MACOS && CPU == .X64] `SIGIO :: 23`
- [OS == .MACOS && CPU == .X64] `SIGXCPU :: 24`
- [OS == .MACOS && CPU == .X64] `SIGXFSZ :: 25`
- [OS == .MACOS && CPU == .X64] `SIGVTALRM :: 26`
- [OS == .MACOS && CPU == .X64] `SIGPROF :: 27`
- [OS == .MACOS && CPU == .X64] `SIGWINCH :: 28`
- [OS == .MACOS && CPU == .X64] `SIGINFO :: 29`
- [OS == .MACOS && CPU == .X64] `SIGUSR1 :: 30`
- [OS == .MACOS && CPU == .X64] `SIGUSR2 :: 31`
- [OS == .MACOS && CPU == .X64] `SIG_DFL :: cast(*void) 0`
- [OS == .MACOS && CPU == .X64] `SIG_IGN :: cast(*void) 1`
- [OS == .MACOS && CPU == .X64] `SIG_HOLD :: cast(*void) 5`
- [OS == .MACOS && CPU == .X64] `SIG_ERR :: cast(*void) -1`
- [OS == .MACOS && CPU == .X64] `SIGEV_NONE :: 0`
- [OS == .MACOS && CPU == .X64] `SIGEV_SIGNAL :: 1`
- [OS == .MACOS && CPU == .X64] `SIGEV_THREAD :: 3`
- [OS == .MACOS && CPU == .X64] `ILL_NOOP :: 0`
- [OS == .MACOS && CPU == .X64] `ILL_ILLOPC :: 1`
- [OS == .MACOS && CPU == .X64] `ILL_ILLTRP :: 2`
- [OS == .MACOS && CPU == .X64] `ILL_PRVOPC :: 3`
- [OS == .MACOS && CPU == .X64] `ILL_ILLOPN :: 4`
- [OS == .MACOS && CPU == .X64] `ILL_ILLADR :: 5`
- [OS == .MACOS && CPU == .X64] `ILL_PRVREG :: 6`
- [OS == .MACOS && CPU == .X64] `ILL_COPROC :: 7`
- [OS == .MACOS && CPU == .X64] `ILL_BADSTK :: 8`
- [OS == .MACOS && CPU == .X64] `FPE_NOOP :: 0`
- [OS == .MACOS && CPU == .X64] `FPE_FLTDIV :: 1`
- [OS == .MACOS && CPU == .X64] `FPE_FLTOVF :: 2`
- [OS == .MACOS && CPU == .X64] `FPE_FLTUND :: 3`
- [OS == .MACOS && CPU == .X64] `FPE_FLTRES :: 4`
- [OS == .MACOS && CPU == .X64] `FPE_FLTINV :: 5`
- [OS == .MACOS && CPU == .X64] `FPE_FLTSUB :: 6`
- [OS == .MACOS && CPU == .X64] `FPE_INTDIV :: 7`
- [OS == .MACOS && CPU == .X64] `FPE_INTOVF :: 8`
- [OS == .MACOS && CPU == .X64] `SEGV_NOOP :: 0`
- [OS == .MACOS && CPU == .X64] `SEGV_MAPERR :: 1`
- [OS == .MACOS && CPU == .X64] `SEGV_ACCERR :: 2`
- [OS == .MACOS && CPU == .X64] `BUS_NOOP :: 0`
- [OS == .MACOS && CPU == .X64] `BUS_ADRALN :: 1`
- [OS == .MACOS && CPU == .X64] `BUS_ADRERR :: 2`
- [OS == .MACOS && CPU == .X64] `BUS_OBJERR :: 3`
- [OS == .MACOS && CPU == .X64] `TRAP_BRKPT :: 1`
- [OS == .MACOS && CPU == .X64] `TRAP_TRACE :: 2`
- [OS == .MACOS && CPU == .X64] `CLD_NOOP :: 0`
- [OS == .MACOS && CPU == .X64] `CLD_EXITED :: 1`
- [OS == .MACOS && CPU == .X64] `CLD_KILLED :: 2`
- [OS == .MACOS && CPU == .X64] `CLD_DUMPED :: 3`
- [OS == .MACOS && CPU == .X64] `CLD_TRAPPED :: 4`
- [OS == .MACOS && CPU == .X64] `CLD_STOPPED :: 5`
- [OS == .MACOS && CPU == .X64] `CLD_CONTINUED :: 6`
- [OS == .MACOS && CPU == .X64] `POLL_IN :: 1`
- [OS == .MACOS && CPU == .X64] `POLL_OUT :: 2`
- [OS == .MACOS && CPU == .X64] `POLL_MSG :: 3`
- [OS == .MACOS && CPU == .X64] `POLL_ERR :: 4`
- [OS == .MACOS && CPU == .X64] `POLL_PRI :: 5`
- [OS == .MACOS && CPU == .X64] `POLL_HUP :: 6`
- [OS == .MACOS && CPU == .X64] `SA_ONSTACK :: 0x0001`
- [OS == .MACOS && CPU == .X64] `SA_RESTART :: 0x0002`
- [OS == .MACOS && CPU == .X64] `SA_RESETHAND :: 0x0004`
- [OS == .MACOS && CPU == .X64] `SA_NOCLDSTOP :: 0x0008`
- [OS == .MACOS && CPU == .X64] `SA_NODEFER :: 0x0010`
- [OS == .MACOS && CPU == .X64] `SA_NOCLDWAIT :: 0x0020`
- [OS == .MACOS && CPU == .X64] `SA_SIGINFO :: 0x0040`
- [OS == .MACOS && CPU == .X64] `SA_USERTRAMP :: 0x0100`
- [OS == .MACOS && CPU == .X64] `SA_64REGSET :: 0x0200`
- [OS == .MACOS && CPU == .X64] `SA_USERSPACE_MASK :: SA_ONSTACK | SA_RESTART | SA_RESETHAND | SA_NOCLDSTOP | SA_NODEFER | SA_NOCLDWAIT | SA_SIGINFO`
- [OS == .MACOS && CPU == .X64] `SIG_BLOCK :: 1`
- [OS == .MACOS && CPU == .X64] `SIG_UNBLOCK :: 2`
- [OS == .MACOS && CPU == .X64] `SIG_SETMASK :: 3`
- [OS == .MACOS && CPU == .X64] `SI_USER :: 0x10001`
- [OS == .MACOS && CPU == .X64] `SI_QUEUE :: 0x10002`
- [OS == .MACOS && CPU == .X64] `SI_TIMER :: 0x10003`
- [OS == .MACOS && CPU == .X64] `SI_ASYNCIO :: 0x10004`
- [OS == .MACOS && CPU == .X64] `SI_MESGQ :: 0x10005`
- [OS == .MACOS && CPU == .X64] `SS_ONSTACK :: 0x0001`
- [OS == .MACOS && CPU == .X64] `SS_DISABLE :: 0x0004`
- [OS == .MACOS && CPU == .X64] `MINSIGSTKSZ :: 32768`
- [OS == .MACOS && CPU == .X64] `SIGSTKSZ :: 131072`
- [OS == .MACOS && CPU == .X64] `SV_ONSTACK :: SA_ONSTACK`
- [OS == .MACOS && CPU == .X64] `SV_INTERRUPT :: SA_RESTART`
- [OS == .MACOS && CPU == .X64] `SV_RESETHAND :: SA_RESETHAND`
- [OS == .MACOS && CPU == .X64] `SV_NODEFER :: SA_NODEFER`
- [OS == .MACOS && CPU == .X64] `SV_NOCLDSTOP :: SA_NOCLDSTOP`
- [OS == .MACOS && CPU == .X64] `SV_SIGINFO :: SA_SIGINFO`
- [OS == .MACOS && CPU == .X64] `BADSIG :: SIG_ERR`
- [OS == .MACOS && CPU == .X64] `__ipc_perm_new :: ipc_perm`
- [OS == .MACOS && CPU == .X64] `IPC_CREAT :: 0x200`
- [OS == .MACOS && CPU == .X64] `IPC_EXCL :: 0x400`
- [OS == .MACOS && CPU == .X64] `IPC_NOWAIT :: 0x800`
- [OS == .MACOS && CPU == .X64] `IPC_PRIVATE :: cast(key_t) 0`
- [OS == .MACOS && CPU == .X64] `IPC_RMID :: 0`
- [OS == .MACOS && CPU == .X64] `IPC_SET :: 1`
- [OS == .MACOS && CPU == .X64] `IPC_STAT :: 2`
- [OS == .MACOS && CPU == .X64] `IPC_R :: 0x100`
- [OS == .MACOS && CPU == .X64] `IPC_W :: 0x80`
- [OS == .MACOS && CPU == .X64] `IPC_M :: 0x1000`
- [OS == .MACOS && CPU == .X64] `PROT_NONE :: 0x00`
- [OS == .MACOS && CPU == .X64] `PROT_READ :: 0x01`
- [OS == .MACOS && CPU == .X64] `PROT_WRITE :: 0x02`
- [OS == .MACOS && CPU == .X64] `PROT_EXEC :: 0x04`
- [OS == .MACOS && CPU == .X64] `MAP_SHARED :: 0x0001`
- [OS == .MACOS && CPU == .X64] `MAP_PRIVATE :: 0x0002`
- [OS == .MACOS && CPU == .X64] `MAP_COPY :: MAP_PRIVATE`
- [OS == .MACOS && CPU == .X64] `MAP_FIXED :: 0x0010`
- [OS == .MACOS && CPU == .X64] `MAP_RENAME :: 0x0020`
- [OS == .MACOS && CPU == .X64] `MAP_NORESERVE :: 0x0040`
- [OS == .MACOS && CPU == .X64] `MAP_RESERVED0080 :: 0x0080`
- [OS == .MACOS && CPU == .X64] `MAP_NOEXTEND :: 0x0100`
- [OS == .MACOS && CPU == .X64] `MAP_HASSEMAPHORE :: 0x0200`
- [OS == .MACOS && CPU == .X64] `MAP_NOCACHE :: 0x0400`
- [OS == .MACOS && CPU == .X64] `MAP_JIT :: 0x0800`
- [OS == .MACOS && CPU == .X64] `MAP_FILE :: 0x0000`
- [OS == .MACOS && CPU == .X64] `MAP_ANON :: 0x1000`
- [OS == .MACOS && CPU == .X64] `MAP_ANONYMOUS :: MAP_ANON`
- [OS == .MACOS && CPU == .X64] `MAP_RESILIENT_CODESIGN :: 0x2000`
- [OS == .MACOS && CPU == .X64] `MAP_RESILIENT_MEDIA :: 0x4000`
- [OS == .MACOS && CPU == .X64] `MCL_CURRENT :: 0x0001`
- [OS == .MACOS && CPU == .X64] `MCL_FUTURE :: 0x0002`
- [OS == .MACOS && CPU == .X64] `MAP_FAILED :: cast,trunc(*void) -1`
- [OS == .MACOS && CPU == .X64] `MS_ASYNC :: 0x0001`
- [OS == .MACOS && CPU == .X64] `MS_INVALIDATE :: 0x0002`
- [OS == .MACOS && CPU == .X64] `MS_SYNC :: 0x0010`
- [OS == .MACOS && CPU == .X64] `MS_KILLPAGES :: 0x0004`
- [OS == .MACOS && CPU == .X64] `MS_DEACTIVATE :: 0x0008`
- [OS == .MACOS && CPU == .X64] `MADV_NORMAL :: 0`
- [OS == .MACOS && CPU == .X64] `MADV_RANDOM :: 1`
- [OS == .MACOS && CPU == .X64] `MADV_SEQUENTIAL :: 2`
- [OS == .MACOS && CPU == .X64] `MADV_WILLNEED :: 3`
- [OS == .MACOS && CPU == .X64] `MADV_DONTNEED :: 4`
- [OS == .MACOS && CPU == .X64] `MADV_FREE :: 5`
- [OS == .MACOS && CPU == .X64] `MADV_ZERO_WIRED_PAGES :: 6`
- [OS == .MACOS && CPU == .X64] `MADV_FREE_REUSABLE :: 7`
- [OS == .MACOS && CPU == .X64] `MADV_FREE_REUSE :: 8`
- [OS == .MACOS && CPU == .X64] `MADV_CAN_REUSE :: 9`
- [OS == .MACOS && CPU == .X64] `MADV_PAGEOUT :: 10`
- [OS == .MACOS && CPU == .X64] `MINCORE_INCORE :: 0x1`
- [OS == .MACOS && CPU == .X64] `MINCORE_REFERENCED :: 0x2`
- [OS == .MACOS && CPU == .X64] `MINCORE_MODIFIED :: 0x4`
- [OS == .MACOS && CPU == .X64] `MINCORE_REFERENCED_OTHER :: 0x8`
- [OS == .MACOS && CPU == .X64] `MINCORE_MODIFIED_OTHER :: 0x10`
- [OS == .MACOS && CPU == .X64] `MINCORE_PAGED_OUT :: 0x20`
- [OS == .MACOS && CPU == .X64] `MINCORE_COPIED :: 0x40`
- [OS == .MACOS && CPU == .X64] `MINCORE_ANONYMOUS :: 0x80`
- [OS == .MACOS && CPU == .X64] `POLLIN :: 0x0001`
- [OS == .MACOS && CPU == .X64] `POLLPRI :: 0x0002`
- [OS == .MACOS && CPU == .X64] `POLLOUT :: 0x0004`
- [OS == .MACOS && CPU == .X64] `POLLRDNORM :: 0x0040`
- [OS == .MACOS && CPU == .X64] `POLLWRNORM :: POLLOUT`
- [OS == .MACOS && CPU == .X64] `POLLRDBAND :: 0x0080`
- [OS == .MACOS && CPU == .X64] `POLLWRBAND :: 0x0100`
- [OS == .MACOS && CPU == .X64] `POLLEXTEND :: 0x0200`
- [OS == .MACOS && CPU == .X64] `POLLATTRIB :: 0x0400`
- [OS == .MACOS && CPU == .X64] `POLLNLINK :: 0x0800`
- [OS == .MACOS && CPU == .X64] `POLLWRITE :: 0x1000`
- [OS == .MACOS && CPU == .X64] `POLLERR :: 0x0008`
- [OS == .MACOS && CPU == .X64] `POLLHUP :: 0x0010`
- [OS == .MACOS && CPU == .X64] `POLLNVAL :: 0x0020`
- [OS == .MACOS && CPU == .X64] `POLLSTANDARD :: POLLIN|POLLPRI|POLLOUT|POLLRDNORM|POLLRDBAND| POLLWRBAND|POLLERR|POLLHUP|POLLNVAL`
- [OS == .MACOS && CPU == .X64] `SHM_RDONLY :: 0x1000`
- [OS == .MACOS && CPU == .X64] `SHM_RND :: 0x2000`
- [OS == .MACOS && CPU == .X64] `SHMLBA :: 4096`
- [OS == .MACOS && CPU == .X64] `SHM_R :: IPC_R`
- [OS == .MACOS && CPU == .X64] `SHM_W :: IPC_W`
- [OS == .MACOS && CPU == .X64] `shmid_ds :: __shmid_ds_new`
- [OS == .MACOS && CPU == .X64] `ITIMER_REAL :: 0`
- [OS == .MACOS && CPU == .X64] `ITIMER_VIRTUAL :: 1`
- [OS == .MACOS && CPU == .X64] `ITIMER_PROF :: 2`
- [OS == .MACOS && CPU == .X64] `DST_NONE :: 0`
- [OS == .MACOS && CPU == .X64] `DST_USA :: 1`
- [OS == .MACOS && CPU == .X64] `DST_AUST :: 2`
- [OS == .MACOS && CPU == .X64] `DST_WET :: 3`
- [OS == .MACOS && CPU == .X64] `DST_MET :: 4`
- [OS == .MACOS && CPU == .X64] `DST_EET :: 5`
- [OS == .MACOS && CPU == .X64] `DST_CAN :: 6`
- [OS == .MACOS && CPU == .X64] `CLOCKS_PER_SEC :: 1000000`
- [OS == .MACOS && CPU == .X64] `_SYS_NAMELEN :: 256`
- [OS == .MACOS && CPU == .X64] `WNOHANG :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `WUNTRACED :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `WCOREFLAG :: 0x80`
- [OS == .MACOS && CPU == .X64] `_WSTOPPED :: 0x7f`
- [OS == .MACOS && CPU == .X64] `WEXITED :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `WSTOPPED :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `WCONTINUED :: 0x00000010`
- [OS == .MACOS && CPU == .X64] `WNOWAIT :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `WAIT_ANY :: -1`
- [OS == .MACOS && CPU == .X64] `WAIT_MYPGRP :: 0`
- [OS == .MACOS && CPU == .X64] `F_OK :: 0`
- [OS == .MACOS && CPU == .X64] `X_OK :: 1<<0`
- [OS == .MACOS && CPU == .X64] `W_OK :: 1<<1`
- [OS == .MACOS && CPU == .X64] `R_OK :: 1<<2`
- [OS == .MACOS && CPU == .X64] `_READ_OK :: 1<<9`
- [OS == .MACOS && CPU == .X64] `_WRITE_OK :: 1<<10`
- [OS == .MACOS && CPU == .X64] `_EXECUTE_OK :: 1<<11`
- [OS == .MACOS && CPU == .X64] `_DELETE_OK :: 1<<12`
- [OS == .MACOS && CPU == .X64] `_APPEND_OK :: 1<<13`
- [OS == .MACOS && CPU == .X64] `_RMFILE_OK :: 1<<14`
- [OS == .MACOS && CPU == .X64] `_RATTR_OK :: 1<<15`
- [OS == .MACOS && CPU == .X64] `_WATTR_OK :: 1<<16`
- [OS == .MACOS && CPU == .X64] `_REXT_OK :: 1<<17`
- [OS == .MACOS && CPU == .X64] `_WEXT_OK :: 1<<18`
- [OS == .MACOS && CPU == .X64] `_RPERM_OK :: 1<<19`
- [OS == .MACOS && CPU == .X64] `_WPERM_OK :: 1<<20`
- [OS == .MACOS && CPU == .X64] `_CHOWN_OK :: 1<<21`
- [OS == .MACOS && CPU == .X64] `_ACCESS_EXTENDED_MASK :: _READ_OK | _WRITE_OK | _EXECUTE_OK | _DELETE_OK | _APPEND_OK | _RMFILE_OK | _REXT_OK | _WEXT_OK | _RATTR_OK | _WATTR_OK | _RPERM_OK | _WPERM_OK | _CHOWN_OK`
- [OS == .MACOS && CPU == .X64] `ACCESSX_MAX_DESCRIPTORS :: 100`
- [OS == .MACOS && CPU == .X64] `ACCESSX_MAX_TABLESIZE :: 16 * 1024`
- [OS == .MACOS && CPU == .X64] `PC :: enum u32`
- [OS == .MACOS && CPU == .X64] `_CS_PATH :: 1`
- [OS == .MACOS && CPU == .X64] `STDIN_FILENO :: 0`
- [OS == .MACOS && CPU == .X64] `STDOUT_FILENO :: 1`
- [OS == .MACOS && CPU == .X64] `STDERR_FILENO :: 2`
- [OS == .MACOS && CPU == .X64] `_XOPEN_VERSION :: 600`
- [OS == .MACOS && CPU == .X64] `_XOPEN_XCU_VERSION :: 4`
- [OS == .MACOS && CPU == .X64] `__ILP32_OFF32 :: -1`
- [OS == .MACOS && CPU == .X64] `__ILP32_OFFBIG :: 1`
- [OS == .MACOS && CPU == .X64] `__LP64_OFF64 :: 1`
- [OS == .MACOS && CPU == .X64] `__LPBIG_OFFBIG :: 1`
- [OS == .MACOS && CPU == .X64] `_V6_ILP32_OFF32 :: __ILP32_OFF32`
- [OS == .MACOS && CPU == .X64] `_V6_ILP32_OFFBIG :: __ILP32_OFFBIG`
- [OS == .MACOS && CPU == .X64] `_V6_LP64_OFF64 :: __LP64_OFF64`
- [OS == .MACOS && CPU == .X64] `_V6_LPBIG_OFFBIG :: __LPBIG_OFFBIG`
- [OS == .MACOS && CPU == .X64] `_XBS5_ILP32_OFF32 :: __ILP32_OFF32`
- [OS == .MACOS && CPU == .X64] `_XBS5_ILP32_OFFBIG :: __ILP32_OFFBIG`
- [OS == .MACOS && CPU == .X64] `_XBS5_LP64_OFF64 :: __LP64_OFF64`
- [OS == .MACOS && CPU == .X64] `_XBS5_LPBIG_OFFBIG :: __LPBIG_OFFBIG`
- [OS == .MACOS && CPU == .X64] `_XOPEN_CRYPT :: 1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_ENH_I18N :: 1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_LEGACY :: -1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_REALTIME :: -1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_REALTIME_THREADS :: -1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_SHM :: 1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_STREAMS :: -1`
- [OS == .MACOS && CPU == .X64] `_XOPEN_UNIX :: 1`
- [OS == .MACOS && CPU == .X64] `using _SC_definitions :: enum s32`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_ILP32_OFF32_CFLAGS :: 2`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_ILP32_OFF32_LDFLAGS :: 3`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_ILP32_OFF32_LIBS :: 4`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_ILP32_OFFBIG_CFLAGS :: 5`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_ILP32_OFFBIG_LDFLAGS :: 6`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_ILP32_OFFBIG_LIBS :: 7`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_LP64_OFF64_CFLAGS :: 8`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_LP64_OFF64_LDFLAGS :: 9`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_LP64_OFF64_LIBS :: 10`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_LPBIG_OFFBIG_CFLAGS :: 11`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_LPBIG_OFFBIG_LDFLAGS :: 12`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_LPBIG_OFFBIG_LIBS :: 13`
- [OS == .MACOS && CPU == .X64] `_CS_POSIX_V6_WIDTH_RESTRICTED_ENVS :: 14`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFF32_CFLAGS :: 20`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFF32_LDFLAGS :: 21`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFF32_LIBS :: 22`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFF32_LINTFLAGS :: 23`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFFBIG_CFLAGS :: 24`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFFBIG_LDFLAGS :: 25`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFFBIG_LIBS :: 26`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_ILP32_OFFBIG_LINTFLAGS :: 27`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LP64_OFF64_CFLAGS :: 28`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LP64_OFF64_LDFLAGS :: 29`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LP64_OFF64_LIBS :: 30`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LP64_OFF64_LINTFLAGS :: 31`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LPBIG_OFFBIG_CFLAGS :: 32`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LPBIG_OFFBIG_LDFLAGS :: 33`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LPBIG_OFFBIG_LIBS :: 34`
- [OS == .MACOS && CPU == .X64] `_CS_XBS5_LPBIG_OFFBIG_LINTFLAGS :: 35`
- [OS == .MACOS && CPU == .X64] `_CS_DARWIN_USER_DIR :: 65536`
- [OS == .MACOS && CPU == .X64] `_CS_DARWIN_USER_TEMP_DIR :: 65537`
- [OS == .MACOS && CPU == .X64] `_CS_DARWIN_USER_CACHE_DIR :: 65538`
- [OS == .MACOS && CPU == .X64] `F_ULOCK :: 0`
- [OS == .MACOS && CPU == .X64] `F_LOCK :: 1`
- [OS == .MACOS && CPU == .X64] `F_TLOCK :: 2`
- [OS == .MACOS && CPU == .X64] `F_TEST :: 3`
- [OS == .MACOS && CPU == .X64] `SYNC_VOLUME_FULLSYNC :: 0x01`
- [OS == .MACOS && CPU == .X64] `SYNC_VOLUME_WAIT :: 0x02`
- [OS == .MACOS && CPU == .X64] `FSOPT_NOFOLLOW :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `FSOPT_NOINMEMUPDATE :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `FSOPT_REPORT_FULLSIZE :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `FSOPT_PACK_INVAL_ATTRS :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `FSOPT_ATTR_CMN_EXTENDED :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `SEARCHFS_MAX_SEARCHPARMS :: 4096`
- [OS == .MACOS && CPU == .X64] `ATTR_BIT_MAP_COUNT :: 5`
- [OS == .MACOS && CPU == .X64] `VOL_CAPABILITIES_FORMAT :: 0`
- [OS == .MACOS && CPU == .X64] `VOL_CAPABILITIES_INTERFACES :: 1`
- [OS == .MACOS && CPU == .X64] `VOL_CAPABILITIES_RESERVED1 :: 2`
- [OS == .MACOS && CPU == .X64] `VOL_CAPABILITIES_RESERVED2 :: 3`
- [OS == .MACOS && CPU == .X64] `ATTR_MAX_BUFFER :: 8192`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_PERSISTENTOBJECTIDS :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_SYMBOLICLINKS :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_HARDLINKS :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_JOURNAL :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_JOURNAL_ACTIVE :: 0x00000010`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_NO_ROOT_TIMES :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_SPARSE_FILES :: 0x00000040`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_ZERO_RUNS :: 0x00000080`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_CASE_SENSITIVE :: 0x00000100`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_CASE_PRESERVING :: 0x00000200`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_FAST_STATFS :: 0x00000400`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_2TB_FILESIZE :: 0x00000800`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_OPENDENYMODES :: 0x00001000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_HIDDEN_FILES :: 0x00002000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_PATH_FROM_ID :: 0x00004000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_NO_VOLUME_SIZES :: 0x00008000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_DECMPFS_COMPRESSION :: 0x00010000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_64BIT_OBJECT_IDS :: 0x00020000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_DIR_HARDLINKS :: 0x00040000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_DOCUMENT_ID :: 0x00080000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_WRITE_GENERATION_COUNT :: 0x00100000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_NO_IMMUTABLE_FILES :: 0x00200000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_FMT_NO_PERMISSIONS :: 0x00400000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_SEARCHFS :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_ATTRLIST :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_NFSEXPORT :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_READDIRATTR :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_EXCHANGEDATA :: 0x00000010`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_COPYFILE :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_ALLOCATE :: 0x00000040`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_VOL_RENAME :: 0x00000080`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_ADVLOCK :: 0x00000100`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_FLOCK :: 0x00000200`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_EXTENDED_SECURITY :: 0x00000400`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_USERACCESS :: 0x00000800`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_MANLOCK :: 0x00001000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_NAMEDSTREAMS :: 0x00002000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_EXTENDED_ATTR :: 0x00004000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_CLONE :: 0x00010000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_SNAPSHOT :: 0x00020000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_RENAME_SWAP :: 0x00040000`
- [OS == .MACOS && CPU == .X64] `VOL_CAP_INT_RENAME_EXCL :: 0x00080000`
- [OS == .MACOS && CPU == .X64] `ATTR_CMN :: enum u32`
- [OS == .MACOS && CPU == .X64] `ATTR_VOL :: enum`
- [OS == .MACOS && CPU == .X64] `ATTR_DIR :: enum`
- [OS == .MACOS && CPU == .X64] `DIR_MNTSTATUS_MNTPOINT :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `DIR_MNTSTATUS_TRIGGER :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `ATTR_FILE :: enum`
- [OS == .MACOS && CPU == .X64] `ATTR_FORK :: enum`
- [OS == .MACOS && CPU == .X64] `ATTR_BULK_REQUIRED :: xx ATTR_CMN.NAME | xx ATTR_CMN.RETURNED_ATTRS`
- [OS == .MACOS && CPU == .X64] `SRCHFS_START :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `SRCHFS_MATCHPARTIALNAMES :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `SRCHFS_MATCHDIRS :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `SRCHFS_MATCHFILES :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `SRCHFS_SKIPLINKS :: 0x00000010`
- [OS == .MACOS && CPU == .X64] `SRCHFS_SKIPINVISIBLE :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `SRCHFS_SKIPPACKAGES :: 0x00000040`
- [OS == .MACOS && CPU == .X64] `SRCHFS_SKIPINAPPROPRIATE :: 0x00000080`
- [OS == .MACOS && CPU == .X64] `SRCHFS_NEGATEPARAMS :: 0x80000000`
- [OS == .MACOS && CPU == .X64] `SRCHFS_VALIDOPTIONSMASK :: 0x800000FF`
- [OS == .MACOS && CPU == .X64] `FST_EOF :: -1`
- [OS == .MACOS && CPU == .X64] `VNODE_READ :: 0x01`
- [OS == .MACOS && CPU == .X64] `VNODE_WRITE :: 0x02`
- [OS == .MACOS && CPU == .X64] `VNODE_BLOCKMAP_NO_TRACK :: 0x04`
- [OS == .MACOS && CPU == .X64] `PREALLOCATE :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `ALLOCATECONTIG :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `ALLOCATEALL :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `FREEREMAINDER :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `ALLOCATEFROMPEOF :: 0x00000010`
- [OS == .MACOS && CPU == .X64] `ALLOCATEFROMVOL :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `VNOVAL :: -1`
- [OS == .MACOS && CPU == .X64] `dl_info :: struct`
- [OS == .MACOS && CPU == .X64] `Dl_info :: dl_info`
- [OS == .MACOS && CPU == .X64] `dladdr :: (unknown0: *void, unknown1: *Dl_info) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dlclose :: (__handle: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dlerror :: () -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dlopen :: (__path: *u8, __mode: s32) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `dlsym :: (__handle: *void, __symbol: *u8) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `dlopen_preflight :: (__path: *u8) -> bool #foreign libc`
- [OS == .MACOS && CPU == .X64] `ino_t :: u64`
- [OS == .MACOS && CPU == .X64] `dirent :: struct`
- [OS == .MACOS && CPU == .X64] `_telldir :: struct`
- [OS == .MACOS && CPU == .X64] `DIR :: struct`
- [OS == .MACOS && CPU == .X64] `closedir :: (unknown0: *DIR) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `opendir :: (unknown0: *u8) -> *DIR #foreign libc "opendir$INODE64"`
- [OS == .MACOS && CPU == .X64] `readdir :: (unknown0: *DIR) -> *dirent #foreign libc "readdir$INODE64"`
- [OS == .MACOS && CPU == .X64] `readdir_r :: (unknown0: *DIR, unknown1: *dirent, unknown2: **dirent) -> s32 #foreign libc "readdir_r$INODE64"`
- [OS == .MACOS && CPU == .X64] `rewinddir :: (unknown0: *DIR) -> void #foreign libc "rewinddir$INODE64"`
- [OS == .MACOS && CPU == .X64] `seekdir :: (unknown0: *DIR, unknown1: s64) -> void #foreign libc "seekdir$INODE64"`
- [OS == .MACOS && CPU == .X64] `telldir :: (unknown0: *DIR) -> s64 #foreign libc "telldir$INODE64"`
- [OS == .MACOS && CPU == .X64] `fdopendir :: (unknown0: s32) -> *DIR #foreign libc "fdopendir$INODE64"`
- [OS == .MACOS && CPU == .X64] `alphasort :: (unknown0: **dirent, unknown1: **dirent) -> s32 #foreign libc "alphasort$INODE64"`
- [OS == .MACOS && CPU == .X64] `dirfd :: (dirp: *DIR) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `scandir :: (unknown0: *u8, unknown1: ***dirent, unknown2: #type (unknown0: *dirent) -> s32 #c_call, unknown3: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc "scandir$INODE64"`
- [OS == .MACOS && CPU == .X64] `__opendir2 :: (unknown0: *u8, unknown1: s32) -> *DIR #foreign libc "__opendir2$INODE64"`
- [OS == .MACOS && CPU == .X64] `__errno_location :: () -> *OS_Error_Code #foreign libc "__error"`
- [OS == .MACOS && CPU == .X64] `backtrace :: (unknown0: **void, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `backtrace_symbols :: (unknown0: **void, unknown1: s32) -> **u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `backtrace_symbols_fd :: (unknown0: **void, unknown1: s32, unknown2: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `fnmatch :: (unknown0: *u8, unknown1: *u8, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `gid_t :: u32`
- [OS == .MACOS && CPU == .X64] `uid_t :: u32`
- [OS == .MACOS && CPU == .X64] `passwd :: struct`
- [OS == .MACOS && CPU == .X64] `getpwuid :: (unknown0: uid_t) -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpwnam :: (unknown0: *u8) -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpwuid_r :: (unknown0: uid_t, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpwnam_r :: (unknown0: *u8, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpwent :: () -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .X64] `setpwent :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `endpwent :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `uuid_t :: [16] u8`
- [OS == .MACOS && CPU == .X64] `setpassent :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `user_from_uid :: (unknown0: uid_t, unknown1: s32) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpwuuid :: (unknown0: *uuid_t) -> *passwd #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpwuuid_r :: (unknown0: *uuid_t, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sig_atomic_t :: s32`
- [OS == .MACOS && CPU == .X64] `__darwin_fp_control :: struct`
- [OS == .MACOS && CPU == .X64] `set___invalid :: (fields: *__darwin_fp_control, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___invalid :: (fields: __darwin_fp_control) -> bool`
- [OS == .MACOS && CPU == .X64] `set___denorm :: (fields: *__darwin_fp_control, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___denorm :: (fields: __darwin_fp_control) -> bool`
- [OS == .MACOS && CPU == .X64] `set___zdiv :: (fields: *__darwin_fp_control, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___zdiv :: (fields: __darwin_fp_control) -> bool`
- [OS == .MACOS && CPU == .X64] `set___ovrfl :: (fields: *__darwin_fp_control, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___ovrfl :: (fields: __darwin_fp_control) -> bool`
- [OS == .MACOS && CPU == .X64] `set___undfl :: (fields: *__darwin_fp_control, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___undfl :: (fields: __darwin_fp_control) -> bool`
- [OS == .MACOS && CPU == .X64] `set___precis :: (fields: *__darwin_fp_control, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___precis :: (fields: __darwin_fp_control) -> bool`
- [OS == .MACOS && CPU == .X64] `set___pc :: (fields: *__darwin_fp_control, value: u16)`
- [OS == .MACOS && CPU == .X64] `get___pc :: (fields: __darwin_fp_control) -> u16`
- [OS == .MACOS && CPU == .X64] `set___rc :: (fields: *__darwin_fp_control, value: u16)`
- [OS == .MACOS && CPU == .X64] `get___rc :: (fields: __darwin_fp_control) -> u16`
- [OS == .MACOS && CPU == .X64] `__darwin_fp_status :: struct`
- [OS == .MACOS && CPU == .X64] `set___invalid :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___invalid :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___denorm :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___denorm :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___zdiv :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___zdiv :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___ovrfl :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___ovrfl :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___undfl :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___undfl :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___precis :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___precis :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___stkflt :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___stkflt :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___errsumm :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___errsumm :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___c0 :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___c0 :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___c1 :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___c1 :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___c2 :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___c2 :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___tos :: (fields: *__darwin_fp_status, value: u16)`
- [OS == .MACOS && CPU == .X64] `get___tos :: (fields: __darwin_fp_status) -> u16`
- [OS == .MACOS && CPU == .X64] `set___c3 :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___c3 :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `set___busy :: (fields: *__darwin_fp_status, value: bool)`
- [OS == .MACOS && CPU == .X64] `get___busy :: (fields: __darwin_fp_status) -> bool`
- [OS == .MACOS && CPU == .X64] `__darwin_mmst_reg :: struct`
- [OS == .MACOS && CPU == .X64] `__darwin_xmm_reg :: struct`
- [OS == .MACOS && CPU == .X64] `__darwin_x86_thread_state64 :: struct`
- [OS == .MACOS && CPU == .X64] `__darwin_x86_float_state64 :: struct`
- [OS == .MACOS && CPU == .X64] `__darwin_x86_exception_state64 :: struct`
- [OS == .MACOS && CPU == .X64] `__darwin_mcontext64 :: struct`
- [OS == .MACOS && CPU == .X64] `stack_t :: struct`
- [OS == .MACOS && CPU == .X64] `ucontext_t :: struct`
- [OS == .MACOS && CPU == .X64] `pid_t :: s32`
- [OS == .MACOS && CPU == .X64] `sigset_t :: u32`
- [OS == .MACOS && CPU == .X64] `sigval :: struct`
- [OS == .MACOS && CPU == .X64] `sigevent :: struct`
- [OS == .MACOS && CPU == .X64] `__siginfo :: struct`
- [OS == .MACOS && CPU == .X64] `siginfo_t :: __siginfo`
- [OS == .MACOS && CPU == .X64] `__sigaction_u :: struct`
- [OS == .MACOS && CPU == .X64] `__sigaction :: struct`
- [OS == .MACOS && CPU == .X64] `sigaction_t :: struct`
- [OS == .MACOS && CPU == .X64] `sig_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .MACOS && CPU == .X64] `sigvec :: struct`
- [OS == .MACOS && CPU == .X64] `signal :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc`
- [OS == .MACOS && CPU == .X64] `raise :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `bsd_signal :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc`
- [OS == .MACOS && CPU == .X64] `kill :: (unknown0: pid_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `killpg :: (unknown0: pid_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigaction :: (unknown0: s32, unknown1: *sigaction_t, unknown2: *sigaction_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigaddset :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigaltstack :: (unknown0: *stack_t, unknown1: *stack_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigdelset :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigemptyset :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigfillset :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sighold :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigignore :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `siginterrupt :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigismember :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigpause :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigpending :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigprocmask :: (unknown0: s32, unknown1: *sigset_t, unknown2: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigrelse :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigset :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigsuspend :: (unknown0: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigwait :: (unknown0: *sigset_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `psignal :: (unknown0: u32, unknown1: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigblock :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigsetmask :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigvec_ :: (unknown0: s32, unknown1: *sigvec, unknown2: *sigvec) -> s32 #foreign libc "sigvec"`
- [OS == .MACOS && CPU == .X64] `mode_t :: u16`
- [OS == .MACOS && CPU == .X64] `key_t :: s32`
- [OS == .MACOS && CPU == .X64] `ipc_perm :: struct`
- [OS == .MACOS && CPU == .X64] `ftok :: (unknown0: *u8, unknown1: s32) -> key_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `mlockall :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `munlockall :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mlock :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mmap :: (unknown0: *void, unknown1: u64, unknown2: s32, unknown3: s32, unknown4: s32, unknown5: s64) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `mprotect :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `msync :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `munlock :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `munmap :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `shm_open :: (unknown0: *u8, unknown1: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `shm_unlink :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `posix_madvise :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `madvise :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mincore :: (unknown0: *void, unknown1: u64, unknown2: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `minherit :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pollfd :: struct`
- [OS == .MACOS && CPU == .X64] `nfds_t :: u32`
- [OS == .MACOS && CPU == .X64] `poll :: (unknown0: *pollfd, unknown1: nfds_t, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `time_t :: s64`
- [OS == .MACOS && CPU == .X64] `shmatt_t :: u16`
- [OS == .MACOS && CPU == .X64] `__shmid_ds_new :: struct`
- [OS == .MACOS && CPU == .X64] `shmsys :: (unknown0: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `shmat :: (unknown0: s32, unknown1: *void, unknown2: s32) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `shmctl :: (unknown0: s32, unknown1: s32, unknown2: *__shmid_ds_new) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `shmdt :: (unknown0: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `shmget :: (unknown0: key_t, unknown1: u64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `timespec :: struct`
- [OS == .MACOS && CPU == .X64] `timeval :: struct`
- [OS == .MACOS && CPU == .X64] `suseconds_t :: s32`
- [OS == .MACOS && CPU == .X64] `itimerval :: struct`
- [OS == .MACOS && CPU == .X64] `timezone :: struct`
- [OS == .MACOS && CPU == .X64] `clockinfo :: struct`
- [OS == .MACOS && CPU == .X64] `clock_t :: u64`
- [OS == .MACOS && CPU == .X64] `tm :: struct`
- [OS == .MACOS && CPU == .X64] `tzname: **u8 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `getdate_err: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `daylight: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `asctime :: (unknown0: *tm) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `clock :: () -> clock_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `ctime :: (unknown0: *time_t) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `difftime :: (unknown0: time_t, unknown1: time_t) -> float64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getdate :: (unknown0: *u8) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .X64] `gmtime :: (unknown0: *time_t) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .X64] `localtime :: (unknown0: *time_t) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .X64] `mktime :: (unknown0: *tm) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `strftime :: (unknown0: *u8, unknown1: u64, unknown2: *u8, unknown3: *tm) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `strptime :: (unknown0: *u8, unknown1: *u8, unknown2: *tm) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `time :: (unknown0: *time_t) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `tzset :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `asctime_r :: (unknown0: *tm, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ctime_r :: (unknown0: *time_t, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `gmtime_r :: (unknown0: *time_t, unknown1: *tm) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .X64] `localtime_r :: (unknown0: *time_t, unknown1: *tm) -> *tm #foreign libc`
- [OS == .MACOS && CPU == .X64] `posix2time :: (unknown0: time_t) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `tzsetwall :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `time2posix :: (unknown0: time_t) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `timelocal :: (unknown0: *tm) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `timegm :: (unknown0: *tm) -> time_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `nanosleep :: (__rqtp: *timespec, __rmtp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `clockid_t :: enum u32`
- [OS == .MACOS && CPU == .X64] `clock_getres :: (__clock_id: clockid_t, __res: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `clock_gettime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `clock_gettime_nsec_np :: (__clock_id: clockid_t) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `clock_settime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `adjtime :: (unknown0: *timeval, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `futimes :: (unknown0: s32, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lutimes :: (unknown0: *u8, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `settimeofday :: (unknown0: *timeval, unknown1: *timezone) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getitimer :: (unknown0: s32, unknown1: *itimerval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `gettimeofday :: (unknown0: *timeval, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setitimer :: (unknown0: s32, unknown1: *itimerval, unknown2: *itimerval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `utimes :: (unknown0: *u8, unknown1: *timeval) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dev_t :: s32`
- [OS == .MACOS && CPU == .X64] `fixpt_t :: u32`
- [OS == .MACOS && CPU == .X64] `blkcnt_t :: s64`
- [OS == .MACOS && CPU == .X64] `blksize_t :: s32`
- [OS == .MACOS && CPU == .X64] `nlink_t :: u16`
- [OS == .MACOS && CPU == .X64] `segsz_t :: s32`
- [OS == .MACOS && CPU == .X64] `useconds_t :: u32`
- [OS == .MACOS && CPU == .X64] `utsname :: struct`
- [OS == .MACOS && CPU == .X64] `uname :: (unknown0: *utsname) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `idtype_t :: enum u32`
- [OS == .MACOS && CPU == .X64] `wait_t :: struct`
- [OS == .MACOS && CPU == .X64] `set_w_T_w_Termsig :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .X64] `get_w_T_w_Termsig :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .X64] `set_w_T_w_Coredump :: (fields: *wait_t, value: bool)`
- [OS == .MACOS && CPU == .X64] `get_w_T_w_Coredump :: (fields: wait_t) -> bool`
- [OS == .MACOS && CPU == .X64] `set_w_T_w_Retcode :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .X64] `get_w_T_w_Retcode :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .X64] `set_w_T_w_Filler :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .X64] `get_w_T_w_Filler :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .X64] `set_w_S_w_Stopval :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .X64] `get_w_S_w_Stopval :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .X64] `set_w_S_w_Stopsig :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .X64] `get_w_S_w_Stopsig :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .X64] `set_w_S_w_Filler :: (fields: *wait_t, value: u32)`
- [OS == .MACOS && CPU == .X64] `get_w_S_w_Filler :: (fields: wait_t) -> u32`
- [OS == .MACOS && CPU == .X64] `wait :: (unknown0: *s32) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `waitpid :: (unknown0: pid_t, unknown1: *s32, unknown2: s32) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `waitid :: (unknown0: idtype_t, unknown1: id_t, unknown2: *siginfo_t, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `wait3 :: (unknown0: *s32, unknown1: s32, unknown2: *rusage) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `wait4 :: (unknown0: pid_t, unknown1: *s32, unknown2: s32, unknown3: *rusage) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `accessx_descriptor :: struct`
- [OS == .MACOS && CPU == .X64] `getattrlistbulk :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getattrlistat :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: *void, unknown4: u64, unknown5: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setattrlistat :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: *void, unknown4: u64, unknown5: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `faccessat :: (unknown0: s32, unknown1: *u8, unknown2: s32, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchownat :: (unknown0: s32, unknown1: *u8, unknown2: uid_t, unknown3: gid_t, unknown4: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `linkat :: (unknown0: s32, unknown1: *u8, unknown2: s32, unknown3: *u8, unknown4: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `readlinkat :: (unknown0: s32, unknown1: *u8, unknown2: *u8, unknown3: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `symlinkat :: (unknown0: *u8, unknown1: s32, unknown2: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `unlinkat :: (unknown0: s32, unknown1: *u8, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `_exit :: (unknown0: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `access :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `alarm :: (unknown0: u32) -> u32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `chdir :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `chown :: (unknown0: *u8, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `close :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dup :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dup2 :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `execl :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `execle :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `execlp :: (__file: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `execve :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fork :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `fpathconf :: (unknown0: s32, __name: PC) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getcwd :: (unknown0: *u8, unknown1: u64) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getegid :: () -> gid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `geteuid :: () -> uid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getgid :: () -> gid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getgroups :: (unknown0: s32, unknown1: *gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getlogin :: () -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpgrp :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpid :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getppid :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getuid :: () -> uid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `isatty :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `link :: (unknown0: *u8, unknown1: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lseek :: (unknown0: s32, unknown1: s64, unknown2: s32) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pathconf :: (unknown0: *u8, __name: PC) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pipe :: (unknown0: *[2] s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `read :: (unknown0: s32, unknown1: *void, unknown2: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `rmdir :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setgid :: (unknown0: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setpgid :: (unknown0: pid_t, unknown1: pid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setsid :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `setuid :: (unknown0: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sleep :: (unknown0: u32) -> u32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sysconf :: (name: _SC_definitions) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `tcgetpgrp :: (unknown0: s32) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `tcsetpgrp :: (unknown0: s32, unknown1: pid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ttyname :: (unknown0: s32) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ttyname_r :: (unknown0: s32, unknown1: *u8, unknown2: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `unlink :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `write :: (__fd: s32, __buf: *void, __nbyte: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `confstr :: (unknown0: s32, unknown1: *u8, unknown2: u64) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getopt :: (unknown0: s32, unknown1: **u8, unknown2: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `optarg: *u8 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `optind: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `opterr: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `optopt: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `brk :: (unknown0: *void) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `chroot :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `crypt :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `encrypt :: (unknown0: *u8, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchdir :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `gethostid :: () -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpgid :: (unknown0: pid_t) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getsid :: (unknown0: pid_t) -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getdtablesize :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpagesize :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpass :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getwd :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lchown :: (unknown0: *u8, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lockf :: (unknown0: s32, unknown1: s32, unknown2: s64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `nice :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pread :: (__fd: s32, __buf: *void, __nbyte: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pwrite :: (__fd: s32, __buf: *void, __nbyte: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sbrk :: (unknown0: s32) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `setpgrp :: () -> pid_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `setregid :: (unknown0: gid_t, unknown1: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setreuid :: (unknown0: uid_t, unknown1: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `swab :: (unknown0: *void, unknown1: *void, unknown2: s64) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `sync :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `truncate :: (unknown0: *u8, unknown1: s64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ualarm :: (unknown0: useconds_t, unknown1: useconds_t) -> useconds_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `usleep :: (unknown0: useconds_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `vfork :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fsync :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ftruncate :: (unknown0: s32, unknown1: s64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getlogin_r :: (unknown0: *u8, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchown :: (unknown0: s32, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `gethostname :: (unknown0: *u8, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `readlink :: (unknown0: *u8, unknown1: *u8, unknown2: u64) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setegid :: (unknown0: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `seteuid :: (unknown0: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `symlink :: (unknown0: *u8, unknown1: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `_Exit :: (unknown0: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `acct :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `endusershell :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `execvP :: (__file: *u8, __searchpath: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fflagstostr :: (unknown0: u64) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getdomainname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getgrouplist :: (unknown0: *u8, unknown1: s32, unknown2: *s32, unknown3: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getmode :: (unknown0: *void, unknown1: mode_t) -> mode_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `getpeereid :: (unknown0: s32, unknown1: *uid_t, unknown2: *gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getsgroups_np :: (unknown0: *s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getusershell :: () -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getwgroups_np :: (unknown0: *s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `initgroups :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `issetugid :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `nfssvc :: (unknown0: s32, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `reboot :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `revoke :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `rcmd :: (unknown0: **u8, unknown1: s32, unknown2: *u8, unknown3: *u8, unknown4: *u8, unknown5: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `rcmd_af :: (unknown0: **u8, unknown1: s32, unknown2: *u8, unknown3: *u8, unknown4: *u8, unknown5: *s32, unknown6: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `rresvport :: (unknown0: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `rresvport_af :: (unknown0: *s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `iruserok :: (unknown0: u64, unknown1: s32, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `iruserok_sa :: (unknown0: *void, unknown1: s32, unknown2: s32, unknown3: *u8, unknown4: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ruserok :: (unknown0: *u8, unknown1: s32, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setdomainname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setgroups :: (unknown0: s32, unknown1: *gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sethostid :: (unknown0: s64) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `sethostname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setkey :: (unknown0: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `setlogin :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setmode :: (unknown0: *u8) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `setrgid :: (unknown0: gid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setruid :: (unknown0: uid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setsgroups_np :: (unknown0: s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setusershell :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `setwgroups_np :: (unknown0: s32, unknown1: *uuid_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `strtofflags :: (unknown0: **u8, unknown1: *u64, unknown2: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `swapon :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ttyslot :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `undelete :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `valloc :: (unknown0: u64) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `suboptarg: *u8 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `getsubopt :: (unknown0: **u8, unknown1: **u8, unknown2: **u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fgetattrlist :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fsetattrlist :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getattrlist :: (unknown0: *u8, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setattrlist :: (unknown0: *u8, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `exchangedata :: (unknown0: *u8, unknown1: *u8, unknown2: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getdirentriesattr :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: *u32, unknown5: *u32, unknown6: *u32, unknown7: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `searchfs :: (unknown0: *u8, unknown1: *fssearchblock, unknown2: *u64, unknown3: u32, unknown4: u32, unknown5: *searchstate) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fsctl :: (unknown0: *u8, unknown1: u64, unknown2: *void, unknown3: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ffsctl :: (unknown0: s32, unknown1: u64, unknown2: *void, unknown3: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fsync_volume_np :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sync_volume_np :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `optreset: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `mach_port_t :: u32`
- [OS == .MACOS && CPU == .X64] `text_encoding_t :: u32`
- [OS == .MACOS && CPU == .X64] `fsobj_type_t :: u32`
- [OS == .MACOS && CPU == .X64] `fsobj_tag_t :: u32`
- [OS == .MACOS && CPU == .X64] `fsfile_type_t :: u32`
- [OS == .MACOS && CPU == .X64] `fsvolid_t :: u32`
- [OS == .MACOS && CPU == .X64] `attrgroup_t :: u32`
- [OS == .MACOS && CPU == .X64] `attrlist :: struct`
- [OS == .MACOS && CPU == .X64] `attribute_set :: struct`
- [OS == .MACOS && CPU == .X64] `attribute_set_t :: attribute_set`
- [OS == .MACOS && CPU == .X64] `attrreference :: struct`
- [OS == .MACOS && CPU == .X64] `attrreference_t :: attrreference`
- [OS == .MACOS && CPU == .X64] `diskextent :: struct`
- [OS == .MACOS && CPU == .X64] `extentrecord :: [8] diskextent`
- [OS == .MACOS && CPU == .X64] `vol_capabilities_set_t :: [4] u32`
- [OS == .MACOS && CPU == .X64] `vol_capabilities_attr :: struct`
- [OS == .MACOS && CPU == .X64] `vol_capabilities_attr_t :: vol_capabilities_attr`
- [OS == .MACOS && CPU == .X64] `vol_attributes_attr :: struct`
- [OS == .MACOS && CPU == .X64] `vol_attributes_attr_t :: vol_attributes_attr`
- [OS == .MACOS && CPU == .X64] `fssearchblock :: struct`
- [OS == .MACOS && CPU == .X64] `searchstate :: struct`
- [OS == .MACOS && CPU == .X64] `iovec :: struct`
- [OS == .MACOS && CPU == .X64] `vtype :: enum u32`
- [OS == .MACOS && CPU == .X64] `vtagtype :: enum u32`
- [OS == .MACOS && CPU == .X64] `__SLBF :: 0x0001`
- [OS == .MACOS && CPU == .X64] `__SNBF :: 0x0002`
- [OS == .MACOS && CPU == .X64] `__SRD :: 0x0004`
- [OS == .MACOS && CPU == .X64] `__SWR :: 0x0008`
- [OS == .MACOS && CPU == .X64] `__SRW :: 0x0010`
- [OS == .MACOS && CPU == .X64] `__SEOF :: 0x0020`
- [OS == .MACOS && CPU == .X64] `__SERR :: 0x0040`
- [OS == .MACOS && CPU == .X64] `__SMBF :: 0x0080`
- [OS == .MACOS && CPU == .X64] `__SAPP :: 0x0100`
- [OS == .MACOS && CPU == .X64] `__SSTR :: 0x0200`
- [OS == .MACOS && CPU == .X64] `__SOPT :: 0x0400`
- [OS == .MACOS && CPU == .X64] `__SNPT :: 0x0800`
- [OS == .MACOS && CPU == .X64] `__SOFF :: 0x1000`
- [OS == .MACOS && CPU == .X64] `__SMOD :: 0x2000`
- [OS == .MACOS && CPU == .X64] `__SALC :: 0x4000`
- [OS == .MACOS && CPU == .X64] `__SIGN :: 0x8000`
- [OS == .MACOS && CPU == .X64] `_IOFBF :: 0`
- [OS == .MACOS && CPU == .X64] `_IOLBF :: 1`
- [OS == .MACOS && CPU == .X64] `_IONBF :: 2`
- [OS == .MACOS && CPU == .X64] `BUFSIZ :: 1024`
- [OS == .MACOS && CPU == .X64] `EOF :: -1`
- [OS == .MACOS && CPU == .X64] `FOPEN_MAX :: 20`
- [OS == .MACOS && CPU == .X64] `FILENAME_MAX :: 1024`
- [OS == .MACOS && CPU == .X64] `P_tmpdir :: "/var/tmp/"`
- [OS == .MACOS && CPU == .X64] `L_tmpnam :: 1024`
- [OS == .MACOS && CPU == .X64] `TMP_MAX :: 308915776`
- [OS == .MACOS && CPU == .X64] `SEEK_SET :: 0`
- [OS == .MACOS && CPU == .X64] `SEEK_CUR :: 1`
- [OS == .MACOS && CPU == .X64] `SEEK_END :: 2`
- [OS == .MACOS && CPU == .X64] `L_ctermid :: 1024`
- [OS == .MACOS && CPU == .X64] `O_RDONLY :: 0x0000`
- [OS == .MACOS && CPU == .X64] `O_WRONLY :: 0x0001`
- [OS == .MACOS && CPU == .X64] `O_RDWR :: 0x0002`
- [OS == .MACOS && CPU == .X64] `O_ACCMODE :: 0x0003`
- [OS == .MACOS && CPU == .X64] `FREAD :: 0x0001`
- [OS == .MACOS && CPU == .X64] `FWRITE :: 0x0002`
- [OS == .MACOS && CPU == .X64] `O_NONBLOCK :: 0x0004`
- [OS == .MACOS && CPU == .X64] `O_APPEND :: 0x0008`
- [OS == .MACOS && CPU == .X64] `O_SYNC :: 0x0080`
- [OS == .MACOS && CPU == .X64] `O_SHLOCK :: 0x0010`
- [OS == .MACOS && CPU == .X64] `O_EXLOCK :: 0x0020`
- [OS == .MACOS && CPU == .X64] `O_ASYNC :: 0x0040`
- [OS == .MACOS && CPU == .X64] `O_FSYNC :: O_SYNC`
- [OS == .MACOS && CPU == .X64] `O_NOFOLLOW :: 0x0100`
- [OS == .MACOS && CPU == .X64] `O_CREAT :: 0x0200`
- [OS == .MACOS && CPU == .X64] `O_TRUNC :: 0x0400`
- [OS == .MACOS && CPU == .X64] `O_EXCL :: 0x0800`
- [OS == .MACOS && CPU == .X64] `AT_FDCWD :: -2`
- [OS == .MACOS && CPU == .X64] `AT_EACCESS :: 0x0010`
- [OS == .MACOS && CPU == .X64] `AT_SYMLINK_NOFOLLOW :: 0x0020`
- [OS == .MACOS && CPU == .X64] `AT_SYMLINK_FOLLOW :: 0x0040`
- [OS == .MACOS && CPU == .X64] `AT_REMOVEDIR :: 0x0080`
- [OS == .MACOS && CPU == .X64] `O_EVTONLY :: 0x8000`
- [OS == .MACOS && CPU == .X64] `O_NOCTTY :: 0x20000`
- [OS == .MACOS && CPU == .X64] `O_DIRECTORY :: 0x100000`
- [OS == .MACOS && CPU == .X64] `O_SYMLINK :: 0x200000`
- [OS == .MACOS && CPU == .X64] `O_CLOEXEC :: 0x1000000`
- [OS == .MACOS && CPU == .X64] `O_DP_GETRAWENCRYPTED :: 0x0001`
- [OS == .MACOS && CPU == .X64] `O_DP_GETRAWUNENCRYPTED :: 0x0002`
- [OS == .MACOS && CPU == .X64] `FAPPEND :: O_APPEND`
- [OS == .MACOS && CPU == .X64] `FASYNC :: O_ASYNC`
- [OS == .MACOS && CPU == .X64] `FFSYNC :: O_FSYNC`
- [OS == .MACOS && CPU == .X64] `FNONBLOCK :: O_NONBLOCK`
- [OS == .MACOS && CPU == .X64] `FNDELAY :: O_NONBLOCK`
- [OS == .MACOS && CPU == .X64] `O_NDELAY :: O_NONBLOCK`
- [OS == .MACOS && CPU == .X64] `CPF_OVERWRITE :: 0x0001`
- [OS == .MACOS && CPU == .X64] `CPF_IGNORE_MODE :: 0x0002`
- [OS == .MACOS && CPU == .X64] `CPF_MASK :: CPF_OVERWRITE|CPF_IGNORE_MODE`
- [OS == .MACOS && CPU == .X64] `F_DUPFD :: 0`
- [OS == .MACOS && CPU == .X64] `F_GETFD :: 1`
- [OS == .MACOS && CPU == .X64] `F_SETFD :: 2`
- [OS == .MACOS && CPU == .X64] `F_GETFL :: 3`
- [OS == .MACOS && CPU == .X64] `F_SETFL :: 4`
- [OS == .MACOS && CPU == .X64] `F_GETOWN :: 5`
- [OS == .MACOS && CPU == .X64] `F_SETOWN :: 6`
- [OS == .MACOS && CPU == .X64] `F_GETLK :: 7`
- [OS == .MACOS && CPU == .X64] `F_SETLK :: 8`
- [OS == .MACOS && CPU == .X64] `F_SETLKW :: 9`
- [OS == .MACOS && CPU == .X64] `F_SETLKWTIMEOUT :: 10`
- [OS == .MACOS && CPU == .X64] `F_FLUSH_DATA :: 40`
- [OS == .MACOS && CPU == .X64] `F_CHKCLEAN :: 41`
- [OS == .MACOS && CPU == .X64] `F_PREALLOCATE :: 42`
- [OS == .MACOS && CPU == .X64] `F_SETSIZE :: 43`
- [OS == .MACOS && CPU == .X64] `F_RDADVISE :: 44`
- [OS == .MACOS && CPU == .X64] `F_RDAHEAD :: 45`
- [OS == .MACOS && CPU == .X64] `F_NOCACHE :: 48`
- [OS == .MACOS && CPU == .X64] `F_LOG2PHYS :: 49`
- [OS == .MACOS && CPU == .X64] `F_GETPATH :: 50`
- [OS == .MACOS && CPU == .X64] `F_FULLFSYNC :: 51`
- [OS == .MACOS && CPU == .X64] `F_PATHPKG_CHECK :: 52`
- [OS == .MACOS && CPU == .X64] `F_FREEZE_FS :: 53`
- [OS == .MACOS && CPU == .X64] `F_THAW_FS :: 54`
- [OS == .MACOS && CPU == .X64] `F_GLOBAL_NOCACHE :: 55`
- [OS == .MACOS && CPU == .X64] `F_ADDSIGS :: 59`
- [OS == .MACOS && CPU == .X64] `F_ADDFILESIGS :: 61`
- [OS == .MACOS && CPU == .X64] `F_NODIRECT :: 62`
- [OS == .MACOS && CPU == .X64] `F_GETPROTECTIONCLASS :: 63`
- [OS == .MACOS && CPU == .X64] `F_SETPROTECTIONCLASS :: 64`
- [OS == .MACOS && CPU == .X64] `F_LOG2PHYS_EXT :: 65`
- [OS == .MACOS && CPU == .X64] `F_GETLKPID :: 66`
- [OS == .MACOS && CPU == .X64] `F_SETBACKINGSTORE :: 70`
- [OS == .MACOS && CPU == .X64] `F_GETPATH_MTMINFO :: 71`
- [OS == .MACOS && CPU == .X64] `F_GETCODEDIR :: 72`
- [OS == .MACOS && CPU == .X64] `F_SETNOSIGPIPE :: 73`
- [OS == .MACOS && CPU == .X64] `F_GETNOSIGPIPE :: 74`
- [OS == .MACOS && CPU == .X64] `F_TRANSCODEKEY :: 75`
- [OS == .MACOS && CPU == .X64] `F_SINGLE_WRITER :: 76`
- [OS == .MACOS && CPU == .X64] `F_GETPROTECTIONLEVEL :: 77`
- [OS == .MACOS && CPU == .X64] `F_FINDSIGS :: 78`
- [OS == .MACOS && CPU == .X64] `F_ADDFILESIGS_FOR_DYLD_SIM :: 83`
- [OS == .MACOS && CPU == .X64] `F_BARRIERFSYNC :: 85`
- [OS == .MACOS && CPU == .X64] `F_ADDFILESIGS_RETURN :: 97`
- [OS == .MACOS && CPU == .X64] `F_CHECK_LV :: 98`
- [OS == .MACOS && CPU == .X64] `F_PUNCHHOLE :: 99`
- [OS == .MACOS && CPU == .X64] `F_TRIM_ACTIVE_FILE :: 100`
- [OS == .MACOS && CPU == .X64] `FCNTL_FS_SPECIFIC_BASE :: 0x00010000`
- [OS == .MACOS && CPU == .X64] `F_DUPFD_CLOEXEC :: 67`
- [OS == .MACOS && CPU == .X64] `FD_CLOEXEC :: 1`
- [OS == .MACOS && CPU == .X64] `F_RDLCK :: 1`
- [OS == .MACOS && CPU == .X64] `F_UNLCK :: 2`
- [OS == .MACOS && CPU == .X64] `F_WRLCK :: 3`
- [OS == .MACOS && CPU == .X64] `S_IFMT :: 0xf000`
- [OS == .MACOS && CPU == .X64] `S_IFIFO :: 0x1000`
- [OS == .MACOS && CPU == .X64] `S_IFCHR :: 0x2000`
- [OS == .MACOS && CPU == .X64] `S_IFDIR :: 0x4000`
- [OS == .MACOS && CPU == .X64] `S_IFBLK :: 0x6000`
- [OS == .MACOS && CPU == .X64] `S_IFREG :: 0x8000`
- [OS == .MACOS && CPU == .X64] `S_IFLNK :: 0xa000`
- [OS == .MACOS && CPU == .X64] `S_IFSOCK :: 0xc000`
- [OS == .MACOS && CPU == .X64] `S_IFWHT :: 0xe000`
- [OS == .MACOS && CPU == .X64] `S_IRWXU :: 0x1c0`
- [OS == .MACOS && CPU == .X64] `S_IRUSR :: 0x100`
- [OS == .MACOS && CPU == .X64] `S_IWUSR :: 0x80`
- [OS == .MACOS && CPU == .X64] `S_IXUSR :: 0x40`
- [OS == .MACOS && CPU == .X64] `S_IRWXG :: 0x38`
- [OS == .MACOS && CPU == .X64] `S_IRGRP :: 0x20`
- [OS == .MACOS && CPU == .X64] `S_IWGRP :: 0x10`
- [OS == .MACOS && CPU == .X64] `S_IXGRP :: 0x8`
- [OS == .MACOS && CPU == .X64] `S_IRWXO :: 0x7`
- [OS == .MACOS && CPU == .X64] `S_IROTH :: 0x4`
- [OS == .MACOS && CPU == .X64] `S_IWOTH :: 0x2`
- [OS == .MACOS && CPU == .X64] `S_IXOTH :: 0x1`
- [OS == .MACOS && CPU == .X64] `S_ISUID :: 0x800`
- [OS == .MACOS && CPU == .X64] `S_ISGID :: 0x400`
- [OS == .MACOS && CPU == .X64] `S_ISVTX :: 0x200`
- [OS == .MACOS && CPU == .X64] `S_ISTXT :: S_ISVTX`
- [OS == .MACOS && CPU == .X64] `S_IREAD :: S_IRUSR`
- [OS == .MACOS && CPU == .X64] `S_IWRITE :: S_IWUSR`
- [OS == .MACOS && CPU == .X64] `S_IEXEC :: S_IXUSR`
- [OS == .MACOS && CPU == .X64] `F_ALLOCATECONTIG :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `F_ALLOCATEALL :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `F_PEOFPOSMODE :: 3`
- [OS == .MACOS && CPU == .X64] `F_VOLPOSMODE :: 4`
- [OS == .MACOS && CPU == .X64] `LOCK_SH :: 0x01`
- [OS == .MACOS && CPU == .X64] `LOCK_EX :: 0x02`
- [OS == .MACOS && CPU == .X64] `LOCK_NB :: 0x04`
- [OS == .MACOS && CPU == .X64] `LOCK_UN :: 0x08`
- [OS == .MACOS && CPU == .X64] `O_POPUP :: 0x80000000`
- [OS == .MACOS && CPU == .X64] `O_ALERT :: 0x20000000`
- [OS == .MACOS && CPU == .X64] `_FILESEC_UNSET_PROPERTY :: cast(*void) 0`
- [OS == .MACOS && CPU == .X64] `_FILESEC_REMOVE_ACL :: cast(*void) 1`
- [OS == .MACOS && CPU == .X64] `ACCESSPERMS :: S_IRWXU|S_IRWXG|S_IRWXO`
- [OS == .MACOS && CPU == .X64] `ALLPERMS :: S_ISUID|S_ISGID|S_ISTXT|S_IRWXU|S_IRWXG|S_IRWXO`
- [OS == .MACOS && CPU == .X64] `DEFFILEMODE :: S_IRUSR|S_IWUSR|S_IRGRP|S_IWGRP|S_IROTH|S_IWOTH`
- [OS == .MACOS && CPU == .X64] `S_BLKSIZE :: 512`
- [OS == .MACOS && CPU == .X64] `UF_SETTABLE :: 0x0000ffff`
- [OS == .MACOS && CPU == .X64] `UF_NODUMP :: 0x00000001`
- [OS == .MACOS && CPU == .X64] `UF_IMMUTABLE :: 0x00000002`
- [OS == .MACOS && CPU == .X64] `UF_APPEND :: 0x00000004`
- [OS == .MACOS && CPU == .X64] `UF_OPAQUE :: 0x00000008`
- [OS == .MACOS && CPU == .X64] `UF_COMPRESSED :: 0x00000020`
- [OS == .MACOS && CPU == .X64] `UF_TRACKED :: 0x00000040`
- [OS == .MACOS && CPU == .X64] `UF_DATAVAULT :: 0x00000080`
- [OS == .MACOS && CPU == .X64] `UF_HIDDEN :: 0x00008000`
- [OS == .MACOS && CPU == .X64] `SF_SUPPORTED :: 0x001f0000`
- [OS == .MACOS && CPU == .X64] `SF_SETTABLE :: 0xffff0000`
- [OS == .MACOS && CPU == .X64] `SF_ARCHIVED :: 0x00010000`
- [OS == .MACOS && CPU == .X64] `SF_IMMUTABLE :: 0x00020000`
- [OS == .MACOS && CPU == .X64] `SF_APPEND :: 0x00040000`
- [OS == .MACOS && CPU == .X64] `SF_RESTRICTED :: 0x00080000`
- [OS == .MACOS && CPU == .X64] `SF_NOUNLINK :: 0x00100000`
- [OS == .MACOS && CPU == .X64] `UTIME_NOW :: -1`
- [OS == .MACOS && CPU == .X64] `UTIME_OMIT :: -2`
- [OS == .MACOS && CPU == .X64] `AIO_ALLDONE :: 0x1`
- [OS == .MACOS && CPU == .X64] `AIO_CANCELED :: 0x2`
- [OS == .MACOS && CPU == .X64] `AIO_NOTCANCELED :: 0x4`
- [OS == .MACOS && CPU == .X64] `LIO_NOP :: 0x0`
- [OS == .MACOS && CPU == .X64] `LIO_READ :: 0x1`
- [OS == .MACOS && CPU == .X64] `LIO_WRITE :: 0x2`
- [OS == .MACOS && CPU == .X64] `LIO_NOWAIT :: 0x1`
- [OS == .MACOS && CPU == .X64] `LIO_WAIT :: 0x2`
- [OS == .MACOS && CPU == .X64] `AIO_LISTIO_MAX :: 16`
- [OS == .MACOS && CPU == .X64] `fpos_t :: s64`
- [OS == .MACOS && CPU == .X64] `stdin: *FILE #elsewhere libc "__stdinp"`
- [OS == .MACOS && CPU == .X64] `stdout: *FILE #elsewhere libc "__stdoutp"`
- [OS == .MACOS && CPU == .X64] `stderr: *FILE #elsewhere libc "__stderrp"`
- [OS == .MACOS && CPU == .X64] `clearerr :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `fclose :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `feof :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ferror :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fflush :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fgetc :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fgetpos :: (unknown0: *FILE, unknown1: *fpos_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fgets :: (unknown0: *u8, unknown1: s32, unknown2: *FILE) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fopen :: (__filename: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `fprintf :: (unknown0: *FILE, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fputc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fputs :: (unknown0: *u8, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fread :: (__ptr: *void, __size: u64, __nitems: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `freopen :: (unknown0: *u8, unknown1: *u8, unknown2: *FILE) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `fscanf :: (unknown0: *FILE, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fseek :: (unknown0: *FILE, unknown1: s64, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fsetpos :: (unknown0: *FILE, unknown1: *fpos_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ftell :: (unknown0: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fwrite :: (__ptr: *void, __size: u64, __nitems: u64, __stream: *FILE) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getc :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getchar :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `gets :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `perror :: (unknown0: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `printf :: (unknown0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `putc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `putchar :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `puts :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `_remove :: (unknown0: *u8) -> s32 #foreign libc "remove"`
- [OS == .MACOS && CPU == .X64] `rename :: (__old: *u8, __new: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `rewind :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `scanf :: (unknown0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setbuf :: (unknown0: *FILE, unknown1: *u8) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `setvbuf :: (unknown0: *FILE, unknown1: *u8, unknown2: s32, unknown3: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sprintf :: (unknown0: *u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sscanf :: (unknown0: *u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `tmpfile :: () -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `tmpnam :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ungetc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ctermid :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fdopen :: (unknown0: s32, unknown1: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `fileno :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pclose :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `popen :: (unknown0: *u8, unknown1: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `__srget :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `__swbuf :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `flockfile :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `ftrylockfile :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `funlockfile :: (unknown0: *FILE) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `getc_unlocked :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getchar_unlocked :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `putc_unlocked :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `putchar_unlocked :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getw :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `putw :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `tempnam :: (__dir: *u8, __prefix: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fseeko :: (__stream: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ftello :: (__stream: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `snprintf :: (__str: *u8, __size: u64, __format: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `dprintf :: (unknown0: s32, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getdelim :: (__linep: **u8, __linecapp: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getline :: (__linep: **u8, __linecapp: *u64, __stream: *FILE) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fmemopen :: (__buf: *void, __size: u64, __mode: *u8) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `open_memstream :: (__bufp: **u8, __sizep: *u64) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `sys_nerr: s32 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `sys_errlist: **u8 #elsewhere libc`
- [OS == .MACOS && CPU == .X64] `asprintf :: (unknown0: **u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ctermid_r :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fgetln :: (unknown0: *FILE, unknown1: *u64) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fmtcheck :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fpurge :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setbuffer :: (unknown0: *FILE, unknown1: *u8, unknown2: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `setlinebuf :: (unknown0: *FILE) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `funopen :: (unknown0: *void, unknown1: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, unknown2: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, unknown3: #type (unknown0: *void, unknown1: fpos_t, unknown2: s32) -> fpos_t #c_call, unknown4: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc`
- [OS == .MACOS && CPU == .X64] `flock :: struct`
- [OS == .MACOS && CPU == .X64] `flocktimeout :: struct`
- [OS == .MACOS && CPU == .X64] `radvisory :: struct`
- [OS == .MACOS && CPU == .X64] `fcodeblobs :: struct`
- [OS == .MACOS && CPU == .X64] `fcodeblobs_t :: fcodeblobs`
- [OS == .MACOS && CPU == .X64] `fsignatures :: struct`
- [OS == .MACOS && CPU == .X64] `fsignatures_t :: fsignatures`
- [OS == .MACOS && CPU == .X64] `fchecklv :: struct`
- [OS == .MACOS && CPU == .X64] `fchecklv_t :: fchecklv`
- [OS == .MACOS && CPU == .X64] `fstore :: struct`
- [OS == .MACOS && CPU == .X64] `fstore_t :: fstore`
- [OS == .MACOS && CPU == .X64] `fpunchhole :: struct`
- [OS == .MACOS && CPU == .X64] `fpunchhole_t :: fpunchhole`
- [OS == .MACOS && CPU == .X64] `ftrimactivefile :: struct`
- [OS == .MACOS && CPU == .X64] `ftrimactivefile_t :: ftrimactivefile`
- [OS == .MACOS && CPU == .X64] `fbootstraptransfer :: struct`
- [OS == .MACOS && CPU == .X64] `fbootstraptransfer_t :: fbootstraptransfer`
- [OS == .MACOS && CPU == .X64] `log2phys :: struct`
- [OS == .MACOS && CPU == .X64] `_filesec :: struct`
- [OS == .MACOS && CPU == .X64] `filesec_t :: *_filesec`
- [OS == .MACOS && CPU == .X64] `filesec_property_t :: enum u32`
- [OS == .MACOS && CPU == .X64] `open :: (unknown0: *u8, unknown1: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `openat :: (unknown0: s32, unknown1: *u8, unknown2: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `creat :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fcntl :: (unknown0: s32, unknown1: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `openx_np :: (unknown0: *u8, unknown1: s32, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `open_dprotected_np :: (unknown0: *u8, unknown1: s32, unknown2: s32, unknown3: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `flock_ :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc "flock"`
- [OS == .MACOS && CPU == .X64] `filesec_init :: () -> filesec_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `filesec_dup :: (unknown0: filesec_t) -> filesec_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `filesec_free :: (unknown0: filesec_t) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `filesec_get_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `filesec_query_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `filesec_set_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `filesec_unset_property :: (unknown0: filesec_t, unknown1: filesec_property_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ostat :: struct`
- [OS == .MACOS && CPU == .X64] `stat_t :: struct`
- [OS == .MACOS && CPU == .X64] `stat64_t :: struct`
- [OS == .MACOS && CPU == .X64] `chmod :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchmod :: (unknown0: s32, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fstat :: (unknown0: s32, unknown1: *stat_t) -> s32 #foreign libc "fstat$INODE64"`
- [OS == .MACOS && CPU == .X64] `lstat :: (unknown0: *u8, unknown1: *stat_t) -> s32 #foreign libc "lstat$INODE64"`
- [OS == .MACOS && CPU == .X64] `mkdir :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkfifo :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `stat :: (unknown0: *u8, unknown1: *stat_t) -> s32 #foreign libc "stat$INODE64"`
- [OS == .MACOS && CPU == .X64] `mknod :: (unknown0: *u8, unknown1: mode_t, unknown2: dev_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `umask :: (unknown0: mode_t) -> mode_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchmodat :: (unknown0: s32, unknown1: *u8, unknown2: mode_t, unknown3: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fstatat :: (unknown0: s32, unknown1: *u8, unknown2: *stat_t, unknown3: s32) -> s32 #foreign libc "fstatat$INODE64"`
- [OS == .MACOS && CPU == .X64] `mkdirat :: (unknown0: s32, unknown1: *u8, unknown2: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `utimensat :: (__fd: s32, __path: *u8, __times: *[2] timespec, __flag: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `chflags :: (unknown0: *u8, unknown1: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `chmodx_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchflags :: (unknown0: s32, unknown1: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fchmodx_np :: (unknown0: s32, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fstatx_np :: (unknown0: s32, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc "fstatx_np$INODE64"`
- [OS == .MACOS && CPU == .X64] `lchflags :: (unknown0: *u8, unknown1: u32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lchmod :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lstatx_np :: (unknown0: *u8, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc "lstatx_np$INODE64"`
- [OS == .MACOS && CPU == .X64] `mkdirx_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkfifox_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `statx_np :: (unknown0: *u8, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc "statx_np$INODE64"`
- [OS == .MACOS && CPU == .X64] `umaskx_np :: (unknown0: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fstatx64_np :: (unknown0: s32, unknown1: *stat64_t, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lstatx64_np :: (unknown0: *u8, unknown1: *stat64_t, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `statx64_np :: (unknown0: *u8, unknown1: *stat64_t, unknown2: filesec_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `fstat64 :: (unknown0: s32, unknown1: *stat64_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lstat64 :: (unknown0: *u8, unknown1: *stat64_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `stat64 :: (unknown0: *u8, unknown1: *stat64_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ttysize :: struct`
- [OS == .MACOS && CPU == .X64] `ioctl :: (unknown0: s32, unknown1: u64, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aiocb :: struct`
- [OS == .MACOS && CPU == .X64] `aio_cancel :: (fd: s32, aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aio_error :: (aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aio_fsync :: (op: s32, aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aio_read :: (aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aio_return :: (aiocbp: *aiocb) -> s64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aio_suspend :: (aiocblist: **aiocb, nent: s32, timeoutp: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `aio_write :: (aiocbp: *aiocb) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `lio_listio :: (mode: s32, aiocblist: **aiocb, nent: s32, sigp: *sigevent) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `FILE :: struct`
- [OS == .MACOS && CPU == .X64] `ftello64 :: ftello`
- [OS == .MACOS && CPU == .X64] `getenv :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `system :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mktemp :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkstemp :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `realpath :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc "realpath$DARWIN_EXTSN"`
- [OS == .MACOS && CPU == .X64] `setenv :: (__name: *u8, __value: *u8, __overwrite: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `unsetenv :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkdtemp :: (unknown0: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkpath_np :: (path: *u8, omode: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkpathat_np :: (dfd: s32, path: *u8, omode: mode_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkstemps :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkostemp :: (path: *u8, oflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkostemps :: (path: *u8, slen: s32, oflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkstemp_dprotected_np :: (path: *u8, dpclass: s32, dpflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkdtempat_np :: (dfd: s32, path: *u8) -> *u8 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkstempsat_np :: (dfd: s32, path: *u8, slen: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `mkostempsat_np :: (dfd: s32, path: *u8, slen: s32, oflags: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `SYS_syscall :: 0`
- [OS == .MACOS && CPU == .X64] `SYS_exit :: 1`
- [OS == .MACOS && CPU == .X64] `SYS_fork :: 2`
- [OS == .MACOS && CPU == .X64] `SYS_read :: 3`
- [OS == .MACOS && CPU == .X64] `SYS_write :: 4`
- [OS == .MACOS && CPU == .X64] `SYS_open :: 5`
- [OS == .MACOS && CPU == .X64] `SYS_close :: 6`
- [OS == .MACOS && CPU == .X64] `SYS_wait4 :: 7`
- [OS == .MACOS && CPU == .X64] `SYS_link :: 9`
- [OS == .MACOS && CPU == .X64] `SYS_unlink :: 10`
- [OS == .MACOS && CPU == .X64] `SYS_chdir :: 12`
- [OS == .MACOS && CPU == .X64] `SYS_fchdir :: 13`
- [OS == .MACOS && CPU == .X64] `SYS_mknod :: 14`
- [OS == .MACOS && CPU == .X64] `SYS_chmod :: 15`
- [OS == .MACOS && CPU == .X64] `SYS_chown :: 16`
- [OS == .MACOS && CPU == .X64] `SYS_getfsstat :: 18`
- [OS == .MACOS && CPU == .X64] `SYS_getpid :: 20`
- [OS == .MACOS && CPU == .X64] `SYS_setuid :: 23`
- [OS == .MACOS && CPU == .X64] `SYS_getuid :: 24`
- [OS == .MACOS && CPU == .X64] `SYS_geteuid :: 25`
- [OS == .MACOS && CPU == .X64] `SYS_ptrace :: 26`
- [OS == .MACOS && CPU == .X64] `SYS_recvmsg :: 27`
- [OS == .MACOS && CPU == .X64] `SYS_sendmsg :: 28`
- [OS == .MACOS && CPU == .X64] `SYS_recvfrom :: 29`
- [OS == .MACOS && CPU == .X64] `SYS_accept :: 30`
- [OS == .MACOS && CPU == .X64] `SYS_getpeername :: 31`
- [OS == .MACOS && CPU == .X64] `SYS_getsockname :: 32`
- [OS == .MACOS && CPU == .X64] `SYS_access :: 33`
- [OS == .MACOS && CPU == .X64] `SYS_chflags :: 34`
- [OS == .MACOS && CPU == .X64] `SYS_fchflags :: 35`
- [OS == .MACOS && CPU == .X64] `SYS_sync :: 36`
- [OS == .MACOS && CPU == .X64] `SYS_kill :: 37`
- [OS == .MACOS && CPU == .X64] `SYS_getppid :: 39`
- [OS == .MACOS && CPU == .X64] `SYS_dup :: 41`
- [OS == .MACOS && CPU == .X64] `SYS_pipe :: 42`
- [OS == .MACOS && CPU == .X64] `SYS_getegid :: 43`
- [OS == .MACOS && CPU == .X64] `SYS_sigaction :: 46`
- [OS == .MACOS && CPU == .X64] `SYS_getgid :: 47`
- [OS == .MACOS && CPU == .X64] `SYS_sigprocmask :: 48`
- [OS == .MACOS && CPU == .X64] `SYS_getlogin :: 49`
- [OS == .MACOS && CPU == .X64] `SYS_setlogin :: 50`
- [OS == .MACOS && CPU == .X64] `SYS_acct :: 51`
- [OS == .MACOS && CPU == .X64] `SYS_sigpending :: 52`
- [OS == .MACOS && CPU == .X64] `SYS_sigaltstack :: 53`
- [OS == .MACOS && CPU == .X64] `SYS_ioctl :: 54`
- [OS == .MACOS && CPU == .X64] `SYS_reboot :: 55`
- [OS == .MACOS && CPU == .X64] `SYS_revoke :: 56`
- [OS == .MACOS && CPU == .X64] `SYS_symlink :: 57`
- [OS == .MACOS && CPU == .X64] `SYS_readlink :: 58`
- [OS == .MACOS && CPU == .X64] `SYS_execve :: 59`
- [OS == .MACOS && CPU == .X64] `SYS_umask :: 60`
- [OS == .MACOS && CPU == .X64] `SYS_chroot :: 61`
- [OS == .MACOS && CPU == .X64] `SYS_msync :: 65`
- [OS == .MACOS && CPU == .X64] `SYS_vfork :: 66`
- [OS == .MACOS && CPU == .X64] `SYS_munmap :: 73`
- [OS == .MACOS && CPU == .X64] `SYS_mprotect :: 74`
- [OS == .MACOS && CPU == .X64] `SYS_madvise :: 75`
- [OS == .MACOS && CPU == .X64] `SYS_mincore :: 78`
- [OS == .MACOS && CPU == .X64] `SYS_getgroups :: 79`
- [OS == .MACOS && CPU == .X64] `SYS_setgroups :: 80`
- [OS == .MACOS && CPU == .X64] `SYS_getpgrp :: 81`
- [OS == .MACOS && CPU == .X64] `SYS_setpgid :: 82`
- [OS == .MACOS && CPU == .X64] `SYS_setitimer :: 83`
- [OS == .MACOS && CPU == .X64] `SYS_swapon :: 85`
- [OS == .MACOS && CPU == .X64] `SYS_getitimer :: 86`
- [OS == .MACOS && CPU == .X64] `SYS_getdtablesize :: 89`
- [OS == .MACOS && CPU == .X64] `SYS_dup2 :: 90`
- [OS == .MACOS && CPU == .X64] `SYS_fcntl :: 92`
- [OS == .MACOS && CPU == .X64] `SYS_select :: 93`
- [OS == .MACOS && CPU == .X64] `SYS_fsync :: 95`
- [OS == .MACOS && CPU == .X64] `SYS_setpriority :: 96`
- [OS == .MACOS && CPU == .X64] `SYS_socket :: 97`
- [OS == .MACOS && CPU == .X64] `SYS_connect :: 98`
- [OS == .MACOS && CPU == .X64] `SYS_getpriority :: 100`
- [OS == .MACOS && CPU == .X64] `SYS_bind :: 104`
- [OS == .MACOS && CPU == .X64] `SYS_setsockopt :: 105`
- [OS == .MACOS && CPU == .X64] `SYS_listen :: 106`
- [OS == .MACOS && CPU == .X64] `SYS_sigsuspend :: 111`
- [OS == .MACOS && CPU == .X64] `SYS_gettimeofday :: 116`
- [OS == .MACOS && CPU == .X64] `SYS_getrusage :: 117`
- [OS == .MACOS && CPU == .X64] `SYS_getsockopt :: 118`
- [OS == .MACOS && CPU == .X64] `SYS_readv :: 120`
- [OS == .MACOS && CPU == .X64] `SYS_writev :: 121`
- [OS == .MACOS && CPU == .X64] `SYS_settimeofday :: 122`
- [OS == .MACOS && CPU == .X64] `SYS_fchown :: 123`
- [OS == .MACOS && CPU == .X64] `SYS_fchmod :: 124`
- [OS == .MACOS && CPU == .X64] `SYS_setreuid :: 126`
- [OS == .MACOS && CPU == .X64] `SYS_setregid :: 127`
- [OS == .MACOS && CPU == .X64] `SYS_rename :: 128`
- [OS == .MACOS && CPU == .X64] `SYS_flock :: 131`
- [OS == .MACOS && CPU == .X64] `SYS_mkfifo :: 132`
- [OS == .MACOS && CPU == .X64] `SYS_sendto :: 133`
- [OS == .MACOS && CPU == .X64] `SYS_shutdown :: 134`
- [OS == .MACOS && CPU == .X64] `SYS_socketpair :: 135`
- [OS == .MACOS && CPU == .X64] `SYS_mkdir :: 136`
- [OS == .MACOS && CPU == .X64] `SYS_rmdir :: 137`
- [OS == .MACOS && CPU == .X64] `SYS_utimes :: 138`
- [OS == .MACOS && CPU == .X64] `SYS_futimes :: 139`
- [OS == .MACOS && CPU == .X64] `SYS_adjtime :: 140`
- [OS == .MACOS && CPU == .X64] `SYS_gethostuuid :: 142`
- [OS == .MACOS && CPU == .X64] `SYS_setsid :: 147`
- [OS == .MACOS && CPU == .X64] `SYS_getpgid :: 151`
- [OS == .MACOS && CPU == .X64] `SYS_setprivexec :: 152`
- [OS == .MACOS && CPU == .X64] `SYS_pread :: 153`
- [OS == .MACOS && CPU == .X64] `SYS_pwrite :: 154`
- [OS == .MACOS && CPU == .X64] `SYS_nfssvc :: 155`
- [OS == .MACOS && CPU == .X64] `SYS_statfs :: 157`
- [OS == .MACOS && CPU == .X64] `SYS_fstatfs :: 158`
- [OS == .MACOS && CPU == .X64] `SYS_unmount :: 159`
- [OS == .MACOS && CPU == .X64] `SYS_getfh :: 161`
- [OS == .MACOS && CPU == .X64] `SYS_quotactl :: 165`
- [OS == .MACOS && CPU == .X64] `SYS_mount :: 167`
- [OS == .MACOS && CPU == .X64] `SYS_csops :: 169`
- [OS == .MACOS && CPU == .X64] `SYS_csops_audittoken :: 170`
- [OS == .MACOS && CPU == .X64] `SYS_waitid :: 173`
- [OS == .MACOS && CPU == .X64] `SYS_kdebug_typefilter :: 177`
- [OS == .MACOS && CPU == .X64] `SYS_kdebug_trace_string :: 178`
- [OS == .MACOS && CPU == .X64] `SYS_kdebug_trace64 :: 179`
- [OS == .MACOS && CPU == .X64] `SYS_kdebug_trace :: 180`
- [OS == .MACOS && CPU == .X64] `SYS_setgid :: 181`
- [OS == .MACOS && CPU == .X64] `SYS_setegid :: 182`
- [OS == .MACOS && CPU == .X64] `SYS_seteuid :: 183`
- [OS == .MACOS && CPU == .X64] `SYS_sigreturn :: 184`
- [OS == .MACOS && CPU == .X64] `SYS_thread_selfcounts :: 186`
- [OS == .MACOS && CPU == .X64] `SYS_fdatasync :: 187`
- [OS == .MACOS && CPU == .X64] `SYS_stat :: 188`
- [OS == .MACOS && CPU == .X64] `SYS_fstat :: 189`
- [OS == .MACOS && CPU == .X64] `SYS_lstat :: 190`
- [OS == .MACOS && CPU == .X64] `SYS_pathconf :: 191`
- [OS == .MACOS && CPU == .X64] `SYS_fpathconf :: 192`
- [OS == .MACOS && CPU == .X64] `SYS_getrlimit :: 194`
- [OS == .MACOS && CPU == .X64] `SYS_setrlimit :: 195`
- [OS == .MACOS && CPU == .X64] `SYS_getdirentries :: 196`
- [OS == .MACOS && CPU == .X64] `SYS_mmap :: 197`
- [OS == .MACOS && CPU == .X64] `SYS_lseek :: 199`
- [OS == .MACOS && CPU == .X64] `SYS_truncate :: 200`
- [OS == .MACOS && CPU == .X64] `SYS_ftruncate :: 201`
- [OS == .MACOS && CPU == .X64] `SYS_sysctl :: 202`
- [OS == .MACOS && CPU == .X64] `SYS_mlock :: 203`
- [OS == .MACOS && CPU == .X64] `SYS_munlock :: 204`
- [OS == .MACOS && CPU == .X64] `SYS_undelete :: 205`
- [OS == .MACOS && CPU == .X64] `SYS_open_dprotected_np :: 216`
- [OS == .MACOS && CPU == .X64] `SYS_getattrlist :: 220`
- [OS == .MACOS && CPU == .X64] `SYS_setattrlist :: 221`
- [OS == .MACOS && CPU == .X64] `SYS_getdirentriesattr :: 222`
- [OS == .MACOS && CPU == .X64] `SYS_exchangedata :: 223`
- [OS == .MACOS && CPU == .X64] `SYS_searchfs :: 225`
- [OS == .MACOS && CPU == .X64] `SYS_delete :: 226`
- [OS == .MACOS && CPU == .X64] `SYS_copyfile :: 227`
- [OS == .MACOS && CPU == .X64] `SYS_fgetattrlist :: 228`
- [OS == .MACOS && CPU == .X64] `SYS_fsetattrlist :: 229`
- [OS == .MACOS && CPU == .X64] `SYS_poll :: 230`
- [OS == .MACOS && CPU == .X64] `SYS_watchevent :: 231`
- [OS == .MACOS && CPU == .X64] `SYS_waitevent :: 232`
- [OS == .MACOS && CPU == .X64] `SYS_modwatch :: 233`
- [OS == .MACOS && CPU == .X64] `SYS_getxattr :: 234`
- [OS == .MACOS && CPU == .X64] `SYS_fgetxattr :: 235`
- [OS == .MACOS && CPU == .X64] `SYS_setxattr :: 236`
- [OS == .MACOS && CPU == .X64] `SYS_fsetxattr :: 237`
- [OS == .MACOS && CPU == .X64] `SYS_removexattr :: 238`
- [OS == .MACOS && CPU == .X64] `SYS_fremovexattr :: 239`
- [OS == .MACOS && CPU == .X64] `SYS_listxattr :: 240`
- [OS == .MACOS && CPU == .X64] `SYS_flistxattr :: 241`
- [OS == .MACOS && CPU == .X64] `SYS_fsctl :: 242`
- [OS == .MACOS && CPU == .X64] `SYS_initgroups :: 243`
- [OS == .MACOS && CPU == .X64] `SYS_posix_spawn :: 244`
- [OS == .MACOS && CPU == .X64] `SYS_ffsctl :: 245`
- [OS == .MACOS && CPU == .X64] `SYS_nfsclnt :: 247`
- [OS == .MACOS && CPU == .X64] `SYS_fhopen :: 248`
- [OS == .MACOS && CPU == .X64] `SYS_minherit :: 250`
- [OS == .MACOS && CPU == .X64] `SYS_semsys :: 251`
- [OS == .MACOS && CPU == .X64] `SYS_msgsys :: 252`
- [OS == .MACOS && CPU == .X64] `SYS_shmsys :: 253`
- [OS == .MACOS && CPU == .X64] `SYS_semctl :: 254`
- [OS == .MACOS && CPU == .X64] `SYS_semget :: 255`
- [OS == .MACOS && CPU == .X64] `SYS_semop :: 256`
- [OS == .MACOS && CPU == .X64] `SYS_msgctl :: 258`
- [OS == .MACOS && CPU == .X64] `SYS_msgget :: 259`
- [OS == .MACOS && CPU == .X64] `SYS_msgsnd :: 260`
- [OS == .MACOS && CPU == .X64] `SYS_msgrcv :: 261`
- [OS == .MACOS && CPU == .X64] `SYS_shmat :: 262`
- [OS == .MACOS && CPU == .X64] `SYS_shmctl :: 263`
- [OS == .MACOS && CPU == .X64] `SYS_shmdt :: 264`
- [OS == .MACOS && CPU == .X64] `SYS_shmget :: 265`
- [OS == .MACOS && CPU == .X64] `SYS_shm_open :: 266`
- [OS == .MACOS && CPU == .X64] `SYS_shm_unlink :: 267`
- [OS == .MACOS && CPU == .X64] `SYS_sem_open :: 268`
- [OS == .MACOS && CPU == .X64] `SYS_sem_close :: 269`
- [OS == .MACOS && CPU == .X64] `SYS_sem_unlink :: 270`
- [OS == .MACOS && CPU == .X64] `SYS_sem_wait :: 271`
- [OS == .MACOS && CPU == .X64] `SYS_sem_trywait :: 272`
- [OS == .MACOS && CPU == .X64] `SYS_sem_post :: 273`
- [OS == .MACOS && CPU == .X64] `SYS_sysctlbyname :: 274`
- [OS == .MACOS && CPU == .X64] `SYS_open_extended :: 277`
- [OS == .MACOS && CPU == .X64] `SYS_umask_extended :: 278`
- [OS == .MACOS && CPU == .X64] `SYS_stat_extended :: 279`
- [OS == .MACOS && CPU == .X64] `SYS_lstat_extended :: 280`
- [OS == .MACOS && CPU == .X64] `SYS_fstat_extended :: 281`
- [OS == .MACOS && CPU == .X64] `SYS_chmod_extended :: 282`
- [OS == .MACOS && CPU == .X64] `SYS_fchmod_extended :: 283`
- [OS == .MACOS && CPU == .X64] `SYS_access_extended :: 284`
- [OS == .MACOS && CPU == .X64] `SYS_settid :: 285`
- [OS == .MACOS && CPU == .X64] `SYS_gettid :: 286`
- [OS == .MACOS && CPU == .X64] `SYS_setsgroups :: 287`
- [OS == .MACOS && CPU == .X64] `SYS_getsgroups :: 288`
- [OS == .MACOS && CPU == .X64] `SYS_setwgroups :: 289`
- [OS == .MACOS && CPU == .X64] `SYS_getwgroups :: 290`
- [OS == .MACOS && CPU == .X64] `SYS_mkfifo_extended :: 291`
- [OS == .MACOS && CPU == .X64] `SYS_mkdir_extended :: 292`
- [OS == .MACOS && CPU == .X64] `SYS_identitysvc :: 293`
- [OS == .MACOS && CPU == .X64] `SYS_shared_region_check_np :: 294`
- [OS == .MACOS && CPU == .X64] `SYS_vm_pressure_monitor :: 296`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_longrdlock :: 297`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_yieldwrlock :: 298`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_downgrade :: 299`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_upgrade :: 300`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_mutexwait :: 301`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_mutexdrop :: 302`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_cvbroad :: 303`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_cvsignal :: 304`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_cvwait :: 305`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_rdlock :: 306`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_wrlock :: 307`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_unlock :: 308`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_rw_unlock2 :: 309`
- [OS == .MACOS && CPU == .X64] `SYS_getsid :: 310`
- [OS == .MACOS && CPU == .X64] `SYS_settid_with_pid :: 311`
- [OS == .MACOS && CPU == .X64] `SYS_psynch_cvclrprepost :: 312`
- [OS == .MACOS && CPU == .X64] `SYS_aio_fsync :: 313`
- [OS == .MACOS && CPU == .X64] `SYS_aio_return :: 314`
- [OS == .MACOS && CPU == .X64] `SYS_aio_suspend :: 315`
- [OS == .MACOS && CPU == .X64] `SYS_aio_cancel :: 316`
- [OS == .MACOS && CPU == .X64] `SYS_aio_error :: 317`
- [OS == .MACOS && CPU == .X64] `SYS_aio_read :: 318`
- [OS == .MACOS && CPU == .X64] `SYS_aio_write :: 319`
- [OS == .MACOS && CPU == .X64] `SYS_lio_listio :: 320`
- [OS == .MACOS && CPU == .X64] `SYS_iopolicysys :: 322`
- [OS == .MACOS && CPU == .X64] `SYS_process_policy :: 323`
- [OS == .MACOS && CPU == .X64] `SYS_mlockall :: 324`
- [OS == .MACOS && CPU == .X64] `SYS_munlockall :: 325`
- [OS == .MACOS && CPU == .X64] `SYS_issetugid :: 327`
- [OS == .MACOS && CPU == .X64] `SYS___pthread_kill :: 328`
- [OS == .MACOS && CPU == .X64] `SYS___pthread_sigmask :: 329`
- [OS == .MACOS && CPU == .X64] `SYS___sigwait :: 330`
- [OS == .MACOS && CPU == .X64] `SYS___disable_threadsignal :: 331`
- [OS == .MACOS && CPU == .X64] `SYS___pthread_markcancel :: 332`
- [OS == .MACOS && CPU == .X64] `SYS___pthread_canceled :: 333`
- [OS == .MACOS && CPU == .X64] `SYS___semwait_signal :: 334`
- [OS == .MACOS && CPU == .X64] `SYS_proc_info :: 336`
- [OS == .MACOS && CPU == .X64] `SYS_sendfile :: 337`
- [OS == .MACOS && CPU == .X64] `SYS_stat64 :: 338`
- [OS == .MACOS && CPU == .X64] `SYS_fstat64 :: 339`
- [OS == .MACOS && CPU == .X64] `SYS_lstat64 :: 340`
- [OS == .MACOS && CPU == .X64] `SYS_stat64_extended :: 341`
- [OS == .MACOS && CPU == .X64] `SYS_lstat64_extended :: 342`
- [OS == .MACOS && CPU == .X64] `SYS_fstat64_extended :: 343`
- [OS == .MACOS && CPU == .X64] `SYS_getdirentries64 :: 344`
- [OS == .MACOS && CPU == .X64] `SYS_statfs64 :: 345`
- [OS == .MACOS && CPU == .X64] `SYS_fstatfs64 :: 346`
- [OS == .MACOS && CPU == .X64] `SYS_getfsstat64 :: 347`
- [OS == .MACOS && CPU == .X64] `SYS___pthread_chdir :: 348`
- [OS == .MACOS && CPU == .X64] `SYS___pthread_fchdir :: 349`
- [OS == .MACOS && CPU == .X64] `SYS_audit :: 350`
- [OS == .MACOS && CPU == .X64] `SYS_auditon :: 351`
- [OS == .MACOS && CPU == .X64] `SYS_getauid :: 353`
- [OS == .MACOS && CPU == .X64] `SYS_setauid :: 354`
- [OS == .MACOS && CPU == .X64] `SYS_getaudit_addr :: 357`
- [OS == .MACOS && CPU == .X64] `SYS_setaudit_addr :: 358`
- [OS == .MACOS && CPU == .X64] `SYS_auditctl :: 359`
- [OS == .MACOS && CPU == .X64] `SYS_bsdthread_create :: 360`
- [OS == .MACOS && CPU == .X64] `SYS_bsdthread_terminate :: 361`
- [OS == .MACOS && CPU == .X64] `SYS_kqueue :: 362`
- [OS == .MACOS && CPU == .X64] `SYS_kevent :: 363`
- [OS == .MACOS && CPU == .X64] `SYS_lchown :: 364`
- [OS == .MACOS && CPU == .X64] `SYS_bsdthread_register :: 366`
- [OS == .MACOS && CPU == .X64] `SYS_workq_open :: 367`
- [OS == .MACOS && CPU == .X64] `SYS_workq_kernreturn :: 368`
- [OS == .MACOS && CPU == .X64] `SYS_kevent64 :: 369`
- [OS == .MACOS && CPU == .X64] `SYS___old_semwait_signal :: 370`
- [OS == .MACOS && CPU == .X64] `SYS___old_semwait_signal_nocancel :: 371`
- [OS == .MACOS && CPU == .X64] `SYS_thread_selfid :: 372`
- [OS == .MACOS && CPU == .X64] `SYS_ledger :: 373`
- [OS == .MACOS && CPU == .X64] `SYS_kevent_qos :: 374`
- [OS == .MACOS && CPU == .X64] `SYS_kevent_id :: 375`
- [OS == .MACOS && CPU == .X64] `SYS___mac_execve :: 380`
- [OS == .MACOS && CPU == .X64] `SYS___mac_syscall :: 381`
- [OS == .MACOS && CPU == .X64] `SYS___mac_get_file :: 382`
- [OS == .MACOS && CPU == .X64] `SYS___mac_set_file :: 383`
- [OS == .MACOS && CPU == .X64] `SYS___mac_get_link :: 384`
- [OS == .MACOS && CPU == .X64] `SYS___mac_set_link :: 385`
- [OS == .MACOS && CPU == .X64] `SYS___mac_get_proc :: 386`
- [OS == .MACOS && CPU == .X64] `SYS___mac_set_proc :: 387`
- [OS == .MACOS && CPU == .X64] `SYS___mac_get_fd :: 388`
- [OS == .MACOS && CPU == .X64] `SYS___mac_set_fd :: 389`
- [OS == .MACOS && CPU == .X64] `SYS___mac_get_pid :: 390`
- [OS == .MACOS && CPU == .X64] `SYS_pselect :: 394`
- [OS == .MACOS && CPU == .X64] `SYS_pselect_nocancel :: 395`
- [OS == .MACOS && CPU == .X64] `SYS_read_nocancel :: 396`
- [OS == .MACOS && CPU == .X64] `SYS_write_nocancel :: 397`
- [OS == .MACOS && CPU == .X64] `SYS_open_nocancel :: 398`
- [OS == .MACOS && CPU == .X64] `SYS_close_nocancel :: 399`
- [OS == .MACOS && CPU == .X64] `SYS_wait4_nocancel :: 400`
- [OS == .MACOS && CPU == .X64] `SYS_recvmsg_nocancel :: 401`
- [OS == .MACOS && CPU == .X64] `SYS_sendmsg_nocancel :: 402`
- [OS == .MACOS && CPU == .X64] `SYS_recvfrom_nocancel :: 403`
- [OS == .MACOS && CPU == .X64] `SYS_accept_nocancel :: 404`
- [OS == .MACOS && CPU == .X64] `SYS_msync_nocancel :: 405`
- [OS == .MACOS && CPU == .X64] `SYS_fcntl_nocancel :: 406`
- [OS == .MACOS && CPU == .X64] `SYS_select_nocancel :: 407`
- [OS == .MACOS && CPU == .X64] `SYS_fsync_nocancel :: 408`
- [OS == .MACOS && CPU == .X64] `SYS_connect_nocancel :: 409`
- [OS == .MACOS && CPU == .X64] `SYS_sigsuspend_nocancel :: 410`
- [OS == .MACOS && CPU == .X64] `SYS_readv_nocancel :: 411`
- [OS == .MACOS && CPU == .X64] `SYS_writev_nocancel :: 412`
- [OS == .MACOS && CPU == .X64] `SYS_sendto_nocancel :: 413`
- [OS == .MACOS && CPU == .X64] `SYS_pread_nocancel :: 414`
- [OS == .MACOS && CPU == .X64] `SYS_pwrite_nocancel :: 415`
- [OS == .MACOS && CPU == .X64] `SYS_waitid_nocancel :: 416`
- [OS == .MACOS && CPU == .X64] `SYS_poll_nocancel :: 417`
- [OS == .MACOS && CPU == .X64] `SYS_msgsnd_nocancel :: 418`
- [OS == .MACOS && CPU == .X64] `SYS_msgrcv_nocancel :: 419`
- [OS == .MACOS && CPU == .X64] `SYS_sem_wait_nocancel :: 420`
- [OS == .MACOS && CPU == .X64] `SYS_aio_suspend_nocancel :: 421`
- [OS == .MACOS && CPU == .X64] `SYS___sigwait_nocancel :: 422`
- [OS == .MACOS && CPU == .X64] `SYS___semwait_signal_nocancel :: 423`
- [OS == .MACOS && CPU == .X64] `SYS___mac_mount :: 424`
- [OS == .MACOS && CPU == .X64] `SYS___mac_get_mount :: 425`
- [OS == .MACOS && CPU == .X64] `SYS___mac_getfsstat :: 426`
- [OS == .MACOS && CPU == .X64] `SYS_fsgetpath :: 427`
- [OS == .MACOS && CPU == .X64] `SYS_audit_session_self :: 428`
- [OS == .MACOS && CPU == .X64] `SYS_audit_session_join :: 429`
- [OS == .MACOS && CPU == .X64] `SYS_fileport_makeport :: 430`
- [OS == .MACOS && CPU == .X64] `SYS_fileport_makefd :: 431`
- [OS == .MACOS && CPU == .X64] `SYS_audit_session_port :: 432`
- [OS == .MACOS && CPU == .X64] `SYS_pid_suspend :: 433`
- [OS == .MACOS && CPU == .X64] `SYS_pid_resume :: 434`
- [OS == .MACOS && CPU == .X64] `SYS_pid_hibernate :: 435`
- [OS == .MACOS && CPU == .X64] `SYS_pid_shutdown_sockets :: 436`
- [OS == .MACOS && CPU == .X64] `SYS_shared_region_map_and_slide_np :: 438`
- [OS == .MACOS && CPU == .X64] `SYS_kas_info :: 439`
- [OS == .MACOS && CPU == .X64] `SYS_memorystatus_control :: 440`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_open_np :: 441`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_close_np :: 442`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_kqueue_np :: 443`
- [OS == .MACOS && CPU == .X64] `SYS_change_fdguard_np :: 444`
- [OS == .MACOS && CPU == .X64] `SYS_usrctl :: 445`
- [OS == .MACOS && CPU == .X64] `SYS_proc_rlimit_control :: 446`
- [OS == .MACOS && CPU == .X64] `SYS_connectx :: 447`
- [OS == .MACOS && CPU == .X64] `SYS_disconnectx :: 448`
- [OS == .MACOS && CPU == .X64] `SYS_peeloff :: 449`
- [OS == .MACOS && CPU == .X64] `SYS_socket_delegate :: 450`
- [OS == .MACOS && CPU == .X64] `SYS_telemetry :: 451`
- [OS == .MACOS && CPU == .X64] `SYS_proc_uuid_policy :: 452`
- [OS == .MACOS && CPU == .X64] `SYS_memorystatus_get_level :: 453`
- [OS == .MACOS && CPU == .X64] `SYS_system_override :: 454`
- [OS == .MACOS && CPU == .X64] `SYS_vfs_purge :: 455`
- [OS == .MACOS && CPU == .X64] `SYS_sfi_ctl :: 456`
- [OS == .MACOS && CPU == .X64] `SYS_sfi_pidctl :: 457`
- [OS == .MACOS && CPU == .X64] `SYS_coalition :: 458`
- [OS == .MACOS && CPU == .X64] `SYS_coalition_info :: 459`
- [OS == .MACOS && CPU == .X64] `SYS_necp_match_policy :: 460`
- [OS == .MACOS && CPU == .X64] `SYS_getattrlistbulk :: 461`
- [OS == .MACOS && CPU == .X64] `SYS_clonefileat :: 462`
- [OS == .MACOS && CPU == .X64] `SYS_openat :: 463`
- [OS == .MACOS && CPU == .X64] `SYS_openat_nocancel :: 464`
- [OS == .MACOS && CPU == .X64] `SYS_renameat :: 465`
- [OS == .MACOS && CPU == .X64] `SYS_faccessat :: 466`
- [OS == .MACOS && CPU == .X64] `SYS_fchmodat :: 467`
- [OS == .MACOS && CPU == .X64] `SYS_fchownat :: 468`
- [OS == .MACOS && CPU == .X64] `SYS_fstatat :: 469`
- [OS == .MACOS && CPU == .X64] `SYS_fstatat64 :: 470`
- [OS == .MACOS && CPU == .X64] `SYS_linkat :: 471`
- [OS == .MACOS && CPU == .X64] `SYS_unlinkat :: 472`
- [OS == .MACOS && CPU == .X64] `SYS_readlinkat :: 473`
- [OS == .MACOS && CPU == .X64] `SYS_symlinkat :: 474`
- [OS == .MACOS && CPU == .X64] `SYS_mkdirat :: 475`
- [OS == .MACOS && CPU == .X64] `SYS_getattrlistat :: 476`
- [OS == .MACOS && CPU == .X64] `SYS_proc_trace_log :: 477`
- [OS == .MACOS && CPU == .X64] `SYS_bsdthread_ctl :: 478`
- [OS == .MACOS && CPU == .X64] `SYS_openbyid_np :: 479`
- [OS == .MACOS && CPU == .X64] `SYS_recvmsg_x :: 480`
- [OS == .MACOS && CPU == .X64] `SYS_sendmsg_x :: 481`
- [OS == .MACOS && CPU == .X64] `SYS_thread_selfusage :: 482`
- [OS == .MACOS && CPU == .X64] `SYS_csrctl :: 483`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_open_dprotected_np :: 484`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_write_np :: 485`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_pwrite_np :: 486`
- [OS == .MACOS && CPU == .X64] `SYS_guarded_writev_np :: 487`
- [OS == .MACOS && CPU == .X64] `SYS_renameatx_np :: 488`
- [OS == .MACOS && CPU == .X64] `SYS_mremap_encrypted :: 489`
- [OS == .MACOS && CPU == .X64] `SYS_netagent_trigger :: 490`
- [OS == .MACOS && CPU == .X64] `SYS_stack_snapshot_with_config :: 491`
- [OS == .MACOS && CPU == .X64] `SYS_microstackshot :: 492`
- [OS == .MACOS && CPU == .X64] `SYS_grab_pgo_data :: 493`
- [OS == .MACOS && CPU == .X64] `SYS_persona :: 494`
- [OS == .MACOS && CPU == .X64] `SYS_work_interval_ctl :: 499`
- [OS == .MACOS && CPU == .X64] `SYS_getentropy :: 500`
- [OS == .MACOS && CPU == .X64] `SYS_necp_open :: 501`
- [OS == .MACOS && CPU == .X64] `SYS_necp_client_action :: 502`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_open :: 503`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_register :: 504`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_deregister :: 505`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_create :: 506`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_destroy :: 507`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_get_opt :: 508`
- [OS == .MACOS && CPU == .X64] `SYS___nexus_set_opt :: 509`
- [OS == .MACOS && CPU == .X64] `SYS___channel_open :: 510`
- [OS == .MACOS && CPU == .X64] `SYS___channel_get_info :: 511`
- [OS == .MACOS && CPU == .X64] `SYS___channel_sync :: 512`
- [OS == .MACOS && CPU == .X64] `SYS___channel_get_opt :: 513`
- [OS == .MACOS && CPU == .X64] `SYS___channel_set_opt :: 514`
- [OS == .MACOS && CPU == .X64] `SYS_ulock_wait :: 515`
- [OS == .MACOS && CPU == .X64] `SYS_ulock_wake :: 516`
- [OS == .MACOS && CPU == .X64] `SYS_fclonefileat :: 517`
- [OS == .MACOS && CPU == .X64] `SYS_fs_snapshot :: 518`
- [OS == .MACOS && CPU == .X64] `SYS_terminate_with_payload :: 520`
- [OS == .MACOS && CPU == .X64] `SYS_abort_with_payload :: 521`
- [OS == .MACOS && CPU == .X64] `SYS_necp_session_open :: 522`
- [OS == .MACOS && CPU == .X64] `SYS_necp_session_action :: 523`
- [OS == .MACOS && CPU == .X64] `SYS_setattrlistat :: 524`
- [OS == .MACOS && CPU == .X64] `SYS_net_qos_guideline :: 525`
- [OS == .MACOS && CPU == .X64] `SYS_fmount :: 526`
- [OS == .MACOS && CPU == .X64] `SYS_ntp_adjtime :: 527`
- [OS == .MACOS && CPU == .X64] `SYS_ntp_gettime :: 528`
- [OS == .MACOS && CPU == .X64] `SYS_os_fault_with_payload :: 529`
- [OS == .MACOS && CPU == .X64] `SYS_MAXSYSCALL :: 530`
- [OS == .MACOS && CPU == .X64] `SYS_invalid :: 63`
- [OS == .MACOS && CPU == .X64] `syscall :: (unknown0: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_SIZE__ :: 8176`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_ATTR_SIZE__ :: 56`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_MUTEXATTR_SIZE__ :: 8`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_MUTEX_SIZE__ :: 56`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_CONDATTR_SIZE__ :: 8`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_COND_SIZE__ :: 40`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_ONCE_SIZE__ :: 8`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_RWLOCK_SIZE__ :: 192`
- [OS == .MACOS && CPU == .X64] `__PTHREAD_RWLOCKATTR_SIZE__ :: 16`
- [OS == .MACOS && CPU == .X64] `QOS_MIN_RELATIVE_PRIORITY :: -15`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CREATE_JOINABLE :: 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CREATE_DETACHED :: 2`
- [OS == .MACOS && CPU == .X64] `PTHREAD_INHERIT_SCHED :: 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_EXPLICIT_SCHED :: 2`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CANCEL_ENABLE :: 0x01`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CANCEL_DISABLE :: 0x00`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CANCEL_DEFERRED :: 0x02`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CANCEL_ASYNCHRONOUS :: 0x00`
- [OS == .MACOS && CPU == .X64] `PTHREAD_CANCELED :: cast(*void) 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_SCOPE_SYSTEM :: 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_SCOPE_PROCESS :: 2`
- [OS == .MACOS && CPU == .X64] `PTHREAD_PROCESS_SHARED :: 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_PROCESS_PRIVATE :: 2`
- [OS == .MACOS && CPU == .X64] `PTHREAD_PRIO_NONE :: 0`
- [OS == .MACOS && CPU == .X64] `PTHREAD_PRIO_INHERIT :: 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_PRIO_PROTECT :: 2`
- [OS == .MACOS && CPU == .X64] `PTHREAD_MUTEX_NORMAL :: 0`
- [OS == .MACOS && CPU == .X64] `PTHREAD_MUTEX_ERRORCHECK :: 1`
- [OS == .MACOS && CPU == .X64] `PTHREAD_MUTEX_RECURSIVE :: 2`
- [OS == .MACOS && CPU == .X64] `PTHREAD_MUTEX_DEFAULT :: PTHREAD_MUTEX_NORMAL`
- [OS == .MACOS && CPU == .X64] `_JBLEN :: (9 * 2) + 3 + 16`
- [OS == .MACOS && CPU == .X64] `_PTHREAD_MUTEX_POLICY_NONE :: 0`
- [OS == .MACOS && CPU == .X64] `_PTHREAD_MUTEX_POLICY_FAIRSHARE :: 1`
- [OS == .MACOS && CPU == .X64] `_PTHREAD_MUTEX_POLICY_FIRSTFIT :: 2`
- [OS == .MACOS && CPU == .X64] `__darwin_pthread_handler_rec :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_attr_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_cond_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_condattr_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_mutex_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_mutexattr_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_once_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_rwlock_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_rwlockattr_t :: struct`
- [OS == .MACOS && CPU == .X64] `_opaque_pthread_t :: struct`
- [OS == .MACOS && CPU == .X64] `sched_param :: struct`
- [OS == .MACOS && CPU == .X64] `sched_yield :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sched_get_priority_min :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sched_get_priority_max :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_t :: _opaque_pthread_attr_t`
- [OS == .MACOS && CPU == .X64] `pthread_cond_t :: _opaque_pthread_cond_t`
- [OS == .MACOS && CPU == .X64] `pthread_condattr_t :: _opaque_pthread_condattr_t`
- [OS == .MACOS && CPU == .X64] `pthread_key_t :: u64`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_t :: _opaque_pthread_mutex_t`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_t :: _opaque_pthread_mutexattr_t`
- [OS == .MACOS && CPU == .X64] `pthread_once_t :: _opaque_pthread_once_t`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_t :: _opaque_pthread_rwlock_t`
- [OS == .MACOS && CPU == .X64] `pthread_rwlockattr_t :: _opaque_pthread_rwlockattr_t`
- [OS == .MACOS && CPU == .X64] `pthread_t :: *_opaque_pthread_t`
- [OS == .MACOS && CPU == .X64] `QOS_CLASS :: enum u32`
- [OS == .MACOS && CPU == .X64] `qos_class_t :: u32`
- [OS == .MACOS && CPU == .X64] `qos_class_self :: () -> qos_class_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `qos_class_main :: () -> qos_class_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_set_qos_class_np :: (__attr: *pthread_attr_t, __qos_class: qos_class_t, __relative_priority: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_get_qos_class_np :: (__attr: *pthread_attr_t, __qos_class: *qos_class_t, __relative_priority: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_set_qos_class_self_np :: (__qos_class: qos_class_t, __relative_priority: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_get_qos_class_np :: (__pthread: pthread_t, __qos_class: *qos_class_t, __relative_priority: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_override_s :: struct`
- [OS == .MACOS && CPU == .X64] `pthread_override_t :: *pthread_override_s`
- [OS == .MACOS && CPU == .X64] `pthread_override_qos_class_start_np :: (__pthread: pthread_t, __qos_class: qos_class_t, __relative_priority: s32) -> pthread_override_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_override_qos_class_end_np :: (__override: pthread_override_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_atfork :: (unknown0: #type () -> void #c_call, unknown1: #type () -> void #c_call, unknown2: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_destroy :: (unknown0: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getdetachstate :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getguardsize :: (unknown0: *pthread_attr_t, unknown1: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getinheritsched :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getschedparam :: (unknown0: *pthread_attr_t, unknown1: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getschedpolicy :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getscope :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getstack :: (unknown0: *pthread_attr_t, unknown1: **void, unknown2: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getstackaddr :: (unknown0: *pthread_attr_t, unknown1: **void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_getstacksize :: (unknown0: *pthread_attr_t, unknown1: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_init :: (unknown0: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setdetachstate :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setguardsize :: (unknown0: *pthread_attr_t, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setinheritsched :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setschedparam :: (unknown0: *pthread_attr_t, unknown1: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setschedpolicy :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setscope :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setstack :: (unknown0: *pthread_attr_t, unknown1: *void, unknown2: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setstackaddr :: (unknown0: *pthread_attr_t, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_attr_setstacksize :: (unknown0: *pthread_attr_t, unknown1: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cancel :: (unknown0: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_broadcast :: (unknown0: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_destroy :: (unknown0: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_init :: (unknown0: *pthread_cond_t, unknown1: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_signal :: (unknown0: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_timedwait :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t, unknown2: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_wait :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_condattr_destroy :: (unknown0: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_condattr_init :: (unknown0: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_condattr_getpshared :: (unknown0: *pthread_condattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_condattr_setpshared :: (unknown0: *pthread_condattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_create :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_detach :: (unknown0: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_equal :: (unknown0: pthread_t, unknown1: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_exit :: (unknown0: *void) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_getconcurrency :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_getschedparam :: (unknown0: pthread_t, unknown1: *s32, unknown2: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_getspecific :: (unknown0: pthread_key_t) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_join :: (unknown0: pthread_t, unknown1: **void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_key_create :: (unknown0: *pthread_key_t, unknown1: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_key_delete :: (unknown0: pthread_key_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_destroy :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_getprioceiling :: (unknown0: *pthread_mutex_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_init :: (unknown0: *pthread_mutex_t, unknown1: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_lock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_setprioceiling :: (unknown0: *pthread_mutex_t, unknown1: s32, unknown2: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_trylock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutex_unlock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_destroy :: (unknown0: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_getprioceiling :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_getprotocol :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_getpshared :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_gettype :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_init :: (unknown0: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_setprioceiling :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_setprotocol :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_setpshared :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_settype :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_once :: (unknown0: *pthread_once_t, unknown1: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_destroy :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_init :: (unknown0: *pthread_rwlock_t, unknown1: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_rdlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_tryrdlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_trywrlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_wrlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlock_unlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlockattr_destroy :: (unknown0: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlockattr_getpshared :: (unknown0: *pthread_rwlockattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlockattr_init :: (unknown0: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_rwlockattr_setpshared :: (unknown0: *pthread_rwlockattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_self :: () -> pthread_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_setcancelstate :: (unknown0: s32, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_setcanceltype :: (unknown0: s32, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_setconcurrency :: (unknown0: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_setschedparam :: (unknown0: pthread_t, unknown1: s32, unknown2: *sched_param) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_setspecific :: (unknown0: pthread_key_t, unknown1: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_testcancel :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_is_threaded_np :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_threadid_np :: (unknown0: pthread_t, unknown1: *u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_getname_np :: (unknown0: pthread_t, unknown1: *u8, unknown2: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_setname_np :: (unknown0: *u8) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_main_np :: () -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mach_thread_np :: (unknown0: pthread_t) -> mach_port_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_get_stacksize_np :: (unknown0: pthread_t) -> u64 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_get_stackaddr_np :: (unknown0: pthread_t) -> *void #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_signal_thread_np :: (unknown0: *pthread_cond_t, unknown1: pthread_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_cond_timedwait_relative_np :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t, unknown2: *timespec) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_create_suspended_np :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_kill :: (unknown0: pthread_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_from_mach_thread_np :: (unknown0: mach_port_t) -> pthread_t #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_sigmask :: (unknown0: s32, unknown1: *sigset_t, unknown2: *sigset_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_yield_np :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `sem_t :: s32`
- [OS == .MACOS && CPU == .X64] `jmp_buf :: [37] s32`
- [OS == .MACOS && CPU == .X64] `sigjmp_buf :: [38] s32`
- [OS == .MACOS && CPU == .X64] `setjmp :: (unknown0: *jmp_buf) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `longjmp :: (unknown0: *jmp_buf, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `_setjmp :: (unknown0: *jmp_buf) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `_longjmp :: (unknown0: *jmp_buf, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `sigsetjmp :: (unknown0: *sigjmp_buf, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `siglongjmp :: (unknown0: *sigjmp_buf, unknown1: s32) -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `longjmperror :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_setpolicy_np :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_mutexattr_getpolicy_np :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `_pthread_mutex_enable_legacy_mode :: () -> void #foreign libc`
- [OS == .MACOS && CPU == .X64] `pthread_create_from_mach_thread :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `PRIO :: enum s32`
- [OS == .MACOS && CPU == .X64] `PRIO_MIN :: -20`
- [OS == .MACOS && CPU == .X64] `PRIO_MAX :: 20`
- [OS == .MACOS && CPU == .X64] `RUSAGE :: enum s32`
- [OS == .MACOS && CPU == .X64] `RUSAGE_INFO :: enum`
- [OS == .MACOS && CPU == .X64] `RLIM_INFINITY :: (cast(u64) 1 << 63) - 1`
- [OS == .MACOS && CPU == .X64] `RLIM_SAVED_MAX :: RLIM_INFINITY`
- [OS == .MACOS && CPU == .X64] `RLIM_SAVED_CUR :: RLIM_INFINITY`
- [OS == .MACOS && CPU == .X64] `RLIMIT_RESOURCE :: enum s32`
- [OS == .MACOS && CPU == .X64] `RLIM_NLIMITS :: 9`
- [OS == .MACOS && CPU == .X64] `_RLIMIT_POSIX_FLAG :: 0x1000`
- [OS == .MACOS && CPU == .X64] `RLIMIT_WAKEUPS_MONITOR :: 0x1`
- [OS == .MACOS && CPU == .X64] `RLIMIT_CPU_USAGE_MONITOR :: 0x2`
- [OS == .MACOS && CPU == .X64] `RLIMIT_THREAD_CPULIMITS :: 0x3`
- [OS == .MACOS && CPU == .X64] `WAKEMON_ENABLE :: 0x01`
- [OS == .MACOS && CPU == .X64] `WAKEMON_DISABLE :: 0x02`
- [OS == .MACOS && CPU == .X64] `WAKEMON_GET_PARAMS :: 0x04`
- [OS == .MACOS && CPU == .X64] `WAKEMON_SET_DEFAULTS :: 0x08`
- [OS == .MACOS && CPU == .X64] `WAKEMON_MAKE_FATAL :: 0x10`
- [OS == .MACOS && CPU == .X64] `CPUMON_MAKE_FATAL :: 0x1000`
- [OS == .MACOS && CPU == .X64] `IOPOL_TYPE_DISK :: 0`
- [OS == .MACOS && CPU == .X64] `IOPOL_SCOPE_PROCESS :: 0`
- [OS == .MACOS && CPU == .X64] `IOPOL_SCOPE_THREAD :: 1`
- [OS == .MACOS && CPU == .X64] `IOPOL_SCOPE_DARWIN_BG :: 2`
- [OS == .MACOS && CPU == .X64] `IOPOL_DEFAULT :: 0`
- [OS == .MACOS && CPU == .X64] `IOPOL_IMPORTANT :: 1`
- [OS == .MACOS && CPU == .X64] `IOPOL_PASSIVE :: 2`
- [OS == .MACOS && CPU == .X64] `IOPOL_THROTTLE :: 3`
- [OS == .MACOS && CPU == .X64] `IOPOL_UTILITY :: 4`
- [OS == .MACOS && CPU == .X64] `IOPOL_STANDARD :: 5`
- [OS == .MACOS && CPU == .X64] `IOPOL_APPLICATION :: IOPOL_STANDARD`
- [OS == .MACOS && CPU == .X64] `IOPOL_NORMAL :: IOPOL_IMPORTANT`
- [OS == .MACOS && CPU == .X64] `id_t :: u32`
- [OS == .MACOS && CPU == .X64] `rlim_t :: u64`
- [OS == .MACOS && CPU == .X64] `rusage :: struct`
- [OS == .MACOS && CPU == .X64] `rusage_info_t :: *void`
- [OS == .MACOS && CPU == .X64] `rusage_info_v0 :: struct`
- [OS == .MACOS && CPU == .X64] `rusage_info_v1 :: struct`
- [OS == .MACOS && CPU == .X64] `rusage_info_v2 :: struct`
- [OS == .MACOS && CPU == .X64] `rusage_info_v3 :: struct`
- [OS == .MACOS && CPU == .X64] `rusage_info_v4 :: struct`
- [OS == .MACOS && CPU == .X64] `rusage_info_current :: rusage_info_v4`
- [OS == .MACOS && CPU == .X64] `rlimit :: struct`
- [OS == .MACOS && CPU == .X64] `proc_rlimit_control_wakeupmon :: struct`
- [OS == .MACOS && CPU == .X64] `getpriority :: (unknown0: PRIO, unknown1: id_t) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getiopolicy_np :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getrlimit :: (unknown0: RLIMIT_RESOURCE, unknown1: *rlimit) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `getrusage :: (unknown0: RUSAGE, unknown1: *rusage) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setpriority :: (unknown0: PRIO, unknown1: id_t, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setiopolicy_np :: (unknown0: s32, unknown1: s32, unknown2: s32) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `setrlimit :: (unknown0: RLIMIT_RESOURCE, unknown1: *rlimit) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `ucred :: struct`
- [OS == .MACOS && CPU == .X64] `P_TRACED :: 0x00000800`
- [OS == .MACOS && CPU == .X64] `CTL_MAXNAME :: 12`
- [OS == .MACOS && CPU == .X64] `CTLTYPE :: 0xf`
- [OS == .MACOS && CPU == .X64] `CTLTYPE_NODE :: 1`
- [OS == .MACOS && CPU == .X64] `CTLTYPE_INT :: 2`
- [OS == .MACOS && CPU == .X64] `CTLTYPE_STRING :: 3`
- [OS == .MACOS && CPU == .X64] `CTLTYPE_QUAD :: 4`
- [OS == .MACOS && CPU == .X64] `CTLTYPE_OPAQUE :: 5`
- [OS == .MACOS && CPU == .X64] `CTLTYPE_STRUCT :: CTLTYPE_OPAQUE`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_RD :: 0x80000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_WR :: 0x40000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_RW :: CTLFLAG_RD|CTLFLAG_WR`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_NOLOCK :: 0x20000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_ANYBODY :: 0x10000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_SECURE :: 0x08000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_MASKED :: 0x04000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_NOAUTO :: 0x02000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_KERN :: 0x01000000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_LOCKED :: 0x00800000`
- [OS == .MACOS && CPU == .X64] `CTLFLAG_OID2 :: 0x00400000`
- [OS == .MACOS && CPU == .X64] `OID_AUTO :: -1`
- [OS == .MACOS && CPU == .X64] `OID_AUTO_START :: 100`
- [OS == .MACOS && CPU == .X64] `CTL_UNSPEC :: 0`
- [OS == .MACOS && CPU == .X64] `CTL_KERN :: 1`
- [OS == .MACOS && CPU == .X64] `CTL_VM :: 2`
- [OS == .MACOS && CPU == .X64] `CTL_VFS :: 3`
- [OS == .MACOS && CPU == .X64] `CTL_NET :: 4`
- [OS == .MACOS && CPU == .X64] `CTL_DEBUG :: 5`
- [OS == .MACOS && CPU == .X64] `CTL_HW :: 6`
- [OS == .MACOS && CPU == .X64] `CTL_MACHDEP :: 7`
- [OS == .MACOS && CPU == .X64] `CTL_USER :: 8`
- [OS == .MACOS && CPU == .X64] `CTL_MAXID :: 9`
- [OS == .MACOS && CPU == .X64] `KERN_OSTYPE :: 1`
- [OS == .MACOS && CPU == .X64] `KERN_OSRELEASE :: 2`
- [OS == .MACOS && CPU == .X64] `KERN_OSREV :: 3`
- [OS == .MACOS && CPU == .X64] `KERN_VERSION :: 4`
- [OS == .MACOS && CPU == .X64] `KERN_MAXVNODES :: 5`
- [OS == .MACOS && CPU == .X64] `KERN_MAXPROC :: 6`
- [OS == .MACOS && CPU == .X64] `KERN_MAXFILES :: 7`
- [OS == .MACOS && CPU == .X64] `KERN_ARGMAX :: 8`
- [OS == .MACOS && CPU == .X64] `KERN_SECURELVL :: 9`
- [OS == .MACOS && CPU == .X64] `KERN_HOSTNAME :: 10`
- [OS == .MACOS && CPU == .X64] `KERN_HOSTID :: 11`
- [OS == .MACOS && CPU == .X64] `KERN_CLOCKRATE :: 12`
- [OS == .MACOS && CPU == .X64] `KERN_VNODE :: 13`
- [OS == .MACOS && CPU == .X64] `KERN_PROC :: 14`
- [OS == .MACOS && CPU == .X64] `KERN_FILE :: 15`
- [OS == .MACOS && CPU == .X64] `KERN_PROF :: 16`
- [OS == .MACOS && CPU == .X64] `KERN_POSIX1 :: 17`
- [OS == .MACOS && CPU == .X64] `KERN_NGROUPS :: 18`
- [OS == .MACOS && CPU == .X64] `KERN_JOB_CONTROL :: 19`
- [OS == .MACOS && CPU == .X64] `KERN_SAVED_IDS :: 20`
- [OS == .MACOS && CPU == .X64] `KERN_BOOTTIME :: 21`
- [OS == .MACOS && CPU == .X64] `KERN_NISDOMAINNAME :: 22`
- [OS == .MACOS && CPU == .X64] `KERN_DOMAINNAME :: KERN_NISDOMAINNAME`
- [OS == .MACOS && CPU == .X64] `KERN_MAXPARTITIONS :: 23`
- [OS == .MACOS && CPU == .X64] `KERN_KDEBUG :: 24`
- [OS == .MACOS && CPU == .X64] `KERN_UPDATEINTERVAL :: 25`
- [OS == .MACOS && CPU == .X64] `KERN_OSRELDATE :: 26`
- [OS == .MACOS && CPU == .X64] `KERN_NTP_PLL :: 27`
- [OS == .MACOS && CPU == .X64] `KERN_BOOTFILE :: 28`
- [OS == .MACOS && CPU == .X64] `KERN_MAXFILESPERPROC :: 29`
- [OS == .MACOS && CPU == .X64] `KERN_MAXPROCPERUID :: 30`
- [OS == .MACOS && CPU == .X64] `KERN_DUMPDEV :: 31`
- [OS == .MACOS && CPU == .X64] `KERN_IPC :: 32`
- [OS == .MACOS && CPU == .X64] `KERN_DUMMY :: 33`
- [OS == .MACOS && CPU == .X64] `KERN_PS_STRINGS :: 34`
- [OS == .MACOS && CPU == .X64] `KERN_USRSTACK32 :: 35`
- [OS == .MACOS && CPU == .X64] `KERN_LOGSIGEXIT :: 36`
- [OS == .MACOS && CPU == .X64] `KERN_SYMFILE :: 37`
- [OS == .MACOS && CPU == .X64] `KERN_PROCARGS :: 38`
- [OS == .MACOS && CPU == .X64] `KERN_NETBOOT :: 40`
- [OS == .MACOS && CPU == .X64] `KERN_SYSV :: 42`
- [OS == .MACOS && CPU == .X64] `KERN_AFFINITY :: 43`
- [OS == .MACOS && CPU == .X64] `KERN_TRANSLATE :: 44`
- [OS == .MACOS && CPU == .X64] `KERN_CLASSIC :: KERN_TRANSLATE`
- [OS == .MACOS && CPU == .X64] `KERN_EXEC :: 45`
- [OS == .MACOS && CPU == .X64] `KERN_CLASSICHANDLER :: KERN_EXEC`
- [OS == .MACOS && CPU == .X64] `KERN_AIOMAX :: 46`
- [OS == .MACOS && CPU == .X64] `KERN_AIOPROCMAX :: 47`
- [OS == .MACOS && CPU == .X64] `KERN_AIOTHREADS :: 48`
- [OS == .MACOS && CPU == .X64] `KERN_PROCARGS2 :: 49`
- [OS == .MACOS && CPU == .X64] `KERN_COREFILE :: 50`
- [OS == .MACOS && CPU == .X64] `KERN_COREDUMP :: 51`
- [OS == .MACOS && CPU == .X64] `KERN_SUGID_COREDUMP :: 52`
- [OS == .MACOS && CPU == .X64] `KERN_PROCDELAYTERM :: 53`
- [OS == .MACOS && CPU == .X64] `KERN_SHREG_PRIVATIZABLE :: 54`
- [OS == .MACOS && CPU == .X64] `KERN_LOW_PRI_WINDOW :: 56`
- [OS == .MACOS && CPU == .X64] `KERN_LOW_PRI_DELAY :: 57`
- [OS == .MACOS && CPU == .X64] `KERN_POSIX :: 58`
- [OS == .MACOS && CPU == .X64] `KERN_USRSTACK64 :: 59`
- [OS == .MACOS && CPU == .X64] `KERN_NX_PROTECTION :: 60`
- [OS == .MACOS && CPU == .X64] `KERN_TFP :: 61`
- [OS == .MACOS && CPU == .X64] `KERN_PROCNAME :: 62`
- [OS == .MACOS && CPU == .X64] `KERN_THALTSTACK :: 63`
- [OS == .MACOS && CPU == .X64] `KERN_SPECULATIVE_READS :: 64`
- [OS == .MACOS && CPU == .X64] `KERN_OSVERSION :: 65`
- [OS == .MACOS && CPU == .X64] `KERN_SAFEBOOT :: 66`
- [OS == .MACOS && CPU == .X64] `KERN_RAGEVNODE :: 68`
- [OS == .MACOS && CPU == .X64] `KERN_TTY :: 69`
- [OS == .MACOS && CPU == .X64] `KERN_CHECKOPENEVT :: 70`
- [OS == .MACOS && CPU == .X64] `KERN_THREADNAME :: 71`
- [OS == .MACOS && CPU == .X64] `KERN_MAXID :: 72`
- [OS == .MACOS && CPU == .X64] `KERN_USRSTACK :: KERN_USRSTACK64`
- [OS == .MACOS && CPU == .X64] `KERN_RAGE_PROC :: 1`
- [OS == .MACOS && CPU == .X64] `KERN_RAGE_THREAD :: 2`
- [OS == .MACOS && CPU == .X64] `KERN_UNRAGE_PROC :: 3`
- [OS == .MACOS && CPU == .X64] `KERN_UNRAGE_THREAD :: 4`
- [OS == .MACOS && CPU == .X64] `KERN_OPENEVT_PROC :: 1`
- [OS == .MACOS && CPU == .X64] `KERN_UNOPENEVT_PROC :: 2`
- [OS == .MACOS && CPU == .X64] `KERN_TFP_POLICY :: 1`
- [OS == .MACOS && CPU == .X64] `KERN_TFP_POLICY_DENY :: 0`
- [OS == .MACOS && CPU == .X64] `KERN_TFP_POLICY_DEFAULT :: 2`
- [OS == .MACOS && CPU == .X64] `KERN_KDEFLAGS :: 1`
- [OS == .MACOS && CPU == .X64] `KERN_KDDFLAGS :: 2`
- [OS == .MACOS && CPU == .X64] `KERN_KDENABLE :: 3`
- [OS == .MACOS && CPU == .X64] `KERN_KDSETBUF :: 4`
- [OS == .MACOS && CPU == .X64] `KERN_KDGETBUF :: 5`
- [OS == .MACOS && CPU == .X64] `KERN_KDSETUP :: 6`
- [OS == .MACOS && CPU == .X64] `KERN_KDREMOVE :: 7`
- [OS == .MACOS && CPU == .X64] `KERN_KDSETREG :: 8`
- [OS == .MACOS && CPU == .X64] `KERN_KDGETREG :: 9`
- [OS == .MACOS && CPU == .X64] `KERN_KDREADTR :: 10`
- [OS == .MACOS && CPU == .X64] `KERN_KDPIDTR :: 11`
- [OS == .MACOS && CPU == .X64] `KERN_KDTHRMAP :: 12`
- [OS == .MACOS && CPU == .X64] `KERN_KDPIDEX :: 14`
- [OS == .MACOS && CPU == .X64] `KERN_KDSETRTCDEC :: 15`
- [OS == .MACOS && CPU == .X64] `KERN_KDGETENTROPY :: 16`
- [OS == .MACOS && CPU == .X64] `KERN_KDWRITETR :: 17`
- [OS == .MACOS && CPU == .X64] `KERN_KDWRITEMAP :: 18`
- [OS == .MACOS && CPU == .X64] `KERN_KDTEST :: 19`
- [OS == .MACOS && CPU == .X64] `KERN_KDREADCURTHRMAP :: 21`
- [OS == .MACOS && CPU == .X64] `KERN_KDSET_TYPEFILTER :: 22`
- [OS == .MACOS && CPU == .X64] `KERN_KDBUFWAIT :: 23`
- [OS == .MACOS && CPU == .X64] `KERN_KDCPUMAP :: 24`
- [OS == .MACOS && CPU == .X64] `KERN_KDWRITEMAP_V3 :: 27`
- [OS == .MACOS && CPU == .X64] `KERN_KDWRITETR_V3 :: 28`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_ALL :: 0`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_PID :: 1`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_PGRP :: 2`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_SESSION :: 3`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_TTY :: 4`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_UID :: 5`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_RUID :: 6`
- [OS == .MACOS && CPU == .X64] `KERN_PROC_LCID :: 7`
- [OS == .MACOS && CPU == .X64] `WMESGLEN :: 7`
- [OS == .MACOS && CPU == .X64] `EPROC_CTTY :: 0x01`
- [OS == .MACOS && CPU == .X64] `EPROC_SLEADER :: 0x02`
- [OS == .MACOS && CPU == .X64] `COMAPT_MAXLOGNAME :: 12`
- [OS == .MACOS && CPU == .X64] `KIPC_MAXSOCKBUF :: 1`
- [OS == .MACOS && CPU == .X64] `KIPC_SOCKBUF_WASTE :: 2`
- [OS == .MACOS && CPU == .X64] `KIPC_SOMAXCONN :: 3`
- [OS == .MACOS && CPU == .X64] `KIPC_MAX_LINKHDR :: 4`
- [OS == .MACOS && CPU == .X64] `KIPC_MAX_PROTOHDR :: 5`
- [OS == .MACOS && CPU == .X64] `KIPC_MAX_HDR :: 6`
- [OS == .MACOS && CPU == .X64] `KIPC_MAX_DATALEN :: 7`
- [OS == .MACOS && CPU == .X64] `KIPC_MBSTAT :: 8`
- [OS == .MACOS && CPU == .X64] `KIPC_NMBCLUSTERS :: 9`
- [OS == .MACOS && CPU == .X64] `KIPC_SOQLIMITCOMPAT :: 10`
- [OS == .MACOS && CPU == .X64] `VM_METER :: 1`
- [OS == .MACOS && CPU == .X64] `VM_LOADAVG :: 2`
- [OS == .MACOS && CPU == .X64] `VM_MACHFACTOR :: 4`
- [OS == .MACOS && CPU == .X64] `VM_SWAPUSAGE :: 5`
- [OS == .MACOS && CPU == .X64] `VM_MAXID :: 6`
- [OS == .MACOS && CPU == .X64] `LSCALE :: 1000`
- [OS == .MACOS && CPU == .X64] `HW_MACHINE :: 1`
- [OS == .MACOS && CPU == .X64] `HW_MODEL :: 2`
- [OS == .MACOS && CPU == .X64] `HW_NCPU :: 3`
- [OS == .MACOS && CPU == .X64] `HW_BYTEORDER :: 4`
- [OS == .MACOS && CPU == .X64] `HW_PHYSMEM :: 5`
- [OS == .MACOS && CPU == .X64] `HW_USERMEM :: 6`
- [OS == .MACOS && CPU == .X64] `HW_PAGESIZE :: 7`
- [OS == .MACOS && CPU == .X64] `HW_DISKNAMES :: 8`
- [OS == .MACOS && CPU == .X64] `HW_DISKSTATS :: 9`
- [OS == .MACOS && CPU == .X64] `HW_EPOCH :: 10`
- [OS == .MACOS && CPU == .X64] `HW_FLOATINGPT :: 11`
- [OS == .MACOS && CPU == .X64] `HW_MACHINE_ARCH :: 12`
- [OS == .MACOS && CPU == .X64] `HW_VECTORUNIT :: 13`
- [OS == .MACOS && CPU == .X64] `HW_BUS_FREQ :: 14`
- [OS == .MACOS && CPU == .X64] `HW_CPU_FREQ :: 15`
- [OS == .MACOS && CPU == .X64] `HW_CACHELINE :: 16`
- [OS == .MACOS && CPU == .X64] `HW_L1ICACHESIZE :: 17`
- [OS == .MACOS && CPU == .X64] `HW_L1DCACHESIZE :: 18`
- [OS == .MACOS && CPU == .X64] `HW_L2SETTINGS :: 19`
- [OS == .MACOS && CPU == .X64] `HW_L2CACHESIZE :: 20`
- [OS == .MACOS && CPU == .X64] `HW_L3SETTINGS :: 21`
- [OS == .MACOS && CPU == .X64] `HW_L3CACHESIZE :: 22`
- [OS == .MACOS && CPU == .X64] `HW_TB_FREQ :: 23`
- [OS == .MACOS && CPU == .X64] `HW_MEMSIZE :: 24`
- [OS == .MACOS && CPU == .X64] `HW_AVAILCPU :: 25`
- [OS == .MACOS && CPU == .X64] `HW_MAXID :: 26`
- [OS == .MACOS && CPU == .X64] `USER_CS_PATH :: 1`
- [OS == .MACOS && CPU == .X64] `USER_BC_BASE_MAX :: 2`
- [OS == .MACOS && CPU == .X64] `USER_BC_DIM_MAX :: 3`
- [OS == .MACOS && CPU == .X64] `USER_BC_SCALE_MAX :: 4`
- [OS == .MACOS && CPU == .X64] `USER_BC_STRING_MAX :: 5`
- [OS == .MACOS && CPU == .X64] `USER_COLL_WEIGHTS_MAX :: 6`
- [OS == .MACOS && CPU == .X64] `USER_EXPR_NEST_MAX :: 7`
- [OS == .MACOS && CPU == .X64] `USER_LINE_MAX :: 8`
- [OS == .MACOS && CPU == .X64] `USER_RE_DUP_MAX :: 9`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_VERSION :: 10`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_C_BIND :: 11`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_C_DEV :: 12`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_CHAR_TERM :: 13`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_FORT_DEV :: 14`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_FORT_RUN :: 15`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_LOCALEDEF :: 16`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_SW_DEV :: 17`
- [OS == .MACOS && CPU == .X64] `USER_POSIX2_UPE :: 18`
- [OS == .MACOS && CPU == .X64] `USER_STREAM_MAX :: 19`
- [OS == .MACOS && CPU == .X64] `USER_TZNAME_MAX :: 20`
- [OS == .MACOS && CPU == .X64] `USER_MAXID :: 21`
- [OS == .MACOS && CPU == .X64] `CTL_DEBUG_NAME :: 0`
- [OS == .MACOS && CPU == .X64] `CTL_DEBUG_VALUE :: 1`
- [OS == .MACOS && CPU == .X64] `CTL_DEBUG_MAXID :: 20`
- [OS == .MACOS && CPU == .X64] `session :: struct`
- [OS == .MACOS && CPU == .X64] `pgrp :: struct`
- [OS == .MACOS && CPU == .X64] `proc :: struct`
- [OS == .MACOS && CPU == .X64] `extern_proc :: struct`
- [OS == .MACOS && CPU == .X64] `vmspace :: struct`
- [OS == .MACOS && CPU == .X64] `ctlname :: struct`
- [OS == .MACOS && CPU == .X64] `_pcred :: struct`
- [OS == .MACOS && CPU == .X64] `_ucred :: struct`
- [OS == .MACOS && CPU == .X64] `kinfo_proc :: struct`
- [OS == .MACOS && CPU == .X64] `xsw_usage :: struct`
- [OS == .MACOS && CPU == .X64] `loadavg :: struct`
- [OS == .MACOS && CPU == .X64] `sysctl :: (unknown0: *s32, unknown1: u32, unknown2: *void, unknown3: *u64, unknown4: *void, unknown5: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sysctlbyname :: (unknown0: *u8, unknown1: *void, unknown2: *u64, unknown3: *void, unknown4: u64) -> s32 #foreign libc`
- [OS == .MACOS && CPU == .X64] `sysctlnametomib :: (unknown0: *u8, unknown1: *s32, unknown2: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `RTLD_LOCAL :: 0`
- [OS == .ANDROID && CPU == .ARM64] `RTLD_LAZY :: 0x00001`
- [OS == .ANDROID && CPU == .ARM64] `RTLD_NOW :: 0x00002`
- [OS == .ANDROID && CPU == .ARM64] `RTLD_NOLOAD :: 0x00004`
- [OS == .ANDROID && CPU == .ARM64] `RTLD_GLOBAL :: 0x00100`
- [OS == .ANDROID && CPU == .ARM64] `RTLD_NODELETE :: 0x01000`
- [OS == .ANDROID && CPU == .ARM64] `__FD_SETSIZE :: 1024`
- [OS == .ANDROID && CPU == .ARM64] `DT :: enum u8`
- [OS == .ANDROID && CPU == .ARM64] `__DIRENT64_INO_T :: ino_t`
- [OS == .ANDROID && CPU == .ARM64] `EPERM :: 1`
- [OS == .ANDROID && CPU == .ARM64] `ENOENT :: 2`
- [OS == .ANDROID && CPU == .ARM64] `ESRCH :: 3`
- [OS == .ANDROID && CPU == .ARM64] `EINTR :: 4`
- [OS == .ANDROID && CPU == .ARM64] `EIO :: 5`
- [OS == .ANDROID && CPU == .ARM64] `ENXIO :: 6`
- [OS == .ANDROID && CPU == .ARM64] `E2BIG :: 7`
- [OS == .ANDROID && CPU == .ARM64] `ENOEXEC :: 8`
- [OS == .ANDROID && CPU == .ARM64] `EBADF :: 9`
- [OS == .ANDROID && CPU == .ARM64] `ECHILD :: 10`
- [OS == .ANDROID && CPU == .ARM64] `EAGAIN :: 11`
- [OS == .ANDROID && CPU == .ARM64] `ENOMEM :: 12`
- [OS == .ANDROID && CPU == .ARM64] `EACCES :: 13`
- [OS == .ANDROID && CPU == .ARM64] `EFAULT :: 14`
- [OS == .ANDROID && CPU == .ARM64] `ENOTBLK :: 15`
- [OS == .ANDROID && CPU == .ARM64] `EBUSY :: 16`
- [OS == .ANDROID && CPU == .ARM64] `EEXIST :: 17`
- [OS == .ANDROID && CPU == .ARM64] `EXDEV :: 18`
- [OS == .ANDROID && CPU == .ARM64] `ENODEV :: 19`
- [OS == .ANDROID && CPU == .ARM64] `ENOTDIR :: 20`
- [OS == .ANDROID && CPU == .ARM64] `EISDIR :: 21`
- [OS == .ANDROID && CPU == .ARM64] `EINVAL :: 22`
- [OS == .ANDROID && CPU == .ARM64] `ENFILE :: 23`
- [OS == .ANDROID && CPU == .ARM64] `EMFILE :: 24`
- [OS == .ANDROID && CPU == .ARM64] `ENOTTY :: 25`
- [OS == .ANDROID && CPU == .ARM64] `ETXTBSY :: 26`
- [OS == .ANDROID && CPU == .ARM64] `EFBIG :: 27`
- [OS == .ANDROID && CPU == .ARM64] `ENOSPC :: 28`
- [OS == .ANDROID && CPU == .ARM64] `ESPIPE :: 29`
- [OS == .ANDROID && CPU == .ARM64] `EROFS :: 30`
- [OS == .ANDROID && CPU == .ARM64] `EMLINK :: 31`
- [OS == .ANDROID && CPU == .ARM64] `EPIPE :: 32`
- [OS == .ANDROID && CPU == .ARM64] `EDOM :: 33`
- [OS == .ANDROID && CPU == .ARM64] `ERANGE :: 34`
- [OS == .ANDROID && CPU == .ARM64] `EDEADLK :: 35`
- [OS == .ANDROID && CPU == .ARM64] `ENAMETOOLONG :: 36`
- [OS == .ANDROID && CPU == .ARM64] `ENOLCK :: 37`
- [OS == .ANDROID && CPU == .ARM64] `ENOSYS :: 38`
- [OS == .ANDROID && CPU == .ARM64] `ENOTEMPTY :: 39`
- [OS == .ANDROID && CPU == .ARM64] `ELOOP :: 40`
- [OS == .ANDROID && CPU == .ARM64] `EWOULDBLOCK :: EAGAIN`
- [OS == .ANDROID && CPU == .ARM64] `ENOMSG :: 42`
- [OS == .ANDROID && CPU == .ARM64] `EIDRM :: 43`
- [OS == .ANDROID && CPU == .ARM64] `ECHRNG :: 44`
- [OS == .ANDROID && CPU == .ARM64] `EL2NSYNC :: 45`
- [OS == .ANDROID && CPU == .ARM64] `EL3HLT :: 46`
- [OS == .ANDROID && CPU == .ARM64] `EL3RST :: 47`
- [OS == .ANDROID && CPU == .ARM64] `ELNRNG :: 48`
- [OS == .ANDROID && CPU == .ARM64] `EUNATCH :: 49`
- [OS == .ANDROID && CPU == .ARM64] `ENOCSI :: 50`
- [OS == .ANDROID && CPU == .ARM64] `EL2HLT :: 51`
- [OS == .ANDROID && CPU == .ARM64] `EBADE :: 52`
- [OS == .ANDROID && CPU == .ARM64] `EBADR :: 53`
- [OS == .ANDROID && CPU == .ARM64] `EXFULL :: 54`
- [OS == .ANDROID && CPU == .ARM64] `ENOANO :: 55`
- [OS == .ANDROID && CPU == .ARM64] `EBADRQC :: 56`
- [OS == .ANDROID && CPU == .ARM64] `EBADSLT :: 57`
- [OS == .ANDROID && CPU == .ARM64] `EDEADLOCK :: EDEADLK`
- [OS == .ANDROID && CPU == .ARM64] `EBFONT :: 59`
- [OS == .ANDROID && CPU == .ARM64] `ENOSTR :: 60`
- [OS == .ANDROID && CPU == .ARM64] `ENODATA :: 61`
- [OS == .ANDROID && CPU == .ARM64] `ETIME :: 62`
- [OS == .ANDROID && CPU == .ARM64] `ENOSR :: 63`
- [OS == .ANDROID && CPU == .ARM64] `ENONET :: 64`
- [OS == .ANDROID && CPU == .ARM64] `ENOPKG :: 65`
- [OS == .ANDROID && CPU == .ARM64] `EREMOTE :: 66`
- [OS == .ANDROID && CPU == .ARM64] `ENOLINK :: 67`
- [OS == .ANDROID && CPU == .ARM64] `EADV :: 68`
- [OS == .ANDROID && CPU == .ARM64] `ESRMNT :: 69`
- [OS == .ANDROID && CPU == .ARM64] `ECOMM :: 70`
- [OS == .ANDROID && CPU == .ARM64] `EPROTO :: 71`
- [OS == .ANDROID && CPU == .ARM64] `EMULTIHOP :: 72`
- [OS == .ANDROID && CPU == .ARM64] `EDOTDOT :: 73`
- [OS == .ANDROID && CPU == .ARM64] `EBADMSG :: 74`
- [OS == .ANDROID && CPU == .ARM64] `EOVERFLOW :: 75`
- [OS == .ANDROID && CPU == .ARM64] `ENOTUNIQ :: 76`
- [OS == .ANDROID && CPU == .ARM64] `EBADFD :: 77`
- [OS == .ANDROID && CPU == .ARM64] `EREMCHG :: 78`
- [OS == .ANDROID && CPU == .ARM64] `ELIBACC :: 79`
- [OS == .ANDROID && CPU == .ARM64] `ELIBBAD :: 80`
- [OS == .ANDROID && CPU == .ARM64] `ELIBSCN :: 81`
- [OS == .ANDROID && CPU == .ARM64] `ELIBMAX :: 82`
- [OS == .ANDROID && CPU == .ARM64] `ELIBEXEC :: 83`
- [OS == .ANDROID && CPU == .ARM64] `EILSEQ :: 84`
- [OS == .ANDROID && CPU == .ARM64] `ERESTART :: 85`
- [OS == .ANDROID && CPU == .ARM64] `ESTRPIPE :: 86`
- [OS == .ANDROID && CPU == .ARM64] `EUSERS :: 87`
- [OS == .ANDROID && CPU == .ARM64] `ENOTSOCK :: 88`
- [OS == .ANDROID && CPU == .ARM64] `EDESTADDRREQ :: 89`
- [OS == .ANDROID && CPU == .ARM64] `EMSGSIZE :: 90`
- [OS == .ANDROID && CPU == .ARM64] `EPROTOTYPE :: 91`
- [OS == .ANDROID && CPU == .ARM64] `ENOPROTOOPT :: 92`
- [OS == .ANDROID && CPU == .ARM64] `EPROTONOSUPPORT :: 93`
- [OS == .ANDROID && CPU == .ARM64] `ESOCKTNOSUPPORT :: 94`
- [OS == .ANDROID && CPU == .ARM64] `EOPNOTSUPP :: 95`
- [OS == .ANDROID && CPU == .ARM64] `EPFNOSUPPORT :: 96`
- [OS == .ANDROID && CPU == .ARM64] `EAFNOSUPPORT :: 97`
- [OS == .ANDROID && CPU == .ARM64] `EADDRINUSE :: 98`
- [OS == .ANDROID && CPU == .ARM64] `EADDRNOTAVAIL :: 99`
- [OS == .ANDROID && CPU == .ARM64] `ENETDOWN :: 100`
- [OS == .ANDROID && CPU == .ARM64] `ENETUNREACH :: 101`
- [OS == .ANDROID && CPU == .ARM64] `ENETRESET :: 102`
- [OS == .ANDROID && CPU == .ARM64] `ECONNABORTED :: 103`
- [OS == .ANDROID && CPU == .ARM64] `ECONNRESET :: 104`
- [OS == .ANDROID && CPU == .ARM64] `ENOBUFS :: 105`
- [OS == .ANDROID && CPU == .ARM64] `EISCONN :: 106`
- [OS == .ANDROID && CPU == .ARM64] `ENOTCONN :: 107`
- [OS == .ANDROID && CPU == .ARM64] `ESHUTDOWN :: 108`
- [OS == .ANDROID && CPU == .ARM64] `ETOOMANYREFS :: 109`
- [OS == .ANDROID && CPU == .ARM64] `ETIMEDOUT :: 110`
- [OS == .ANDROID && CPU == .ARM64] `ECONNREFUSED :: 111`
- [OS == .ANDROID && CPU == .ARM64] `EHOSTDOWN :: 112`
- [OS == .ANDROID && CPU == .ARM64] `EHOSTUNREACH :: 113`
- [OS == .ANDROID && CPU == .ARM64] `EALREADY :: 114`
- [OS == .ANDROID && CPU == .ARM64] `EINPROGRESS :: 115`
- [OS == .ANDROID && CPU == .ARM64] `ESTALE :: 116`
- [OS == .ANDROID && CPU == .ARM64] `EUCLEAN :: 117`
- [OS == .ANDROID && CPU == .ARM64] `ENOTNAM :: 118`
- [OS == .ANDROID && CPU == .ARM64] `ENAVAIL :: 119`
- [OS == .ANDROID && CPU == .ARM64] `EISNAM :: 120`
- [OS == .ANDROID && CPU == .ARM64] `EREMOTEIO :: 121`
- [OS == .ANDROID && CPU == .ARM64] `EDQUOT :: 122`
- [OS == .ANDROID && CPU == .ARM64] `ENOMEDIUM :: 123`
- [OS == .ANDROID && CPU == .ARM64] `EMEDIUMTYPE :: 124`
- [OS == .ANDROID && CPU == .ARM64] `ECANCELED :: 125`
- [OS == .ANDROID && CPU == .ARM64] `ENOKEY :: 126`
- [OS == .ANDROID && CPU == .ARM64] `EKEYEXPIRED :: 127`
- [OS == .ANDROID && CPU == .ARM64] `EKEYREVOKED :: 128`
- [OS == .ANDROID && CPU == .ARM64] `EKEYREJECTED :: 129`
- [OS == .ANDROID && CPU == .ARM64] `EOWNERDEAD :: 130`
- [OS == .ANDROID && CPU == .ARM64] `ENOTRECOVERABLE :: 131`
- [OS == .ANDROID && CPU == .ARM64] `ERFKILL :: 132`
- [OS == .ANDROID && CPU == .ARM64] `EHWPOISON :: 133`
- [OS == .ANDROID && CPU == .ARM64] `ENOTSUP :: EOPNOTSUPP`
- [OS == .ANDROID && CPU == .ARM64] `FNM_NOMATCH :: 1`
- [OS == .ANDROID && CPU == .ARM64] `FNM_NOSYS :: 2`
- [OS == .ANDROID && CPU == .ARM64] `FNM_NOESCAPE :: 0x01`
- [OS == .ANDROID && CPU == .ARM64] `FNM_PATHNAME :: 0x02`
- [OS == .ANDROID && CPU == .ARM64] `FNM_PERIOD :: 0x04`
- [OS == .ANDROID && CPU == .ARM64] `FNM_LEADING_DIR :: 0x08`
- [OS == .ANDROID && CPU == .ARM64] `FNM_CASEFOLD :: 0x10`
- [OS == .ANDROID && CPU == .ARM64] `FNM_IGNORECASE :: FNM_CASEFOLD`
- [OS == .ANDROID && CPU == .ARM64] `FNM_FILE_NAME :: FNM_PATHNAME`
- [OS == .ANDROID && CPU == .ARM64] `SA_RESTORER :: 0x04000000`
- [OS == .ANDROID && CPU == .ARM64] `MINSIGSTKSZ :: 5120`
- [OS == .ANDROID && CPU == .ARM64] `SIGSTKSZ :: 16384`
- [OS == .ANDROID && CPU == .ARM64] `_KERNEL__NSIG :: 64`
- [OS == .ANDROID && CPU == .ARM64] `SIGHUP :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SIGINT :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SIGQUIT :: 3`
- [OS == .ANDROID && CPU == .ARM64] `SIGILL :: 4`
- [OS == .ANDROID && CPU == .ARM64] `SIGTRAP :: 5`
- [OS == .ANDROID && CPU == .ARM64] `SIGABRT :: 6`
- [OS == .ANDROID && CPU == .ARM64] `SIGIOT :: 6`
- [OS == .ANDROID && CPU == .ARM64] `SIGBUS :: 7`
- [OS == .ANDROID && CPU == .ARM64] `SIGFPE :: 8`
- [OS == .ANDROID && CPU == .ARM64] `SIGKILL :: 9`
- [OS == .ANDROID && CPU == .ARM64] `SIGUSR1 :: 10`
- [OS == .ANDROID && CPU == .ARM64] `SIGSEGV :: 11`
- [OS == .ANDROID && CPU == .ARM64] `SIGUSR2 :: 12`
- [OS == .ANDROID && CPU == .ARM64] `SIGPIPE :: 13`
- [OS == .ANDROID && CPU == .ARM64] `SIGALRM :: 14`
- [OS == .ANDROID && CPU == .ARM64] `SIGTERM :: 15`
- [OS == .ANDROID && CPU == .ARM64] `SIGSTKFLT :: 16`
- [OS == .ANDROID && CPU == .ARM64] `SIGCHLD :: 17`
- [OS == .ANDROID && CPU == .ARM64] `SIGCONT :: 18`
- [OS == .ANDROID && CPU == .ARM64] `SIGSTOP :: 19`
- [OS == .ANDROID && CPU == .ARM64] `SIGTSTP :: 20`
- [OS == .ANDROID && CPU == .ARM64] `SIGTTIN :: 21`
- [OS == .ANDROID && CPU == .ARM64] `SIGTTOU :: 22`
- [OS == .ANDROID && CPU == .ARM64] `SIGURG :: 23`
- [OS == .ANDROID && CPU == .ARM64] `SIGXCPU :: 24`
- [OS == .ANDROID && CPU == .ARM64] `SIGXFSZ :: 25`
- [OS == .ANDROID && CPU == .ARM64] `SIGVTALRM :: 26`
- [OS == .ANDROID && CPU == .ARM64] `SIGPROF :: 27`
- [OS == .ANDROID && CPU == .ARM64] `SIGWINCH :: 28`
- [OS == .ANDROID && CPU == .ARM64] `SIGIO :: 29`
- [OS == .ANDROID && CPU == .ARM64] `SIGPOLL :: SIGIO`
- [OS == .ANDROID && CPU == .ARM64] `SIGPWR :: 30`
- [OS == .ANDROID && CPU == .ARM64] `SIGSYS :: 31`
- [OS == .ANDROID && CPU == .ARM64] `SIGUNUSED :: 31`
- [OS == .ANDROID && CPU == .ARM64] `__SIGRTMIN :: 32`
- [OS == .ANDROID && CPU == .ARM64] `__SIGRTMAX :: _KERNEL__NSIG`
- [OS == .ANDROID && CPU == .ARM64] `SI_MAX_SIZE :: 128`
- [OS == .ANDROID && CPU == .ARM64] `__ARCH_SI_CLOCK_T :: s64`
- [OS == .ANDROID && CPU == .ARM64] `SI_USER :: 0`
- [OS == .ANDROID && CPU == .ARM64] `SI_KERNEL :: 0x80`
- [OS == .ANDROID && CPU == .ARM64] `SI_QUEUE :: - 1`
- [OS == .ANDROID && CPU == .ARM64] `SI_TIMER :: - 2`
- [OS == .ANDROID && CPU == .ARM64] `SI_MESGQ :: - 3`
- [OS == .ANDROID && CPU == .ARM64] `SI_ASYNCIO :: - 4`
- [OS == .ANDROID && CPU == .ARM64] `SI_SIGIO :: - 5`
- [OS == .ANDROID && CPU == .ARM64] `SI_TKILL :: - 6`
- [OS == .ANDROID && CPU == .ARM64] `SI_DETHREAD :: - 7`
- [OS == .ANDROID && CPU == .ARM64] `SI_ASYNCNL :: - 60`
- [OS == .ANDROID && CPU == .ARM64] `ILL_ILLOPC :: 1`
- [OS == .ANDROID && CPU == .ARM64] `ILL_ILLOPN :: 2`
- [OS == .ANDROID && CPU == .ARM64] `ILL_ILLADR :: 3`
- [OS == .ANDROID && CPU == .ARM64] `ILL_ILLTRP :: 4`
- [OS == .ANDROID && CPU == .ARM64] `ILL_PRVOPC :: 5`
- [OS == .ANDROID && CPU == .ARM64] `ILL_PRVREG :: 6`
- [OS == .ANDROID && CPU == .ARM64] `ILL_COPROC :: 7`
- [OS == .ANDROID && CPU == .ARM64] `ILL_BADSTK :: 8`
- [OS == .ANDROID && CPU == .ARM64] `ILL_BADIADDR :: 9`
- [OS == .ANDROID && CPU == .ARM64] `__ILL_BREAK :: 10`
- [OS == .ANDROID && CPU == .ARM64] `__ILL_BNDMOD :: 11`
- [OS == .ANDROID && CPU == .ARM64] `NSIGILL :: 11`
- [OS == .ANDROID && CPU == .ARM64] `FPE_INTDIV :: 1`
- [OS == .ANDROID && CPU == .ARM64] `FPE_INTOVF :: 2`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTDIV :: 3`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTOVF :: 4`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTUND :: 5`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTRES :: 6`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTINV :: 7`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTSUB :: 8`
- [OS == .ANDROID && CPU == .ARM64] `__FPE_DECOVF :: 9`
- [OS == .ANDROID && CPU == .ARM64] `__FPE_DECDIV :: 10`
- [OS == .ANDROID && CPU == .ARM64] `__FPE_DECERR :: 11`
- [OS == .ANDROID && CPU == .ARM64] `__FPE_INVASC :: 12`
- [OS == .ANDROID && CPU == .ARM64] `__FPE_INVDEC :: 13`
- [OS == .ANDROID && CPU == .ARM64] `FPE_FLTUNK :: 14`
- [OS == .ANDROID && CPU == .ARM64] `FPE_CONDTRAP :: 15`
- [OS == .ANDROID && CPU == .ARM64] `NSIGFPE :: 15`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_MAPERR :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_ACCERR :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_BNDERR :: 3`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_PKUERR :: 4`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_ACCADI :: 5`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_ADIDERR :: 6`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_ADIPERR :: 7`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_MTEAERR :: 8`
- [OS == .ANDROID && CPU == .ARM64] `SEGV_MTESERR :: 9`
- [OS == .ANDROID && CPU == .ARM64] `NSIGSEGV :: 9`
- [OS == .ANDROID && CPU == .ARM64] `BUS_ADRALN :: 1`
- [OS == .ANDROID && CPU == .ARM64] `BUS_ADRERR :: 2`
- [OS == .ANDROID && CPU == .ARM64] `BUS_OBJERR :: 3`
- [OS == .ANDROID && CPU == .ARM64] `BUS_MCEERR_AR :: 4`
- [OS == .ANDROID && CPU == .ARM64] `BUS_MCEERR_AO :: 5`
- [OS == .ANDROID && CPU == .ARM64] `NSIGBUS :: 5`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_BRKPT :: 1`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_TRACE :: 2`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_BRANCH :: 3`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_HWBKPT :: 4`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_UNK :: 5`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_PERF :: 6`
- [OS == .ANDROID && CPU == .ARM64] `NSIGTRAP :: 6`
- [OS == .ANDROID && CPU == .ARM64] `TRAP_PERF_FLAG_ASYNC :: 1 << 0`
- [OS == .ANDROID && CPU == .ARM64] `CLD_EXITED :: 1`
- [OS == .ANDROID && CPU == .ARM64] `CLD_KILLED :: 2`
- [OS == .ANDROID && CPU == .ARM64] `CLD_DUMPED :: 3`
- [OS == .ANDROID && CPU == .ARM64] `CLD_TRAPPED :: 4`
- [OS == .ANDROID && CPU == .ARM64] `CLD_STOPPED :: 5`
- [OS == .ANDROID && CPU == .ARM64] `CLD_CONTINUED :: 6`
- [OS == .ANDROID && CPU == .ARM64] `NSIGCHLD :: 6`
- [OS == .ANDROID && CPU == .ARM64] `POLL_IN :: 1`
- [OS == .ANDROID && CPU == .ARM64] `POLL_OUT :: 2`
- [OS == .ANDROID && CPU == .ARM64] `POLL_MSG :: 3`
- [OS == .ANDROID && CPU == .ARM64] `POLL_ERR :: 4`
- [OS == .ANDROID && CPU == .ARM64] `POLL_PRI :: 5`
- [OS == .ANDROID && CPU == .ARM64] `POLL_HUP :: 6`
- [OS == .ANDROID && CPU == .ARM64] `NSIGPOLL :: 6`
- [OS == .ANDROID && CPU == .ARM64] `SYS_SECCOMP :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SYS_USER_DISPATCH :: 2`
- [OS == .ANDROID && CPU == .ARM64] `NSIGSYS :: 2`
- [OS == .ANDROID && CPU == .ARM64] `EMT_TAGOVF :: 1`
- [OS == .ANDROID && CPU == .ARM64] `NSIGEMT :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SIGEV_SIGNAL :: 0`
- [OS == .ANDROID && CPU == .ARM64] `SIGEV_NONE :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SIGEV_THREAD :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SIGEV_THREAD_ID :: 4`
- [OS == .ANDROID && CPU == .ARM64] `SIGEV_MAX_SIZE :: 64`
- [OS == .ANDROID && CPU == .ARM64] `SS_ONSTACK :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SS_DISABLE :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SS_AUTODISARM :: 1 << 31`
- [OS == .ANDROID && CPU == .ARM64] `SS_FLAG_BITS :: SS_AUTODISARM`
- [OS == .ANDROID && CPU == .ARM64] `_NSIG :: _KERNEL__NSIG + 1`
- [OS == .ANDROID && CPU == .ARM64] `NSIG :: _NSIG`
- [OS == .ANDROID && CPU == .ARM64] `PAGE_SIZE :: 4096`
- [OS == .ANDROID && CPU == .ARM64] `PAGE_MASK :: ~(PAGE_SIZE - 1)`
- [OS == .ANDROID && CPU == .ARM64] `NGREG :: 34`
- [OS == .ANDROID && CPU == .ARM64] `IPC_PRIVATE :: cast(s32) 0`
- [OS == .ANDROID && CPU == .ARM64] `IPC_CREAT :: 0x200`
- [OS == .ANDROID && CPU == .ARM64] `IPC_EXCL :: 0x400`
- [OS == .ANDROID && CPU == .ARM64] `IPC_NOWAIT :: 0x800`
- [OS == .ANDROID && CPU == .ARM64] `IPC_DIPC :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `IPC_OWN :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `IPC_RMID :: 0`
- [OS == .ANDROID && CPU == .ARM64] `IPC_SET :: 1`
- [OS == .ANDROID && CPU == .ARM64] `IPC_STAT :: 2`
- [OS == .ANDROID && CPU == .ARM64] `IPC_INFO :: 3`
- [OS == .ANDROID && CPU == .ARM64] `IPC_OLD :: 0`
- [OS == .ANDROID && CPU == .ARM64] `IPC_64 :: 0x0100`
- [OS == .ANDROID && CPU == .ARM64] `SEMOP :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SEMGET :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SEMCTL :: 3`
- [OS == .ANDROID && CPU == .ARM64] `SEMTIMEDOP :: 4`
- [OS == .ANDROID && CPU == .ARM64] `MSGSND :: 11`
- [OS == .ANDROID && CPU == .ARM64] `MSGRCV :: 12`
- [OS == .ANDROID && CPU == .ARM64] `MSGGET :: 13`
- [OS == .ANDROID && CPU == .ARM64] `MSGCTL :: 14`
- [OS == .ANDROID && CPU == .ARM64] `SHMAT :: 21`
- [OS == .ANDROID && CPU == .ARM64] `SHMDT :: 22`
- [OS == .ANDROID && CPU == .ARM64] `SHMGET :: 23`
- [OS == .ANDROID && CPU == .ARM64] `SHMCTL :: 24`
- [OS == .ANDROID && CPU == .ARM64] `DIPC :: 25`
- [OS == .ANDROID && CPU == .ARM64] `PROT_READ :: 0x1`
- [OS == .ANDROID && CPU == .ARM64] `PROT_WRITE :: 0x2`
- [OS == .ANDROID && CPU == .ARM64] `PROT_EXEC :: 0x4`
- [OS == .ANDROID && CPU == .ARM64] `PROT_SEM :: 0x8`
- [OS == .ANDROID && CPU == .ARM64] `PROT_NONE :: 0x0`
- [OS == .ANDROID && CPU == .ARM64] `PROT_GROWSDOWN :: 0x01000000`
- [OS == .ANDROID && CPU == .ARM64] `PROT_GROWSUP :: 0x02000000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_TYPE :: 0x0f`
- [OS == .ANDROID && CPU == .ARM64] `MAP_FIXED :: 0x10`
- [OS == .ANDROID && CPU == .ARM64] `MAP_ANONYMOUS :: 0x20`
- [OS == .ANDROID && CPU == .ARM64] `MAP_POPULATE :: 0x008000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_NONBLOCK :: 0x010000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_STACK :: 0x020000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_HUGETLB :: 0x040000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_SYNC :: 0x080000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_FIXED_NOREPLACE :: 0x100000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_UNINITIALIZED :: 0x4000000`
- [OS == .ANDROID && CPU == .ARM64] `MLOCK_ONFAULT :: 0x01`
- [OS == .ANDROID && CPU == .ARM64] `MS_ASYNC :: 1`
- [OS == .ANDROID && CPU == .ARM64] `MS_INVALIDATE :: 2`
- [OS == .ANDROID && CPU == .ARM64] `MS_SYNC :: 4`
- [OS == .ANDROID && CPU == .ARM64] `MADV_NORMAL :: 0`
- [OS == .ANDROID && CPU == .ARM64] `MADV_RANDOM :: 1`
- [OS == .ANDROID && CPU == .ARM64] `MADV_SEQUENTIAL :: 2`
- [OS == .ANDROID && CPU == .ARM64] `MADV_WILLNEED :: 3`
- [OS == .ANDROID && CPU == .ARM64] `MADV_DONTNEED :: 4`
- [OS == .ANDROID && CPU == .ARM64] `MADV_FREE :: 8`
- [OS == .ANDROID && CPU == .ARM64] `MADV_REMOVE :: 9`
- [OS == .ANDROID && CPU == .ARM64] `MADV_DONTFORK :: 10`
- [OS == .ANDROID && CPU == .ARM64] `MADV_DOFORK :: 11`
- [OS == .ANDROID && CPU == .ARM64] `MADV_HWPOISON :: 100`
- [OS == .ANDROID && CPU == .ARM64] `MADV_SOFT_OFFLINE :: 101`
- [OS == .ANDROID && CPU == .ARM64] `MADV_MERGEABLE :: 12`
- [OS == .ANDROID && CPU == .ARM64] `MADV_UNMERGEABLE :: 13`
- [OS == .ANDROID && CPU == .ARM64] `MADV_HUGEPAGE :: 14`
- [OS == .ANDROID && CPU == .ARM64] `MADV_NOHUGEPAGE :: 15`
- [OS == .ANDROID && CPU == .ARM64] `MADV_DONTDUMP :: 16`
- [OS == .ANDROID && CPU == .ARM64] `MADV_DODUMP :: 17`
- [OS == .ANDROID && CPU == .ARM64] `MADV_WIPEONFORK :: 18`
- [OS == .ANDROID && CPU == .ARM64] `MADV_KEEPONFORK :: 19`
- [OS == .ANDROID && CPU == .ARM64] `MADV_COLD :: 20`
- [OS == .ANDROID && CPU == .ARM64] `MADV_PAGEOUT :: 21`
- [OS == .ANDROID && CPU == .ARM64] `MADV_POPULATE_READ :: 22`
- [OS == .ANDROID && CPU == .ARM64] `MADV_POPULATE_WRITE :: 23`
- [OS == .ANDROID && CPU == .ARM64] `MADV_DONTNEED_LOCKED :: 24`
- [OS == .ANDROID && CPU == .ARM64] `MADV_COLLAPSE :: 25`
- [OS == .ANDROID && CPU == .ARM64] `MAP_FILE :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PKEY_DISABLE_ACCESS :: 0x1`
- [OS == .ANDROID && CPU == .ARM64] `PKEY_DISABLE_WRITE :: 0x2`
- [OS == .ANDROID && CPU == .ARM64] `PKEY_ACCESS_MASK :: PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE`
- [OS == .ANDROID && CPU == .ARM64] `MAP_GROWSDOWN :: 0x0100`
- [OS == .ANDROID && CPU == .ARM64] `MAP_DENYWRITE :: 0x0800`
- [OS == .ANDROID && CPU == .ARM64] `MAP_EXECUTABLE :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_LOCKED :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `MAP_NORESERVE :: 0x4000`
- [OS == .ANDROID && CPU == .ARM64] `MCL_CURRENT :: 1`
- [OS == .ANDROID && CPU == .ARM64] `MCL_FUTURE :: 2`
- [OS == .ANDROID && CPU == .ARM64] `MCL_ONFAULT :: 4`
- [OS == .ANDROID && CPU == .ARM64] `PROT_BTI :: 0x10`
- [OS == .ANDROID && CPU == .ARM64] `PROT_MTE :: 0x20`
- [OS == .ANDROID && CPU == .ARM64] `MREMAP_MAYMOVE :: 1`
- [OS == .ANDROID && CPU == .ARM64] `MREMAP_FIXED :: 2`
- [OS == .ANDROID && CPU == .ARM64] `MREMAP_DONTUNMAP :: 4`
- [OS == .ANDROID && CPU == .ARM64] `OVERCOMMIT_GUESS :: 0`
- [OS == .ANDROID && CPU == .ARM64] `OVERCOMMIT_ALWAYS :: 1`
- [OS == .ANDROID && CPU == .ARM64] `OVERCOMMIT_NEVER :: 2`
- [OS == .ANDROID && CPU == .ARM64] `MAP_SHARED :: 0x01`
- [OS == .ANDROID && CPU == .ARM64] `MAP_PRIVATE :: 0x02`
- [OS == .ANDROID && CPU == .ARM64] `MAP_SHARED_VALIDATE :: 0x03`
- [OS == .ANDROID && CPU == .ARM64] `MAP_ANON :: MAP_ANONYMOUS`
- [OS == .ANDROID && CPU == .ARM64] `POLLIN :: 0x0001`
- [OS == .ANDROID && CPU == .ARM64] `POLLPRI :: 0x0002`
- [OS == .ANDROID && CPU == .ARM64] `POLLOUT :: 0x0004`
- [OS == .ANDROID && CPU == .ARM64] `POLLERR :: 0x0008`
- [OS == .ANDROID && CPU == .ARM64] `POLLHUP :: 0x0010`
- [OS == .ANDROID && CPU == .ARM64] `POLLNVAL :: 0x0020`
- [OS == .ANDROID && CPU == .ARM64] `POLLRDNORM :: 0x0040`
- [OS == .ANDROID && CPU == .ARM64] `POLLRDBAND :: 0x0080`
- [OS == .ANDROID && CPU == .ARM64] `POLLWRNORM :: 0x0100`
- [OS == .ANDROID && CPU == .ARM64] `POLLWRBAND :: 0x0200`
- [OS == .ANDROID && CPU == .ARM64] `POLLMSG :: 0x0400`
- [OS == .ANDROID && CPU == .ARM64] `POLLREMOVE :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `POLLRDHUP :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `POLLFREE :: cast(u32) 0x4000`
- [OS == .ANDROID && CPU == .ARM64] `POLL_BUSY_LOOP :: cast(u32) 0x8000`
- [OS == .ANDROID && CPU == .ARM64] `ITIMER_REAL :: 0`
- [OS == .ANDROID && CPU == .ARM64] `ITIMER_VIRTUAL :: 1`
- [OS == .ANDROID && CPU == .ARM64] `ITIMER_PROF :: 2`
- [OS == .ANDROID && CPU == .ARM64] `clockid_t :: enum u32`
- [OS == .ANDROID && CPU == .ARM64] `MAX_CLOCKS :: 16`
- [OS == .ANDROID && CPU == .ARM64] `CLOCKS_MASK :: xx clockid_t.REALTIME | xx clockid_t.MONOTONIC`
- [OS == .ANDROID && CPU == .ARM64] `CLOCKS_MONO :: xx clockid_t.MONOTONIC`
- [OS == .ANDROID && CPU == .ARM64] `TIMER_ABSTIME :: 0x01`
- [OS == .ANDROID && CPU == .ARM64] `CLOCKS_PER_SEC :: 1000000`
- [OS == .ANDROID && CPU == .ARM64] `TIME_UTC :: xx clockid_t.REALTIME+1`
- [OS == .ANDROID && CPU == .ARM64] `TIME_MONOTONIC :: xx clockid_t.MONOTONIC+1`
- [OS == .ANDROID && CPU == .ARM64] `TIME_ACTIVE :: xx clockid_t.PROCESS_CPUTIME_ID+1`
- [OS == .ANDROID && CPU == .ARM64] `TIME_THREAD_ACTIVE :: xx clockid_t.THREAD_CPUTIME_ID+1`
- [OS == .ANDROID && CPU == .ARM64] `SEEK_SET :: 0`
- [OS == .ANDROID && CPU == .ARM64] `SEEK_CUR :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SEEK_END :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SEEK_DATA :: 3`
- [OS == .ANDROID && CPU == .ARM64] `SEEK_HOLE :: 4`
- [OS == .ANDROID && CPU == .ARM64] `using _SC_definitions :: enum s32`
- [OS == .ANDROID && CPU == .ARM64] `STDIN_FILENO :: 0`
- [OS == .ANDROID && CPU == .ARM64] `STDOUT_FILENO :: 1`
- [OS == .ANDROID && CPU == .ARM64] `STDERR_FILENO :: 2`
- [OS == .ANDROID && CPU == .ARM64] `F_OK :: 0`
- [OS == .ANDROID && CPU == .ARM64] `X_OK :: 1`
- [OS == .ANDROID && CPU == .ARM64] `W_OK :: 2`
- [OS == .ANDROID && CPU == .ARM64] `R_OK :: 4`
- [OS == .ANDROID && CPU == .ARM64] `PC :: enum u32`
- [OS == .ANDROID && CPU == .ARM64] `SHMMIN :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SHMMNI :: 4096`
- [OS == .ANDROID && CPU == .ARM64] `SHMSEG :: SHMMNI`
- [OS == .ANDROID && CPU == .ARM64] `SHM_R :: 0x100`
- [OS == .ANDROID && CPU == .ARM64] `SHM_W :: 0x80`
- [OS == .ANDROID && CPU == .ARM64] `SHM_HUGETLB :: 0x800`
- [OS == .ANDROID && CPU == .ARM64] `SHM_NORESERVE :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `SHM_RDONLY :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `SHM_RND :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `SHM_REMAP :: 0x4000`
- [OS == .ANDROID && CPU == .ARM64] `SHM_EXEC :: 0x8000`
- [OS == .ANDROID && CPU == .ARM64] `SHM_LOCK :: 11`
- [OS == .ANDROID && CPU == .ARM64] `SHM_UNLOCK :: 12`
- [OS == .ANDROID && CPU == .ARM64] `SHM_STAT :: 13`
- [OS == .ANDROID && CPU == .ARM64] `SHM_INFO :: 14`
- [OS == .ANDROID && CPU == .ARM64] `SHM_STAT_ANY :: 15`
- [OS == .ANDROID && CPU == .ARM64] `SHMLBA :: 4096`
- [OS == .ANDROID && CPU == .ARM64] `SYS_NMLN :: 65`
- [OS == .ANDROID && CPU == .ARM64] `WNOHANG :: 0x00000001`
- [OS == .ANDROID && CPU == .ARM64] `WUNTRACED :: 0x00000002`
- [OS == .ANDROID && CPU == .ARM64] `WSTOPPED :: WUNTRACED`
- [OS == .ANDROID && CPU == .ARM64] `WEXITED :: 0x00000004`
- [OS == .ANDROID && CPU == .ARM64] `WCONTINUED :: 0x00000008`
- [OS == .ANDROID && CPU == .ARM64] `WNOWAIT :: 0x01000000`
- [OS == .ANDROID && CPU == .ARM64] `__WNOTHREAD :: 0x20000000`
- [OS == .ANDROID && CPU == .ARM64] `__WALL :: 0x40000000`
- [OS == .ANDROID && CPU == .ARM64] `__WCLONE :: 0x80000000`
- [OS == .ANDROID && CPU == .ARM64] `P_ALL :: 0`
- [OS == .ANDROID && CPU == .ARM64] `P_PID :: 1`
- [OS == .ANDROID && CPU == .ARM64] `P_PGID :: 2`
- [OS == .ANDROID && CPU == .ARM64] `P_PIDFD :: 3`
- [OS == .ANDROID && CPU == .ARM64] `ptrdiff_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `Dl_info :: struct`
- [OS == .ANDROID && CPU == .ARM64] `dlopen :: (__filename: *u8, __flag: s32) -> *void #foreign libdl`
- [OS == .ANDROID && CPU == .ARM64] `dlclose :: (__handle: *void) -> s32 #foreign libdl`
- [OS == .ANDROID && CPU == .ARM64] `dlerror :: () -> *u8 #foreign libdl`
- [OS == .ANDROID && CPU == .ARM64] `dlsym :: (__handle: *void, __symbol: *u8) -> *void #foreign libdl`
- [OS == .ANDROID && CPU == .ARM64] `dlvsym :: (__handle: *void, __symbol: *u8, __version: *u8) -> *void #foreign libdl`
- [OS == .ANDROID && CPU == .ARM64] `dladdr :: (__addr: *void, __info: *Dl_info) -> s32 #foreign libdl`
- [OS == .ANDROID && CPU == .ARM64] `__kernel_fd_set :: struct`
- [OS == .ANDROID && CPU == .ARM64] `__kernel_fsid_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `gid_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `uid_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `pid_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `blkcnt_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `blksize_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `clock_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `fsblkcnt_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `fsfilcnt_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `mode_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `key_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `ino_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `ino64_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `nlink_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `timer_t :: *void`
- [OS == .ANDROID && CPU == .ARM64] `suseconds_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `useconds_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `dev_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `time_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `uint_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `dirent :: struct`
- [OS == .ANDROID && CPU == .ARM64] `dirent64 :: struct`
- [OS == .ANDROID && CPU == .ARM64] `DIR :: struct`
- [OS == .ANDROID && CPU == .ARM64] `opendir :: (__path: *u8) -> *DIR #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fdopendir :: (__dir_fd: s32) -> *DIR #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readdir :: (__dir: *DIR) -> *dirent #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readdir64 :: (__dir: *DIR) -> *dirent64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readdir_r :: (__dir: *DIR, __entry: *dirent, __buffer: **dirent) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readdir64_r :: (__dir: *DIR, __entry: *dirent64, __buffer: **dirent64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `closedir :: (__dir: *DIR) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `rewinddir :: (__dir: *DIR) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `seekdir :: (__dir: *DIR, __location: s64) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `telldir :: (__dir: *DIR) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `dirfd :: (__dir: *DIR) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `alphasort :: (__lhs: **dirent, __rhs: **dirent) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `alphasort64 :: (__lhs: **dirent64, __rhs: **dirent64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `scandir :: (__path: *u8, __name_list: ***dirent, __filter: #type (unknown0: *dirent) -> s32 #c_call, __comparator: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `scandir64 :: (__path: *u8, __name_list: ***dirent64, __filter: #type (unknown0: *dirent64) -> s32 #c_call, __comparator: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `scandirat64 :: (__dir_fd: s32, __path: *u8, __name_list: ***dirent64, __filter: #type (unknown0: *dirent64) -> s32 #c_call, __comparator: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `scandirat :: (__dir_fd: s32, __path: *u8, __name_list: ***dirent, __filter: #type (unknown0: *dirent) -> s32 #c_call, __comparator: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__errno_location :: () -> *OS_Error_Code #foreign libc "__errno"`
- [OS == .ANDROID && CPU == .ARM64] `backtrace :: (buffer: **void, size: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `backtrace_symbols :: (buffer: **void, size: s32) -> **u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `backtrace_symbols_fd :: (buffer: **void, size: s32, fd: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fnmatch :: (__pattern: *u8, __string: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `passwd :: struct`
- [OS == .ANDROID && CPU == .ARM64] `getpwnam :: (__name: *u8) -> *passwd #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpwuid :: (__uid: uid_t) -> *passwd #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpwent :: () -> *passwd #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setpwent :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `endpwent :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpwnam_r :: (__name: *u8, __pwd: *passwd, __buf: *u8, __n: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpwuid_r :: (__uid: uid_t, __pwd: *passwd, __buf: *u8, __n: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigcontext :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sigset_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `old_sigset_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `__kernel_sigaction :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sigaltstack_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `stack_t :: sigaltstack_t`
- [OS == .ANDROID && CPU == .ARM64] `sigval :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sigval_t :: sigval`
- [OS == .ANDROID && CPU == .ARM64] `__sifields :: struct`
- [OS == .ANDROID && CPU == .ARM64] `siginfo :: struct`
- [OS == .ANDROID && CPU == .ARM64] `siginfo_t :: siginfo`
- [OS == .ANDROID && CPU == .ARM64] `sigevent :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sigevent_t :: sigevent`
- [OS == .ANDROID && CPU == .ARM64] `sig_atomic_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `sig_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .ANDROID && CPU == .ARM64] `sighandler_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .ANDROID && CPU == .ARM64] `sigset64_t :: sigset_t`
- [OS == .ANDROID && CPU == .ARM64] `sigaction_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sigaction64_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `timespec :: struct`
- [OS == .ANDROID && CPU == .ARM64] `user_regs_struct :: struct`
- [OS == .ANDROID && CPU == .ARM64] `user_fpsimd_struct :: struct`
- [OS == .ANDROID && CPU == .ARM64] `greg_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `gregset_t :: [34] greg_t`
- [OS == .ANDROID && CPU == .ARM64] `fpregset_t :: user_fpsimd_struct`
- [OS == .ANDROID && CPU == .ARM64] `mcontext_t :: sigcontext`
- [OS == .ANDROID && CPU == .ARM64] `ucontext :: struct`
- [OS == .ANDROID && CPU == .ARM64] `ucontext_t :: ucontext`
- [OS == .ANDROID && CPU == .ARM64] `__libc_current_sigrtmin :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__libc_current_sigrtmax :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sys_siglist: [65] *u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `sys_signame: [65] *u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `sigaction :: (__signal: s32, __new_action: *sigaction_t, __old_action: *sigaction_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigaction64 :: (__signal: s32, __new_action: *sigaction64_t, __old_action: *sigaction64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `siginterrupt :: (__signal: s32, __flag: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `signal :: (__signal: s32, __handler: sighandler_t) -> sighandler_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigaddset :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigaddset64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigdelset :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigdelset64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigemptyset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigemptyset64 :: (__set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigfillset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigfillset64 :: (__set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigismember :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigismember64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigpending :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigpending64 :: (__set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigprocmask :: (__how: s32, __new_set: *sigset_t, __old_set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigprocmask64 :: (__how: s32, __new_set: *sigset64_t, __old_set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigsuspend :: (__mask: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigsuspend64 :: (__mask: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigwait :: (__set: *sigset_t, __signal: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigwait64 :: (__set: *sigset64_t, __signal: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sighold :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigignore :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigpause :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigrelse :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigset :: (__signal: s32, __handler: sighandler_t) -> sighandler_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `raise :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `kill :: (__pid: pid_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `killpg :: (__pgrp: s32, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tgkill :: (__tgid: s32, __tid: s32, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigaltstack :: (__new_signal_stack: *stack_t, __old_signal_stack: *stack_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `psiginfo :: (__info: *siginfo_t, __msg: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `psignal :: (__signal: s32, __msg: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigqueue :: (__pid: pid_t, __signal: s32, __value: sigval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigtimedwait :: (__set: *sigset_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigtimedwait64 :: (__set: *sigset64_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigwaitinfo :: (__set: *sigset_t, __info: *siginfo_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigwaitinfo64 :: (__set: *sigset64_t, __info: *siginfo_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ipc_kludge :: struct`
- [OS == .ANDROID && CPU == .ARM64] `ftok :: (__path: *u8, __id: s32) -> key_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `iovec :: struct`
- [OS == .ANDROID && CPU == .ARM64] `mmap :: (__addr: *void, __size: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mmap64 :: (__addr: *void, __size: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `munmap :: (__addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `msync :: (__addr: *void, __size: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mprotect :: (__addr: *void, __size: u64, __prot: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mremap :: (__old_addr: *void, __old_size: u64, __new_size: u64, __flags: s32, __args: ..Any) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mlockall :: (__flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `munlockall :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mlock :: (__addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mlock2 :: (__addr: *void, __size: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `munlock :: (__addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mincore :: (__addr: *void, __size: u64, __vector: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `madvise :: (__addr: *void, __size: u64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `process_madvise :: (__pid_fd: s32, __iov: *iovec, __count: u64, __advice: s32, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `memfd_create :: (__name: *u8, __flags: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `posix_madvise :: (__addr: *void, __size: u64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pollfd :: struct`
- [OS == .ANDROID && CPU == .ARM64] `timeval :: struct`
- [OS == .ANDROID && CPU == .ARM64] `itimerspec :: struct`
- [OS == .ANDROID && CPU == .ARM64] `itimerval :: struct`
- [OS == .ANDROID && CPU == .ARM64] `timezone :: struct`
- [OS == .ANDROID && CPU == .ARM64] `gettimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `settimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getitimer :: (__which: s32, __current_value: *itimerval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setitimer :: (__which: s32, __new_value: *itimerval, __old_value: *itimerval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `utimes :: (__path: *u8, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `futimes :: (__fd: s32, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `lutimes :: (__path: *u8, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `futimesat :: (__dir_fd: s32, __path: *u8, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__locale_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `locale_t :: *__locale_t`
- [OS == .ANDROID && CPU == .ARM64] `tzname: **u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `daylight: s32 #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `tm :: struct`
- [OS == .ANDROID && CPU == .ARM64] `time :: (__t: *time_t) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `nanosleep :: (__request: *timespec, __remainder: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `asctime :: (__tm: *tm) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `asctime_r :: (__tm: *tm, __buf: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `difftime :: (__lhs: time_t, __rhs: time_t) -> float64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mktime :: (__tm: *tm) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `localtime :: (__t: *time_t) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `localtime_r :: (__t: *time_t, __tm: *tm) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timelocal :: (__tm: *tm) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `gmtime :: (__t: *time_t) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `gmtime_r :: (__t: *time_t, __tm: *tm) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timegm :: (__tm: *tm) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `strptime :: (__s: *u8, __fmt: *u8, __tm: *tm) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `strptime_l :: (__s: *u8, __fmt: *u8, __tm: *tm, __l: locale_t) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `strftime :: (__buf: *u8, __n: u64, __fmt: *u8, __tm: *tm) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `strftime_l :: (__buf: *u8, __n: u64, __fmt: *u8, __tm: *tm, __l: locale_t) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ctime :: (__t: *time_t) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ctime_r :: (__t: *time_t, __buf: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tzset :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clock :: () -> clock_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clock_getcpuclockid :: (__pid: pid_t, __clock: *clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clock_getres :: (__clock: clockid_t, __resolution: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clock_gettime :: (__clock: clockid_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clock_nanosleep :: (__clock: clockid_t, __flags: s32, __request: *timespec, __remainder: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clock_settime :: (__clock: clockid_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timer_create :: (__clock: clockid_t, __event: *sigevent, __timer_ptr: *timer_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timer_delete :: (__timer: timer_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timer_settime :: (__timer: timer_t, __flags: s32, __new_value: *itimerspec, __old_value: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timer_gettime :: (_timer: timer_t, __ts: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timer_getoverrun :: (__timer: timer_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timespec_get :: (__ts: *timespec, __base: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `nfds_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `poll :: (__fds: *pollfd, __count: nfds_t, __timeout_ms: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ppoll :: (__fds: *pollfd, __count: nfds_t, __timeout: *timespec, __mask: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ppoll64 :: (__fds: *pollfd, __count: nfds_t, __timeout: *timespec, __mask: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sysconf :: (name: _SC_definitions) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `environ: **u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `_exit :: (__status: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fork :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `vfork :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `gettid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpgid :: (__pid: pid_t) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setpgid :: (__pid: pid_t, __pgid: pid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getppid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpgrp :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setpgrp :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getsid :: (__pid: pid_t) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setsid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execvpe :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execve :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execl :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execlp :: (__file: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `execle :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fexecve :: (__fd: s32, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `nice :: (__incr: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setegid :: (__gid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `seteuid :: (__uid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setgid :: (__gid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setregid :: (__rgid: gid_t, __egid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setresgid :: (__rgid: gid_t, __egid: gid_t, __sgid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setresuid :: (__ruid: uid_t, __euid: uid_t, __suid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setreuid :: (__ruid: uid_t, __euid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setuid :: (__uid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getuid :: () -> uid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `geteuid :: () -> uid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getgid :: () -> gid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getegid :: () -> gid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getgroups :: (__size: s32, __list: *gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setgroups :: (__size: u64, __list: *gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getresuid :: (__ruid: *uid_t, __euid: *uid_t, __suid: *uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getresgid :: (__rgid: *gid_t, __egid: *gid_t, __sgid: *gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getlogin :: () -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getlogin_r :: (__buffer: *u8, __buffer_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fpathconf :: (__fd: s32, __name: PC) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pathconf :: (__path: *u8, __name: PC) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `access :: (__path: *u8, __mode: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `faccessat :: (__dirfd: s32, __path: *u8, __mode: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `link :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `linkat :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `unlink :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `unlinkat :: (__dirfd: s32, __path: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `chdir :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fchdir :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `rmdir :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pipe :: (__fds: *[2] s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pipe2 :: (__fds: *[2] s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `chroot :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `symlink :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `symlinkat :: (__old_path: *u8, __new_dir_fd: s32, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readlink :: (__path: *u8, __buf: *u8, __buf_size: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readlinkat :: (__dir_fd: s32, __path: *u8, __buf: *u8, __buf_size: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `chown :: (__path: *u8, __owner: uid_t, __group: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fchown :: (__fd: s32, __owner: uid_t, __group: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fchownat :: (__dir_fd: s32, __path: *u8, __owner: uid_t, __group: gid_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `lchown :: (__path: *u8, __owner: uid_t, __group: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getcwd :: (__buf: *u8, __size: u64) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sync :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `syncfs :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `close :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `read :: (__fd: s32, __buf: *void, __count: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `write :: (__fd: s32, __buf: *void, __count: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `dup :: (__old_fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `dup2 :: (__old_fd: s32, __new_fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `dup3 :: (__old_fd: s32, __new_fd: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fsync :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fdatasync :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `truncate :: (__path: *u8, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `lseek :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pread :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pwrite :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ftruncate :: (__fd: s32, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `truncate64 :: (__path: *u8, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `lseek64 :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pread64 :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pwrite64 :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ftruncate64 :: (__fd: s32, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pause :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `alarm :: (__seconds: u32) -> u32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sleep :: (__seconds: u32) -> u32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `usleep :: (__microseconds: useconds_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `gethostname :: (_buf: *u8, __buf_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sethostname :: (__name: *u8, __n: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `brk :: (__addr: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sbrk :: (__increment: ptrdiff_t) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `isatty :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ttyname :: (__fd: s32) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ttyname_r :: (__fd: s32, __buf: *u8, __buf_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `acct :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpagesize :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `daemon :: (__no_chdir: s32, __no_close: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tcgetpgrp :: (__fd: s32) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tcsetpgrp :: (__fd: s32, __pid: pid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getdomainname :: (__buf: *u8, __buf_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setdomainname :: (__name: *u8, __n: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `copy_file_range :: (__fd_in: s32, __off_in: *s64, __fd_out: s32, __off_out: *s64, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `swab :: (__src: *void, __dst: *void, __byte_count: s64) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `close_range :: (__min_fd: u32, __max_fd: u32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `shminfo :: struct`
- [OS == .ANDROID && CPU == .ARM64] `shm_info :: struct`
- [OS == .ANDROID && CPU == .ARM64] `shmatt_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `shmat :: (__shm_id: s32, __addr: *void, __flags: s32) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `shmdt :: (__addr: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `shmget :: (key: key_t, __size: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `utsname :: struct`
- [OS == .ANDROID && CPU == .ARM64] `uname :: (__buf: *utsname) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `wait :: (__status: *s32) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `waitpid :: (__pid: pid_t, __status: *s32, __options: s32) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `wait4 :: (__pid: pid_t, __status: *s32, __options: s32, __rusage: *rusage) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `idtype_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `waitid :: (__type: idtype_t, __id: id_t, __info: *siginfo_t, __options: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `_GNU_SOURCE :: 1`
- [OS == .ANDROID && CPU == .ARM64] `_IOFBF :: 0`
- [OS == .ANDROID && CPU == .ARM64] `_IOLBF :: 1`
- [OS == .ANDROID && CPU == .ARM64] `_IONBF :: 2`
- [OS == .ANDROID && CPU == .ARM64] `BUFSIZ :: 1024`
- [OS == .ANDROID && CPU == .ARM64] `EOF :: -1`
- [OS == .ANDROID && CPU == .ARM64] `FOPEN_MAX :: 20`
- [OS == .ANDROID && CPU == .ARM64] `FILENAME_MAX :: 4096`
- [OS == .ANDROID && CPU == .ARM64] `L_tmpnam :: 4096`
- [OS == .ANDROID && CPU == .ARM64] `TMP_MAX :: 308915776`
- [OS == .ANDROID && CPU == .ARM64] `P_tmpdir :: "/tmp/"`
- [OS == .ANDROID && CPU == .ARM64] `RENAME_NOREPLACE :: 1<<0`
- [OS == .ANDROID && CPU == .ARM64] `RENAME_EXCHANGE :: 1<<1`
- [OS == .ANDROID && CPU == .ARM64] `RENAME_WHITEOUT :: 1<<2`
- [OS == .ANDROID && CPU == .ARM64] `L_ctermid :: 1024`
- [OS == .ANDROID && CPU == .ARM64] `O_DIRECTORY :: 0x4000`
- [OS == .ANDROID && CPU == .ARM64] `O_NOFOLLOW :: 0x8000`
- [OS == .ANDROID && CPU == .ARM64] `O_DIRECT :: 0x10000`
- [OS == .ANDROID && CPU == .ARM64] `O_LARGEFILE :: 0x20000`
- [OS == .ANDROID && CPU == .ARM64] `O_ACCMODE :: 0x3`
- [OS == .ANDROID && CPU == .ARM64] `O_RDONLY :: 0x0`
- [OS == .ANDROID && CPU == .ARM64] `O_WRONLY :: 0x1`
- [OS == .ANDROID && CPU == .ARM64] `O_RDWR :: 0x2`
- [OS == .ANDROID && CPU == .ARM64] `O_CREAT :: 0x40`
- [OS == .ANDROID && CPU == .ARM64] `O_EXCL :: 0x80`
- [OS == .ANDROID && CPU == .ARM64] `O_NOCTTY :: 0x100`
- [OS == .ANDROID && CPU == .ARM64] `O_TRUNC :: 0x200`
- [OS == .ANDROID && CPU == .ARM64] `O_APPEND :: 0x400`
- [OS == .ANDROID && CPU == .ARM64] `O_NONBLOCK :: 0x800`
- [OS == .ANDROID && CPU == .ARM64] `O_DSYNC :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `FASYNC :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `O_NOATIME :: 0x40000`
- [OS == .ANDROID && CPU == .ARM64] `O_CLOEXEC :: 0x80000`
- [OS == .ANDROID && CPU == .ARM64] `__O_SYNC :: 0x100000`
- [OS == .ANDROID && CPU == .ARM64] `O_SYNC :: __O_SYNC | O_DSYNC`
- [OS == .ANDROID && CPU == .ARM64] `O_PATH :: 0x200000`
- [OS == .ANDROID && CPU == .ARM64] `__O_TMPFILE :: 0x400000`
- [OS == .ANDROID && CPU == .ARM64] `O_TMPFILE :: __O_TMPFILE | O_DIRECTORY`
- [OS == .ANDROID && CPU == .ARM64] `O_TMPFILE_MASK :: __O_TMPFILE | O_DIRECTORY | O_CREAT`
- [OS == .ANDROID && CPU == .ARM64] `O_NDELAY :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .ARM64] `F_DUPFD :: 0`
- [OS == .ANDROID && CPU == .ARM64] `F_GETFD :: 1`
- [OS == .ANDROID && CPU == .ARM64] `F_SETFD :: 2`
- [OS == .ANDROID && CPU == .ARM64] `F_GETFL :: 3`
- [OS == .ANDROID && CPU == .ARM64] `F_SETFL :: 4`
- [OS == .ANDROID && CPU == .ARM64] `F_GETLK :: 5`
- [OS == .ANDROID && CPU == .ARM64] `F_SETLK :: 6`
- [OS == .ANDROID && CPU == .ARM64] `F_SETLKW :: 7`
- [OS == .ANDROID && CPU == .ARM64] `F_SETOWN :: 8`
- [OS == .ANDROID && CPU == .ARM64] `F_GETOWN :: 9`
- [OS == .ANDROID && CPU == .ARM64] `F_SETSIG :: 10`
- [OS == .ANDROID && CPU == .ARM64] `F_GETSIG :: 11`
- [OS == .ANDROID && CPU == .ARM64] `F_SETOWN_EX :: 15`
- [OS == .ANDROID && CPU == .ARM64] `F_GETOWN_EX :: 16`
- [OS == .ANDROID && CPU == .ARM64] `F_GETOWNER_UIDS :: 17`
- [OS == .ANDROID && CPU == .ARM64] `F_OFD_GETLK :: 36`
- [OS == .ANDROID && CPU == .ARM64] `F_OFD_SETLK :: 37`
- [OS == .ANDROID && CPU == .ARM64] `F_OFD_SETLKW :: 38`
- [OS == .ANDROID && CPU == .ARM64] `F_OWNER_TID :: 0`
- [OS == .ANDROID && CPU == .ARM64] `F_OWNER_PID :: 1`
- [OS == .ANDROID && CPU == .ARM64] `F_OWNER_PGRP :: 2`
- [OS == .ANDROID && CPU == .ARM64] `FD_CLOEXEC :: 1`
- [OS == .ANDROID && CPU == .ARM64] `F_RDLCK :: 0`
- [OS == .ANDROID && CPU == .ARM64] `F_WRLCK :: 1`
- [OS == .ANDROID && CPU == .ARM64] `F_UNLCK :: 2`
- [OS == .ANDROID && CPU == .ARM64] `F_EXLCK :: 4`
- [OS == .ANDROID && CPU == .ARM64] `F_SHLCK :: 8`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_SH :: 1`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_EX :: 2`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_NB :: 4`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_UN :: 8`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_MAND :: 32`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_READ :: 64`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_WRITE :: 128`
- [OS == .ANDROID && CPU == .ARM64] `LOCK_RW :: 192`
- [OS == .ANDROID && CPU == .ARM64] `F_LINUX_SPECIFIC_BASE :: 1024`
- [OS == .ANDROID && CPU == .ARM64] `F_SETLEASE :: F_LINUX_SPECIFIC_BASE + 0`
- [OS == .ANDROID && CPU == .ARM64] `F_GETLEASE :: F_LINUX_SPECIFIC_BASE + 1`
- [OS == .ANDROID && CPU == .ARM64] `F_CANCELLK :: F_LINUX_SPECIFIC_BASE + 5`
- [OS == .ANDROID && CPU == .ARM64] `F_DUPFD_CLOEXEC :: F_LINUX_SPECIFIC_BASE + 6`
- [OS == .ANDROID && CPU == .ARM64] `F_NOTIFY :: F_LINUX_SPECIFIC_BASE + 2`
- [OS == .ANDROID && CPU == .ARM64] `F_SETPIPE_SZ :: F_LINUX_SPECIFIC_BASE + 7`
- [OS == .ANDROID && CPU == .ARM64] `F_GETPIPE_SZ :: F_LINUX_SPECIFIC_BASE + 8`
- [OS == .ANDROID && CPU == .ARM64] `F_ADD_SEALS :: F_LINUX_SPECIFIC_BASE + 9`
- [OS == .ANDROID && CPU == .ARM64] `F_GET_SEALS :: F_LINUX_SPECIFIC_BASE + 10`
- [OS == .ANDROID && CPU == .ARM64] `F_SEAL_SEAL :: 0x0001`
- [OS == .ANDROID && CPU == .ARM64] `F_SEAL_SHRINK :: 0x0002`
- [OS == .ANDROID && CPU == .ARM64] `F_SEAL_GROW :: 0x0004`
- [OS == .ANDROID && CPU == .ARM64] `F_SEAL_WRITE :: 0x0008`
- [OS == .ANDROID && CPU == .ARM64] `F_SEAL_FUTURE_WRITE :: 0x0010`
- [OS == .ANDROID && CPU == .ARM64] `F_SEAL_EXEC :: 0x0020`
- [OS == .ANDROID && CPU == .ARM64] `F_GET_RW_HINT :: F_LINUX_SPECIFIC_BASE + 11`
- [OS == .ANDROID && CPU == .ARM64] `F_SET_RW_HINT :: F_LINUX_SPECIFIC_BASE + 12`
- [OS == .ANDROID && CPU == .ARM64] `F_GET_FILE_RW_HINT :: F_LINUX_SPECIFIC_BASE + 13`
- [OS == .ANDROID && CPU == .ARM64] `F_SET_FILE_RW_HINT :: F_LINUX_SPECIFIC_BASE + 14`
- [OS == .ANDROID && CPU == .ARM64] `RWH_WRITE_LIFE_NOT_SET :: 0`
- [OS == .ANDROID && CPU == .ARM64] `RWH_WRITE_LIFE_NONE :: 1`
- [OS == .ANDROID && CPU == .ARM64] `RWH_WRITE_LIFE_SHORT :: 2`
- [OS == .ANDROID && CPU == .ARM64] `RWH_WRITE_LIFE_MEDIUM :: 3`
- [OS == .ANDROID && CPU == .ARM64] `RWH_WRITE_LIFE_LONG :: 4`
- [OS == .ANDROID && CPU == .ARM64] `RWH_WRITE_LIFE_EXTREME :: 5`
- [OS == .ANDROID && CPU == .ARM64] `RWF_WRITE_LIFE_NOT_SET :: RWH_WRITE_LIFE_NOT_SET`
- [OS == .ANDROID && CPU == .ARM64] `DN_ACCESS :: 0x00000001`
- [OS == .ANDROID && CPU == .ARM64] `DN_MODIFY :: 0x00000002`
- [OS == .ANDROID && CPU == .ARM64] `DN_CREATE :: 0x00000004`
- [OS == .ANDROID && CPU == .ARM64] `DN_DELETE :: 0x00000008`
- [OS == .ANDROID && CPU == .ARM64] `DN_RENAME :: 0x00000010`
- [OS == .ANDROID && CPU == .ARM64] `DN_ATTRIB :: 0x00000020`
- [OS == .ANDROID && CPU == .ARM64] `DN_MULTISHOT :: 0x80000000`
- [OS == .ANDROID && CPU == .ARM64] `AT_FDCWD :: - 100`
- [OS == .ANDROID && CPU == .ARM64] `AT_SYMLINK_NOFOLLOW :: 0x100`
- [OS == .ANDROID && CPU == .ARM64] `AT_EACCESS :: 0x200`
- [OS == .ANDROID && CPU == .ARM64] `AT_REMOVEDIR :: 0x200`
- [OS == .ANDROID && CPU == .ARM64] `AT_SYMLINK_FOLLOW :: 0x400`
- [OS == .ANDROID && CPU == .ARM64] `AT_NO_AUTOMOUNT :: 0x800`
- [OS == .ANDROID && CPU == .ARM64] `AT_EMPTY_PATH :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `AT_STATX_SYNC_TYPE :: 0x6000`
- [OS == .ANDROID && CPU == .ARM64] `AT_STATX_SYNC_AS_STAT :: 0x0000`
- [OS == .ANDROID && CPU == .ARM64] `AT_STATX_FORCE_SYNC :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `AT_STATX_DONT_SYNC :: 0x4000`
- [OS == .ANDROID && CPU == .ARM64] `AT_RECURSIVE :: 0x8000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFMT :: 0xf000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFSOCK :: 0xc000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFLNK :: 0xa000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFREG :: 0x8000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFBLK :: 0x6000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFDIR :: 0x4000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFCHR :: 0x2000`
- [OS == .ANDROID && CPU == .ARM64] `S_IFIFO :: 0x1000`
- [OS == .ANDROID && CPU == .ARM64] `S_ISUID :: 0x800`
- [OS == .ANDROID && CPU == .ARM64] `S_ISGID :: 0x400`
- [OS == .ANDROID && CPU == .ARM64] `S_ISVTX :: 0x200`
- [OS == .ANDROID && CPU == .ARM64] `S_IRWXU :: 0x1c0`
- [OS == .ANDROID && CPU == .ARM64] `S_IRUSR :: 0x100`
- [OS == .ANDROID && CPU == .ARM64] `S_IWUSR :: 0x80`
- [OS == .ANDROID && CPU == .ARM64] `S_IXUSR :: 0x40`
- [OS == .ANDROID && CPU == .ARM64] `S_IRWXG :: 0x38`
- [OS == .ANDROID && CPU == .ARM64] `S_IRGRP :: 0x20`
- [OS == .ANDROID && CPU == .ARM64] `S_IWGRP :: 0x10`
- [OS == .ANDROID && CPU == .ARM64] `S_IXGRP :: 0x8`
- [OS == .ANDROID && CPU == .ARM64] `S_IRWXO :: 0x7`
- [OS == .ANDROID && CPU == .ARM64] `S_IROTH :: 0x4`
- [OS == .ANDROID && CPU == .ARM64] `S_IWOTH :: 0x2`
- [OS == .ANDROID && CPU == .ARM64] `S_IXOTH :: 0x1`
- [OS == .ANDROID && CPU == .ARM64] `STATX_TYPE :: 0x00000001`
- [OS == .ANDROID && CPU == .ARM64] `STATX_MODE :: 0x00000002`
- [OS == .ANDROID && CPU == .ARM64] `STATX_NLINK :: 0x00000004`
- [OS == .ANDROID && CPU == .ARM64] `STATX_UID :: 0x00000008`
- [OS == .ANDROID && CPU == .ARM64] `STATX_GID :: 0x00000010`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATIME :: 0x00000020`
- [OS == .ANDROID && CPU == .ARM64] `STATX_MTIME :: 0x00000040`
- [OS == .ANDROID && CPU == .ARM64] `STATX_CTIME :: 0x00000080`
- [OS == .ANDROID && CPU == .ARM64] `STATX_INO :: 0x00000100`
- [OS == .ANDROID && CPU == .ARM64] `STATX_SIZE :: 0x00000200`
- [OS == .ANDROID && CPU == .ARM64] `STATX_BLOCKS :: 0x00000400`
- [OS == .ANDROID && CPU == .ARM64] `STATX_BASIC_STATS :: 0x000007ff`
- [OS == .ANDROID && CPU == .ARM64] `STATX_BTIME :: 0x00000800`
- [OS == .ANDROID && CPU == .ARM64] `STATX_MNT_ID :: 0x00001000`
- [OS == .ANDROID && CPU == .ARM64] `STATX_DIOALIGN :: 0x00002000`
- [OS == .ANDROID && CPU == .ARM64] `STATX__RESERVED :: 0x80000000`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ALL :: 0x00000fff`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_COMPRESSED :: 0x00000004`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_IMMUTABLE :: 0x00000010`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_APPEND :: 0x00000020`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_NODUMP :: 0x00000040`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_ENCRYPTED :: 0x00000800`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_AUTOMOUNT :: 0x00001000`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_MOUNT_ROOT :: 0x00002000`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_VERITY :: 0x00100000`
- [OS == .ANDROID && CPU == .ARM64] `STATX_ATTR_DAX :: 0x00200000`
- [OS == .ANDROID && CPU == .ARM64] `F_GETLK64 :: F_GETLK`
- [OS == .ANDROID && CPU == .ARM64] `F_SETLK64 :: F_SETLK`
- [OS == .ANDROID && CPU == .ARM64] `F_SETLKW64 :: F_SETLKW`
- [OS == .ANDROID && CPU == .ARM64] `O_ASYNC :: FASYNC`
- [OS == .ANDROID && CPU == .ARM64] `O_RSYNC :: O_SYNC`
- [OS == .ANDROID && CPU == .ARM64] `SPLICE_F_MOVE :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SPLICE_F_NONBLOCK :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SPLICE_F_MORE :: 4`
- [OS == .ANDROID && CPU == .ARM64] `SPLICE_F_GIFT :: 8`
- [OS == .ANDROID && CPU == .ARM64] `SYNC_FILE_RANGE_WAIT_BEFORE :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SYNC_FILE_RANGE_WRITE :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SYNC_FILE_RANGE_WAIT_AFTER :: 4`
- [OS == .ANDROID && CPU == .ARM64] `ACCESSPERMS :: S_IRWXU | S_IRWXG | S_IRWXO`
- [OS == .ANDROID && CPU == .ARM64] `ALLPERMS :: S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO`
- [OS == .ANDROID && CPU == .ARM64] `DEFFILEMODE :: S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH`
- [OS == .ANDROID && CPU == .ARM64] `S_IREAD :: S_IRUSR`
- [OS == .ANDROID && CPU == .ARM64] `S_IWRITE :: S_IWUSR`
- [OS == .ANDROID && CPU == .ARM64] `S_IEXEC :: S_IXUSR`
- [OS == .ANDROID && CPU == .ARM64] `UTIME_NOW :: (1 << 30) - 1`
- [OS == .ANDROID && CPU == .ARM64] `UTIME_OMIT :: (1 << 30) - 2`
- [OS == .ANDROID && CPU == .ARM64] `TCGETS :: 0x5401`
- [OS == .ANDROID && CPU == .ARM64] `TCSETS :: 0x5402`
- [OS == .ANDROID && CPU == .ARM64] `TCSETSW :: 0x5403`
- [OS == .ANDROID && CPU == .ARM64] `TCSETSF :: 0x5404`
- [OS == .ANDROID && CPU == .ARM64] `TCGETA :: 0x5405`
- [OS == .ANDROID && CPU == .ARM64] `TCSETA :: 0x5406`
- [OS == .ANDROID && CPU == .ARM64] `TCSETAW :: 0x5407`
- [OS == .ANDROID && CPU == .ARM64] `TCSETAF :: 0x5408`
- [OS == .ANDROID && CPU == .ARM64] `TCSBRK :: 0x5409`
- [OS == .ANDROID && CPU == .ARM64] `TCXONC :: 0x540A`
- [OS == .ANDROID && CPU == .ARM64] `TCFLSH :: 0x540B`
- [OS == .ANDROID && CPU == .ARM64] `TIOCEXCL :: 0x540C`
- [OS == .ANDROID && CPU == .ARM64] `TIOCNXCL :: 0x540D`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSCTTY :: 0x540E`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGPGRP :: 0x540F`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSPGRP :: 0x5410`
- [OS == .ANDROID && CPU == .ARM64] `TIOCOUTQ :: 0x5411`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSTI :: 0x5412`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGWINSZ :: 0x5413`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSWINSZ :: 0x5414`
- [OS == .ANDROID && CPU == .ARM64] `TIOCMGET :: 0x5415`
- [OS == .ANDROID && CPU == .ARM64] `TIOCMBIS :: 0x5416`
- [OS == .ANDROID && CPU == .ARM64] `TIOCMBIC :: 0x5417`
- [OS == .ANDROID && CPU == .ARM64] `TIOCMSET :: 0x5418`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGSOFTCAR :: 0x5419`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSSOFTCAR :: 0x541A`
- [OS == .ANDROID && CPU == .ARM64] `FIONREAD :: 0x541B`
- [OS == .ANDROID && CPU == .ARM64] `TIOCINQ :: FIONREAD`
- [OS == .ANDROID && CPU == .ARM64] `TIOCLINUX :: 0x541C`
- [OS == .ANDROID && CPU == .ARM64] `TIOCCONS :: 0x541D`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGSERIAL :: 0x541E`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSSERIAL :: 0x541F`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT :: 0x5420`
- [OS == .ANDROID && CPU == .ARM64] `FIONBIO :: 0x5421`
- [OS == .ANDROID && CPU == .ARM64] `TIOCNOTTY :: 0x5422`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSETD :: 0x5423`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGETD :: 0x5424`
- [OS == .ANDROID && CPU == .ARM64] `TCSBRKP :: 0x5425`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSBRK :: 0x5427`
- [OS == .ANDROID && CPU == .ARM64] `TIOCCBRK :: 0x5428`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGSID :: 0x5429`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGRS485 :: 0x542E`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSRS485 :: 0x542F`
- [OS == .ANDROID && CPU == .ARM64] `TCGETX :: 0x5432`
- [OS == .ANDROID && CPU == .ARM64] `TCSETX :: 0x5433`
- [OS == .ANDROID && CPU == .ARM64] `TCSETXF :: 0x5434`
- [OS == .ANDROID && CPU == .ARM64] `TCSETXW :: 0x5435`
- [OS == .ANDROID && CPU == .ARM64] `TIOCVHANGUP :: 0x5437`
- [OS == .ANDROID && CPU == .ARM64] `FIONCLEX :: 0x5450`
- [OS == .ANDROID && CPU == .ARM64] `FIOCLEX :: 0x5451`
- [OS == .ANDROID && CPU == .ARM64] `FIOASYNC :: 0x5452`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERCONFIG :: 0x5453`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERGWILD :: 0x5454`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERSWILD :: 0x5455`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGLCKTRMIOS :: 0x5456`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSLCKTRMIOS :: 0x5457`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERGSTRUCT :: 0x5458`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERGETLSR :: 0x5459`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERGETMULTI :: 0x545A`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSERSETMULTI :: 0x545B`
- [OS == .ANDROID && CPU == .ARM64] `TIOCMIWAIT :: 0x545C`
- [OS == .ANDROID && CPU == .ARM64] `TIOCGICOUNT :: 0x545D`
- [OS == .ANDROID && CPU == .ARM64] `FIOQSIZE :: 0x5460`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_DATA :: 0`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_FLUSHREAD :: 1`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_FLUSHWRITE :: 2`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_STOP :: 4`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_START :: 8`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_NOSTOP :: 16`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_DOSTOP :: 32`
- [OS == .ANDROID && CPU == .ARM64] `TIOCPKT_IOCTL :: 64`
- [OS == .ANDROID && CPU == .ARM64] `TIOCSER_TEMT :: 0x01`
- [OS == .ANDROID && CPU == .ARM64] `EFD_SEMAPHORE :: 1 << 0`
- [OS == .ANDROID && CPU == .ARM64] `EFD_CLOEXEC :: O_CLOEXEC`
- [OS == .ANDROID && CPU == .ARM64] `EFD_NONBLOCK :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .ARM64] `TFD_CLOEXEC :: O_CLOEXEC`
- [OS == .ANDROID && CPU == .ARM64] `TFD_NONBLOCK :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .ARM64] `TFD_TIMER_ABSTIME :: 1 << 0`
- [OS == .ANDROID && CPU == .ARM64] `TFD_TIMER_CANCEL_ON_SET :: 1 << 1`
- [OS == .ANDROID && CPU == .ARM64] `SFD_CLOEXEC :: O_CLOEXEC`
- [OS == .ANDROID && CPU == .ARM64] `SFD_NONBLOCK :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .ARM64] `fpos_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `fpos64_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `__sFILE :: struct`
- [OS == .ANDROID && CPU == .ARM64] `FILE :: __sFILE`
- [OS == .ANDROID && CPU == .ARM64] `stdin: *FILE #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `stdout: *FILE #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `stderr: *FILE #elsewhere libc`
- [OS == .ANDROID && CPU == .ARM64] `clearerr :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fclose :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `feof :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ferror :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fflush :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgetc :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgets :: (__buf: *u8, __size: s32, __fp: *FILE) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fprintf :: (__fp: *FILE, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fputc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fputs :: (__s: *u8, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fread :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fscanf :: (__fp: *FILE, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fwrite :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getc :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getchar :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getdelim :: (__line_ptr: **u8, __line_length_ptr: *u64, __delimiter: s32, __fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getline :: (__line_ptr: **u8, __line_length_ptr: *u64, __fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `perror :: (__msg: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `printf :: (__fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `putc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `putchar :: (__ch: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `puts :: (__s: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `_remove :: (__path: *u8) -> s32 #foreign libc "remove"`
- [OS == .ANDROID && CPU == .ARM64] `rewind :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `scanf :: (__fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setbuf :: (__fp: *FILE, __buf: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setvbuf :: (__fp: *FILE, __buf: *u8, __mode: s32, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sscanf :: (__s: *u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ungetc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `dprintf :: (__fd: s32, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sprintf :: (__s: *u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tmpnam :: (__s: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tempnam :: (__dir: *u8, __prefix: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `rename :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `renameat :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `renameat2 :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8, __flags: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fseek :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ftell :: (__fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgetpos :: (__fp: *FILE, __pos: *fpos_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fsetpos :: (__fp: *FILE, __pos: *fpos_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fseeko :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ftello :: (__fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `funopen :: (__cookie: *void, __read_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __write_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __seek_fn: #type (unknown0: *void, unknown1: fpos_t, unknown2: s32) -> fpos_t #c_call, __close_fn: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgetpos64 :: (__fp: *FILE, __pos: *fpos64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fsetpos64 :: (__fp: *FILE, __pos: *fpos64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fseeko64 :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ftello64 :: (__fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `funopen64 :: (__cookie: *void, __read_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __write_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __seek_fn: #type (unknown0: *void, unknown1: fpos64_t, unknown2: s32) -> fpos64_t #c_call, __close_fn: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fopen :: (__path: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fopen64 :: (__path: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `freopen :: (__path: *u8, __mode: *u8, __fp: *FILE) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `freopen64 :: (__path: *u8, __mode: *u8, __fp: *FILE) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tmpfile :: () -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tmpfile64 :: () -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `snprintf :: (__buf: *u8, __size: u64, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ctermid :: (__buf: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fdopen :: (__fd: s32, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fileno :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pclose :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `popen :: (__command: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `flockfile :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ftrylockfile :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `funlockfile :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getc_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getchar_unlocked :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `putc_unlocked :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `putchar_unlocked :: (__ch: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fmemopen :: (__buf: *void, __size: u64, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `open_memstream :: (__ptr: **u8, __size_ptr: *u64) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `asprintf :: (__s_ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgetln :: (__fp: *FILE, __length_ptr: *u64) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fpurge :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setbuffer :: (__fp: *FILE, __buf: *u8, __size: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setlinebuf :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clearerr_unlocked :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `feof_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `ferror_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fileno_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fflush_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgetc_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fputc_unlocked :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fread_unlocked :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fwrite_unlocked :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fputs_unlocked :: (__s: *u8, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fgets_unlocked :: (__buf: *u8, __size: s32, __fp: *FILE) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `f_owner_ex :: struct`
- [OS == .ANDROID && CPU == .ARM64] `statx_timestamp :: struct`
- [OS == .ANDROID && CPU == .ARM64] `statx_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `fcntl :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `creat :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `creat64 :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `openat :: (__dir_fd: s32, __path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `openat64 :: (__dir_fd: s32, __path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `open :: (__path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `open64 :: (__path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `splice :: (__in_fd: s32, __in_offset: *s64, __out_fd: s32, __out_offset: *s64, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `tee :: (__in_fd: s32, __out_fd: s32, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `vmsplice :: (__fd: s32, __iov: *iovec, __count: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fallocate :: (__fd: s32, __mode: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fallocate64 :: (__fd: s32, __mode: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `posix_fadvise :: (__fd: s32, __offset: s64, __length: s64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `posix_fadvise64 :: (__fd: s32, __offset: s64, __length: s64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `posix_fallocate :: (__fd: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `posix_fallocate64 :: (__fd: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `readahead :: (__fd: s32, __offset: s64, __length: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sync_file_range :: (__fd: s32, __offset: s64, __length: s64, __flags: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `stat_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `stat64_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `chmod :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fchmod :: (__fd: s32, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkdir :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fstat :: (__fd: s32, __buf: *stat_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fstat64 :: (__fd: s32, __buf: *stat64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fstatat :: (__dir_fd: s32, __path: *u8, __buf: *stat_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fstatat64 :: (__dir_fd: s32, __path: *u8, __buf: *stat64_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `lstat :: (__path: *u8, __buf: *stat_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `lstat64 :: (__path: *u8, __buf: *stat64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `stat :: (__path: *u8, __buf: *stat_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `stat64 :: (__path: *u8, __buf: *stat64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mknod :: (__path: *u8, __mode: mode_t, __dev: dev_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `umask :: (__mask: mode_t) -> mode_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkfifo :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkfifoat :: (__dir_fd: s32, __path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `fchmodat :: (__dir_fd: s32, __path: *u8, __mode: mode_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkdirat :: (__dir_fd: s32, __path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mknodat :: (__dir_fd: s32, __path: *u8, __mode: mode_t, __dev: dev_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `utimensat :: (__dir_fd: s32, __path: *u8, __times: *[2] timespec, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `statx :: (__dir_fd: s32, __path: *u8, __flags: s32, __mask: u32, __buf: *statx_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `eventfd :: (__initial_value: u32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `eventfd_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `eventfd_read :: (__fd: s32, __value: *eventfd_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `eventfd_write :: (__fd: s32, __value: eventfd_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timerfd_create :: (__clock: clockid_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timerfd_settime :: (__fd: s32, __flags: s32, __new_value: *itimerspec, __old_value: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `timerfd_gettime :: (__fd: s32, __current_value: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `signalfd_siginfo :: struct`
- [OS == .ANDROID && CPU == .ARM64] `signalfd :: (__fd: s32, __mask: *sigset_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `signalfd64 :: (__fd: s32, __mask: *sigset64_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getenv :: (__name: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setenv :: (__name: *u8, __value: *u8, __overwrite: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `unsetenv :: (__name: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkdtemp :: (__template: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mktemp :: (__template: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkostemp :: (__template: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkostemps :: (__template: *u8, __suffix_length: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkstemp :: (__template: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `mkstemps :: (__template: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `realpath :: (__path: *u8, __resolved: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `system :: (__command: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `SYS_accept :: 202`
- [OS == .ANDROID && CPU == .ARM64] `SYS_accept4 :: 242`
- [OS == .ANDROID && CPU == .ARM64] `SYS_acct :: 89`
- [OS == .ANDROID && CPU == .ARM64] `SYS_add_key :: 217`
- [OS == .ANDROID && CPU == .ARM64] `SYS_adjtimex :: 171`
- [OS == .ANDROID && CPU == .ARM64] `SYS_arch_specific_syscall :: 244`
- [OS == .ANDROID && CPU == .ARM64] `SYS_bind :: 200`
- [OS == .ANDROID && CPU == .ARM64] `SYS_bpf :: 280`
- [OS == .ANDROID && CPU == .ARM64] `SYS_brk :: 214`
- [OS == .ANDROID && CPU == .ARM64] `SYS_capget :: 90`
- [OS == .ANDROID && CPU == .ARM64] `SYS_capset :: 91`
- [OS == .ANDROID && CPU == .ARM64] `SYS_chdir :: 49`
- [OS == .ANDROID && CPU == .ARM64] `SYS_chroot :: 51`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clock_adjtime :: 266`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clock_getres :: 114`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clock_gettime :: 113`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clock_nanosleep :: 115`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clock_settime :: 112`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clone :: 220`
- [OS == .ANDROID && CPU == .ARM64] `SYS_clone3 :: 435`
- [OS == .ANDROID && CPU == .ARM64] `SYS_close :: 57`
- [OS == .ANDROID && CPU == .ARM64] `SYS_close_range :: 436`
- [OS == .ANDROID && CPU == .ARM64] `SYS_connect :: 203`
- [OS == .ANDROID && CPU == .ARM64] `SYS_copy_file_range :: 285`
- [OS == .ANDROID && CPU == .ARM64] `SYS_delete_module :: 106`
- [OS == .ANDROID && CPU == .ARM64] `SYS_dup :: 23`
- [OS == .ANDROID && CPU == .ARM64] `SYS_dup3 :: 24`
- [OS == .ANDROID && CPU == .ARM64] `SYS_epoll_create1 :: 20`
- [OS == .ANDROID && CPU == .ARM64] `SYS_epoll_ctl :: 21`
- [OS == .ANDROID && CPU == .ARM64] `SYS_epoll_pwait :: 22`
- [OS == .ANDROID && CPU == .ARM64] `SYS_epoll_pwait2 :: 441`
- [OS == .ANDROID && CPU == .ARM64] `SYS_eventfd2 :: 19`
- [OS == .ANDROID && CPU == .ARM64] `SYS_execve :: 221`
- [OS == .ANDROID && CPU == .ARM64] `SYS_execveat :: 281`
- [OS == .ANDROID && CPU == .ARM64] `SYS_exit :: 93`
- [OS == .ANDROID && CPU == .ARM64] `SYS_exit_group :: 94`
- [OS == .ANDROID && CPU == .ARM64] `SYS_faccessat :: 48`
- [OS == .ANDROID && CPU == .ARM64] `SYS_faccessat2 :: 439`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fadvise64 :: 223`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fallocate :: 47`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fanotify_init :: 262`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fanotify_mark :: 263`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fchdir :: 50`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fchmod :: 52`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fchmodat :: 53`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fchown :: 55`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fchownat :: 54`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fcntl :: 25`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fdatasync :: 83`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fgetxattr :: 10`
- [OS == .ANDROID && CPU == .ARM64] `SYS_finit_module :: 273`
- [OS == .ANDROID && CPU == .ARM64] `SYS_flistxattr :: 13`
- [OS == .ANDROID && CPU == .ARM64] `SYS_flock :: 32`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fremovexattr :: 16`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fsconfig :: 431`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fsetxattr :: 7`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fsmount :: 432`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fsopen :: 430`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fspick :: 433`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fstat :: 80`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fstatfs :: 44`
- [OS == .ANDROID && CPU == .ARM64] `SYS_fsync :: 82`
- [OS == .ANDROID && CPU == .ARM64] `SYS_ftruncate :: 46`
- [OS == .ANDROID && CPU == .ARM64] `SYS_futex :: 98`
- [OS == .ANDROID && CPU == .ARM64] `SYS_futex_waitv :: 449`
- [OS == .ANDROID && CPU == .ARM64] `SYS_get_mempolicy :: 236`
- [OS == .ANDROID && CPU == .ARM64] `SYS_get_robust_list :: 100`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getcpu :: 168`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getcwd :: 17`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getdents64 :: 61`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getegid :: 177`
- [OS == .ANDROID && CPU == .ARM64] `SYS_geteuid :: 175`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getgid :: 176`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getgroups :: 158`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getitimer :: 102`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getpeername :: 205`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getpgid :: 155`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getpid :: 172`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getppid :: 173`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getpriority :: 141`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getrandom :: 278`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getresgid :: 150`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getresuid :: 148`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getrlimit :: 163`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getrusage :: 165`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getsid :: 156`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getsockname :: 204`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getsockopt :: 209`
- [OS == .ANDROID && CPU == .ARM64] `SYS_gettid :: 178`
- [OS == .ANDROID && CPU == .ARM64] `SYS_gettimeofday :: 169`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getuid :: 174`
- [OS == .ANDROID && CPU == .ARM64] `SYS_getxattr :: 8`
- [OS == .ANDROID && CPU == .ARM64] `SYS_init_module :: 105`
- [OS == .ANDROID && CPU == .ARM64] `SYS_inotify_add_watch :: 27`
- [OS == .ANDROID && CPU == .ARM64] `SYS_inotify_init1 :: 26`
- [OS == .ANDROID && CPU == .ARM64] `SYS_inotify_rm_watch :: 28`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_cancel :: 3`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_destroy :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_getevents :: 4`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_pgetevents :: 292`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_setup :: 0`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_submit :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_uring_enter :: 426`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_uring_register :: 427`
- [OS == .ANDROID && CPU == .ARM64] `SYS_io_uring_setup :: 425`
- [OS == .ANDROID && CPU == .ARM64] `SYS_ioctl :: 29`
- [OS == .ANDROID && CPU == .ARM64] `SYS_ioprio_get :: 31`
- [OS == .ANDROID && CPU == .ARM64] `SYS_ioprio_set :: 30`
- [OS == .ANDROID && CPU == .ARM64] `SYS_kcmp :: 272`
- [OS == .ANDROID && CPU == .ARM64] `SYS_kexec_file_load :: 294`
- [OS == .ANDROID && CPU == .ARM64] `SYS_kexec_load :: 104`
- [OS == .ANDROID && CPU == .ARM64] `SYS_keyctl :: 219`
- [OS == .ANDROID && CPU == .ARM64] `SYS_kill :: 129`
- [OS == .ANDROID && CPU == .ARM64] `SYS_landlock_add_rule :: 445`
- [OS == .ANDROID && CPU == .ARM64] `SYS_landlock_create_ruleset :: 444`
- [OS == .ANDROID && CPU == .ARM64] `SYS_landlock_restrict_self :: 446`
- [OS == .ANDROID && CPU == .ARM64] `SYS_lgetxattr :: 9`
- [OS == .ANDROID && CPU == .ARM64] `SYS_linkat :: 37`
- [OS == .ANDROID && CPU == .ARM64] `SYS_listen :: 201`
- [OS == .ANDROID && CPU == .ARM64] `SYS_listxattr :: 11`
- [OS == .ANDROID && CPU == .ARM64] `SYS_llistxattr :: 12`
- [OS == .ANDROID && CPU == .ARM64] `SYS_lookup_dcookie :: 18`
- [OS == .ANDROID && CPU == .ARM64] `SYS_lremovexattr :: 15`
- [OS == .ANDROID && CPU == .ARM64] `SYS_lseek :: 62`
- [OS == .ANDROID && CPU == .ARM64] `SYS_lsetxattr :: 6`
- [OS == .ANDROID && CPU == .ARM64] `SYS_madvise :: 233`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mbind :: 235`
- [OS == .ANDROID && CPU == .ARM64] `SYS_membarrier :: 283`
- [OS == .ANDROID && CPU == .ARM64] `SYS_memfd_create :: 279`
- [OS == .ANDROID && CPU == .ARM64] `SYS_memfd_secret :: 447`
- [OS == .ANDROID && CPU == .ARM64] `SYS_migrate_pages :: 238`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mincore :: 232`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mkdirat :: 34`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mknodat :: 33`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mlock :: 228`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mlock2 :: 284`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mlockall :: 230`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mmap :: 222`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mount :: 40`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mount_setattr :: 442`
- [OS == .ANDROID && CPU == .ARM64] `SYS_move_mount :: 429`
- [OS == .ANDROID && CPU == .ARM64] `SYS_move_pages :: 239`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mprotect :: 226`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mq_getsetattr :: 185`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mq_notify :: 184`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mq_open :: 180`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mq_timedreceive :: 183`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mq_timedsend :: 182`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mq_unlink :: 181`
- [OS == .ANDROID && CPU == .ARM64] `SYS_mremap :: 216`
- [OS == .ANDROID && CPU == .ARM64] `SYS_msgctl :: 187`
- [OS == .ANDROID && CPU == .ARM64] `SYS_msgget :: 186`
- [OS == .ANDROID && CPU == .ARM64] `SYS_msgrcv :: 188`
- [OS == .ANDROID && CPU == .ARM64] `SYS_msgsnd :: 189`
- [OS == .ANDROID && CPU == .ARM64] `SYS_msync :: 227`
- [OS == .ANDROID && CPU == .ARM64] `SYS_munlock :: 229`
- [OS == .ANDROID && CPU == .ARM64] `SYS_munlockall :: 231`
- [OS == .ANDROID && CPU == .ARM64] `SYS_munmap :: 215`
- [OS == .ANDROID && CPU == .ARM64] `SYS_name_to_handle_at :: 264`
- [OS == .ANDROID && CPU == .ARM64] `SYS_nanosleep :: 101`
- [OS == .ANDROID && CPU == .ARM64] `SYS_newfstatat :: 79`
- [OS == .ANDROID && CPU == .ARM64] `SYS_nfsservctl :: 42`
- [OS == .ANDROID && CPU == .ARM64] `SYS_open_by_handle_at :: 265`
- [OS == .ANDROID && CPU == .ARM64] `SYS_open_tree :: 428`
- [OS == .ANDROID && CPU == .ARM64] `SYS_openat :: 56`
- [OS == .ANDROID && CPU == .ARM64] `SYS_openat2 :: 437`
- [OS == .ANDROID && CPU == .ARM64] `SYS_perf_event_open :: 241`
- [OS == .ANDROID && CPU == .ARM64] `SYS_personality :: 92`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pidfd_getfd :: 438`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pidfd_open :: 434`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pidfd_send_signal :: 424`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pipe2 :: 59`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pivot_root :: 41`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pkey_alloc :: 289`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pkey_free :: 290`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pkey_mprotect :: 288`
- [OS == .ANDROID && CPU == .ARM64] `SYS_ppoll :: 73`
- [OS == .ANDROID && CPU == .ARM64] `SYS_prctl :: 167`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pread64 :: 67`
- [OS == .ANDROID && CPU == .ARM64] `SYS_preadv :: 69`
- [OS == .ANDROID && CPU == .ARM64] `SYS_preadv2 :: 286`
- [OS == .ANDROID && CPU == .ARM64] `SYS_prlimit64 :: 261`
- [OS == .ANDROID && CPU == .ARM64] `SYS_process_madvise :: 440`
- [OS == .ANDROID && CPU == .ARM64] `SYS_process_mrelease :: 448`
- [OS == .ANDROID && CPU == .ARM64] `SYS_process_vm_readv :: 270`
- [OS == .ANDROID && CPU == .ARM64] `SYS_process_vm_writev :: 271`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pselect6 :: 72`
- [OS == .ANDROID && CPU == .ARM64] `SYS_ptrace :: 117`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pwrite64 :: 68`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pwritev :: 70`
- [OS == .ANDROID && CPU == .ARM64] `SYS_pwritev2 :: 287`
- [OS == .ANDROID && CPU == .ARM64] `SYS_quotactl :: 60`
- [OS == .ANDROID && CPU == .ARM64] `SYS_quotactl_fd :: 443`
- [OS == .ANDROID && CPU == .ARM64] `SYS_read :: 63`
- [OS == .ANDROID && CPU == .ARM64] `SYS_readahead :: 213`
- [OS == .ANDROID && CPU == .ARM64] `SYS_readlinkat :: 78`
- [OS == .ANDROID && CPU == .ARM64] `SYS_readv :: 65`
- [OS == .ANDROID && CPU == .ARM64] `SYS_reboot :: 142`
- [OS == .ANDROID && CPU == .ARM64] `SYS_recvfrom :: 207`
- [OS == .ANDROID && CPU == .ARM64] `SYS_recvmmsg :: 243`
- [OS == .ANDROID && CPU == .ARM64] `SYS_recvmsg :: 212`
- [OS == .ANDROID && CPU == .ARM64] `SYS_remap_file_pages :: 234`
- [OS == .ANDROID && CPU == .ARM64] `SYS_removexattr :: 14`
- [OS == .ANDROID && CPU == .ARM64] `SYS_renameat :: 38`
- [OS == .ANDROID && CPU == .ARM64] `SYS_renameat2 :: 276`
- [OS == .ANDROID && CPU == .ARM64] `SYS_request_key :: 218`
- [OS == .ANDROID && CPU == .ARM64] `SYS_restart_syscall :: 128`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rseq :: 293`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigaction :: 134`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigpending :: 136`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigprocmask :: 135`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigqueueinfo :: 138`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigreturn :: 139`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigsuspend :: 133`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_sigtimedwait :: 137`
- [OS == .ANDROID && CPU == .ARM64] `SYS_rt_tgsigqueueinfo :: 240`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_get_priority_max :: 125`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_get_priority_min :: 126`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_getaffinity :: 123`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_getattr :: 275`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_getparam :: 121`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_getscheduler :: 120`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_rr_get_interval :: 127`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_setaffinity :: 122`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_setattr :: 274`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_setparam :: 118`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_setscheduler :: 119`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sched_yield :: 124`
- [OS == .ANDROID && CPU == .ARM64] `SYS_seccomp :: 277`
- [OS == .ANDROID && CPU == .ARM64] `SYS_semctl :: 191`
- [OS == .ANDROID && CPU == .ARM64] `SYS_semget :: 190`
- [OS == .ANDROID && CPU == .ARM64] `SYS_semop :: 193`
- [OS == .ANDROID && CPU == .ARM64] `SYS_semtimedop :: 192`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sendfile :: 71`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sendmmsg :: 269`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sendmsg :: 211`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sendto :: 206`
- [OS == .ANDROID && CPU == .ARM64] `SYS_set_mempolicy :: 237`
- [OS == .ANDROID && CPU == .ARM64] `SYS_set_mempolicy_home_node :: 450`
- [OS == .ANDROID && CPU == .ARM64] `SYS_set_robust_list :: 99`
- [OS == .ANDROID && CPU == .ARM64] `SYS_set_tid_address :: 96`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setdomainname :: 162`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setfsgid :: 152`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setfsuid :: 151`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setgid :: 144`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setgroups :: 159`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sethostname :: 161`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setitimer :: 103`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setns :: 268`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setpgid :: 154`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setpriority :: 140`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setregid :: 143`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setresgid :: 149`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setresuid :: 147`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setreuid :: 145`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setrlimit :: 164`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setsid :: 157`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setsockopt :: 208`
- [OS == .ANDROID && CPU == .ARM64] `SYS_settimeofday :: 170`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setuid :: 146`
- [OS == .ANDROID && CPU == .ARM64] `SYS_setxattr :: 5`
- [OS == .ANDROID && CPU == .ARM64] `SYS_shmat :: 196`
- [OS == .ANDROID && CPU == .ARM64] `SYS_shmctl :: 195`
- [OS == .ANDROID && CPU == .ARM64] `SYS_shmdt :: 197`
- [OS == .ANDROID && CPU == .ARM64] `SYS_shmget :: 194`
- [OS == .ANDROID && CPU == .ARM64] `SYS_shutdown :: 210`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sigaltstack :: 132`
- [OS == .ANDROID && CPU == .ARM64] `SYS_signalfd4 :: 74`
- [OS == .ANDROID && CPU == .ARM64] `SYS_socket :: 198`
- [OS == .ANDROID && CPU == .ARM64] `SYS_socketpair :: 199`
- [OS == .ANDROID && CPU == .ARM64] `SYS_splice :: 76`
- [OS == .ANDROID && CPU == .ARM64] `SYS_statfs :: 43`
- [OS == .ANDROID && CPU == .ARM64] `SYS_statx :: 291`
- [OS == .ANDROID && CPU == .ARM64] `SYS_swapoff :: 225`
- [OS == .ANDROID && CPU == .ARM64] `SYS_swapon :: 224`
- [OS == .ANDROID && CPU == .ARM64] `SYS_symlinkat :: 36`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sync :: 81`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sync_file_range :: 84`
- [OS == .ANDROID && CPU == .ARM64] `SYS_syncfs :: 267`
- [OS == .ANDROID && CPU == .ARM64] `SYS_syscalls :: 451`
- [OS == .ANDROID && CPU == .ARM64] `SYS_sysinfo :: 179`
- [OS == .ANDROID && CPU == .ARM64] `SYS_syslog :: 116`
- [OS == .ANDROID && CPU == .ARM64] `SYS_tee :: 77`
- [OS == .ANDROID && CPU == .ARM64] `SYS_tgkill :: 131`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timer_create :: 107`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timer_delete :: 111`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timer_getoverrun :: 109`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timer_gettime :: 108`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timer_settime :: 110`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timerfd_create :: 85`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timerfd_gettime :: 87`
- [OS == .ANDROID && CPU == .ARM64] `SYS_timerfd_settime :: 86`
- [OS == .ANDROID && CPU == .ARM64] `SYS_times :: 153`
- [OS == .ANDROID && CPU == .ARM64] `SYS_tkill :: 130`
- [OS == .ANDROID && CPU == .ARM64] `SYS_truncate :: 45`
- [OS == .ANDROID && CPU == .ARM64] `SYS_umask :: 166`
- [OS == .ANDROID && CPU == .ARM64] `SYS_umount2 :: 39`
- [OS == .ANDROID && CPU == .ARM64] `SYS_uname :: 160`
- [OS == .ANDROID && CPU == .ARM64] `SYS_unlinkat :: 35`
- [OS == .ANDROID && CPU == .ARM64] `SYS_unshare :: 97`
- [OS == .ANDROID && CPU == .ARM64] `SYS_userfaultfd :: 282`
- [OS == .ANDROID && CPU == .ARM64] `SYS_utimensat :: 88`
- [OS == .ANDROID && CPU == .ARM64] `SYS_vhangup :: 58`
- [OS == .ANDROID && CPU == .ARM64] `SYS_vmsplice :: 75`
- [OS == .ANDROID && CPU == .ARM64] `SYS_wait4 :: 260`
- [OS == .ANDROID && CPU == .ARM64] `SYS_waitid :: 95`
- [OS == .ANDROID && CPU == .ARM64] `SYS_write :: 64`
- [OS == .ANDROID && CPU == .ARM64] `SYS_writev :: 66`
- [OS == .ANDROID && CPU == .ARM64] `syscall :: (__number: s64, __args: ..Any) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `CSIGNAL :: 0x000000ff`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_VM :: 0x00000100`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_FS :: 0x00000200`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_FILES :: 0x00000400`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_SIGHAND :: 0x00000800`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_PIDFD :: 0x00001000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_PTRACE :: 0x00002000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_VFORK :: 0x00004000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_PARENT :: 0x00008000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_THREAD :: 0x00010000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWNS :: 0x00020000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_SYSVSEM :: 0x00040000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_SETTLS :: 0x00080000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_PARENT_SETTID :: 0x00100000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_CHILD_CLEARTID :: 0x00200000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_DETACHED :: 0x00400000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_UNTRACED :: 0x00800000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_CHILD_SETTID :: 0x01000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWCGROUP :: 0x02000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWUTS :: 0x04000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWIPC :: 0x08000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWUSER :: 0x10000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWPID :: 0x20000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWNET :: 0x40000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_IO :: 0x80000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_CLEAR_SIGHAND :: 0x100000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_INTO_CGROUP :: 0x200000000`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_NEWTIME :: 0x00000080`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_ARGS_SIZE_VER0 :: 64`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_ARGS_SIZE_VER1 :: 80`
- [OS == .ANDROID && CPU == .ARM64] `CLONE_ARGS_SIZE_VER2 :: 88`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_NORMAL :: 0`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FIFO :: 1`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_RR :: 2`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_BATCH :: 3`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_IDLE :: 5`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_DEADLINE :: 6`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_RESET_ON_FORK :: 0x40000000`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_RESET_ON_FORK :: 0x01`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_RECLAIM :: 0x02`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_DL_OVERRUN :: 0x04`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_KEEP_POLICY :: 0x08`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_KEEP_PARAMS :: 0x10`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_UTIL_CLAMP_MIN :: 0x20`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_UTIL_CLAMP_MAX :: 0x40`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_KEEP_ALL :: SCHED_FLAG_KEEP_POLICY | SCHED_FLAG_KEEP_PARAMS`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_UTIL_CLAMP :: SCHED_FLAG_UTIL_CLAMP_MIN | SCHED_FLAG_UTIL_CLAMP_MAX`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_FLAG_ALL :: SCHED_FLAG_RESET_ON_FORK | SCHED_FLAG_RECLAIM | SCHED_FLAG_DL_OVERRUN | SCHED_FLAG_KEEP_ALL | SCHED_FLAG_UTIL_CLAMP`
- [OS == .ANDROID && CPU == .ARM64] `SCHED_OTHER :: SCHED_NORMAL`
- [OS == .ANDROID && CPU == .ARM64] `CPU_SETSIZE :: 1024`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_ONCE_INIT :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_BARRIER_SERIAL_THREAD :: -1`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_CREATE_DETACHED :: 1`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_CREATE_JOINABLE :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_EXPLICIT_SCHED :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_INHERIT_SCHED :: 1`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_PRIO_NONE :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_PRIO_INHERIT :: 1`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_PROCESS_PRIVATE :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_PROCESS_SHARED :: 1`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_SCOPE_SYSTEM :: 0`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_SCOPE_PROCESS :: 1`
- [OS == .ANDROID && CPU == .ARM64] `_JBLEN :: 32`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrier_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrierattr_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `pthread_key_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `pthread_once_t :: s32`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `pthread_spinlock_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `pthread_t :: s64`
- [OS == .ANDROID && CPU == .ARM64] `clone_args :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sched_param :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sched_setscheduler :: (__pid: pid_t, __policy: s32, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_getscheduler :: (__pid: pid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_yield :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_get_priority_max :: (__policy: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_get_priority_min :: (__policy: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_setparam :: (__pid: pid_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_getparam :: (__pid: pid_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_rr_get_interval :: (__pid: pid_t, __quantum: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `clone :: (__fn: #type (unknown0: *void) -> s32 #c_call, __child_stack: *void, __flags: s32, __arg: *void, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `unshare :: (__flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setns :: (__fd: s32, __ns_type: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_getcpu :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `cpu_set_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sched_setaffinity :: (__pid: pid_t, __set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sched_getaffinity :: (__pid: pid_t, __set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__sched_cpucount :: (__set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__sched_cpualloc :: (__count: u64) -> *cpu_set_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__sched_cpufree :: (__set: *cpu_set_t) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_MUTEX :: enum u32`
- [OS == .ANDROID && CPU == .ARM64] `PTHREAD_RWLOCK_PREFER :: enum u32`
- [OS == .ANDROID && CPU == .ARM64] `pthread_atfork :: (__prepare: #type () -> void #c_call, __parent: #type () -> void #c_call, __child: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_destroy :: (__attr: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getdetachstate :: (__attr: *pthread_attr_t, __state: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getguardsize :: (__attr: *pthread_attr_t, __size: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getinheritsched :: (__attr: *pthread_attr_t, __flag: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getschedpolicy :: (__attr: *pthread_attr_t, __policy: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getscope :: (__attr: *pthread_attr_t, __scope: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getstack :: (__attr: *pthread_attr_t, __addr: **void, __size: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_getstacksize :: (__attr: *pthread_attr_t, __size: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_init :: (__attr: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setdetachstate :: (__attr: *pthread_attr_t, __state: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setguardsize :: (__attr: *pthread_attr_t, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setinheritsched :: (__attr: *pthread_attr_t, __flag: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setschedpolicy :: (__attr: *pthread_attr_t, __policy: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setscope :: (__attr: *pthread_attr_t, __scope: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setstack :: (__attr: *pthread_attr_t, __addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_attr_setstacksize :: (__addr: *pthread_attr_t, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_destroy :: (__attr: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_getclock :: (__attr: *pthread_condattr_t, __clock: *clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_getpshared :: (__attr: *pthread_condattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_init :: (__attr: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_setclock :: (__attr: *pthread_condattr_t, __clock: clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_condattr_setpshared :: (__attr: *pthread_condattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_broadcast :: (__cond: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_clockwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_destroy :: (__cond: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_init :: (__cond: *pthread_cond_t, __attr: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_signal :: (__cond: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_timedwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_timedwait_monotonic_np :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_cond_wait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_create :: (__pthread_ptr: *pthread_t, __attr: *pthread_attr_t, __start_routine: #type (unknown0: *void) -> *void #c_call, unknown0: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_detach :: (__pthread: pthread_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_exit :: (__return_value: *void) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_equal :: (__lhs: pthread_t, __rhs: pthread_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_getattr_np :: (__pthread: pthread_t, __attr: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_getcpuclockid :: (__pthread: pthread_t, __clock: *clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_getspecific :: (__key: pthread_key_t) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_gettid_np :: (__pthread: pthread_t) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_join :: (__pthread: pthread_t, __return_value_ptr: **void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_key_create :: (__key_ptr: *pthread_key_t, __key_destructor: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_key_delete :: (__key: pthread_key_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_destroy :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_getpshared :: (__attr: *pthread_mutexattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_gettype :: (__attr: *pthread_mutexattr_t, __type: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_getprotocol :: (__attr: *pthread_mutexattr_t, __protocol: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_init :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_setpshared :: (__attr: *pthread_mutexattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_settype :: (__attr: *pthread_mutexattr_t, __type: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutexattr_setprotocol :: (__attr: *pthread_mutexattr_t, __protocol: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_clocklock :: (__mutex: *pthread_mutex_t, __clock: clockid_t, __abstime: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_destroy :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_init :: (__mutex: *pthread_mutex_t, __attr: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_lock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_timedlock :: (__mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_timedlock_monotonic_np :: (__mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_trylock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_mutex_unlock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_once :: (__once: *pthread_once_t, __init_routine: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_init :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_destroy :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_getpshared :: (__attr: *pthread_rwlockattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_setpshared :: (__attr: *pthread_rwlockattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_getkind_np :: (__attr: *pthread_rwlockattr_t, __kind: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlockattr_setkind_np :: (__attr: *pthread_rwlockattr_t, __kind: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_clockrdlock :: (__rwlock: *pthread_rwlock_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_clockwrlock :: (__rwlock: *pthread_rwlock_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_destroy :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_init :: (__rwlock: *pthread_rwlock_t, __attr: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_rdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_timedrdlock :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_timedrdlock_monotonic_np :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_timedwrlock :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_timedwrlock_monotonic_np :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_tryrdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_trywrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_unlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_rwlock_wrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrierattr_init :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrierattr_destroy :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrierattr_getpshared :: (__attr: *pthread_barrierattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrierattr_setpshared :: (__attr: *pthread_barrierattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrier_init :: (__barrier: *pthread_barrier_t, __attr: *pthread_barrierattr_t, __count: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrier_destroy :: (__barrier: *pthread_barrier_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_barrier_wait :: (__barrier: *pthread_barrier_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_spin_destroy :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_spin_init :: (__spinlock: *pthread_spinlock_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_spin_lock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_spin_trylock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_spin_unlock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_self :: () -> pthread_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_getname_np :: (__pthread: pthread_t, __buf: *u8, __n: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_setname_np :: (__pthread: pthread_t, __name: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_setschedparam :: (__pthread: pthread_t, __policy: s32, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_getschedparam :: (__pthread: pthread_t, __policy: *s32, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_setschedprio :: (__pthread: pthread_t, __priority: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `pthread_setspecific :: (__key: pthread_key_t, __value: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__pthread_cleanup_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `__pthread_cleanup_push :: (c: *__pthread_cleanup_t, unknown0: #type () -> void #c_call, unknown1: *void) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `__pthread_cleanup_pop :: (unknown0: *__pthread_cleanup_t, unknown1: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_t :: struct`
- [OS == .ANDROID && CPU == .ARM64] `sem_clockwait :: (__sem: *sem_t, __clock: clockid_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_destroy :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_getvalue :: (__sem: *sem_t, __value: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_init :: (__sem: *sem_t, __shared: s32, __value: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_post :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_timedwait :: (__sem: *sem_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_timedwait_monotonic_np :: (__sem: *sem_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_trywait :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_wait :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_open :: (__name: *u8, _flags: s32, __args: ..Any) -> *sem_t #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_close :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sem_unlink :: (__name: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigjmp_buf :: [33] s64`
- [OS == .ANDROID && CPU == .ARM64] `jmp_buf :: [32] s64`
- [OS == .ANDROID && CPU == .ARM64] `_setjmp :: (__env: *jmp_buf) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `_longjmp :: (__env: *jmp_buf, __value: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setjmp :: (__env: *jmp_buf) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `longjmp :: (__env: *jmp_buf, __value: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `sigsetjmp :: (__env: *sigjmp_buf, __save_signal_mask: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `siglongjmp :: (__env: *sigjmp_buf, __value: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `RUSAGE :: enum s32`
- [OS == .ANDROID && CPU == .ARM64] `RLIM64_INFINITY :: ~0`
- [OS == .ANDROID && CPU == .ARM64] `PRIO_MIN :: - 20`
- [OS == .ANDROID && CPU == .ARM64] `PRIO_MAX :: 20`
- [OS == .ANDROID && CPU == .ARM64] `PRIO :: enum s32`
- [OS == .ANDROID && CPU == .ARM64] `_STK_LIM :: 8 * 1024 * 1024`
- [OS == .ANDROID && CPU == .ARM64] `MLOCK_LIMIT :: 8 * 1024 * 1024`
- [OS == .ANDROID && CPU == .ARM64] `RLIMIT_RESOURCE :: enum s32`
- [OS == .ANDROID && CPU == .ARM64] `RLIM_NLIMITS :: 16`
- [OS == .ANDROID && CPU == .ARM64] `RLIM_INFINITY :: ~0`
- [OS == .ANDROID && CPU == .ARM64] `RLIM_SAVED_CUR :: RLIM_INFINITY`
- [OS == .ANDROID && CPU == .ARM64] `RLIM_SAVED_MAX :: RLIM_INFINITY`
- [OS == .ANDROID && CPU == .ARM64] `id_t :: u32`
- [OS == .ANDROID && CPU == .ARM64] `rusage :: struct`
- [OS == .ANDROID && CPU == .ARM64] `rlimit :: struct`
- [OS == .ANDROID && CPU == .ARM64] `rlimit64 :: struct`
- [OS == .ANDROID && CPU == .ARM64] `rlim_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `rlim64_t :: u64`
- [OS == .ANDROID && CPU == .ARM64] `getrlimit :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setrlimit :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getrlimit64 :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setrlimit64 :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getpriority :: (__which: PRIO, __who: id_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `setpriority :: (__which: PRIO, __who: id_t, __priority: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `getrusage :: (__who: RUSAGE, __usage: *rusage) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `prlimit :: (__pid: pid_t, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit, __old_limit: *rlimit) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .ARM64] `prlimit64 :: (__pid: pid_t, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit64, __old_limit: *rlimit64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `RTLD_LOCAL :: 0`
- [OS == .ANDROID && CPU == .X64] `RTLD_LAZY :: 0x00001`
- [OS == .ANDROID && CPU == .X64] `RTLD_NOW :: 0x00002`
- [OS == .ANDROID && CPU == .X64] `RTLD_NOLOAD :: 0x00004`
- [OS == .ANDROID && CPU == .X64] `RTLD_GLOBAL :: 0x00100`
- [OS == .ANDROID && CPU == .X64] `RTLD_NODELETE :: 0x01000`
- [OS == .ANDROID && CPU == .X64] `__FD_SETSIZE :: 1024`
- [OS == .ANDROID && CPU == .X64] `DT :: enum u8`
- [OS == .ANDROID && CPU == .X64] `__DIRENT64_INO_T :: ino_t`
- [OS == .ANDROID && CPU == .X64] `EPERM :: 1`
- [OS == .ANDROID && CPU == .X64] `ENOENT :: 2`
- [OS == .ANDROID && CPU == .X64] `ESRCH :: 3`
- [OS == .ANDROID && CPU == .X64] `EINTR :: 4`
- [OS == .ANDROID && CPU == .X64] `EIO :: 5`
- [OS == .ANDROID && CPU == .X64] `ENXIO :: 6`
- [OS == .ANDROID && CPU == .X64] `E2BIG :: 7`
- [OS == .ANDROID && CPU == .X64] `ENOEXEC :: 8`
- [OS == .ANDROID && CPU == .X64] `EBADF :: 9`
- [OS == .ANDROID && CPU == .X64] `ECHILD :: 10`
- [OS == .ANDROID && CPU == .X64] `EAGAIN :: 11`
- [OS == .ANDROID && CPU == .X64] `ENOMEM :: 12`
- [OS == .ANDROID && CPU == .X64] `EACCES :: 13`
- [OS == .ANDROID && CPU == .X64] `EFAULT :: 14`
- [OS == .ANDROID && CPU == .X64] `ENOTBLK :: 15`
- [OS == .ANDROID && CPU == .X64] `EBUSY :: 16`
- [OS == .ANDROID && CPU == .X64] `EEXIST :: 17`
- [OS == .ANDROID && CPU == .X64] `EXDEV :: 18`
- [OS == .ANDROID && CPU == .X64] `ENODEV :: 19`
- [OS == .ANDROID && CPU == .X64] `ENOTDIR :: 20`
- [OS == .ANDROID && CPU == .X64] `EISDIR :: 21`
- [OS == .ANDROID && CPU == .X64] `EINVAL :: 22`
- [OS == .ANDROID && CPU == .X64] `ENFILE :: 23`
- [OS == .ANDROID && CPU == .X64] `EMFILE :: 24`
- [OS == .ANDROID && CPU == .X64] `ENOTTY :: 25`
- [OS == .ANDROID && CPU == .X64] `ETXTBSY :: 26`
- [OS == .ANDROID && CPU == .X64] `EFBIG :: 27`
- [OS == .ANDROID && CPU == .X64] `ENOSPC :: 28`
- [OS == .ANDROID && CPU == .X64] `ESPIPE :: 29`
- [OS == .ANDROID && CPU == .X64] `EROFS :: 30`
- [OS == .ANDROID && CPU == .X64] `EMLINK :: 31`
- [OS == .ANDROID && CPU == .X64] `EPIPE :: 32`
- [OS == .ANDROID && CPU == .X64] `EDOM :: 33`
- [OS == .ANDROID && CPU == .X64] `ERANGE :: 34`
- [OS == .ANDROID && CPU == .X64] `EDEADLK :: 35`
- [OS == .ANDROID && CPU == .X64] `ENAMETOOLONG :: 36`
- [OS == .ANDROID && CPU == .X64] `ENOLCK :: 37`
- [OS == .ANDROID && CPU == .X64] `ENOSYS :: 38`
- [OS == .ANDROID && CPU == .X64] `ENOTEMPTY :: 39`
- [OS == .ANDROID && CPU == .X64] `ELOOP :: 40`
- [OS == .ANDROID && CPU == .X64] `EWOULDBLOCK :: EAGAIN`
- [OS == .ANDROID && CPU == .X64] `ENOMSG :: 42`
- [OS == .ANDROID && CPU == .X64] `EIDRM :: 43`
- [OS == .ANDROID && CPU == .X64] `ECHRNG :: 44`
- [OS == .ANDROID && CPU == .X64] `EL2NSYNC :: 45`
- [OS == .ANDROID && CPU == .X64] `EL3HLT :: 46`
- [OS == .ANDROID && CPU == .X64] `EL3RST :: 47`
- [OS == .ANDROID && CPU == .X64] `ELNRNG :: 48`
- [OS == .ANDROID && CPU == .X64] `EUNATCH :: 49`
- [OS == .ANDROID && CPU == .X64] `ENOCSI :: 50`
- [OS == .ANDROID && CPU == .X64] `EL2HLT :: 51`
- [OS == .ANDROID && CPU == .X64] `EBADE :: 52`
- [OS == .ANDROID && CPU == .X64] `EBADR :: 53`
- [OS == .ANDROID && CPU == .X64] `EXFULL :: 54`
- [OS == .ANDROID && CPU == .X64] `ENOANO :: 55`
- [OS == .ANDROID && CPU == .X64] `EBADRQC :: 56`
- [OS == .ANDROID && CPU == .X64] `EBADSLT :: 57`
- [OS == .ANDROID && CPU == .X64] `EDEADLOCK :: EDEADLK`
- [OS == .ANDROID && CPU == .X64] `EBFONT :: 59`
- [OS == .ANDROID && CPU == .X64] `ENOSTR :: 60`
- [OS == .ANDROID && CPU == .X64] `ENODATA :: 61`
- [OS == .ANDROID && CPU == .X64] `ETIME :: 62`
- [OS == .ANDROID && CPU == .X64] `ENOSR :: 63`
- [OS == .ANDROID && CPU == .X64] `ENONET :: 64`
- [OS == .ANDROID && CPU == .X64] `ENOPKG :: 65`
- [OS == .ANDROID && CPU == .X64] `EREMOTE :: 66`
- [OS == .ANDROID && CPU == .X64] `ENOLINK :: 67`
- [OS == .ANDROID && CPU == .X64] `EADV :: 68`
- [OS == .ANDROID && CPU == .X64] `ESRMNT :: 69`
- [OS == .ANDROID && CPU == .X64] `ECOMM :: 70`
- [OS == .ANDROID && CPU == .X64] `EPROTO :: 71`
- [OS == .ANDROID && CPU == .X64] `EMULTIHOP :: 72`
- [OS == .ANDROID && CPU == .X64] `EDOTDOT :: 73`
- [OS == .ANDROID && CPU == .X64] `EBADMSG :: 74`
- [OS == .ANDROID && CPU == .X64] `EOVERFLOW :: 75`
- [OS == .ANDROID && CPU == .X64] `ENOTUNIQ :: 76`
- [OS == .ANDROID && CPU == .X64] `EBADFD :: 77`
- [OS == .ANDROID && CPU == .X64] `EREMCHG :: 78`
- [OS == .ANDROID && CPU == .X64] `ELIBACC :: 79`
- [OS == .ANDROID && CPU == .X64] `ELIBBAD :: 80`
- [OS == .ANDROID && CPU == .X64] `ELIBSCN :: 81`
- [OS == .ANDROID && CPU == .X64] `ELIBMAX :: 82`
- [OS == .ANDROID && CPU == .X64] `ELIBEXEC :: 83`
- [OS == .ANDROID && CPU == .X64] `EILSEQ :: 84`
- [OS == .ANDROID && CPU == .X64] `ERESTART :: 85`
- [OS == .ANDROID && CPU == .X64] `ESTRPIPE :: 86`
- [OS == .ANDROID && CPU == .X64] `EUSERS :: 87`
- [OS == .ANDROID && CPU == .X64] `ENOTSOCK :: 88`
- [OS == .ANDROID && CPU == .X64] `EDESTADDRREQ :: 89`
- [OS == .ANDROID && CPU == .X64] `EMSGSIZE :: 90`
- [OS == .ANDROID && CPU == .X64] `EPROTOTYPE :: 91`
- [OS == .ANDROID && CPU == .X64] `ENOPROTOOPT :: 92`
- [OS == .ANDROID && CPU == .X64] `EPROTONOSUPPORT :: 93`
- [OS == .ANDROID && CPU == .X64] `ESOCKTNOSUPPORT :: 94`
- [OS == .ANDROID && CPU == .X64] `EOPNOTSUPP :: 95`
- [OS == .ANDROID && CPU == .X64] `EPFNOSUPPORT :: 96`
- [OS == .ANDROID && CPU == .X64] `EAFNOSUPPORT :: 97`
- [OS == .ANDROID && CPU == .X64] `EADDRINUSE :: 98`
- [OS == .ANDROID && CPU == .X64] `EADDRNOTAVAIL :: 99`
- [OS == .ANDROID && CPU == .X64] `ENETDOWN :: 100`
- [OS == .ANDROID && CPU == .X64] `ENETUNREACH :: 101`
- [OS == .ANDROID && CPU == .X64] `ENETRESET :: 102`
- [OS == .ANDROID && CPU == .X64] `ECONNABORTED :: 103`
- [OS == .ANDROID && CPU == .X64] `ECONNRESET :: 104`
- [OS == .ANDROID && CPU == .X64] `ENOBUFS :: 105`
- [OS == .ANDROID && CPU == .X64] `EISCONN :: 106`
- [OS == .ANDROID && CPU == .X64] `ENOTCONN :: 107`
- [OS == .ANDROID && CPU == .X64] `ESHUTDOWN :: 108`
- [OS == .ANDROID && CPU == .X64] `ETOOMANYREFS :: 109`
- [OS == .ANDROID && CPU == .X64] `ETIMEDOUT :: 110`
- [OS == .ANDROID && CPU == .X64] `ECONNREFUSED :: 111`
- [OS == .ANDROID && CPU == .X64] `EHOSTDOWN :: 112`
- [OS == .ANDROID && CPU == .X64] `EHOSTUNREACH :: 113`
- [OS == .ANDROID && CPU == .X64] `EALREADY :: 114`
- [OS == .ANDROID && CPU == .X64] `EINPROGRESS :: 115`
- [OS == .ANDROID && CPU == .X64] `ESTALE :: 116`
- [OS == .ANDROID && CPU == .X64] `EUCLEAN :: 117`
- [OS == .ANDROID && CPU == .X64] `ENOTNAM :: 118`
- [OS == .ANDROID && CPU == .X64] `ENAVAIL :: 119`
- [OS == .ANDROID && CPU == .X64] `EISNAM :: 120`
- [OS == .ANDROID && CPU == .X64] `EREMOTEIO :: 121`
- [OS == .ANDROID && CPU == .X64] `EDQUOT :: 122`
- [OS == .ANDROID && CPU == .X64] `ENOMEDIUM :: 123`
- [OS == .ANDROID && CPU == .X64] `EMEDIUMTYPE :: 124`
- [OS == .ANDROID && CPU == .X64] `ECANCELED :: 125`
- [OS == .ANDROID && CPU == .X64] `ENOKEY :: 126`
- [OS == .ANDROID && CPU == .X64] `EKEYEXPIRED :: 127`
- [OS == .ANDROID && CPU == .X64] `EKEYREVOKED :: 128`
- [OS == .ANDROID && CPU == .X64] `EKEYREJECTED :: 129`
- [OS == .ANDROID && CPU == .X64] `EOWNERDEAD :: 130`
- [OS == .ANDROID && CPU == .X64] `ENOTRECOVERABLE :: 131`
- [OS == .ANDROID && CPU == .X64] `ERFKILL :: 132`
- [OS == .ANDROID && CPU == .X64] `EHWPOISON :: 133`
- [OS == .ANDROID && CPU == .X64] `ENOTSUP :: EOPNOTSUPP`
- [OS == .ANDROID && CPU == .X64] `FNM_NOMATCH :: 1`
- [OS == .ANDROID && CPU == .X64] `FNM_NOSYS :: 2`
- [OS == .ANDROID && CPU == .X64] `FNM_NOESCAPE :: 0x01`
- [OS == .ANDROID && CPU == .X64] `FNM_PATHNAME :: 0x02`
- [OS == .ANDROID && CPU == .X64] `FNM_PERIOD :: 0x04`
- [OS == .ANDROID && CPU == .X64] `FNM_LEADING_DIR :: 0x08`
- [OS == .ANDROID && CPU == .X64] `FNM_CASEFOLD :: 0x10`
- [OS == .ANDROID && CPU == .X64] `FNM_IGNORECASE :: FNM_CASEFOLD`
- [OS == .ANDROID && CPU == .X64] `FNM_FILE_NAME :: FNM_PATHNAME`
- [OS == .ANDROID && CPU == .X64] `ITIMER_REAL :: 0`
- [OS == .ANDROID && CPU == .X64] `ITIMER_VIRTUAL :: 1`
- [OS == .ANDROID && CPU == .X64] `ITIMER_PROF :: 2`
- [OS == .ANDROID && CPU == .X64] `clockid_t :: enum u32`
- [OS == .ANDROID && CPU == .X64] `MAX_CLOCKS :: 16`
- [OS == .ANDROID && CPU == .X64] `CLOCKS_MASK :: xx clockid_t.REALTIME | xx clockid_t.MONOTONIC`
- [OS == .ANDROID && CPU == .X64] `CLOCKS_MONO :: xx clockid_t.MONOTONIC`
- [OS == .ANDROID && CPU == .X64] `TIMER_ABSTIME :: 0x01`
- [OS == .ANDROID && CPU == .X64] `_KERNEL_NSIG :: 32`
- [OS == .ANDROID && CPU == .X64] `SIGHUP :: 1`
- [OS == .ANDROID && CPU == .X64] `SIGINT :: 2`
- [OS == .ANDROID && CPU == .X64] `SIGQUIT :: 3`
- [OS == .ANDROID && CPU == .X64] `SIGILL :: 4`
- [OS == .ANDROID && CPU == .X64] `SIGTRAP :: 5`
- [OS == .ANDROID && CPU == .X64] `SIGABRT :: 6`
- [OS == .ANDROID && CPU == .X64] `SIGIOT :: 6`
- [OS == .ANDROID && CPU == .X64] `SIGBUS :: 7`
- [OS == .ANDROID && CPU == .X64] `SIGFPE :: 8`
- [OS == .ANDROID && CPU == .X64] `SIGKILL :: 9`
- [OS == .ANDROID && CPU == .X64] `SIGUSR1 :: 10`
- [OS == .ANDROID && CPU == .X64] `SIGSEGV :: 11`
- [OS == .ANDROID && CPU == .X64] `SIGUSR2 :: 12`
- [OS == .ANDROID && CPU == .X64] `SIGPIPE :: 13`
- [OS == .ANDROID && CPU == .X64] `SIGALRM :: 14`
- [OS == .ANDROID && CPU == .X64] `SIGTERM :: 15`
- [OS == .ANDROID && CPU == .X64] `SIGSTKFLT :: 16`
- [OS == .ANDROID && CPU == .X64] `SIGCHLD :: 17`
- [OS == .ANDROID && CPU == .X64] `SIGCONT :: 18`
- [OS == .ANDROID && CPU == .X64] `SIGSTOP :: 19`
- [OS == .ANDROID && CPU == .X64] `SIGTSTP :: 20`
- [OS == .ANDROID && CPU == .X64] `SIGTTIN :: 21`
- [OS == .ANDROID && CPU == .X64] `SIGTTOU :: 22`
- [OS == .ANDROID && CPU == .X64] `SIGURG :: 23`
- [OS == .ANDROID && CPU == .X64] `SIGXCPU :: 24`
- [OS == .ANDROID && CPU == .X64] `SIGXFSZ :: 25`
- [OS == .ANDROID && CPU == .X64] `SIGVTALRM :: 26`
- [OS == .ANDROID && CPU == .X64] `SIGPROF :: 27`
- [OS == .ANDROID && CPU == .X64] `SIGWINCH :: 28`
- [OS == .ANDROID && CPU == .X64] `SIGIO :: 29`
- [OS == .ANDROID && CPU == .X64] `SIGPOLL :: SIGIO`
- [OS == .ANDROID && CPU == .X64] `SIGPWR :: 30`
- [OS == .ANDROID && CPU == .X64] `SIGSYS :: 31`
- [OS == .ANDROID && CPU == .X64] `SIGUNUSED :: 31`
- [OS == .ANDROID && CPU == .X64] `__SIGRTMIN :: 32`
- [OS == .ANDROID && CPU == .X64] `__SIGRTMAX :: _KERNEL__NSIG`
- [OS == .ANDROID && CPU == .X64] `SA_RESTORER :: 0x04000000`
- [OS == .ANDROID && CPU == .X64] `MINSIGSTKSZ :: 2048`
- [OS == .ANDROID && CPU == .X64] `SIGSTKSZ :: 8192`
- [OS == .ANDROID && CPU == .X64] `SI_MAX_SIZE :: 128`
- [OS == .ANDROID && CPU == .X64] `__ARCH_SI_CLOCK_T :: s64`
- [OS == .ANDROID && CPU == .X64] `SI_USER :: 0`
- [OS == .ANDROID && CPU == .X64] `SI_KERNEL :: 0x80`
- [OS == .ANDROID && CPU == .X64] `SI_QUEUE :: - 1`
- [OS == .ANDROID && CPU == .X64] `SI_TIMER :: - 2`
- [OS == .ANDROID && CPU == .X64] `SI_MESGQ :: - 3`
- [OS == .ANDROID && CPU == .X64] `SI_ASYNCIO :: - 4`
- [OS == .ANDROID && CPU == .X64] `SI_SIGIO :: - 5`
- [OS == .ANDROID && CPU == .X64] `SI_TKILL :: - 6`
- [OS == .ANDROID && CPU == .X64] `SI_DETHREAD :: - 7`
- [OS == .ANDROID && CPU == .X64] `SI_ASYNCNL :: - 60`
- [OS == .ANDROID && CPU == .X64] `ILL_ILLOPC :: 1`
- [OS == .ANDROID && CPU == .X64] `ILL_ILLOPN :: 2`
- [OS == .ANDROID && CPU == .X64] `ILL_ILLADR :: 3`
- [OS == .ANDROID && CPU == .X64] `ILL_ILLTRP :: 4`
- [OS == .ANDROID && CPU == .X64] `ILL_PRVOPC :: 5`
- [OS == .ANDROID && CPU == .X64] `ILL_PRVREG :: 6`
- [OS == .ANDROID && CPU == .X64] `ILL_COPROC :: 7`
- [OS == .ANDROID && CPU == .X64] `ILL_BADSTK :: 8`
- [OS == .ANDROID && CPU == .X64] `ILL_BADIADDR :: 9`
- [OS == .ANDROID && CPU == .X64] `__ILL_BREAK :: 10`
- [OS == .ANDROID && CPU == .X64] `__ILL_BNDMOD :: 11`
- [OS == .ANDROID && CPU == .X64] `NSIGILL :: 11`
- [OS == .ANDROID && CPU == .X64] `FPE_INTDIV :: 1`
- [OS == .ANDROID && CPU == .X64] `FPE_INTOVF :: 2`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTDIV :: 3`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTOVF :: 4`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTUND :: 5`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTRES :: 6`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTINV :: 7`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTSUB :: 8`
- [OS == .ANDROID && CPU == .X64] `__FPE_DECOVF :: 9`
- [OS == .ANDROID && CPU == .X64] `__FPE_DECDIV :: 10`
- [OS == .ANDROID && CPU == .X64] `__FPE_DECERR :: 11`
- [OS == .ANDROID && CPU == .X64] `__FPE_INVASC :: 12`
- [OS == .ANDROID && CPU == .X64] `__FPE_INVDEC :: 13`
- [OS == .ANDROID && CPU == .X64] `FPE_FLTUNK :: 14`
- [OS == .ANDROID && CPU == .X64] `FPE_CONDTRAP :: 15`
- [OS == .ANDROID && CPU == .X64] `NSIGFPE :: 15`
- [OS == .ANDROID && CPU == .X64] `SEGV_MAPERR :: 1`
- [OS == .ANDROID && CPU == .X64] `SEGV_ACCERR :: 2`
- [OS == .ANDROID && CPU == .X64] `SEGV_BNDERR :: 3`
- [OS == .ANDROID && CPU == .X64] `SEGV_PKUERR :: 4`
- [OS == .ANDROID && CPU == .X64] `SEGV_ACCADI :: 5`
- [OS == .ANDROID && CPU == .X64] `SEGV_ADIDERR :: 6`
- [OS == .ANDROID && CPU == .X64] `SEGV_ADIPERR :: 7`
- [OS == .ANDROID && CPU == .X64] `SEGV_MTEAERR :: 8`
- [OS == .ANDROID && CPU == .X64] `SEGV_MTESERR :: 9`
- [OS == .ANDROID && CPU == .X64] `NSIGSEGV :: 9`
- [OS == .ANDROID && CPU == .X64] `BUS_ADRALN :: 1`
- [OS == .ANDROID && CPU == .X64] `BUS_ADRERR :: 2`
- [OS == .ANDROID && CPU == .X64] `BUS_OBJERR :: 3`
- [OS == .ANDROID && CPU == .X64] `BUS_MCEERR_AR :: 4`
- [OS == .ANDROID && CPU == .X64] `BUS_MCEERR_AO :: 5`
- [OS == .ANDROID && CPU == .X64] `NSIGBUS :: 5`
- [OS == .ANDROID && CPU == .X64] `TRAP_BRKPT :: 1`
- [OS == .ANDROID && CPU == .X64] `TRAP_TRACE :: 2`
- [OS == .ANDROID && CPU == .X64] `TRAP_BRANCH :: 3`
- [OS == .ANDROID && CPU == .X64] `TRAP_HWBKPT :: 4`
- [OS == .ANDROID && CPU == .X64] `TRAP_UNK :: 5`
- [OS == .ANDROID && CPU == .X64] `TRAP_PERF :: 6`
- [OS == .ANDROID && CPU == .X64] `NSIGTRAP :: 6`
- [OS == .ANDROID && CPU == .X64] `TRAP_PERF_FLAG_ASYNC :: 1 << 0`
- [OS == .ANDROID && CPU == .X64] `CLD_EXITED :: 1`
- [OS == .ANDROID && CPU == .X64] `CLD_KILLED :: 2`
- [OS == .ANDROID && CPU == .X64] `CLD_DUMPED :: 3`
- [OS == .ANDROID && CPU == .X64] `CLD_TRAPPED :: 4`
- [OS == .ANDROID && CPU == .X64] `CLD_STOPPED :: 5`
- [OS == .ANDROID && CPU == .X64] `CLD_CONTINUED :: 6`
- [OS == .ANDROID && CPU == .X64] `NSIGCHLD :: 6`
- [OS == .ANDROID && CPU == .X64] `POLL_IN :: 1`
- [OS == .ANDROID && CPU == .X64] `POLL_OUT :: 2`
- [OS == .ANDROID && CPU == .X64] `POLL_MSG :: 3`
- [OS == .ANDROID && CPU == .X64] `POLL_ERR :: 4`
- [OS == .ANDROID && CPU == .X64] `POLL_PRI :: 5`
- [OS == .ANDROID && CPU == .X64] `POLL_HUP :: 6`
- [OS == .ANDROID && CPU == .X64] `NSIGPOLL :: 6`
- [OS == .ANDROID && CPU == .X64] `SYS_SECCOMP :: 1`
- [OS == .ANDROID && CPU == .X64] `SYS_USER_DISPATCH :: 2`
- [OS == .ANDROID && CPU == .X64] `NSIGSYS :: 2`
- [OS == .ANDROID && CPU == .X64] `EMT_TAGOVF :: 1`
- [OS == .ANDROID && CPU == .X64] `NSIGEMT :: 1`
- [OS == .ANDROID && CPU == .X64] `SIGEV_SIGNAL :: 0`
- [OS == .ANDROID && CPU == .X64] `SIGEV_NONE :: 1`
- [OS == .ANDROID && CPU == .X64] `SIGEV_THREAD :: 2`
- [OS == .ANDROID && CPU == .X64] `SIGEV_THREAD_ID :: 4`
- [OS == .ANDROID && CPU == .X64] `SIGEV_MAX_SIZE :: 64`
- [OS == .ANDROID && CPU == .X64] `SS_ONSTACK :: 1`
- [OS == .ANDROID && CPU == .X64] `SS_DISABLE :: 2`
- [OS == .ANDROID && CPU == .X64] `SS_AUTODISARM :: 1 << 31`
- [OS == .ANDROID && CPU == .X64] `SS_FLAG_BITS :: SS_AUTODISARM`
- [OS == .ANDROID && CPU == .X64] `_KERNEL__NSIG :: 64`
- [OS == .ANDROID && CPU == .X64] `_NSIG :: _KERNEL__NSIG + 1`
- [OS == .ANDROID && CPU == .X64] `NSIG :: _NSIG`
- [OS == .ANDROID && CPU == .X64] `PAGE_SIZE :: 4096`
- [OS == .ANDROID && CPU == .X64] `PAGE_MASK :: ~(PAGE_SIZE - 1)`
- [OS == .ANDROID && CPU == .X64] `IPC_PRIVATE :: cast(s32) 0`
- [OS == .ANDROID && CPU == .X64] `IPC_CREAT :: 0x200`
- [OS == .ANDROID && CPU == .X64] `IPC_EXCL :: 0x400`
- [OS == .ANDROID && CPU == .X64] `IPC_NOWAIT :: 0x800`
- [OS == .ANDROID && CPU == .X64] `IPC_DIPC :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `IPC_OWN :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `IPC_RMID :: 0`
- [OS == .ANDROID && CPU == .X64] `IPC_SET :: 1`
- [OS == .ANDROID && CPU == .X64] `IPC_STAT :: 2`
- [OS == .ANDROID && CPU == .X64] `IPC_INFO :: 3`
- [OS == .ANDROID && CPU == .X64] `IPC_OLD :: 0`
- [OS == .ANDROID && CPU == .X64] `IPC_64 :: 0x0100`
- [OS == .ANDROID && CPU == .X64] `SEMOP :: 1`
- [OS == .ANDROID && CPU == .X64] `SEMGET :: 2`
- [OS == .ANDROID && CPU == .X64] `SEMCTL :: 3`
- [OS == .ANDROID && CPU == .X64] `SEMTIMEDOP :: 4`
- [OS == .ANDROID && CPU == .X64] `MSGSND :: 11`
- [OS == .ANDROID && CPU == .X64] `MSGRCV :: 12`
- [OS == .ANDROID && CPU == .X64] `MSGGET :: 13`
- [OS == .ANDROID && CPU == .X64] `MSGCTL :: 14`
- [OS == .ANDROID && CPU == .X64] `SHMAT :: 21`
- [OS == .ANDROID && CPU == .X64] `SHMDT :: 22`
- [OS == .ANDROID && CPU == .X64] `SHMGET :: 23`
- [OS == .ANDROID && CPU == .X64] `SHMCTL :: 24`
- [OS == .ANDROID && CPU == .X64] `DIPC :: 25`
- [OS == .ANDROID && CPU == .X64] `MAP_32BIT :: 0x40`
- [OS == .ANDROID && CPU == .X64] `PROT_READ :: 0x1`
- [OS == .ANDROID && CPU == .X64] `PROT_WRITE :: 0x2`
- [OS == .ANDROID && CPU == .X64] `PROT_EXEC :: 0x4`
- [OS == .ANDROID && CPU == .X64] `PROT_SEM :: 0x8`
- [OS == .ANDROID && CPU == .X64] `PROT_NONE :: 0x0`
- [OS == .ANDROID && CPU == .X64] `PROT_GROWSDOWN :: 0x01000000`
- [OS == .ANDROID && CPU == .X64] `PROT_GROWSUP :: 0x02000000`
- [OS == .ANDROID && CPU == .X64] `MAP_TYPE :: 0x0f`
- [OS == .ANDROID && CPU == .X64] `MAP_FIXED :: 0x10`
- [OS == .ANDROID && CPU == .X64] `MAP_ANONYMOUS :: 0x20`
- [OS == .ANDROID && CPU == .X64] `MAP_POPULATE :: 0x008000`
- [OS == .ANDROID && CPU == .X64] `MAP_NONBLOCK :: 0x010000`
- [OS == .ANDROID && CPU == .X64] `MAP_STACK :: 0x020000`
- [OS == .ANDROID && CPU == .X64] `MAP_HUGETLB :: 0x040000`
- [OS == .ANDROID && CPU == .X64] `MAP_SYNC :: 0x080000`
- [OS == .ANDROID && CPU == .X64] `MAP_FIXED_NOREPLACE :: 0x100000`
- [OS == .ANDROID && CPU == .X64] `MAP_UNINITIALIZED :: 0x4000000`
- [OS == .ANDROID && CPU == .X64] `MLOCK_ONFAULT :: 0x01`
- [OS == .ANDROID && CPU == .X64] `MS_ASYNC :: 1`
- [OS == .ANDROID && CPU == .X64] `MS_INVALIDATE :: 2`
- [OS == .ANDROID && CPU == .X64] `MS_SYNC :: 4`
- [OS == .ANDROID && CPU == .X64] `MADV_NORMAL :: 0`
- [OS == .ANDROID && CPU == .X64] `MADV_RANDOM :: 1`
- [OS == .ANDROID && CPU == .X64] `MADV_SEQUENTIAL :: 2`
- [OS == .ANDROID && CPU == .X64] `MADV_WILLNEED :: 3`
- [OS == .ANDROID && CPU == .X64] `MADV_DONTNEED :: 4`
- [OS == .ANDROID && CPU == .X64] `MADV_FREE :: 8`
- [OS == .ANDROID && CPU == .X64] `MADV_REMOVE :: 9`
- [OS == .ANDROID && CPU == .X64] `MADV_DONTFORK :: 10`
- [OS == .ANDROID && CPU == .X64] `MADV_DOFORK :: 11`
- [OS == .ANDROID && CPU == .X64] `MADV_HWPOISON :: 100`
- [OS == .ANDROID && CPU == .X64] `MADV_SOFT_OFFLINE :: 101`
- [OS == .ANDROID && CPU == .X64] `MADV_MERGEABLE :: 12`
- [OS == .ANDROID && CPU == .X64] `MADV_UNMERGEABLE :: 13`
- [OS == .ANDROID && CPU == .X64] `MADV_HUGEPAGE :: 14`
- [OS == .ANDROID && CPU == .X64] `MADV_NOHUGEPAGE :: 15`
- [OS == .ANDROID && CPU == .X64] `MADV_DONTDUMP :: 16`
- [OS == .ANDROID && CPU == .X64] `MADV_DODUMP :: 17`
- [OS == .ANDROID && CPU == .X64] `MADV_WIPEONFORK :: 18`
- [OS == .ANDROID && CPU == .X64] `MADV_KEEPONFORK :: 19`
- [OS == .ANDROID && CPU == .X64] `MADV_COLD :: 20`
- [OS == .ANDROID && CPU == .X64] `MADV_PAGEOUT :: 21`
- [OS == .ANDROID && CPU == .X64] `MADV_POPULATE_READ :: 22`
- [OS == .ANDROID && CPU == .X64] `MADV_POPULATE_WRITE :: 23`
- [OS == .ANDROID && CPU == .X64] `MADV_DONTNEED_LOCKED :: 24`
- [OS == .ANDROID && CPU == .X64] `MADV_COLLAPSE :: 25`
- [OS == .ANDROID && CPU == .X64] `MAP_FILE :: 0`
- [OS == .ANDROID && CPU == .X64] `PKEY_DISABLE_ACCESS :: 0x1`
- [OS == .ANDROID && CPU == .X64] `PKEY_DISABLE_WRITE :: 0x2`
- [OS == .ANDROID && CPU == .X64] `PKEY_ACCESS_MASK :: PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE`
- [OS == .ANDROID && CPU == .X64] `MAP_GROWSDOWN :: 0x0100`
- [OS == .ANDROID && CPU == .X64] `MAP_DENYWRITE :: 0x0800`
- [OS == .ANDROID && CPU == .X64] `MAP_EXECUTABLE :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `MAP_LOCKED :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `MAP_NORESERVE :: 0x4000`
- [OS == .ANDROID && CPU == .X64] `MCL_CURRENT :: 1`
- [OS == .ANDROID && CPU == .X64] `MCL_FUTURE :: 2`
- [OS == .ANDROID && CPU == .X64] `MCL_ONFAULT :: 4`
- [OS == .ANDROID && CPU == .X64] `MREMAP_MAYMOVE :: 1`
- [OS == .ANDROID && CPU == .X64] `MREMAP_FIXED :: 2`
- [OS == .ANDROID && CPU == .X64] `MREMAP_DONTUNMAP :: 4`
- [OS == .ANDROID && CPU == .X64] `OVERCOMMIT_GUESS :: 0`
- [OS == .ANDROID && CPU == .X64] `OVERCOMMIT_ALWAYS :: 1`
- [OS == .ANDROID && CPU == .X64] `OVERCOMMIT_NEVER :: 2`
- [OS == .ANDROID && CPU == .X64] `MAP_SHARED :: 0x01`
- [OS == .ANDROID && CPU == .X64] `MAP_PRIVATE :: 0x02`
- [OS == .ANDROID && CPU == .X64] `MAP_SHARED_VALIDATE :: 0x03`
- [OS == .ANDROID && CPU == .X64] `MAP_ANON :: MAP_ANONYMOUS`
- [OS == .ANDROID && CPU == .X64] `POLLIN :: 0x0001`
- [OS == .ANDROID && CPU == .X64] `POLLPRI :: 0x0002`
- [OS == .ANDROID && CPU == .X64] `POLLOUT :: 0x0004`
- [OS == .ANDROID && CPU == .X64] `POLLERR :: 0x0008`
- [OS == .ANDROID && CPU == .X64] `POLLHUP :: 0x0010`
- [OS == .ANDROID && CPU == .X64] `POLLNVAL :: 0x0020`
- [OS == .ANDROID && CPU == .X64] `POLLRDNORM :: 0x0040`
- [OS == .ANDROID && CPU == .X64] `POLLRDBAND :: 0x0080`
- [OS == .ANDROID && CPU == .X64] `POLLWRNORM :: 0x0100`
- [OS == .ANDROID && CPU == .X64] `POLLWRBAND :: 0x0200`
- [OS == .ANDROID && CPU == .X64] `POLLMSG :: 0x0400`
- [OS == .ANDROID && CPU == .X64] `POLLREMOVE :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `POLLRDHUP :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `POLLFREE :: cast(u32) 0x4000`
- [OS == .ANDROID && CPU == .X64] `POLL_BUSY_LOOP :: cast(u32) 0x8000`
- [OS == .ANDROID && CPU == .X64] `CLOCKS_PER_SEC :: 1000000`
- [OS == .ANDROID && CPU == .X64] `TIME_UTC :: xx clockid_t.REALTIME+1`
- [OS == .ANDROID && CPU == .X64] `TIME_MONOTONIC :: xx clockid_t.MONOTONIC+1`
- [OS == .ANDROID && CPU == .X64] `TIME_ACTIVE :: xx clockid_t.PROCESS_CPUTIME_ID+1`
- [OS == .ANDROID && CPU == .X64] `TIME_THREAD_ACTIVE :: xx clockid_t.THREAD_CPUTIME_ID+1`
- [OS == .ANDROID && CPU == .X64] `SEEK_SET :: 0`
- [OS == .ANDROID && CPU == .X64] `SEEK_CUR :: 1`
- [OS == .ANDROID && CPU == .X64] `SEEK_END :: 2`
- [OS == .ANDROID && CPU == .X64] `SEEK_DATA :: 3`
- [OS == .ANDROID && CPU == .X64] `SEEK_HOLE :: 4`
- [OS == .ANDROID && CPU == .X64] `using _SC_definitions :: enum s32`
- [OS == .ANDROID && CPU == .X64] `STDIN_FILENO :: 0`
- [OS == .ANDROID && CPU == .X64] `STDOUT_FILENO :: 1`
- [OS == .ANDROID && CPU == .X64] `STDERR_FILENO :: 2`
- [OS == .ANDROID && CPU == .X64] `F_OK :: 0`
- [OS == .ANDROID && CPU == .X64] `X_OK :: 1`
- [OS == .ANDROID && CPU == .X64] `W_OK :: 2`
- [OS == .ANDROID && CPU == .X64] `R_OK :: 4`
- [OS == .ANDROID && CPU == .X64] `PC :: enum u32`
- [OS == .ANDROID && CPU == .X64] `SHMMIN :: 1`
- [OS == .ANDROID && CPU == .X64] `SHMMNI :: 4096`
- [OS == .ANDROID && CPU == .X64] `SHMSEG :: SHMMNI`
- [OS == .ANDROID && CPU == .X64] `SHM_R :: 0x100`
- [OS == .ANDROID && CPU == .X64] `SHM_W :: 0x80`
- [OS == .ANDROID && CPU == .X64] `SHM_HUGETLB :: 0x800`
- [OS == .ANDROID && CPU == .X64] `SHM_NORESERVE :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `SHM_RDONLY :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `SHM_RND :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `SHM_REMAP :: 0x4000`
- [OS == .ANDROID && CPU == .X64] `SHM_EXEC :: 0x8000`
- [OS == .ANDROID && CPU == .X64] `SHM_LOCK :: 11`
- [OS == .ANDROID && CPU == .X64] `SHM_UNLOCK :: 12`
- [OS == .ANDROID && CPU == .X64] `SHM_STAT :: 13`
- [OS == .ANDROID && CPU == .X64] `SHM_INFO :: 14`
- [OS == .ANDROID && CPU == .X64] `SHM_STAT_ANY :: 15`
- [OS == .ANDROID && CPU == .X64] `SHMLBA :: 4096`
- [OS == .ANDROID && CPU == .X64] `SYS_NMLN :: 65`
- [OS == .ANDROID && CPU == .X64] `WNOHANG :: 0x00000001`
- [OS == .ANDROID && CPU == .X64] `WUNTRACED :: 0x00000002`
- [OS == .ANDROID && CPU == .X64] `WSTOPPED :: WUNTRACED`
- [OS == .ANDROID && CPU == .X64] `WEXITED :: 0x00000004`
- [OS == .ANDROID && CPU == .X64] `WCONTINUED :: 0x00000008`
- [OS == .ANDROID && CPU == .X64] `WNOWAIT :: 0x01000000`
- [OS == .ANDROID && CPU == .X64] `__WNOTHREAD :: 0x20000000`
- [OS == .ANDROID && CPU == .X64] `__WALL :: 0x40000000`
- [OS == .ANDROID && CPU == .X64] `__WCLONE :: 0x80000000`
- [OS == .ANDROID && CPU == .X64] `P_ALL :: 0`
- [OS == .ANDROID && CPU == .X64] `P_PID :: 1`
- [OS == .ANDROID && CPU == .X64] `P_PGID :: 2`
- [OS == .ANDROID && CPU == .X64] `P_PIDFD :: 3`
- [OS == .ANDROID && CPU == .X64] `ptrdiff_t :: s64`
- [OS == .ANDROID && CPU == .X64] `Dl_info :: struct`
- [OS == .ANDROID && CPU == .X64] `dlopen :: (__filename: *u8, __flag: s32) -> *void #foreign libdl`
- [OS == .ANDROID && CPU == .X64] `dlclose :: (__handle: *void) -> s32 #foreign libdl`
- [OS == .ANDROID && CPU == .X64] `dlerror :: () -> *u8 #foreign libdl`
- [OS == .ANDROID && CPU == .X64] `dlsym :: (__handle: *void, __symbol: *u8) -> *void #foreign libdl`
- [OS == .ANDROID && CPU == .X64] `dlvsym :: (__handle: *void, __symbol: *u8, __version: *u8) -> *void #foreign libdl`
- [OS == .ANDROID && CPU == .X64] `dladdr :: (__addr: *void, __info: *Dl_info) -> s32 #foreign libdl`
- [OS == .ANDROID && CPU == .X64] `__kernel_fd_set :: struct`
- [OS == .ANDROID && CPU == .X64] `__kernel_fsid_t :: struct`
- [OS == .ANDROID && CPU == .X64] `gid_t :: u32`
- [OS == .ANDROID && CPU == .X64] `uid_t :: u32`
- [OS == .ANDROID && CPU == .X64] `pid_t :: s32`
- [OS == .ANDROID && CPU == .X64] `blkcnt_t :: u64`
- [OS == .ANDROID && CPU == .X64] `blksize_t :: u64`
- [OS == .ANDROID && CPU == .X64] `clock_t :: s64`
- [OS == .ANDROID && CPU == .X64] `fsblkcnt_t :: u64`
- [OS == .ANDROID && CPU == .X64] `fsfilcnt_t :: u64`
- [OS == .ANDROID && CPU == .X64] `mode_t :: u32`
- [OS == .ANDROID && CPU == .X64] `key_t :: s32`
- [OS == .ANDROID && CPU == .X64] `ino_t :: u64`
- [OS == .ANDROID && CPU == .X64] `ino64_t :: u64`
- [OS == .ANDROID && CPU == .X64] `nlink_t :: u32`
- [OS == .ANDROID && CPU == .X64] `timer_t :: *void`
- [OS == .ANDROID && CPU == .X64] `suseconds_t :: s64`
- [OS == .ANDROID && CPU == .X64] `useconds_t :: u32`
- [OS == .ANDROID && CPU == .X64] `dev_t :: u64`
- [OS == .ANDROID && CPU == .X64] `time_t :: s64`
- [OS == .ANDROID && CPU == .X64] `uint_t :: u32`
- [OS == .ANDROID && CPU == .X64] `dirent :: struct`
- [OS == .ANDROID && CPU == .X64] `dirent64 :: struct`
- [OS == .ANDROID && CPU == .X64] `DIR :: struct`
- [OS == .ANDROID && CPU == .X64] `opendir :: (__path: *u8) -> *DIR #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fdopendir :: (__dir_fd: s32) -> *DIR #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readdir :: (__dir: *DIR) -> *dirent #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readdir64 :: (__dir: *DIR) -> *dirent64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readdir_r :: (__dir: *DIR, __entry: *dirent, __buffer: **dirent) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readdir64_r :: (__dir: *DIR, __entry: *dirent64, __buffer: **dirent64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `closedir :: (__dir: *DIR) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `rewinddir :: (__dir: *DIR) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `seekdir :: (__dir: *DIR, __location: s64) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `telldir :: (__dir: *DIR) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `dirfd :: (__dir: *DIR) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `alphasort :: (__lhs: **dirent, __rhs: **dirent) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `alphasort64 :: (__lhs: **dirent64, __rhs: **dirent64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `scandir :: (__path: *u8, __name_list: ***dirent, __filter: #type (unknown0: *dirent) -> s32 #c_call, __comparator: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `scandir64 :: (__path: *u8, __name_list: ***dirent64, __filter: #type (unknown0: *dirent64) -> s32 #c_call, __comparator: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `scandirat64 :: (__dir_fd: s32, __path: *u8, __name_list: ***dirent64, __filter: #type (unknown0: *dirent64) -> s32 #c_call, __comparator: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `scandirat :: (__dir_fd: s32, __path: *u8, __name_list: ***dirent, __filter: #type (unknown0: *dirent) -> s32 #c_call, __comparator: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__errno_location :: () -> *OS_Error_Code #foreign libc "__errno"`
- [OS == .ANDROID && CPU == .X64] `backtrace :: (buffer: **void, size: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `backtrace_symbols :: (buffer: **void, size: s32) -> **u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `backtrace_symbols_fd :: (buffer: **void, size: s32, fd: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fnmatch :: (__pattern: *u8, __string: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `passwd :: struct`
- [OS == .ANDROID && CPU == .X64] `getpwnam :: (__name: *u8) -> *passwd #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpwuid :: (__uid: uid_t) -> *passwd #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpwent :: () -> *passwd #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setpwent :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `endpwent :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpwnam_r :: (__name: *u8, __pwd: *passwd, __buf: *u8, __n: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpwuid_r :: (__uid: uid_t, __pwd: *passwd, __buf: *u8, __n: u64, __result: **passwd) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `_fpx_sw_bytes :: struct`
- [OS == .ANDROID && CPU == .X64] `_fpstate_64 :: struct`
- [OS == .ANDROID && CPU == .X64] `sigcontext :: struct`
- [OS == .ANDROID && CPU == .X64] `timespec :: struct`
- [OS == .ANDROID && CPU == .X64] `timeval :: struct`
- [OS == .ANDROID && CPU == .X64] `itimerspec :: struct`
- [OS == .ANDROID && CPU == .X64] `itimerval :: struct`
- [OS == .ANDROID && CPU == .X64] `timezone :: struct`
- [OS == .ANDROID && CPU == .X64] `sigset_t :: u64`
- [OS == .ANDROID && CPU == .X64] `__kernel_sigaction :: struct`
- [OS == .ANDROID && CPU == .X64] `sigaltstack_t :: struct`
- [OS == .ANDROID && CPU == .X64] `stack_t :: sigaltstack_t`
- [OS == .ANDROID && CPU == .X64] `sigval :: struct`
- [OS == .ANDROID && CPU == .X64] `sigval_t :: sigval`
- [OS == .ANDROID && CPU == .X64] `__sifields :: struct`
- [OS == .ANDROID && CPU == .X64] `siginfo :: struct`
- [OS == .ANDROID && CPU == .X64] `siginfo_t :: siginfo`
- [OS == .ANDROID && CPU == .X64] `sigevent :: struct`
- [OS == .ANDROID && CPU == .X64] `sigevent_t :: sigevent`
- [OS == .ANDROID && CPU == .X64] `sig_atomic_t :: s32`
- [OS == .ANDROID && CPU == .X64] `sig_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .ANDROID && CPU == .X64] `sighandler_t :: #type (unknown0: s32) -> void #c_call`
- [OS == .ANDROID && CPU == .X64] `sigset64_t :: sigset_t`
- [OS == .ANDROID && CPU == .X64] `sigaction_t :: struct`
- [OS == .ANDROID && CPU == .X64] `sigaction64_t :: struct`
- [OS == .ANDROID && CPU == .X64] `user_fpregs_struct :: struct`
- [OS == .ANDROID && CPU == .X64] `user_regs_struct :: struct`
- [OS == .ANDROID && CPU == .X64] `user :: struct`
- [OS == .ANDROID && CPU == .X64] `REG :: enum u32`
- [OS == .ANDROID && CPU == .X64] `greg_t :: s64`
- [OS == .ANDROID && CPU == .X64] `gregset_t :: [23] greg_t`
- [OS == .ANDROID && CPU == .X64] `_libc_fpxreg :: struct`
- [OS == .ANDROID && CPU == .X64] `_libc_xmmreg :: struct`
- [OS == .ANDROID && CPU == .X64] `_libc_fpstate :: struct`
- [OS == .ANDROID && CPU == .X64] `fpregset_t :: *_libc_fpstate`
- [OS == .ANDROID && CPU == .X64] `mcontext_t :: struct`
- [OS == .ANDROID && CPU == .X64] `ucontext :: struct`
- [OS == .ANDROID && CPU == .X64] `ucontext_t :: ucontext`
- [OS == .ANDROID && CPU == .X64] `__libc_current_sigrtmin :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__libc_current_sigrtmax :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sys_siglist: [65] *u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `sys_signame: [65] *u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `sigaction :: (__signal: s32, __new_action: *sigaction_t, __old_action: *sigaction_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigaction64 :: (__signal: s32, __new_action: *sigaction64_t, __old_action: *sigaction64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `siginterrupt :: (__signal: s32, __flag: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `signal :: (__signal: s32, __handler: sighandler_t) -> sighandler_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigaddset :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigaddset64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigdelset :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigdelset64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigemptyset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigemptyset64 :: (__set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigfillset :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigfillset64 :: (__set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigismember :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigismember64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigpending :: (__set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigpending64 :: (__set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigprocmask :: (__how: s32, __new_set: *sigset_t, __old_set: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigprocmask64 :: (__how: s32, __new_set: *sigset64_t, __old_set: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigsuspend :: (__mask: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigsuspend64 :: (__mask: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigwait :: (__set: *sigset_t, __signal: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigwait64 :: (__set: *sigset64_t, __signal: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sighold :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigignore :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigpause :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigrelse :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigset :: (__signal: s32, __handler: sighandler_t) -> sighandler_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `raise :: (__signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `kill :: (__pid: pid_t, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `killpg :: (__pgrp: s32, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tgkill :: (__tgid: s32, __tid: s32, __signal: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigaltstack :: (__new_signal_stack: *stack_t, __old_signal_stack: *stack_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `psiginfo :: (__info: *siginfo_t, __msg: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `psignal :: (__signal: s32, __msg: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigqueue :: (__pid: pid_t, __signal: s32, __value: sigval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigtimedwait :: (__set: *sigset_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigtimedwait64 :: (__set: *sigset64_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigwaitinfo :: (__set: *sigset_t, __info: *siginfo_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigwaitinfo64 :: (__set: *sigset64_t, __info: *siginfo_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ipc_kludge :: struct`
- [OS == .ANDROID && CPU == .X64] `ftok :: (__path: *u8, __id: s32) -> key_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `iovec :: struct`
- [OS == .ANDROID && CPU == .X64] `mmap :: (__addr: *void, __size: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mmap64 :: (__addr: *void, __size: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `munmap :: (__addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `msync :: (__addr: *void, __size: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mprotect :: (__addr: *void, __size: u64, __prot: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mremap :: (__old_addr: *void, __old_size: u64, __new_size: u64, __flags: s32, __args: ..Any) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mlockall :: (__flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `munlockall :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mlock :: (__addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mlock2 :: (__addr: *void, __size: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `munlock :: (__addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mincore :: (__addr: *void, __size: u64, __vector: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `madvise :: (__addr: *void, __size: u64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `process_madvise :: (__pid_fd: s32, __iov: *iovec, __count: u64, __advice: s32, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `memfd_create :: (__name: *u8, __flags: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `posix_madvise :: (__addr: *void, __size: u64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pollfd :: struct`
- [OS == .ANDROID && CPU == .X64] `gettimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `settimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getitimer :: (__which: s32, __current_value: *itimerval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setitimer :: (__which: s32, __new_value: *itimerval, __old_value: *itimerval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `utimes :: (__path: *u8, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `futimes :: (__fd: s32, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `lutimes :: (__path: *u8, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `futimesat :: (__dir_fd: s32, __path: *u8, __times: *[2] timeval) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__locale_t :: struct`
- [OS == .ANDROID && CPU == .X64] `locale_t :: *__locale_t`
- [OS == .ANDROID && CPU == .X64] `tzname: **u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `daylight: s32 #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `tm :: struct`
- [OS == .ANDROID && CPU == .X64] `time :: (__t: *time_t) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `nanosleep :: (__request: *timespec, __remainder: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `asctime :: (__tm: *tm) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `asctime_r :: (__tm: *tm, __buf: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `difftime :: (__lhs: time_t, __rhs: time_t) -> float64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mktime :: (__tm: *tm) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `localtime :: (__t: *time_t) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .X64] `localtime_r :: (__t: *time_t, __tm: *tm) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timelocal :: (__tm: *tm) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `gmtime :: (__t: *time_t) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .X64] `gmtime_r :: (__t: *time_t, __tm: *tm) -> *tm #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timegm :: (__tm: *tm) -> time_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `strptime :: (__s: *u8, __fmt: *u8, __tm: *tm) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `strptime_l :: (__s: *u8, __fmt: *u8, __tm: *tm, __l: locale_t) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `strftime :: (__buf: *u8, __n: u64, __fmt: *u8, __tm: *tm) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `strftime_l :: (__buf: *u8, __n: u64, __fmt: *u8, __tm: *tm, __l: locale_t) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ctime :: (__t: *time_t) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ctime_r :: (__t: *time_t, __buf: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tzset :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clock :: () -> clock_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clock_getcpuclockid :: (__pid: pid_t, __clock: *clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clock_getres :: (__clock: clockid_t, __resolution: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clock_gettime :: (__clock: clockid_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clock_nanosleep :: (__clock: clockid_t, __flags: s32, __request: *timespec, __remainder: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clock_settime :: (__clock: clockid_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timer_create :: (__clock: clockid_t, __event: *sigevent, __timer_ptr: *timer_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timer_delete :: (__timer: timer_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timer_settime :: (__timer: timer_t, __flags: s32, __new_value: *itimerspec, __old_value: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timer_gettime :: (_timer: timer_t, __ts: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timer_getoverrun :: (__timer: timer_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timespec_get :: (__ts: *timespec, __base: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `nfds_t :: u32`
- [OS == .ANDROID && CPU == .X64] `poll :: (__fds: *pollfd, __count: nfds_t, __timeout_ms: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ppoll :: (__fds: *pollfd, __count: nfds_t, __timeout: *timespec, __mask: *sigset_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ppoll64 :: (__fds: *pollfd, __count: nfds_t, __timeout: *timespec, __mask: *sigset64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sysconf :: (name: _SC_definitions) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `environ: **u8 #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `_exit :: (__status: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fork :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `vfork :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `gettid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpgid :: (__pid: pid_t) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setpgid :: (__pid: pid_t, __pgid: pid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getppid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpgrp :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setpgrp :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getsid :: (__pid: pid_t) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setsid :: () -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execvpe :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execve :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execl :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execlp :: (__file: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `execle :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fexecve :: (__fd: s32, __argv: **u8, __envp: **u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `nice :: (__incr: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setegid :: (__gid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `seteuid :: (__uid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setgid :: (__gid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setregid :: (__rgid: gid_t, __egid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setresgid :: (__rgid: gid_t, __egid: gid_t, __sgid: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setresuid :: (__ruid: uid_t, __euid: uid_t, __suid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setreuid :: (__ruid: uid_t, __euid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setuid :: (__uid: uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getuid :: () -> uid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `geteuid :: () -> uid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getgid :: () -> gid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getegid :: () -> gid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getgroups :: (__size: s32, __list: *gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setgroups :: (__size: u64, __list: *gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getresuid :: (__ruid: *uid_t, __euid: *uid_t, __suid: *uid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getresgid :: (__rgid: *gid_t, __egid: *gid_t, __sgid: *gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getlogin :: () -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getlogin_r :: (__buffer: *u8, __buffer_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fpathconf :: (__fd: s32, __name: PC) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pathconf :: (__path: *u8, __name: PC) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `access :: (__path: *u8, __mode: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `faccessat :: (__dirfd: s32, __path: *u8, __mode: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `link :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `linkat :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `unlink :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `unlinkat :: (__dirfd: s32, __path: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `chdir :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fchdir :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `rmdir :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pipe :: (__fds: *[2] s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pipe2 :: (__fds: *[2] s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `chroot :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `symlink :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `symlinkat :: (__old_path: *u8, __new_dir_fd: s32, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readlink :: (__path: *u8, __buf: *u8, __buf_size: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readlinkat :: (__dir_fd: s32, __path: *u8, __buf: *u8, __buf_size: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `chown :: (__path: *u8, __owner: uid_t, __group: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fchown :: (__fd: s32, __owner: uid_t, __group: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fchownat :: (__dir_fd: s32, __path: *u8, __owner: uid_t, __group: gid_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `lchown :: (__path: *u8, __owner: uid_t, __group: gid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getcwd :: (__buf: *u8, __size: u64) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sync :: () -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `syncfs :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `close :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `read :: (__fd: s32, __buf: *void, __count: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `write :: (__fd: s32, __buf: *void, __count: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `dup :: (__old_fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `dup2 :: (__old_fd: s32, __new_fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `dup3 :: (__old_fd: s32, __new_fd: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fsync :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fdatasync :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `truncate :: (__path: *u8, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `lseek :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pread :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pwrite :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ftruncate :: (__fd: s32, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `truncate64 :: (__path: *u8, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `lseek64 :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pread64 :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pwrite64 :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ftruncate64 :: (__fd: s32, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `alarm :: (__seconds: u32) -> u32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sleep :: (__seconds: u32) -> u32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `usleep :: (__microseconds: useconds_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `gethostname :: (_buf: *u8, __buf_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sethostname :: (__name: *u8, __n: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `brk :: (__addr: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sbrk :: (__increment: ptrdiff_t) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `isatty :: (__fd: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ttyname :: (__fd: s32) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ttyname_r :: (__fd: s32, __buf: *u8, __buf_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `acct :: (__path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpagesize :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `daemon :: (__no_chdir: s32, __no_close: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tcgetpgrp :: (__fd: s32) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tcsetpgrp :: (__fd: s32, __pid: pid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getdomainname :: (__buf: *u8, __buf_size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setdomainname :: (__name: *u8, __n: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `copy_file_range :: (__fd_in: s32, __off_in: *s64, __fd_out: s32, __off_out: *s64, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `swab :: (__src: *void, __dst: *void, __byte_count: s64) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `close_range :: (__min_fd: u32, __max_fd: u32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `shminfo :: struct`
- [OS == .ANDROID && CPU == .X64] `shm_info :: struct`
- [OS == .ANDROID && CPU == .X64] `shmatt_t :: u64`
- [OS == .ANDROID && CPU == .X64] `shmat :: (__shm_id: s32, __addr: *void, __flags: s32) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `shmdt :: (__addr: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `shmget :: (key: key_t, __size: u64, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `utsname :: struct`
- [OS == .ANDROID && CPU == .X64] `uname :: (__buf: *utsname) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `wait :: (__status: *s32) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `waitpid :: (__pid: pid_t, __status: *s32, __options: s32) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `wait4 :: (__pid: pid_t, __status: *s32, __options: s32, __rusage: *rusage) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `idtype_t :: s32`
- [OS == .ANDROID && CPU == .X64] `waitid :: (__type: idtype_t, __id: id_t, __info: *siginfo_t, __options: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `_GNU_SOURCE :: 1`
- [OS == .ANDROID && CPU == .X64] `_IOFBF :: 0`
- [OS == .ANDROID && CPU == .X64] `_IOLBF :: 1`
- [OS == .ANDROID && CPU == .X64] `_IONBF :: 2`
- [OS == .ANDROID && CPU == .X64] `BUFSIZ :: 1024`
- [OS == .ANDROID && CPU == .X64] `EOF :: -1`
- [OS == .ANDROID && CPU == .X64] `FOPEN_MAX :: 20`
- [OS == .ANDROID && CPU == .X64] `FILENAME_MAX :: 4096`
- [OS == .ANDROID && CPU == .X64] `L_tmpnam :: 4096`
- [OS == .ANDROID && CPU == .X64] `TMP_MAX :: 308915776`
- [OS == .ANDROID && CPU == .X64] `P_tmpdir :: "/tmp/"`
- [OS == .ANDROID && CPU == .X64] `RENAME_NOREPLACE :: 1<<0`
- [OS == .ANDROID && CPU == .X64] `RENAME_EXCHANGE :: 1<<1`
- [OS == .ANDROID && CPU == .X64] `RENAME_WHITEOUT :: 1<<2`
- [OS == .ANDROID && CPU == .X64] `L_ctermid :: 1024`
- [OS == .ANDROID && CPU == .X64] `O_ACCMODE :: 0x3`
- [OS == .ANDROID && CPU == .X64] `O_RDONLY :: 0x0`
- [OS == .ANDROID && CPU == .X64] `O_WRONLY :: 0x1`
- [OS == .ANDROID && CPU == .X64] `O_RDWR :: 0x2`
- [OS == .ANDROID && CPU == .X64] `O_CREAT :: 0x40`
- [OS == .ANDROID && CPU == .X64] `O_EXCL :: 0x80`
- [OS == .ANDROID && CPU == .X64] `O_NOCTTY :: 0x100`
- [OS == .ANDROID && CPU == .X64] `O_TRUNC :: 0x200`
- [OS == .ANDROID && CPU == .X64] `O_APPEND :: 0x400`
- [OS == .ANDROID && CPU == .X64] `O_NONBLOCK :: 0x800`
- [OS == .ANDROID && CPU == .X64] `O_DSYNC :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `FASYNC :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `O_DIRECT :: 0x4000`
- [OS == .ANDROID && CPU == .X64] `O_LARGEFILE :: 0x8000`
- [OS == .ANDROID && CPU == .X64] `O_DIRECTORY :: 0x10000`
- [OS == .ANDROID && CPU == .X64] `O_NOFOLLOW :: 0x20000`
- [OS == .ANDROID && CPU == .X64] `O_NOATIME :: 0x40000`
- [OS == .ANDROID && CPU == .X64] `O_CLOEXEC :: 0x80000`
- [OS == .ANDROID && CPU == .X64] `__O_SYNC :: 0x100000`
- [OS == .ANDROID && CPU == .X64] `O_SYNC :: __O_SYNC | O_DSYNC`
- [OS == .ANDROID && CPU == .X64] `O_PATH :: 0x200000`
- [OS == .ANDROID && CPU == .X64] `__O_TMPFILE :: 0x400000`
- [OS == .ANDROID && CPU == .X64] `O_TMPFILE :: __O_TMPFILE | O_DIRECTORY`
- [OS == .ANDROID && CPU == .X64] `O_TMPFILE_MASK :: __O_TMPFILE | O_DIRECTORY | O_CREAT`
- [OS == .ANDROID && CPU == .X64] `O_NDELAY :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .X64] `F_DUPFD :: 0`
- [OS == .ANDROID && CPU == .X64] `F_GETFD :: 1`
- [OS == .ANDROID && CPU == .X64] `F_SETFD :: 2`
- [OS == .ANDROID && CPU == .X64] `F_GETFL :: 3`
- [OS == .ANDROID && CPU == .X64] `F_SETFL :: 4`
- [OS == .ANDROID && CPU == .X64] `F_GETLK :: 5`
- [OS == .ANDROID && CPU == .X64] `F_SETLK :: 6`
- [OS == .ANDROID && CPU == .X64] `F_SETLKW :: 7`
- [OS == .ANDROID && CPU == .X64] `F_SETOWN :: 8`
- [OS == .ANDROID && CPU == .X64] `F_GETOWN :: 9`
- [OS == .ANDROID && CPU == .X64] `F_SETSIG :: 10`
- [OS == .ANDROID && CPU == .X64] `F_GETSIG :: 11`
- [OS == .ANDROID && CPU == .X64] `F_SETOWN_EX :: 15`
- [OS == .ANDROID && CPU == .X64] `F_GETOWN_EX :: 16`
- [OS == .ANDROID && CPU == .X64] `F_GETOWNER_UIDS :: 17`
- [OS == .ANDROID && CPU == .X64] `F_OFD_GETLK :: 36`
- [OS == .ANDROID && CPU == .X64] `F_OFD_SETLK :: 37`
- [OS == .ANDROID && CPU == .X64] `F_OFD_SETLKW :: 38`
- [OS == .ANDROID && CPU == .X64] `F_OWNER_TID :: 0`
- [OS == .ANDROID && CPU == .X64] `F_OWNER_PID :: 1`
- [OS == .ANDROID && CPU == .X64] `F_OWNER_PGRP :: 2`
- [OS == .ANDROID && CPU == .X64] `FD_CLOEXEC :: 1`
- [OS == .ANDROID && CPU == .X64] `F_RDLCK :: 0`
- [OS == .ANDROID && CPU == .X64] `F_WRLCK :: 1`
- [OS == .ANDROID && CPU == .X64] `F_UNLCK :: 2`
- [OS == .ANDROID && CPU == .X64] `F_EXLCK :: 4`
- [OS == .ANDROID && CPU == .X64] `F_SHLCK :: 8`
- [OS == .ANDROID && CPU == .X64] `LOCK_SH :: 1`
- [OS == .ANDROID && CPU == .X64] `LOCK_EX :: 2`
- [OS == .ANDROID && CPU == .X64] `LOCK_NB :: 4`
- [OS == .ANDROID && CPU == .X64] `LOCK_UN :: 8`
- [OS == .ANDROID && CPU == .X64] `LOCK_MAND :: 32`
- [OS == .ANDROID && CPU == .X64] `LOCK_READ :: 64`
- [OS == .ANDROID && CPU == .X64] `LOCK_WRITE :: 128`
- [OS == .ANDROID && CPU == .X64] `LOCK_RW :: 192`
- [OS == .ANDROID && CPU == .X64] `F_LINUX_SPECIFIC_BASE :: 1024`
- [OS == .ANDROID && CPU == .X64] `F_SETLEASE :: F_LINUX_SPECIFIC_BASE + 0`
- [OS == .ANDROID && CPU == .X64] `F_GETLEASE :: F_LINUX_SPECIFIC_BASE + 1`
- [OS == .ANDROID && CPU == .X64] `F_CANCELLK :: F_LINUX_SPECIFIC_BASE + 5`
- [OS == .ANDROID && CPU == .X64] `F_DUPFD_CLOEXEC :: F_LINUX_SPECIFIC_BASE + 6`
- [OS == .ANDROID && CPU == .X64] `F_NOTIFY :: F_LINUX_SPECIFIC_BASE + 2`
- [OS == .ANDROID && CPU == .X64] `F_SETPIPE_SZ :: F_LINUX_SPECIFIC_BASE + 7`
- [OS == .ANDROID && CPU == .X64] `F_GETPIPE_SZ :: F_LINUX_SPECIFIC_BASE + 8`
- [OS == .ANDROID && CPU == .X64] `F_ADD_SEALS :: F_LINUX_SPECIFIC_BASE + 9`
- [OS == .ANDROID && CPU == .X64] `F_GET_SEALS :: F_LINUX_SPECIFIC_BASE + 10`
- [OS == .ANDROID && CPU == .X64] `F_SEAL_SEAL :: 0x0001`
- [OS == .ANDROID && CPU == .X64] `F_SEAL_SHRINK :: 0x0002`
- [OS == .ANDROID && CPU == .X64] `F_SEAL_GROW :: 0x0004`
- [OS == .ANDROID && CPU == .X64] `F_SEAL_WRITE :: 0x0008`
- [OS == .ANDROID && CPU == .X64] `F_SEAL_FUTURE_WRITE :: 0x0010`
- [OS == .ANDROID && CPU == .X64] `F_SEAL_EXEC :: 0x0020`
- [OS == .ANDROID && CPU == .X64] `F_GET_RW_HINT :: F_LINUX_SPECIFIC_BASE + 11`
- [OS == .ANDROID && CPU == .X64] `F_SET_RW_HINT :: F_LINUX_SPECIFIC_BASE + 12`
- [OS == .ANDROID && CPU == .X64] `F_GET_FILE_RW_HINT :: F_LINUX_SPECIFIC_BASE + 13`
- [OS == .ANDROID && CPU == .X64] `F_SET_FILE_RW_HINT :: F_LINUX_SPECIFIC_BASE + 14`
- [OS == .ANDROID && CPU == .X64] `RWH_WRITE_LIFE_NOT_SET :: 0`
- [OS == .ANDROID && CPU == .X64] `RWH_WRITE_LIFE_NONE :: 1`
- [OS == .ANDROID && CPU == .X64] `RWH_WRITE_LIFE_SHORT :: 2`
- [OS == .ANDROID && CPU == .X64] `RWH_WRITE_LIFE_MEDIUM :: 3`
- [OS == .ANDROID && CPU == .X64] `RWH_WRITE_LIFE_LONG :: 4`
- [OS == .ANDROID && CPU == .X64] `RWH_WRITE_LIFE_EXTREME :: 5`
- [OS == .ANDROID && CPU == .X64] `RWF_WRITE_LIFE_NOT_SET :: RWH_WRITE_LIFE_NOT_SET`
- [OS == .ANDROID && CPU == .X64] `DN_ACCESS :: 0x00000001`
- [OS == .ANDROID && CPU == .X64] `DN_MODIFY :: 0x00000002`
- [OS == .ANDROID && CPU == .X64] `DN_CREATE :: 0x00000004`
- [OS == .ANDROID && CPU == .X64] `DN_DELETE :: 0x00000008`
- [OS == .ANDROID && CPU == .X64] `DN_RENAME :: 0x00000010`
- [OS == .ANDROID && CPU == .X64] `DN_ATTRIB :: 0x00000020`
- [OS == .ANDROID && CPU == .X64] `DN_MULTISHOT :: 0x80000000`
- [OS == .ANDROID && CPU == .X64] `AT_FDCWD :: - 100`
- [OS == .ANDROID && CPU == .X64] `AT_SYMLINK_NOFOLLOW :: 0x100`
- [OS == .ANDROID && CPU == .X64] `AT_EACCESS :: 0x200`
- [OS == .ANDROID && CPU == .X64] `AT_REMOVEDIR :: 0x200`
- [OS == .ANDROID && CPU == .X64] `AT_SYMLINK_FOLLOW :: 0x400`
- [OS == .ANDROID && CPU == .X64] `AT_NO_AUTOMOUNT :: 0x800`
- [OS == .ANDROID && CPU == .X64] `AT_EMPTY_PATH :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `AT_STATX_SYNC_TYPE :: 0x6000`
- [OS == .ANDROID && CPU == .X64] `AT_STATX_SYNC_AS_STAT :: 0x0000`
- [OS == .ANDROID && CPU == .X64] `AT_STATX_FORCE_SYNC :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `AT_STATX_DONT_SYNC :: 0x4000`
- [OS == .ANDROID && CPU == .X64] `AT_RECURSIVE :: 0x8000`
- [OS == .ANDROID && CPU == .X64] `S_IFMT :: 0xf000`
- [OS == .ANDROID && CPU == .X64] `S_IFSOCK :: 0xc000`
- [OS == .ANDROID && CPU == .X64] `S_IFLNK :: 0xa000`
- [OS == .ANDROID && CPU == .X64] `S_IFREG :: 0x8000`
- [OS == .ANDROID && CPU == .X64] `S_IFBLK :: 0x6000`
- [OS == .ANDROID && CPU == .X64] `S_IFDIR :: 0x4000`
- [OS == .ANDROID && CPU == .X64] `S_IFCHR :: 0x2000`
- [OS == .ANDROID && CPU == .X64] `S_IFIFO :: 0x1000`
- [OS == .ANDROID && CPU == .X64] `S_ISUID :: 0x800`
- [OS == .ANDROID && CPU == .X64] `S_ISGID :: 0x400`
- [OS == .ANDROID && CPU == .X64] `S_ISVTX :: 0x200`
- [OS == .ANDROID && CPU == .X64] `S_IRWXU :: 0x1c0`
- [OS == .ANDROID && CPU == .X64] `S_IRUSR :: 0x100`
- [OS == .ANDROID && CPU == .X64] `S_IWUSR :: 0x80`
- [OS == .ANDROID && CPU == .X64] `S_IXUSR :: 0x40`
- [OS == .ANDROID && CPU == .X64] `S_IRWXG :: 0x38`
- [OS == .ANDROID && CPU == .X64] `S_IRGRP :: 0x20`
- [OS == .ANDROID && CPU == .X64] `S_IWGRP :: 0x10`
- [OS == .ANDROID && CPU == .X64] `S_IXGRP :: 0x8`
- [OS == .ANDROID && CPU == .X64] `S_IRWXO :: 0x7`
- [OS == .ANDROID && CPU == .X64] `S_IROTH :: 0x4`
- [OS == .ANDROID && CPU == .X64] `S_IWOTH :: 0x2`
- [OS == .ANDROID && CPU == .X64] `S_IXOTH :: 0x1`
- [OS == .ANDROID && CPU == .X64] `STATX_TYPE :: 0x00000001`
- [OS == .ANDROID && CPU == .X64] `STATX_MODE :: 0x00000002`
- [OS == .ANDROID && CPU == .X64] `STATX_NLINK :: 0x00000004`
- [OS == .ANDROID && CPU == .X64] `STATX_UID :: 0x00000008`
- [OS == .ANDROID && CPU == .X64] `STATX_GID :: 0x00000010`
- [OS == .ANDROID && CPU == .X64] `STATX_ATIME :: 0x00000020`
- [OS == .ANDROID && CPU == .X64] `STATX_MTIME :: 0x00000040`
- [OS == .ANDROID && CPU == .X64] `STATX_CTIME :: 0x00000080`
- [OS == .ANDROID && CPU == .X64] `STATX_INO :: 0x00000100`
- [OS == .ANDROID && CPU == .X64] `STATX_SIZE :: 0x00000200`
- [OS == .ANDROID && CPU == .X64] `STATX_BLOCKS :: 0x00000400`
- [OS == .ANDROID && CPU == .X64] `STATX_BASIC_STATS :: 0x000007ff`
- [OS == .ANDROID && CPU == .X64] `STATX_BTIME :: 0x00000800`
- [OS == .ANDROID && CPU == .X64] `STATX_MNT_ID :: 0x00001000`
- [OS == .ANDROID && CPU == .X64] `STATX_DIOALIGN :: 0x00002000`
- [OS == .ANDROID && CPU == .X64] `STATX__RESERVED :: 0x80000000`
- [OS == .ANDROID && CPU == .X64] `STATX_ALL :: 0x00000fff`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_COMPRESSED :: 0x00000004`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_IMMUTABLE :: 0x00000010`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_APPEND :: 0x00000020`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_NODUMP :: 0x00000040`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_ENCRYPTED :: 0x00000800`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_AUTOMOUNT :: 0x00001000`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_MOUNT_ROOT :: 0x00002000`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_VERITY :: 0x00100000`
- [OS == .ANDROID && CPU == .X64] `STATX_ATTR_DAX :: 0x00200000`
- [OS == .ANDROID && CPU == .X64] `F_GETLK64 :: F_GETLK`
- [OS == .ANDROID && CPU == .X64] `F_SETLK64 :: F_SETLK`
- [OS == .ANDROID && CPU == .X64] `F_SETLKW64 :: F_SETLKW`
- [OS == .ANDROID && CPU == .X64] `O_ASYNC :: FASYNC`
- [OS == .ANDROID && CPU == .X64] `O_RSYNC :: O_SYNC`
- [OS == .ANDROID && CPU == .X64] `SPLICE_F_MOVE :: 1`
- [OS == .ANDROID && CPU == .X64] `SPLICE_F_NONBLOCK :: 2`
- [OS == .ANDROID && CPU == .X64] `SPLICE_F_MORE :: 4`
- [OS == .ANDROID && CPU == .X64] `SPLICE_F_GIFT :: 8`
- [OS == .ANDROID && CPU == .X64] `SYNC_FILE_RANGE_WAIT_BEFORE :: 1`
- [OS == .ANDROID && CPU == .X64] `SYNC_FILE_RANGE_WRITE :: 2`
- [OS == .ANDROID && CPU == .X64] `SYNC_FILE_RANGE_WAIT_AFTER :: 4`
- [OS == .ANDROID && CPU == .X64] `ACCESSPERMS :: S_IRWXU | S_IRWXG | S_IRWXO`
- [OS == .ANDROID && CPU == .X64] `ALLPERMS :: S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO`
- [OS == .ANDROID && CPU == .X64] `DEFFILEMODE :: S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH`
- [OS == .ANDROID && CPU == .X64] `S_IREAD :: S_IRUSR`
- [OS == .ANDROID && CPU == .X64] `S_IWRITE :: S_IWUSR`
- [OS == .ANDROID && CPU == .X64] `S_IEXEC :: S_IXUSR`
- [OS == .ANDROID && CPU == .X64] `UTIME_NOW :: (1 << 30) - 1`
- [OS == .ANDROID && CPU == .X64] `UTIME_OMIT :: (1 << 30) - 2`
- [OS == .ANDROID && CPU == .X64] `TCGETS :: 0x5401`
- [OS == .ANDROID && CPU == .X64] `TCSETS :: 0x5402`
- [OS == .ANDROID && CPU == .X64] `TCSETSW :: 0x5403`
- [OS == .ANDROID && CPU == .X64] `TCSETSF :: 0x5404`
- [OS == .ANDROID && CPU == .X64] `TCGETA :: 0x5405`
- [OS == .ANDROID && CPU == .X64] `TCSETA :: 0x5406`
- [OS == .ANDROID && CPU == .X64] `TCSETAW :: 0x5407`
- [OS == .ANDROID && CPU == .X64] `TCSETAF :: 0x5408`
- [OS == .ANDROID && CPU == .X64] `TCSBRK :: 0x5409`
- [OS == .ANDROID && CPU == .X64] `TCXONC :: 0x540A`
- [OS == .ANDROID && CPU == .X64] `TCFLSH :: 0x540B`
- [OS == .ANDROID && CPU == .X64] `TIOCEXCL :: 0x540C`
- [OS == .ANDROID && CPU == .X64] `TIOCNXCL :: 0x540D`
- [OS == .ANDROID && CPU == .X64] `TIOCSCTTY :: 0x540E`
- [OS == .ANDROID && CPU == .X64] `TIOCGPGRP :: 0x540F`
- [OS == .ANDROID && CPU == .X64] `TIOCSPGRP :: 0x5410`
- [OS == .ANDROID && CPU == .X64] `TIOCOUTQ :: 0x5411`
- [OS == .ANDROID && CPU == .X64] `TIOCSTI :: 0x5412`
- [OS == .ANDROID && CPU == .X64] `TIOCGWINSZ :: 0x5413`
- [OS == .ANDROID && CPU == .X64] `TIOCSWINSZ :: 0x5414`
- [OS == .ANDROID && CPU == .X64] `TIOCMGET :: 0x5415`
- [OS == .ANDROID && CPU == .X64] `TIOCMBIS :: 0x5416`
- [OS == .ANDROID && CPU == .X64] `TIOCMBIC :: 0x5417`
- [OS == .ANDROID && CPU == .X64] `TIOCMSET :: 0x5418`
- [OS == .ANDROID && CPU == .X64] `TIOCGSOFTCAR :: 0x5419`
- [OS == .ANDROID && CPU == .X64] `TIOCSSOFTCAR :: 0x541A`
- [OS == .ANDROID && CPU == .X64] `FIONREAD :: 0x541B`
- [OS == .ANDROID && CPU == .X64] `TIOCINQ :: FIONREAD`
- [OS == .ANDROID && CPU == .X64] `TIOCLINUX :: 0x541C`
- [OS == .ANDROID && CPU == .X64] `TIOCCONS :: 0x541D`
- [OS == .ANDROID && CPU == .X64] `TIOCGSERIAL :: 0x541E`
- [OS == .ANDROID && CPU == .X64] `TIOCSSERIAL :: 0x541F`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT :: 0x5420`
- [OS == .ANDROID && CPU == .X64] `FIONBIO :: 0x5421`
- [OS == .ANDROID && CPU == .X64] `TIOCNOTTY :: 0x5422`
- [OS == .ANDROID && CPU == .X64] `TIOCSETD :: 0x5423`
- [OS == .ANDROID && CPU == .X64] `TIOCGETD :: 0x5424`
- [OS == .ANDROID && CPU == .X64] `TCSBRKP :: 0x5425`
- [OS == .ANDROID && CPU == .X64] `TIOCSBRK :: 0x5427`
- [OS == .ANDROID && CPU == .X64] `TIOCCBRK :: 0x5428`
- [OS == .ANDROID && CPU == .X64] `TIOCGSID :: 0x5429`
- [OS == .ANDROID && CPU == .X64] `TIOCGRS485 :: 0x542E`
- [OS == .ANDROID && CPU == .X64] `TIOCSRS485 :: 0x542F`
- [OS == .ANDROID && CPU == .X64] `TCGETX :: 0x5432`
- [OS == .ANDROID && CPU == .X64] `TCSETX :: 0x5433`
- [OS == .ANDROID && CPU == .X64] `TCSETXF :: 0x5434`
- [OS == .ANDROID && CPU == .X64] `TCSETXW :: 0x5435`
- [OS == .ANDROID && CPU == .X64] `TIOCVHANGUP :: 0x5437`
- [OS == .ANDROID && CPU == .X64] `FIONCLEX :: 0x5450`
- [OS == .ANDROID && CPU == .X64] `FIOCLEX :: 0x5451`
- [OS == .ANDROID && CPU == .X64] `FIOASYNC :: 0x5452`
- [OS == .ANDROID && CPU == .X64] `TIOCSERCONFIG :: 0x5453`
- [OS == .ANDROID && CPU == .X64] `TIOCSERGWILD :: 0x5454`
- [OS == .ANDROID && CPU == .X64] `TIOCSERSWILD :: 0x5455`
- [OS == .ANDROID && CPU == .X64] `TIOCGLCKTRMIOS :: 0x5456`
- [OS == .ANDROID && CPU == .X64] `TIOCSLCKTRMIOS :: 0x5457`
- [OS == .ANDROID && CPU == .X64] `TIOCSERGSTRUCT :: 0x5458`
- [OS == .ANDROID && CPU == .X64] `TIOCSERGETLSR :: 0x5459`
- [OS == .ANDROID && CPU == .X64] `TIOCSERGETMULTI :: 0x545A`
- [OS == .ANDROID && CPU == .X64] `TIOCSERSETMULTI :: 0x545B`
- [OS == .ANDROID && CPU == .X64] `TIOCMIWAIT :: 0x545C`
- [OS == .ANDROID && CPU == .X64] `TIOCGICOUNT :: 0x545D`
- [OS == .ANDROID && CPU == .X64] `FIOQSIZE :: 0x5460`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_DATA :: 0`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_FLUSHREAD :: 1`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_FLUSHWRITE :: 2`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_STOP :: 4`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_START :: 8`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_NOSTOP :: 16`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_DOSTOP :: 32`
- [OS == .ANDROID && CPU == .X64] `TIOCPKT_IOCTL :: 64`
- [OS == .ANDROID && CPU == .X64] `TIOCSER_TEMT :: 0x01`
- [OS == .ANDROID && CPU == .X64] `EFD_SEMAPHORE :: 1 << 0`
- [OS == .ANDROID && CPU == .X64] `EFD_CLOEXEC :: O_CLOEXEC`
- [OS == .ANDROID && CPU == .X64] `EFD_NONBLOCK :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .X64] `TFD_CLOEXEC :: O_CLOEXEC`
- [OS == .ANDROID && CPU == .X64] `TFD_NONBLOCK :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .X64] `TFD_TIMER_ABSTIME :: 1 << 0`
- [OS == .ANDROID && CPU == .X64] `TFD_TIMER_CANCEL_ON_SET :: 1 << 1`
- [OS == .ANDROID && CPU == .X64] `SFD_CLOEXEC :: O_CLOEXEC`
- [OS == .ANDROID && CPU == .X64] `SFD_NONBLOCK :: O_NONBLOCK`
- [OS == .ANDROID && CPU == .X64] `fpos_t :: s64`
- [OS == .ANDROID && CPU == .X64] `fpos64_t :: s64`
- [OS == .ANDROID && CPU == .X64] `__sFILE :: struct`
- [OS == .ANDROID && CPU == .X64] `FILE :: __sFILE`
- [OS == .ANDROID && CPU == .X64] `stdin: *FILE #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `stdout: *FILE #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `stderr: *FILE #elsewhere libc`
- [OS == .ANDROID && CPU == .X64] `clearerr :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fclose :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `feof :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ferror :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fflush :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgetc :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgets :: (__buf: *u8, __size: s32, __fp: *FILE) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fprintf :: (__fp: *FILE, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fputc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fputs :: (__s: *u8, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fread :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fscanf :: (__fp: *FILE, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fwrite :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getc :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getchar :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getdelim :: (__line_ptr: **u8, __line_length_ptr: *u64, __delimiter: s32, __fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getline :: (__line_ptr: **u8, __line_length_ptr: *u64, __fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `perror :: (__msg: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `printf :: (__fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `putc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `putchar :: (__ch: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `puts :: (__s: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `_remove :: (__path: *u8) -> s32 #foreign libc "remove"`
- [OS == .ANDROID && CPU == .X64] `rewind :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `scanf :: (__fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setbuf :: (__fp: *FILE, __buf: *u8) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setvbuf :: (__fp: *FILE, __buf: *u8, __mode: s32, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sscanf :: (__s: *u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ungetc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `dprintf :: (__fd: s32, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sprintf :: (__s: *u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tmpnam :: (__s: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tempnam :: (__dir: *u8, __prefix: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `rename :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `renameat :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `renameat2 :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8, __flags: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fseek :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ftell :: (__fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgetpos :: (__fp: *FILE, __pos: *fpos_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fsetpos :: (__fp: *FILE, __pos: *fpos_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fseeko :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ftello :: (__fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `funopen :: (__cookie: *void, __read_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __write_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __seek_fn: #type (unknown0: *void, unknown1: fpos_t, unknown2: s32) -> fpos_t #c_call, __close_fn: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgetpos64 :: (__fp: *FILE, __pos: *fpos64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fsetpos64 :: (__fp: *FILE, __pos: *fpos64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fseeko64 :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ftello64 :: (__fp: *FILE) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `funopen64 :: (__cookie: *void, __read_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __write_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __seek_fn: #type (unknown0: *void, unknown1: fpos64_t, unknown2: s32) -> fpos64_t #c_call, __close_fn: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fopen :: (__path: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fopen64 :: (__path: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `freopen :: (__path: *u8, __mode: *u8, __fp: *FILE) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `freopen64 :: (__path: *u8, __mode: *u8, __fp: *FILE) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tmpfile :: () -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tmpfile64 :: () -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `snprintf :: (__buf: *u8, __size: u64, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ctermid :: (__buf: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fdopen :: (__fd: s32, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fileno :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pclose :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `popen :: (__command: *u8, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `flockfile :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ftrylockfile :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `funlockfile :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getc_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getchar_unlocked :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `putc_unlocked :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `putchar_unlocked :: (__ch: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fmemopen :: (__buf: *void, __size: u64, __mode: *u8) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `open_memstream :: (__ptr: **u8, __size_ptr: *u64) -> *FILE #foreign libc`
- [OS == .ANDROID && CPU == .X64] `asprintf :: (__s_ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgetln :: (__fp: *FILE, __length_ptr: *u64) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fpurge :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setbuffer :: (__fp: *FILE, __buf: *u8, __size: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setlinebuf :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clearerr_unlocked :: (__fp: *FILE) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `feof_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `ferror_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fileno_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fflush_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgetc_unlocked :: (__fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fputc_unlocked :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fread_unlocked :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fwrite_unlocked :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fputs_unlocked :: (__s: *u8, __fp: *FILE) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fgets_unlocked :: (__buf: *u8, __size: s32, __fp: *FILE) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `f_owner_ex :: struct`
- [OS == .ANDROID && CPU == .X64] `statx_timestamp :: struct`
- [OS == .ANDROID && CPU == .X64] `statx_t :: struct`
- [OS == .ANDROID && CPU == .X64] `fcntl :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `creat :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `creat64 :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `openat :: (__dir_fd: s32, __path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `openat64 :: (__dir_fd: s32, __path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `open :: (__path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `open64 :: (__path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `splice :: (__in_fd: s32, __in_offset: *s64, __out_fd: s32, __out_offset: *s64, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `tee :: (__in_fd: s32, __out_fd: s32, __length: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `vmsplice :: (__fd: s32, __iov: *iovec, __count: u64, __flags: u32) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fallocate :: (__fd: s32, __mode: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fallocate64 :: (__fd: s32, __mode: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `posix_fadvise :: (__fd: s32, __offset: s64, __length: s64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `posix_fadvise64 :: (__fd: s32, __offset: s64, __length: s64, __advice: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `posix_fallocate :: (__fd: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `posix_fallocate64 :: (__fd: s32, __offset: s64, __length: s64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `readahead :: (__fd: s32, __offset: s64, __length: u64) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sync_file_range :: (__fd: s32, __offset: s64, __length: s64, __flags: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `stat_t :: struct`
- [OS == .ANDROID && CPU == .X64] `stat64_t :: struct`
- [OS == .ANDROID && CPU == .X64] `chmod :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fchmod :: (__fd: s32, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkdir :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fstat :: (__fd: s32, __buf: *stat_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fstat64 :: (__fd: s32, __buf: *stat64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fstatat :: (__dir_fd: s32, __path: *u8, __buf: *stat_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fstatat64 :: (__dir_fd: s32, __path: *u8, __buf: *stat64_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `lstat :: (__path: *u8, __buf: *stat_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `lstat64 :: (__path: *u8, __buf: *stat64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `stat :: (__path: *u8, __buf: *stat_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `stat64 :: (__path: *u8, __buf: *stat64_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mknod :: (__path: *u8, __mode: mode_t, __dev: dev_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `umask :: (__mask: mode_t) -> mode_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkfifo :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkfifoat :: (__dir_fd: s32, __path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `fchmodat :: (__dir_fd: s32, __path: *u8, __mode: mode_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkdirat :: (__dir_fd: s32, __path: *u8, __mode: mode_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mknodat :: (__dir_fd: s32, __path: *u8, __mode: mode_t, __dev: dev_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `utimensat :: (__dir_fd: s32, __path: *u8, __times: *[2] timespec, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `statx :: (__dir_fd: s32, __path: *u8, __flags: s32, __mask: u32, __buf: *statx_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `eventfd :: (__initial_value: u32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `eventfd_t :: u64`
- [OS == .ANDROID && CPU == .X64] `eventfd_read :: (__fd: s32, __value: *eventfd_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `eventfd_write :: (__fd: s32, __value: eventfd_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timerfd_create :: (__clock: clockid_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timerfd_settime :: (__fd: s32, __flags: s32, __new_value: *itimerspec, __old_value: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `timerfd_gettime :: (__fd: s32, __current_value: *itimerspec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `signalfd_siginfo :: struct`
- [OS == .ANDROID && CPU == .X64] `signalfd :: (__fd: s32, __mask: *sigset_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `signalfd64 :: (__fd: s32, __mask: *sigset64_t, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getenv :: (__name: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setenv :: (__name: *u8, __value: *u8, __overwrite: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `unsetenv :: (__name: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkdtemp :: (__template: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mktemp :: (__template: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkostemp :: (__template: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkostemps :: (__template: *u8, __suffix_length: s32, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkstemp :: (__template: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `mkstemps :: (__template: *u8, __flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `realpath :: (__path: *u8, __resolved: *u8) -> *u8 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `system :: (__command: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `SYS__sysctl :: 156`
- [OS == .ANDROID && CPU == .X64] `SYS_accept :: 43`
- [OS == .ANDROID && CPU == .X64] `SYS_accept4 :: 288`
- [OS == .ANDROID && CPU == .X64] `SYS_access :: 21`
- [OS == .ANDROID && CPU == .X64] `SYS_acct :: 163`
- [OS == .ANDROID && CPU == .X64] `SYS_add_key :: 248`
- [OS == .ANDROID && CPU == .X64] `SYS_adjtimex :: 159`
- [OS == .ANDROID && CPU == .X64] `SYS_afs_syscall :: 183`
- [OS == .ANDROID && CPU == .X64] `SYS_alarm :: 37`
- [OS == .ANDROID && CPU == .X64] `SYS_arch_prctl :: 158`
- [OS == .ANDROID && CPU == .X64] `SYS_bind :: 49`
- [OS == .ANDROID && CPU == .X64] `SYS_bpf :: 321`
- [OS == .ANDROID && CPU == .X64] `SYS_brk :: 12`
- [OS == .ANDROID && CPU == .X64] `SYS_capget :: 125`
- [OS == .ANDROID && CPU == .X64] `SYS_capset :: 126`
- [OS == .ANDROID && CPU == .X64] `SYS_chdir :: 80`
- [OS == .ANDROID && CPU == .X64] `SYS_chmod :: 90`
- [OS == .ANDROID && CPU == .X64] `SYS_chown :: 92`
- [OS == .ANDROID && CPU == .X64] `SYS_chroot :: 161`
- [OS == .ANDROID && CPU == .X64] `SYS_clock_adjtime :: 305`
- [OS == .ANDROID && CPU == .X64] `SYS_clock_getres :: 229`
- [OS == .ANDROID && CPU == .X64] `SYS_clock_gettime :: 228`
- [OS == .ANDROID && CPU == .X64] `SYS_clock_nanosleep :: 230`
- [OS == .ANDROID && CPU == .X64] `SYS_clock_settime :: 227`
- [OS == .ANDROID && CPU == .X64] `SYS_clone :: 56`
- [OS == .ANDROID && CPU == .X64] `SYS_clone3 :: 435`
- [OS == .ANDROID && CPU == .X64] `SYS_close :: 3`
- [OS == .ANDROID && CPU == .X64] `SYS_close_range :: 436`
- [OS == .ANDROID && CPU == .X64] `SYS_connect :: 42`
- [OS == .ANDROID && CPU == .X64] `SYS_copy_file_range :: 326`
- [OS == .ANDROID && CPU == .X64] `SYS_creat :: 85`
- [OS == .ANDROID && CPU == .X64] `SYS_create_module :: 174`
- [OS == .ANDROID && CPU == .X64] `SYS_delete_module :: 176`
- [OS == .ANDROID && CPU == .X64] `SYS_dup :: 32`
- [OS == .ANDROID && CPU == .X64] `SYS_dup2 :: 33`
- [OS == .ANDROID && CPU == .X64] `SYS_dup3 :: 292`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_create :: 213`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_create1 :: 291`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_ctl :: 233`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_ctl_old :: 214`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_pwait :: 281`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_pwait2 :: 441`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_wait :: 232`
- [OS == .ANDROID && CPU == .X64] `SYS_epoll_wait_old :: 215`
- [OS == .ANDROID && CPU == .X64] `SYS_eventfd :: 284`
- [OS == .ANDROID && CPU == .X64] `SYS_eventfd2 :: 290`
- [OS == .ANDROID && CPU == .X64] `SYS_execve :: 59`
- [OS == .ANDROID && CPU == .X64] `SYS_execveat :: 322`
- [OS == .ANDROID && CPU == .X64] `SYS_exit :: 60`
- [OS == .ANDROID && CPU == .X64] `SYS_exit_group :: 231`
- [OS == .ANDROID && CPU == .X64] `SYS_faccessat :: 269`
- [OS == .ANDROID && CPU == .X64] `SYS_faccessat2 :: 439`
- [OS == .ANDROID && CPU == .X64] `SYS_fadvise64 :: 221`
- [OS == .ANDROID && CPU == .X64] `SYS_fallocate :: 285`
- [OS == .ANDROID && CPU == .X64] `SYS_fanotify_init :: 300`
- [OS == .ANDROID && CPU == .X64] `SYS_fanotify_mark :: 301`
- [OS == .ANDROID && CPU == .X64] `SYS_fchdir :: 81`
- [OS == .ANDROID && CPU == .X64] `SYS_fchmod :: 91`
- [OS == .ANDROID && CPU == .X64] `SYS_fchmodat :: 268`
- [OS == .ANDROID && CPU == .X64] `SYS_fchown :: 93`
- [OS == .ANDROID && CPU == .X64] `SYS_fchownat :: 260`
- [OS == .ANDROID && CPU == .X64] `SYS_fcntl :: 72`
- [OS == .ANDROID && CPU == .X64] `SYS_fdatasync :: 75`
- [OS == .ANDROID && CPU == .X64] `SYS_fgetxattr :: 193`
- [OS == .ANDROID && CPU == .X64] `SYS_finit_module :: 313`
- [OS == .ANDROID && CPU == .X64] `SYS_flistxattr :: 196`
- [OS == .ANDROID && CPU == .X64] `SYS_flock :: 73`
- [OS == .ANDROID && CPU == .X64] `SYS_fork :: 57`
- [OS == .ANDROID && CPU == .X64] `SYS_fremovexattr :: 199`
- [OS == .ANDROID && CPU == .X64] `SYS_fsconfig :: 431`
- [OS == .ANDROID && CPU == .X64] `SYS_fsetxattr :: 190`
- [OS == .ANDROID && CPU == .X64] `SYS_fsmount :: 432`
- [OS == .ANDROID && CPU == .X64] `SYS_fsopen :: 430`
- [OS == .ANDROID && CPU == .X64] `SYS_fspick :: 433`
- [OS == .ANDROID && CPU == .X64] `SYS_fstat :: 5`
- [OS == .ANDROID && CPU == .X64] `SYS_fstatfs :: 138`
- [OS == .ANDROID && CPU == .X64] `SYS_fsync :: 74`
- [OS == .ANDROID && CPU == .X64] `SYS_ftruncate :: 77`
- [OS == .ANDROID && CPU == .X64] `SYS_futex :: 202`
- [OS == .ANDROID && CPU == .X64] `SYS_futex_waitv :: 449`
- [OS == .ANDROID && CPU == .X64] `SYS_futimesat :: 261`
- [OS == .ANDROID && CPU == .X64] `SYS_get_kernel_syms :: 177`
- [OS == .ANDROID && CPU == .X64] `SYS_get_mempolicy :: 239`
- [OS == .ANDROID && CPU == .X64] `SYS_get_robust_list :: 274`
- [OS == .ANDROID && CPU == .X64] `SYS_get_thread_area :: 211`
- [OS == .ANDROID && CPU == .X64] `SYS_getcpu :: 309`
- [OS == .ANDROID && CPU == .X64] `SYS_getcwd :: 79`
- [OS == .ANDROID && CPU == .X64] `SYS_getdents :: 78`
- [OS == .ANDROID && CPU == .X64] `SYS_getdents64 :: 217`
- [OS == .ANDROID && CPU == .X64] `SYS_getegid :: 108`
- [OS == .ANDROID && CPU == .X64] `SYS_geteuid :: 107`
- [OS == .ANDROID && CPU == .X64] `SYS_getgid :: 104`
- [OS == .ANDROID && CPU == .X64] `SYS_getgroups :: 115`
- [OS == .ANDROID && CPU == .X64] `SYS_getitimer :: 36`
- [OS == .ANDROID && CPU == .X64] `SYS_getpeername :: 52`
- [OS == .ANDROID && CPU == .X64] `SYS_getpgid :: 121`
- [OS == .ANDROID && CPU == .X64] `SYS_getpgrp :: 111`
- [OS == .ANDROID && CPU == .X64] `SYS_getpid :: 39`
- [OS == .ANDROID && CPU == .X64] `SYS_getpmsg :: 181`
- [OS == .ANDROID && CPU == .X64] `SYS_getppid :: 110`
- [OS == .ANDROID && CPU == .X64] `SYS_getpriority :: 140`
- [OS == .ANDROID && CPU == .X64] `SYS_getrandom :: 318`
- [OS == .ANDROID && CPU == .X64] `SYS_getresgid :: 120`
- [OS == .ANDROID && CPU == .X64] `SYS_getresuid :: 118`
- [OS == .ANDROID && CPU == .X64] `SYS_getrlimit :: 97`
- [OS == .ANDROID && CPU == .X64] `SYS_getrusage :: 98`
- [OS == .ANDROID && CPU == .X64] `SYS_getsid :: 124`
- [OS == .ANDROID && CPU == .X64] `SYS_getsockname :: 51`
- [OS == .ANDROID && CPU == .X64] `SYS_getsockopt :: 55`
- [OS == .ANDROID && CPU == .X64] `SYS_gettid :: 186`
- [OS == .ANDROID && CPU == .X64] `SYS_gettimeofday :: 96`
- [OS == .ANDROID && CPU == .X64] `SYS_getuid :: 102`
- [OS == .ANDROID && CPU == .X64] `SYS_getxattr :: 191`
- [OS == .ANDROID && CPU == .X64] `SYS_init_module :: 175`
- [OS == .ANDROID && CPU == .X64] `SYS_inotify_add_watch :: 254`
- [OS == .ANDROID && CPU == .X64] `SYS_inotify_init :: 253`
- [OS == .ANDROID && CPU == .X64] `SYS_inotify_init1 :: 294`
- [OS == .ANDROID && CPU == .X64] `SYS_inotify_rm_watch :: 255`
- [OS == .ANDROID && CPU == .X64] `SYS_io_cancel :: 210`
- [OS == .ANDROID && CPU == .X64] `SYS_io_destroy :: 207`
- [OS == .ANDROID && CPU == .X64] `SYS_io_getevents :: 208`
- [OS == .ANDROID && CPU == .X64] `SYS_io_pgetevents :: 333`
- [OS == .ANDROID && CPU == .X64] `SYS_io_setup :: 206`
- [OS == .ANDROID && CPU == .X64] `SYS_io_submit :: 209`
- [OS == .ANDROID && CPU == .X64] `SYS_io_uring_enter :: 426`
- [OS == .ANDROID && CPU == .X64] `SYS_io_uring_register :: 427`
- [OS == .ANDROID && CPU == .X64] `SYS_io_uring_setup :: 425`
- [OS == .ANDROID && CPU == .X64] `SYS_ioctl :: 16`
- [OS == .ANDROID && CPU == .X64] `SYS_ioperm :: 173`
- [OS == .ANDROID && CPU == .X64] `SYS_iopl :: 172`
- [OS == .ANDROID && CPU == .X64] `SYS_ioprio_get :: 252`
- [OS == .ANDROID && CPU == .X64] `SYS_ioprio_set :: 251`
- [OS == .ANDROID && CPU == .X64] `SYS_kcmp :: 312`
- [OS == .ANDROID && CPU == .X64] `SYS_kexec_file_load :: 320`
- [OS == .ANDROID && CPU == .X64] `SYS_kexec_load :: 246`
- [OS == .ANDROID && CPU == .X64] `SYS_keyctl :: 250`
- [OS == .ANDROID && CPU == .X64] `SYS_kill :: 62`
- [OS == .ANDROID && CPU == .X64] `SYS_landlock_add_rule :: 445`
- [OS == .ANDROID && CPU == .X64] `SYS_landlock_create_ruleset :: 444`
- [OS == .ANDROID && CPU == .X64] `SYS_landlock_restrict_self :: 446`
- [OS == .ANDROID && CPU == .X64] `SYS_lchown :: 94`
- [OS == .ANDROID && CPU == .X64] `SYS_lgetxattr :: 192`
- [OS == .ANDROID && CPU == .X64] `SYS_link :: 86`
- [OS == .ANDROID && CPU == .X64] `SYS_linkat :: 265`
- [OS == .ANDROID && CPU == .X64] `SYS_listen :: 50`
- [OS == .ANDROID && CPU == .X64] `SYS_listxattr :: 194`
- [OS == .ANDROID && CPU == .X64] `SYS_llistxattr :: 195`
- [OS == .ANDROID && CPU == .X64] `SYS_lookup_dcookie :: 212`
- [OS == .ANDROID && CPU == .X64] `SYS_lremovexattr :: 198`
- [OS == .ANDROID && CPU == .X64] `SYS_lseek :: 8`
- [OS == .ANDROID && CPU == .X64] `SYS_lsetxattr :: 189`
- [OS == .ANDROID && CPU == .X64] `SYS_lstat :: 6`
- [OS == .ANDROID && CPU == .X64] `SYS_madvise :: 28`
- [OS == .ANDROID && CPU == .X64] `SYS_mbind :: 237`
- [OS == .ANDROID && CPU == .X64] `SYS_membarrier :: 324`
- [OS == .ANDROID && CPU == .X64] `SYS_memfd_create :: 319`
- [OS == .ANDROID && CPU == .X64] `SYS_memfd_secret :: 447`
- [OS == .ANDROID && CPU == .X64] `SYS_migrate_pages :: 256`
- [OS == .ANDROID && CPU == .X64] `SYS_mincore :: 27`
- [OS == .ANDROID && CPU == .X64] `SYS_mkdir :: 83`
- [OS == .ANDROID && CPU == .X64] `SYS_mkdirat :: 258`
- [OS == .ANDROID && CPU == .X64] `SYS_mknod :: 133`
- [OS == .ANDROID && CPU == .X64] `SYS_mknodat :: 259`
- [OS == .ANDROID && CPU == .X64] `SYS_mlock :: 149`
- [OS == .ANDROID && CPU == .X64] `SYS_mlock2 :: 325`
- [OS == .ANDROID && CPU == .X64] `SYS_mlockall :: 151`
- [OS == .ANDROID && CPU == .X64] `SYS_mmap :: 9`
- [OS == .ANDROID && CPU == .X64] `SYS_modify_ldt :: 154`
- [OS == .ANDROID && CPU == .X64] `SYS_mount :: 165`
- [OS == .ANDROID && CPU == .X64] `SYS_mount_setattr :: 442`
- [OS == .ANDROID && CPU == .X64] `SYS_move_mount :: 429`
- [OS == .ANDROID && CPU == .X64] `SYS_move_pages :: 279`
- [OS == .ANDROID && CPU == .X64] `SYS_mprotect :: 10`
- [OS == .ANDROID && CPU == .X64] `SYS_mq_getsetattr :: 245`
- [OS == .ANDROID && CPU == .X64] `SYS_mq_notify :: 244`
- [OS == .ANDROID && CPU == .X64] `SYS_mq_open :: 240`
- [OS == .ANDROID && CPU == .X64] `SYS_mq_timedreceive :: 243`
- [OS == .ANDROID && CPU == .X64] `SYS_mq_timedsend :: 242`
- [OS == .ANDROID && CPU == .X64] `SYS_mq_unlink :: 241`
- [OS == .ANDROID && CPU == .X64] `SYS_mremap :: 25`
- [OS == .ANDROID && CPU == .X64] `SYS_msgctl :: 71`
- [OS == .ANDROID && CPU == .X64] `SYS_msgget :: 68`
- [OS == .ANDROID && CPU == .X64] `SYS_msgrcv :: 70`
- [OS == .ANDROID && CPU == .X64] `SYS_msgsnd :: 69`
- [OS == .ANDROID && CPU == .X64] `SYS_msync :: 26`
- [OS == .ANDROID && CPU == .X64] `SYS_munlock :: 150`
- [OS == .ANDROID && CPU == .X64] `SYS_munlockall :: 152`
- [OS == .ANDROID && CPU == .X64] `SYS_munmap :: 11`
- [OS == .ANDROID && CPU == .X64] `SYS_name_to_handle_at :: 303`
- [OS == .ANDROID && CPU == .X64] `SYS_nanosleep :: 35`
- [OS == .ANDROID && CPU == .X64] `SYS_newfstatat :: 262`
- [OS == .ANDROID && CPU == .X64] `SYS_nfsservctl :: 180`
- [OS == .ANDROID && CPU == .X64] `SYS_open :: 2`
- [OS == .ANDROID && CPU == .X64] `SYS_open_by_handle_at :: 304`
- [OS == .ANDROID && CPU == .X64] `SYS_open_tree :: 428`
- [OS == .ANDROID && CPU == .X64] `SYS_openat :: 257`
- [OS == .ANDROID && CPU == .X64] `SYS_openat2 :: 437`
- [OS == .ANDROID && CPU == .X64] `SYS_pause :: 34`
- [OS == .ANDROID && CPU == .X64] `SYS_perf_event_open :: 298`
- [OS == .ANDROID && CPU == .X64] `SYS_personality :: 135`
- [OS == .ANDROID && CPU == .X64] `SYS_pidfd_getfd :: 438`
- [OS == .ANDROID && CPU == .X64] `SYS_pidfd_open :: 434`
- [OS == .ANDROID && CPU == .X64] `SYS_pidfd_send_signal :: 424`
- [OS == .ANDROID && CPU == .X64] `SYS_pipe :: 22`
- [OS == .ANDROID && CPU == .X64] `SYS_pipe2 :: 293`
- [OS == .ANDROID && CPU == .X64] `SYS_pivot_root :: 155`
- [OS == .ANDROID && CPU == .X64] `SYS_pkey_alloc :: 330`
- [OS == .ANDROID && CPU == .X64] `SYS_pkey_free :: 331`
- [OS == .ANDROID && CPU == .X64] `SYS_pkey_mprotect :: 329`
- [OS == .ANDROID && CPU == .X64] `SYS_poll :: 7`
- [OS == .ANDROID && CPU == .X64] `SYS_ppoll :: 271`
- [OS == .ANDROID && CPU == .X64] `SYS_prctl :: 157`
- [OS == .ANDROID && CPU == .X64] `SYS_pread64 :: 17`
- [OS == .ANDROID && CPU == .X64] `SYS_preadv :: 295`
- [OS == .ANDROID && CPU == .X64] `SYS_preadv2 :: 327`
- [OS == .ANDROID && CPU == .X64] `SYS_prlimit64 :: 302`
- [OS == .ANDROID && CPU == .X64] `SYS_process_madvise :: 440`
- [OS == .ANDROID && CPU == .X64] `SYS_process_mrelease :: 448`
- [OS == .ANDROID && CPU == .X64] `SYS_process_vm_readv :: 310`
- [OS == .ANDROID && CPU == .X64] `SYS_process_vm_writev :: 311`
- [OS == .ANDROID && CPU == .X64] `SYS_pselect6 :: 270`
- [OS == .ANDROID && CPU == .X64] `SYS_ptrace :: 101`
- [OS == .ANDROID && CPU == .X64] `SYS_putpmsg :: 182`
- [OS == .ANDROID && CPU == .X64] `SYS_pwrite64 :: 18`
- [OS == .ANDROID && CPU == .X64] `SYS_pwritev :: 296`
- [OS == .ANDROID && CPU == .X64] `SYS_pwritev2 :: 328`
- [OS == .ANDROID && CPU == .X64] `SYS_query_module :: 178`
- [OS == .ANDROID && CPU == .X64] `SYS_quotactl :: 179`
- [OS == .ANDROID && CPU == .X64] `SYS_quotactl_fd :: 443`
- [OS == .ANDROID && CPU == .X64] `SYS_read :: 0`
- [OS == .ANDROID && CPU == .X64] `SYS_readahead :: 187`
- [OS == .ANDROID && CPU == .X64] `SYS_readlink :: 89`
- [OS == .ANDROID && CPU == .X64] `SYS_readlinkat :: 267`
- [OS == .ANDROID && CPU == .X64] `SYS_readv :: 19`
- [OS == .ANDROID && CPU == .X64] `SYS_reboot :: 169`
- [OS == .ANDROID && CPU == .X64] `SYS_recvfrom :: 45`
- [OS == .ANDROID && CPU == .X64] `SYS_recvmmsg :: 299`
- [OS == .ANDROID && CPU == .X64] `SYS_recvmsg :: 47`
- [OS == .ANDROID && CPU == .X64] `SYS_remap_file_pages :: 216`
- [OS == .ANDROID && CPU == .X64] `SYS_removexattr :: 197`
- [OS == .ANDROID && CPU == .X64] `SYS_rename :: 82`
- [OS == .ANDROID && CPU == .X64] `SYS_renameat :: 264`
- [OS == .ANDROID && CPU == .X64] `SYS_renameat2 :: 316`
- [OS == .ANDROID && CPU == .X64] `SYS_request_key :: 249`
- [OS == .ANDROID && CPU == .X64] `SYS_restart_syscall :: 219`
- [OS == .ANDROID && CPU == .X64] `SYS_rmdir :: 84`
- [OS == .ANDROID && CPU == .X64] `SYS_rseq :: 334`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigaction :: 13`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigpending :: 127`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigprocmask :: 14`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigqueueinfo :: 129`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigreturn :: 15`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigsuspend :: 130`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_sigtimedwait :: 128`
- [OS == .ANDROID && CPU == .X64] `SYS_rt_tgsigqueueinfo :: 297`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_get_priority_max :: 146`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_get_priority_min :: 147`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_getaffinity :: 204`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_getattr :: 315`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_getparam :: 143`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_getscheduler :: 145`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_rr_get_interval :: 148`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_setaffinity :: 203`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_setattr :: 314`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_setparam :: 142`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_setscheduler :: 144`
- [OS == .ANDROID && CPU == .X64] `SYS_sched_yield :: 24`
- [OS == .ANDROID && CPU == .X64] `SYS_seccomp :: 317`
- [OS == .ANDROID && CPU == .X64] `SYS_security :: 185`
- [OS == .ANDROID && CPU == .X64] `SYS_select :: 23`
- [OS == .ANDROID && CPU == .X64] `SYS_semctl :: 66`
- [OS == .ANDROID && CPU == .X64] `SYS_semget :: 64`
- [OS == .ANDROID && CPU == .X64] `SYS_semop :: 65`
- [OS == .ANDROID && CPU == .X64] `SYS_semtimedop :: 220`
- [OS == .ANDROID && CPU == .X64] `SYS_sendfile :: 40`
- [OS == .ANDROID && CPU == .X64] `SYS_sendmmsg :: 307`
- [OS == .ANDROID && CPU == .X64] `SYS_sendmsg :: 46`
- [OS == .ANDROID && CPU == .X64] `SYS_sendto :: 44`
- [OS == .ANDROID && CPU == .X64] `SYS_set_mempolicy :: 238`
- [OS == .ANDROID && CPU == .X64] `SYS_set_mempolicy_home_node :: 450`
- [OS == .ANDROID && CPU == .X64] `SYS_set_robust_list :: 273`
- [OS == .ANDROID && CPU == .X64] `SYS_set_thread_area :: 205`
- [OS == .ANDROID && CPU == .X64] `SYS_set_tid_address :: 218`
- [OS == .ANDROID && CPU == .X64] `SYS_setdomainname :: 171`
- [OS == .ANDROID && CPU == .X64] `SYS_setfsgid :: 123`
- [OS == .ANDROID && CPU == .X64] `SYS_setfsuid :: 122`
- [OS == .ANDROID && CPU == .X64] `SYS_setgid :: 106`
- [OS == .ANDROID && CPU == .X64] `SYS_setgroups :: 116`
- [OS == .ANDROID && CPU == .X64] `SYS_sethostname :: 170`
- [OS == .ANDROID && CPU == .X64] `SYS_setitimer :: 38`
- [OS == .ANDROID && CPU == .X64] `SYS_setns :: 308`
- [OS == .ANDROID && CPU == .X64] `SYS_setpgid :: 109`
- [OS == .ANDROID && CPU == .X64] `SYS_setpriority :: 141`
- [OS == .ANDROID && CPU == .X64] `SYS_setregid :: 114`
- [OS == .ANDROID && CPU == .X64] `SYS_setresgid :: 119`
- [OS == .ANDROID && CPU == .X64] `SYS_setresuid :: 117`
- [OS == .ANDROID && CPU == .X64] `SYS_setreuid :: 113`
- [OS == .ANDROID && CPU == .X64] `SYS_setrlimit :: 160`
- [OS == .ANDROID && CPU == .X64] `SYS_setsid :: 112`
- [OS == .ANDROID && CPU == .X64] `SYS_setsockopt :: 54`
- [OS == .ANDROID && CPU == .X64] `SYS_settimeofday :: 164`
- [OS == .ANDROID && CPU == .X64] `SYS_setuid :: 105`
- [OS == .ANDROID && CPU == .X64] `SYS_setxattr :: 188`
- [OS == .ANDROID && CPU == .X64] `SYS_shmat :: 30`
- [OS == .ANDROID && CPU == .X64] `SYS_shmctl :: 31`
- [OS == .ANDROID && CPU == .X64] `SYS_shmdt :: 67`
- [OS == .ANDROID && CPU == .X64] `SYS_shmget :: 29`
- [OS == .ANDROID && CPU == .X64] `SYS_shutdown :: 48`
- [OS == .ANDROID && CPU == .X64] `SYS_sigaltstack :: 131`
- [OS == .ANDROID && CPU == .X64] `SYS_signalfd :: 282`
- [OS == .ANDROID && CPU == .X64] `SYS_signalfd4 :: 289`
- [OS == .ANDROID && CPU == .X64] `SYS_socket :: 41`
- [OS == .ANDROID && CPU == .X64] `SYS_socketpair :: 53`
- [OS == .ANDROID && CPU == .X64] `SYS_splice :: 275`
- [OS == .ANDROID && CPU == .X64] `SYS_stat :: 4`
- [OS == .ANDROID && CPU == .X64] `SYS_statfs :: 137`
- [OS == .ANDROID && CPU == .X64] `SYS_statx :: 332`
- [OS == .ANDROID && CPU == .X64] `SYS_swapoff :: 168`
- [OS == .ANDROID && CPU == .X64] `SYS_swapon :: 167`
- [OS == .ANDROID && CPU == .X64] `SYS_symlink :: 88`
- [OS == .ANDROID && CPU == .X64] `SYS_symlinkat :: 266`
- [OS == .ANDROID && CPU == .X64] `SYS_sync :: 162`
- [OS == .ANDROID && CPU == .X64] `SYS_sync_file_range :: 277`
- [OS == .ANDROID && CPU == .X64] `SYS_syncfs :: 306`
- [OS == .ANDROID && CPU == .X64] `SYS_sysfs :: 139`
- [OS == .ANDROID && CPU == .X64] `SYS_sysinfo :: 99`
- [OS == .ANDROID && CPU == .X64] `SYS_syslog :: 103`
- [OS == .ANDROID && CPU == .X64] `SYS_tee :: 276`
- [OS == .ANDROID && CPU == .X64] `SYS_tgkill :: 234`
- [OS == .ANDROID && CPU == .X64] `SYS_time :: 201`
- [OS == .ANDROID && CPU == .X64] `SYS_timer_create :: 222`
- [OS == .ANDROID && CPU == .X64] `SYS_timer_delete :: 226`
- [OS == .ANDROID && CPU == .X64] `SYS_timer_getoverrun :: 225`
- [OS == .ANDROID && CPU == .X64] `SYS_timer_gettime :: 224`
- [OS == .ANDROID && CPU == .X64] `SYS_timer_settime :: 223`
- [OS == .ANDROID && CPU == .X64] `SYS_timerfd_create :: 283`
- [OS == .ANDROID && CPU == .X64] `SYS_timerfd_gettime :: 287`
- [OS == .ANDROID && CPU == .X64] `SYS_timerfd_settime :: 286`
- [OS == .ANDROID && CPU == .X64] `SYS_times :: 100`
- [OS == .ANDROID && CPU == .X64] `SYS_tkill :: 200`
- [OS == .ANDROID && CPU == .X64] `SYS_truncate :: 76`
- [OS == .ANDROID && CPU == .X64] `SYS_tuxcall :: 184`
- [OS == .ANDROID && CPU == .X64] `SYS_umask :: 95`
- [OS == .ANDROID && CPU == .X64] `SYS_umount2 :: 166`
- [OS == .ANDROID && CPU == .X64] `SYS_uname :: 63`
- [OS == .ANDROID && CPU == .X64] `SYS_unlink :: 87`
- [OS == .ANDROID && CPU == .X64] `SYS_unlinkat :: 263`
- [OS == .ANDROID && CPU == .X64] `SYS_unshare :: 272`
- [OS == .ANDROID && CPU == .X64] `SYS_uselib :: 134`
- [OS == .ANDROID && CPU == .X64] `SYS_userfaultfd :: 323`
- [OS == .ANDROID && CPU == .X64] `SYS_ustat :: 136`
- [OS == .ANDROID && CPU == .X64] `SYS_utime :: 132`
- [OS == .ANDROID && CPU == .X64] `SYS_utimensat :: 280`
- [OS == .ANDROID && CPU == .X64] `SYS_utimes :: 235`
- [OS == .ANDROID && CPU == .X64] `SYS_vfork :: 58`
- [OS == .ANDROID && CPU == .X64] `SYS_vhangup :: 153`
- [OS == .ANDROID && CPU == .X64] `SYS_vmsplice :: 278`
- [OS == .ANDROID && CPU == .X64] `SYS_vserver :: 236`
- [OS == .ANDROID && CPU == .X64] `SYS_wait4 :: 61`
- [OS == .ANDROID && CPU == .X64] `SYS_waitid :: 247`
- [OS == .ANDROID && CPU == .X64] `SYS_write :: 1`
- [OS == .ANDROID && CPU == .X64] `SYS_writev :: 20`
- [OS == .ANDROID && CPU == .X64] `syscall :: (__number: s64, __args: ..Any) -> s64 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `CSIGNAL :: 0x000000ff`
- [OS == .ANDROID && CPU == .X64] `CLONE_VM :: 0x00000100`
- [OS == .ANDROID && CPU == .X64] `CLONE_FS :: 0x00000200`
- [OS == .ANDROID && CPU == .X64] `CLONE_FILES :: 0x00000400`
- [OS == .ANDROID && CPU == .X64] `CLONE_SIGHAND :: 0x00000800`
- [OS == .ANDROID && CPU == .X64] `CLONE_PIDFD :: 0x00001000`
- [OS == .ANDROID && CPU == .X64] `CLONE_PTRACE :: 0x00002000`
- [OS == .ANDROID && CPU == .X64] `CLONE_VFORK :: 0x00004000`
- [OS == .ANDROID && CPU == .X64] `CLONE_PARENT :: 0x00008000`
- [OS == .ANDROID && CPU == .X64] `CLONE_THREAD :: 0x00010000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWNS :: 0x00020000`
- [OS == .ANDROID && CPU == .X64] `CLONE_SYSVSEM :: 0x00040000`
- [OS == .ANDROID && CPU == .X64] `CLONE_SETTLS :: 0x00080000`
- [OS == .ANDROID && CPU == .X64] `CLONE_PARENT_SETTID :: 0x00100000`
- [OS == .ANDROID && CPU == .X64] `CLONE_CHILD_CLEARTID :: 0x00200000`
- [OS == .ANDROID && CPU == .X64] `CLONE_DETACHED :: 0x00400000`
- [OS == .ANDROID && CPU == .X64] `CLONE_UNTRACED :: 0x00800000`
- [OS == .ANDROID && CPU == .X64] `CLONE_CHILD_SETTID :: 0x01000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWCGROUP :: 0x02000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWUTS :: 0x04000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWIPC :: 0x08000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWUSER :: 0x10000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWPID :: 0x20000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWNET :: 0x40000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_IO :: 0x80000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_CLEAR_SIGHAND :: 0x100000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_INTO_CGROUP :: 0x200000000`
- [OS == .ANDROID && CPU == .X64] `CLONE_NEWTIME :: 0x00000080`
- [OS == .ANDROID && CPU == .X64] `CLONE_ARGS_SIZE_VER0 :: 64`
- [OS == .ANDROID && CPU == .X64] `CLONE_ARGS_SIZE_VER1 :: 80`
- [OS == .ANDROID && CPU == .X64] `CLONE_ARGS_SIZE_VER2 :: 88`
- [OS == .ANDROID && CPU == .X64] `SCHED_NORMAL :: 0`
- [OS == .ANDROID && CPU == .X64] `SCHED_FIFO :: 1`
- [OS == .ANDROID && CPU == .X64] `SCHED_RR :: 2`
- [OS == .ANDROID && CPU == .X64] `SCHED_BATCH :: 3`
- [OS == .ANDROID && CPU == .X64] `SCHED_IDLE :: 5`
- [OS == .ANDROID && CPU == .X64] `SCHED_DEADLINE :: 6`
- [OS == .ANDROID && CPU == .X64] `SCHED_RESET_ON_FORK :: 0x40000000`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_RESET_ON_FORK :: 0x01`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_RECLAIM :: 0x02`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_DL_OVERRUN :: 0x04`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_KEEP_POLICY :: 0x08`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_KEEP_PARAMS :: 0x10`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_UTIL_CLAMP_MIN :: 0x20`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_UTIL_CLAMP_MAX :: 0x40`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_KEEP_ALL :: SCHED_FLAG_KEEP_POLICY | SCHED_FLAG_KEEP_PARAMS`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_UTIL_CLAMP :: SCHED_FLAG_UTIL_CLAMP_MIN | SCHED_FLAG_UTIL_CLAMP_MAX`
- [OS == .ANDROID && CPU == .X64] `SCHED_FLAG_ALL :: SCHED_FLAG_RESET_ON_FORK | SCHED_FLAG_RECLAIM | SCHED_FLAG_DL_OVERRUN | SCHED_FLAG_KEEP_ALL | SCHED_FLAG_UTIL_CLAMP`
- [OS == .ANDROID && CPU == .X64] `SCHED_OTHER :: SCHED_NORMAL`
- [OS == .ANDROID && CPU == .X64] `CPU_SETSIZE :: 1024`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_ONCE_INIT :: 0`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_BARRIER_SERIAL_THREAD :: -1`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_CREATE_DETACHED :: 1`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_CREATE_JOINABLE :: 0`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_EXPLICIT_SCHED :: 0`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_INHERIT_SCHED :: 1`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_PRIO_NONE :: 0`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_PRIO_INHERIT :: 1`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_PROCESS_PRIVATE :: 0`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_PROCESS_SHARED :: 1`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_SCOPE_SYSTEM :: 0`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_SCOPE_PROCESS :: 1`
- [OS == .ANDROID && CPU == .X64] `_JBLEN :: 11`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_t :: struct`
- [OS == .ANDROID && CPU == .X64] `pthread_barrier_t :: struct`
- [OS == .ANDROID && CPU == .X64] `pthread_barrierattr_t :: s32`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_t :: struct`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_t :: s64`
- [OS == .ANDROID && CPU == .X64] `pthread_key_t :: s32`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_t :: struct`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_t :: s64`
- [OS == .ANDROID && CPU == .X64] `pthread_once_t :: s32`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_t :: struct`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_t :: s64`
- [OS == .ANDROID && CPU == .X64] `pthread_spinlock_t :: struct`
- [OS == .ANDROID && CPU == .X64] `pthread_t :: s64`
- [OS == .ANDROID && CPU == .X64] `clone_args :: struct`
- [OS == .ANDROID && CPU == .X64] `sched_param :: struct`
- [OS == .ANDROID && CPU == .X64] `sched_setscheduler :: (__pid: pid_t, __policy: s32, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_getscheduler :: (__pid: pid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_yield :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_get_priority_max :: (__policy: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_get_priority_min :: (__policy: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_setparam :: (__pid: pid_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_getparam :: (__pid: pid_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_rr_get_interval :: (__pid: pid_t, __quantum: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `clone :: (__fn: #type (unknown0: *void) -> s32 #c_call, __child_stack: *void, __flags: s32, __arg: *void, __args: ..Any) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `unshare :: (__flags: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setns :: (__fd: s32, __ns_type: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_getcpu :: () -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `cpu_set_t :: struct`
- [OS == .ANDROID && CPU == .X64] `sched_setaffinity :: (__pid: pid_t, __set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sched_getaffinity :: (__pid: pid_t, __set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__sched_cpucount :: (__set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__sched_cpualloc :: (__count: u64) -> *cpu_set_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__sched_cpufree :: (__set: *cpu_set_t) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_MUTEX :: enum u32`
- [OS == .ANDROID && CPU == .X64] `PTHREAD_RWLOCK_PREFER :: enum u32`
- [OS == .ANDROID && CPU == .X64] `pthread_atfork :: (__prepare: #type () -> void #c_call, __parent: #type () -> void #c_call, __child: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_destroy :: (__attr: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getdetachstate :: (__attr: *pthread_attr_t, __state: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getguardsize :: (__attr: *pthread_attr_t, __size: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getinheritsched :: (__attr: *pthread_attr_t, __flag: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getschedpolicy :: (__attr: *pthread_attr_t, __policy: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getscope :: (__attr: *pthread_attr_t, __scope: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getstack :: (__attr: *pthread_attr_t, __addr: **void, __size: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_getstacksize :: (__attr: *pthread_attr_t, __size: *u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_init :: (__attr: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setdetachstate :: (__attr: *pthread_attr_t, __state: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setguardsize :: (__attr: *pthread_attr_t, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setinheritsched :: (__attr: *pthread_attr_t, __flag: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setschedpolicy :: (__attr: *pthread_attr_t, __policy: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setscope :: (__attr: *pthread_attr_t, __scope: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setstack :: (__attr: *pthread_attr_t, __addr: *void, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_attr_setstacksize :: (__addr: *pthread_attr_t, __size: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_destroy :: (__attr: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_getclock :: (__attr: *pthread_condattr_t, __clock: *clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_getpshared :: (__attr: *pthread_condattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_init :: (__attr: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_setclock :: (__attr: *pthread_condattr_t, __clock: clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_condattr_setpshared :: (__attr: *pthread_condattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_broadcast :: (__cond: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_clockwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_destroy :: (__cond: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_init :: (__cond: *pthread_cond_t, __attr: *pthread_condattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_signal :: (__cond: *pthread_cond_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_timedwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_timedwait_monotonic_np :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_cond_wait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_create :: (__pthread_ptr: *pthread_t, __attr: *pthread_attr_t, __start_routine: #type (unknown0: *void) -> *void #c_call, unknown0: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_detach :: (__pthread: pthread_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_exit :: (__return_value: *void) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_equal :: (__lhs: pthread_t, __rhs: pthread_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_getattr_np :: (__pthread: pthread_t, __attr: *pthread_attr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_getcpuclockid :: (__pthread: pthread_t, __clock: *clockid_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_getspecific :: (__key: pthread_key_t) -> *void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_gettid_np :: (__pthread: pthread_t) -> pid_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_join :: (__pthread: pthread_t, __return_value_ptr: **void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_key_create :: (__key_ptr: *pthread_key_t, __key_destructor: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_key_delete :: (__key: pthread_key_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_destroy :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_getpshared :: (__attr: *pthread_mutexattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_gettype :: (__attr: *pthread_mutexattr_t, __type: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_getprotocol :: (__attr: *pthread_mutexattr_t, __protocol: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_init :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_setpshared :: (__attr: *pthread_mutexattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_settype :: (__attr: *pthread_mutexattr_t, __type: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutexattr_setprotocol :: (__attr: *pthread_mutexattr_t, __protocol: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_clocklock :: (__mutex: *pthread_mutex_t, __clock: clockid_t, __abstime: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_destroy :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_init :: (__mutex: *pthread_mutex_t, __attr: *pthread_mutexattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_lock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_timedlock :: (__mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_timedlock_monotonic_np :: (__mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_trylock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_mutex_unlock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_once :: (__once: *pthread_once_t, __init_routine: #type () -> void #c_call) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_init :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_destroy :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_getpshared :: (__attr: *pthread_rwlockattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_setpshared :: (__attr: *pthread_rwlockattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_getkind_np :: (__attr: *pthread_rwlockattr_t, __kind: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlockattr_setkind_np :: (__attr: *pthread_rwlockattr_t, __kind: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_clockrdlock :: (__rwlock: *pthread_rwlock_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_clockwrlock :: (__rwlock: *pthread_rwlock_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_destroy :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_init :: (__rwlock: *pthread_rwlock_t, __attr: *pthread_rwlockattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_rdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_timedrdlock :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_timedrdlock_monotonic_np :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_timedwrlock :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_timedwrlock_monotonic_np :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_tryrdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_trywrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_unlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_rwlock_wrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrierattr_init :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrierattr_destroy :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrierattr_getpshared :: (__attr: *pthread_barrierattr_t, __shared: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrierattr_setpshared :: (__attr: *pthread_barrierattr_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrier_init :: (__barrier: *pthread_barrier_t, __attr: *pthread_barrierattr_t, __count: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrier_destroy :: (__barrier: *pthread_barrier_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_barrier_wait :: (__barrier: *pthread_barrier_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_spin_destroy :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_spin_init :: (__spinlock: *pthread_spinlock_t, __shared: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_spin_lock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_spin_trylock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_spin_unlock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_self :: () -> pthread_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_getname_np :: (__pthread: pthread_t, __buf: *u8, __n: u64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_setname_np :: (__pthread: pthread_t, __name: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_setschedparam :: (__pthread: pthread_t, __policy: s32, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_getschedparam :: (__pthread: pthread_t, __policy: *s32, __param: *sched_param) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_setschedprio :: (__pthread: pthread_t, __priority: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `pthread_setspecific :: (__key: pthread_key_t, __value: *void) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__pthread_cleanup_t :: struct`
- [OS == .ANDROID && CPU == .X64] `__pthread_cleanup_push :: (c: *__pthread_cleanup_t, unknown0: #type () -> void #c_call, unknown1: *void) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `__pthread_cleanup_pop :: (unknown0: *__pthread_cleanup_t, unknown1: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_t :: struct`
- [OS == .ANDROID && CPU == .X64] `sem_clockwait :: (__sem: *sem_t, __clock: clockid_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_destroy :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_getvalue :: (__sem: *sem_t, __value: *s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_init :: (__sem: *sem_t, __shared: s32, __value: u32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_post :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_timedwait :: (__sem: *sem_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_timedwait_monotonic_np :: (__sem: *sem_t, __ts: *timespec) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_trywait :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_wait :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_open :: (__name: *u8, _flags: s32, __args: ..Any) -> *sem_t #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_close :: (__sem: *sem_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sem_unlink :: (__name: *u8) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigjmp_buf :: [12] s64`
- [OS == .ANDROID && CPU == .X64] `jmp_buf :: [11] s64`
- [OS == .ANDROID && CPU == .X64] `_setjmp :: (__env: *jmp_buf) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `_longjmp :: (__env: *jmp_buf, __value: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setjmp :: (__env: *jmp_buf) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `longjmp :: (__env: *jmp_buf, __value: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `sigsetjmp :: (__env: *sigjmp_buf, __save_signal_mask: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `siglongjmp :: (__env: *sigjmp_buf, __value: s32) -> void #foreign libc`
- [OS == .ANDROID && CPU == .X64] `RUSAGE :: enum s32`
- [OS == .ANDROID && CPU == .X64] `RLIM64_INFINITY :: ~0`
- [OS == .ANDROID && CPU == .X64] `PRIO_MIN :: - 20`
- [OS == .ANDROID && CPU == .X64] `PRIO_MAX :: 20`
- [OS == .ANDROID && CPU == .X64] `PRIO :: enum s32`
- [OS == .ANDROID && CPU == .X64] `_STK_LIM :: 8 * 1024 * 1024`
- [OS == .ANDROID && CPU == .X64] `MLOCK_LIMIT :: 8 * 1024 * 1024`
- [OS == .ANDROID && CPU == .X64] `RLIMIT_RESOURCE :: enum s32`
- [OS == .ANDROID && CPU == .X64] `RLIM_NLIMITS :: 16`
- [OS == .ANDROID && CPU == .X64] `RLIM_INFINITY :: ~0`
- [OS == .ANDROID && CPU == .X64] `RLIM_SAVED_CUR :: RLIM_INFINITY`
- [OS == .ANDROID && CPU == .X64] `RLIM_SAVED_MAX :: RLIM_INFINITY`
- [OS == .ANDROID && CPU == .X64] `id_t :: u32`
- [OS == .ANDROID && CPU == .X64] `rusage :: struct`
- [OS == .ANDROID && CPU == .X64] `rlimit :: struct`
- [OS == .ANDROID && CPU == .X64] `rlimit64 :: struct`
- [OS == .ANDROID && CPU == .X64] `rlim_t :: u64`
- [OS == .ANDROID && CPU == .X64] `rlim64_t :: u64`
- [OS == .ANDROID && CPU == .X64] `getrlimit :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setrlimit :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getrlimit64 :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setrlimit64 :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit64) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getpriority :: (__which: PRIO, __who: id_t) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `setpriority :: (__which: PRIO, __who: id_t, __priority: s32) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `getrusage :: (__who: RUSAGE, __usage: *rusage) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `prlimit :: (__pid: pid_t, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit, __old_limit: *rlimit) -> s32 #foreign libc`
- [OS == .ANDROID && CPU == .X64] `prlimit64 :: (__pid: pid_t, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit64, __old_limit: *rlimit64) -> s32 #foreign libc`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `AIO`

```jai
AIO :: enum u32 {
    CANCELED    :: 0;

    NOTCANCELED :: 1;

    ALLDONE     :: 2;

    AIO_CANCELED    :: CANCELED;

    AIO_NOTCANCELED :: NOTCANCELED;

    AIO_ALLDONE     :: ALLDONE;
}
```
### `ATTR_CMN`

Variant 1 (macOS, ARM64):

```jai
ATTR_CMN :: enum u32 {
    NAME                   :: 0x00000001;
    DEVID                  :: 0x00000002;
    FSID                   :: 0x00000004;
    OBJTYPE                :: 0x00000008;
    OBJTAG                 :: 0x00000010;
    OBJID                  :: 0x00000020;
    OBJPERMANENTID         :: 0x00000040;
    PAROBJID               :: 0x00000080;
    SCRIPT                 :: 0x00000100;
    CRTIME                 :: 0x00000200;
    MODTIME                :: 0x00000400;
    CHGTIME                :: 0x00000800;
    ACCTIME                :: 0x00001000;
    BKUPTIME               :: 0x00002000;
    FNDRINFO               :: 0x00004000;
    OWNERID                :: 0x00008000;
    GRPID                  :: 0x00010000;
    ACCESSMASK             :: 0x00020000;
    FLAGS                  :: 0x00040000;

    GEN_COUNT              :: 0x00080000;
    DOCUMENT_ID            :: 0x00100000;

    USERACCESS             :: 0x00200000;
    EXTENDED_SECURITY      :: 0x00400000;
    UUID                   :: 0x00800000;
    GRPUUID                :: 0x01000000;
    FILEID                 :: 0x02000000;
    PARENTID               :: 0x04000000;
    FULLPATH               :: 0x08000000;
    ADDEDTIME              :: 0x10000000;
    ERROR                  :: 0x20000000;
    DATA_PROTECT_FLAGS     :: 0x40000000;

    RETURNED_ATTRS         :: 0x80000000;

    VALIDMASK              :: 0xFFFFFFFF;

    SETMASK                :: 0x51C7FF00;
    VOLSETMASK             :: 0x00006700;

    EXT_RELPATH            :: 0x00000004;
    EXT_PRIVATESIZE        :: 0x00000008;
    EXT_LINKID             :: 0x00000010;
    EXT_NOFIRMLINKPATH     :: 0x00000020;
    EXT_REALDEVID          :: 0x00000040;
    EXT_REALFSID           :: 0x00000080;
    EXT_CLONEID            :: 0x00000100;
    EXT_EXT_FLAGS          :: 0x00000200;
    EXT_RECURSIVE_GENCOUNT :: 0x00000400;

    EXT_VALIDMASK          :: 0x000007fc;
    EXT_SETMASK            :: 0x00000000;

    NAMEDATTRCOUNT         :: 0x00080000;
    NAMEDATTRLIST          :: 0x00100000;

    ATTR_CMN_NAME                  :: NAME;
    ATTR_CMN_DEVID                 :: DEVID;
    ATTR_CMN_FSID                  :: FSID;
    ATTR_CMN_OBJTYPE               :: OBJTYPE;
    ATTR_CMN_OBJTAG                :: OBJTAG;
    ATTR_CMN_OBJID                 :: OBJID;
    ATTR_CMN_OBJPERMANENTID        :: OBJPERMANENTID;
    ATTR_CMN_PAROBJID              :: PAROBJID;
    ATTR_CMN_SCRIPT                :: SCRIPT;
    ATTR_CMN_CRTIME                :: CRTIME;
    ATTR_CMN_MODTIME               :: MODTIME;
    ATTR_CMN_CHGTIME               :: CHGTIME;
    ATTR_CMN_ACCTIME               :: ACCTIME;
    ATTR_CMN_BKUPTIME              :: BKUPTIME;
    ATTR_CMN_FNDRINFO              :: FNDRINFO;
    ATTR_CMN_OWNERID               :: OWNERID;
    ATTR_CMN_GRPID                 :: GRPID;
    ATTR_CMN_ACCESSMASK            :: ACCESSMASK;
    ATTR_CMN_FLAGS                 :: FLAGS;

    ATTR_CMN_GEN_COUNT             :: GEN_COUNT;
    ATTR_CMN_DOCUMENT_ID           :: DOCUMENT_ID;

    ATTR_CMN_USERACCESS            :: USERACCESS;
    ATTR_CMN_EXTENDED_SECURITY     :: EXTENDED_SECURITY;
    ATTR_CMN_UUID                  :: UUID;
    ATTR_CMN_GRPUUID               :: GRPUUID;
    ATTR_CMN_FILEID                :: FILEID;
    ATTR_CMN_PARENTID              :: PARENTID;
    ATTR_CMN_FULLPATH              :: FULLPATH;
    ATTR_CMN_ADDEDTIME             :: ADDEDTIME;
    ATTR_CMN_ERROR                 :: ERROR;
    ATTR_CMN_DATA_PROTECT_FLAGS    :: DATA_PROTECT_FLAGS;

    ATTR_CMN_RETURNED_ATTRS        :: RETURNED_ATTRS;

    ATTR_CMN_VALIDMASK             :: VALIDMASK;

    ATTR_CMN_SETMASK               :: SETMASK;
    ATTR_CMN_VOLSETMASK            :: VOLSETMASK;

    ATTR_CMNEXT_RELPATH            :: EXT_RELPATH;
    ATTR_CMNEXT_PRIVATESIZE        :: EXT_PRIVATESIZE;
    ATTR_CMNEXT_LINKID             :: EXT_LINKID;
    ATTR_CMNEXT_NOFIRMLINKPATH     :: EXT_NOFIRMLINKPATH;
    ATTR_CMNEXT_REALDEVID          :: EXT_REALDEVID;
    ATTR_CMNEXT_REALFSID           :: EXT_REALFSID;
    ATTR_CMNEXT_CLONEID            :: EXT_CLONEID;
    ATTR_CMNEXT_EXT_FLAGS          :: EXT_EXT_FLAGS;
    ATTR_CMNEXT_RECURSIVE_GENCOUNT :: EXT_RECURSIVE_GENCOUNT;

    ATTR_CMNEXT_VALIDMASK          :: EXT_VALIDMASK;
    ATTR_CMNEXT_SETMASK            :: EXT_SETMASK;

    ATTR_CMN_NAMEDATTRCOUNT        :: NAMEDATTRCOUNT;
    ATTR_CMN_NAMEDATTRLIST         :: NAMEDATTRLIST;
}
```
Variant 2 (macOS, x64):

```jai
ATTR_CMN :: enum u32 {
    NAME               :: 0x00000001;
    DEVID              :: 0x00000002;
    FSID               :: 0x00000004;
    OBJTYPE            :: 0x00000008;
    OBJTAG             :: 0x00000010;
    OBJID              :: 0x00000020;
    OBJPERMANENTID     :: 0x00000040;
    PAROBJID           :: 0x00000080;
    SCRIPT             :: 0x00000100;
    CRTIME             :: 0x00000200;
    MODTIME            :: 0x00000400;
    CHGTIME            :: 0x00000800;
    ACCTIME            :: 0x00001000;
    BKUPTIME           :: 0x00002000;
    FNDRINFO           :: 0x00004000;
    OWNERID            :: 0x00008000;
    GRPID              :: 0x00010000;
    ACCESSMASK         :: 0x00020000;
    FLAGS              :: 0x00040000;

    GEN_COUNT          :: 0x00080000;
    DOCUMENT_ID        :: 0x00100000;

    USERACCESS         :: 0x00200000;
    EXTENDED_SECURITY  :: 0x00400000;
    UUID               :: 0x00800000;
    GRPUUID            :: 0x01000000;
    FILEID             :: 0x02000000;
    PARENTID           :: 0x04000000;
    FULLPATH           :: 0x08000000;
    ADDEDTIME          :: 0x10000000;
    ERROR              :: 0x20000000;
    DATA_PROTECT_FLAGS :: 0x40000000;

    RETURNED_ATTRS     :: 0x80000000;

    VALIDMASK          :: 0xFFFFFFFF;

    SETMASK            :: 0x51C7FF00;
    VOLSETMASK         :: 0x00006700;

    EXT_RELPATH        :: 0x00000004;
    EXT_PRIVATESIZE    :: 0x00000008;
    EXT_LINKID         :: 0x00000010;

    EXT_VALIDMASK      :: 0x0000001c;
    EXT_SETMASK        :: 0x00000000;

    NAMEDATTRCOUNT     :: 0x00080000;
    NAMEDATTRLIST      :: 0x00100000;

    ATTR_CMN_NAME               :: NAME;
    ATTR_CMN_DEVID              :: DEVID;
    ATTR_CMN_FSID               :: FSID;
    ATTR_CMN_OBJTYPE            :: OBJTYPE;
    ATTR_CMN_OBJTAG             :: OBJTAG;
    ATTR_CMN_OBJID              :: OBJID;
    ATTR_CMN_OBJPERMANENTID     :: OBJPERMANENTID;
    ATTR_CMN_PAROBJID           :: PAROBJID;
    ATTR_CMN_SCRIPT             :: SCRIPT;
    ATTR_CMN_CRTIME             :: CRTIME;
    ATTR_CMN_MODTIME            :: MODTIME;
    ATTR_CMN_CHGTIME            :: CHGTIME;
    ATTR_CMN_ACCTIME            :: ACCTIME;
    ATTR_CMN_BKUPTIME           :: BKUPTIME;
    ATTR_CMN_FNDRINFO           :: FNDRINFO;
    ATTR_CMN_OWNERID            :: OWNERID;
    ATTR_CMN_GRPID              :: GRPID;
    ATTR_CMN_ACCESSMASK         :: ACCESSMASK;
    ATTR_CMN_FLAGS              :: FLAGS;

    ATTR_CMN_GEN_COUNT          :: GEN_COUNT;
    ATTR_CMN_DOCUMENT_ID        :: DOCUMENT_ID;

    ATTR_CMN_USERACCESS         :: USERACCESS;
    ATTR_CMN_EXTENDED_SECURITY  :: EXTENDED_SECURITY;
    ATTR_CMN_UUID               :: UUID;
    ATTR_CMN_GRPUUID            :: GRPUUID;
    ATTR_CMN_FILEID             :: FILEID;
    ATTR_CMN_PARENTID           :: PARENTID;
    ATTR_CMN_FULLPATH           :: FULLPATH;
    ATTR_CMN_ADDEDTIME          :: ADDEDTIME;
    ATTR_CMN_ERROR              :: ERROR;
    ATTR_CMN_DATA_PROTECT_FLAGS :: DATA_PROTECT_FLAGS;

    ATTR_CMN_RETURNED_ATTRS     :: RETURNED_ATTRS;

    ATTR_CMN_VALIDMASK          :: VALIDMASK;

    ATTR_CMN_SETMASK            :: SETMASK;
    ATTR_CMN_VOLSETMASK         :: VOLSETMASK;

    ATTR_CMNEXT_RELPATH         :: EXT_RELPATH;
    ATTR_CMNEXT_PRIVATESIZE     :: EXT_PRIVATESIZE;
    ATTR_CMNEXT_LINKID          :: EXT_LINKID;

    ATTR_CMNEXT_VALIDMASK       :: EXT_VALIDMASK;
    ATTR_CMNEXT_SETMASK         :: EXT_SETMASK;

    ATTR_CMN_NAMEDATTRCOUNT     :: NAMEDATTRCOUNT;
    ATTR_CMN_NAMEDATTRLIST      :: NAMEDATTRLIST;
}
```
### `ATTR_DIR`

```jai
ATTR_DIR :: enum {
    LINKCOUNT   :: 0x00000001;
    ENTRYCOUNT  :: 0x00000002;
    MOUNTSTATUS :: 0x00000004;
    ALLOCSIZE   :: 0x00000008;
    IOBLOCKSIZE :: 0x00000010;
    DATALENGTH  :: 0x00000020;

    VALIDMASK   :: 0x0000003f;
    SETMASK     :: 0x00000000;

    ATTR_DIR_LINKCOUNT   :: LINKCOUNT;
    ATTR_DIR_ENTRYCOUNT  :: ENTRYCOUNT;
    ATTR_DIR_MOUNTSTATUS :: MOUNTSTATUS;
    ATTR_DIR_ALLOCSIZE   :: ALLOCSIZE;
    ATTR_DIR_IOBLOCKSIZE :: IOBLOCKSIZE;
    ATTR_DIR_DATALENGTH  :: DATALENGTH;

    ATTR_DIR_VALIDMASK   :: VALIDMASK;
    ATTR_DIR_SETMASK     :: SETMASK;
}
```
### `ATTR_FILE`

```jai
ATTR_FILE :: enum {
    LINKCOUNT     :: 0x00000001;
    TOTALSIZE     :: 0x00000002;
    ALLOCSIZE     :: 0x00000004;
    IOBLOCKSIZE   :: 0x00000008;
    DEVTYPE       :: 0x00000020;
    FORKCOUNT     :: 0x00000080;
    FORKLIST      :: 0x00000100;
    DATALENGTH    :: 0x00000200;
    DATAALLOCSIZE :: 0x00000400;
    RSRCLENGTH    :: 0x00001000;
    RSRCALLOCSIZE :: 0x00002000;

    VALIDMASK     :: 0x000037FF;

    SETMASK       :: 0x00000020;

    CLUMPSIZE     :: 0x00000010;
    FILETYPE      :: 0x00000040;
    DATAEXTENTS   :: 0x00000800;
    RSRCEXTENTS   :: 0x00004000;

    ATTR_FILE_LINKCOUNT     :: LINKCOUNT;
    ATTR_FILE_TOTALSIZE     :: TOTALSIZE;
    ATTR_FILE_ALLOCSIZE     :: ALLOCSIZE;
    ATTR_FILE_IOBLOCKSIZE   :: IOBLOCKSIZE;
    ATTR_FILE_DEVTYPE       :: DEVTYPE;
    ATTR_FILE_FORKCOUNT     :: FORKCOUNT;
    ATTR_FILE_FORKLIST      :: FORKLIST;
    ATTR_FILE_DATALENGTH    :: DATALENGTH;
    ATTR_FILE_DATAALLOCSIZE :: DATAALLOCSIZE;
    ATTR_FILE_RSRCLENGTH    :: RSRCLENGTH;
    ATTR_FILE_RSRCALLOCSIZE :: RSRCALLOCSIZE;

    ATTR_FILE_VALIDMASK     :: VALIDMASK;

    ATTR_FILE_SETMASK       :: SETMASK;

    ATTR_FILE_CLUMPSIZE     :: CLUMPSIZE;
    ATTR_FILE_FILETYPE      :: FILETYPE;
    ATTR_FILE_DATAEXTENTS   :: DATAEXTENTS;
    ATTR_FILE_RSRCEXTENTS   :: RSRCEXTENTS;
}
```
### `ATTR_FORK`

```jai
ATTR_FORK :: enum {
    TOTALSIZE :: 0x00000001;
    ALLOCSIZE :: 0x00000002;
    RESERVED  :: 0xffffffff;

    VALIDMASK :: 0x00000003;
    SETMASK   :: 0x00000000;

    ATTR_FORK_TOTALSIZE :: TOTALSIZE;
    ATTR_FORK_ALLOCSIZE :: ALLOCSIZE;
    ATTR_FORK_RESERVED  :: RESERVED;

    ATTR_FORK_VALIDMASK :: VALIDMASK;
    ATTR_FORK_SETMASK   :: SETMASK;
}
```
### `ATTR_VOL`

```jai
ATTR_VOL :: enum {
    FSTYPE          :: 0x00000001;
    SIGNATURE       :: 0x00000002;
    SIZE            :: 0x00000004;
    SPACEFREE       :: 0x00000008;
    SPACEAVAIL      :: 0x00000010;
    MINALLOCATION   :: 0x00000020;
    ALLOCATIONCLUMP :: 0x00000040;
    IOBLOCKSIZE     :: 0x00000080;
    OBJCOUNT        :: 0x00000100;
    FILECOUNT       :: 0x00000200;
    DIRCOUNT        :: 0x00000400;
    MAXOBJCOUNT     :: 0x00000800;
    MOUNTPOINT      :: 0x00001000;
    NAME            :: 0x00002000;
    MOUNTFLAGS      :: 0x00004000;
    MOUNTEDDEVICE   :: 0x00008000;
    ENCODINGSUSED   :: 0x00010000;
    CAPABILITIES    :: 0x00020000;
    UUID            :: 0x00040000;
    QUOTA_SIZE      :: 0x10000000;
    RESERVED_SIZE   :: 0x20000000;
    ATTRIBUTES      :: 0x40000000;
    INFO            :: 0x80000000;

    VALIDMASK       :: 0xF007FFFF;

    SETMASK         :: 0x80002000;

    ATTR_VOL_FSTYPE          :: FSTYPE;
    ATTR_VOL_SIGNATURE       :: SIGNATURE;
    ATTR_VOL_SIZE            :: SIZE;
    ATTR_VOL_SPACEFREE       :: SPACEFREE;
    ATTR_VOL_SPACEAVAIL      :: SPACEAVAIL;
    ATTR_VOL_MINALLOCATION   :: MINALLOCATION;
    ATTR_VOL_ALLOCATIONCLUMP :: ALLOCATIONCLUMP;
    ATTR_VOL_IOBLOCKSIZE     :: IOBLOCKSIZE;
    ATTR_VOL_OBJCOUNT        :: OBJCOUNT;
    ATTR_VOL_FILECOUNT       :: FILECOUNT;
    ATTR_VOL_DIRCOUNT        :: DIRCOUNT;
    ATTR_VOL_MAXOBJCOUNT     :: MAXOBJCOUNT;
    ATTR_VOL_MOUNTPOINT      :: MOUNTPOINT;
    ATTR_VOL_NAME            :: NAME;
    ATTR_VOL_MOUNTFLAGS      :: MOUNTFLAGS;
    ATTR_VOL_MOUNTEDDEVICE   :: MOUNTEDDEVICE;
    ATTR_VOL_ENCODINGSUSED   :: ENCODINGSUSED;
    ATTR_VOL_CAPABILITIES    :: CAPABILITIES;
    ATTR_VOL_UUID            :: UUID;
    ATTR_VOL_QUOTA_SIZE      :: QUOTA_SIZE;
    ATTR_VOL_RESERVED_SIZE   :: RESERVED_SIZE;
    ATTR_VOL_ATTRIBUTES      :: ATTRIBUTES;
    ATTR_VOL_INFO            :: INFO;

    ATTR_VOL_VALIDMASK       :: VALIDMASK;

    ATTR_VOL_SETMASK         :: SETMASK;
}
```
### `CS`

```jai
CS :: enum u32 {
    PATH                            :: 0;

    V6_WIDTH_RESTRICTED_ENVS        :: 1;

    GNU_LIBC_VERSION                :: 2;

    GNU_LIBPTHREAD_VERSION          :: 3;

    V5_WIDTH_RESTRICTED_ENVS        :: 4;

    V7_WIDTH_RESTRICTED_ENVS        :: 5;

    LFS_CFLAGS                      :: 1000;

    LFS_LDFLAGS                     :: 1001;

    LFS_LIBS                        :: 1002;

    LFS_LINTFLAGS                   :: 1003;

    LFS64_CFLAGS                    :: 1004;

    LFS64_LDFLAGS                   :: 1005;

    LFS64_LIBS                      :: 1006;

    LFS64_LINTFLAGS                 :: 1007;

    XBS5_ILP32_OFF32_CFLAGS         :: 1100;

    XBS5_ILP32_OFF32_LDFLAGS        :: 1101;

    XBS5_ILP32_OFF32_LIBS           :: 1102;

    XBS5_ILP32_OFF32_LINTFLAGS      :: 1103;

    XBS5_ILP32_OFFBIG_CFLAGS        :: 1104;

    XBS5_ILP32_OFFBIG_LDFLAGS       :: 1105;

    XBS5_ILP32_OFFBIG_LIBS          :: 1106;

    XBS5_ILP32_OFFBIG_LINTFLAGS     :: 1107;

    XBS5_LP64_OFF64_CFLAGS          :: 1108;

    XBS5_LP64_OFF64_LDFLAGS         :: 1109;

    XBS5_LP64_OFF64_LIBS            :: 1110;

    XBS5_LP64_OFF64_LINTFLAGS       :: 1111;

    XBS5_LPBIG_OFFBIG_CFLAGS        :: 1112;

    XBS5_LPBIG_OFFBIG_LDFLAGS       :: 1113;

    XBS5_LPBIG_OFFBIG_LIBS          :: 1114;

    XBS5_LPBIG_OFFBIG_LINTFLAGS     :: 1115;

    POSIX_V6_ILP32_OFF32_CFLAGS     :: 1116;

    POSIX_V6_ILP32_OFF32_LDFLAGS    :: 1117;

    POSIX_V6_ILP32_OFF32_LIBS       :: 1118;

    POSIX_V6_ILP32_OFF32_LINTFLAGS  :: 1119;

    POSIX_V6_ILP32_OFFBIG_CFLAGS    :: 1120;

    POSIX_V6_ILP32_OFFBIG_LDFLAGS   :: 1121;

    POSIX_V6_ILP32_OFFBIG_LIBS      :: 1122;

    POSIX_V6_ILP32_OFFBIG_LINTFLAGS :: 1123;

    POSIX_V6_LP64_OFF64_CFLAGS      :: 1124;

    POSIX_V6_LP64_OFF64_LDFLAGS     :: 1125;

    POSIX_V6_LP64_OFF64_LIBS        :: 1126;

    POSIX_V6_LP64_OFF64_LINTFLAGS   :: 1127;

    POSIX_V6_LPBIG_OFFBIG_CFLAGS    :: 1128;

    POSIX_V6_LPBIG_OFFBIG_LDFLAGS   :: 1129;

    POSIX_V6_LPBIG_OFFBIG_LIBS      :: 1130;

    POSIX_V6_LPBIG_OFFBIG_LINTFLAGS :: 1131;

    POSIX_V7_ILP32_OFF32_CFLAGS     :: 1132;

    POSIX_V7_ILP32_OFF32_LDFLAGS    :: 1133;

    POSIX_V7_ILP32_OFF32_LIBS       :: 1134;

    POSIX_V7_ILP32_OFF32_LINTFLAGS  :: 1135;

    POSIX_V7_ILP32_OFFBIG_CFLAGS    :: 1136;

    POSIX_V7_ILP32_OFFBIG_LDFLAGS   :: 1137;

    POSIX_V7_ILP32_OFFBIG_LIBS      :: 1138;

    POSIX_V7_ILP32_OFFBIG_LINTFLAGS :: 1139;

    POSIX_V7_LP64_OFF64_CFLAGS      :: 1140;

    POSIX_V7_LP64_OFF64_LDFLAGS     :: 1141;

    POSIX_V7_LP64_OFF64_LIBS        :: 1142;

    POSIX_V7_LP64_OFF64_LINTFLAGS   :: 1143;

    POSIX_V7_LPBIG_OFFBIG_CFLAGS    :: 1144;

    POSIX_V7_LPBIG_OFFBIG_LDFLAGS   :: 1145;

    POSIX_V7_LPBIG_OFFBIG_LIBS      :: 1146;

    POSIX_V7_LPBIG_OFFBIG_LINTFLAGS :: 1147;

    V6_ENV                          :: 1148;

    V7_ENV                          :: 1149;

    _CS_PATH                            :: PATH;

    _CS_V6_WIDTH_RESTRICTED_ENVS        :: V6_WIDTH_RESTRICTED_ENVS;

    _CS_GNU_LIBC_VERSION                :: GNU_LIBC_VERSION;

    _CS_GNU_LIBPTHREAD_VERSION          :: GNU_LIBPTHREAD_VERSION;

    _CS_V5_WIDTH_RESTRICTED_ENVS        :: V5_WIDTH_RESTRICTED_ENVS;

    _CS_V7_WIDTH_RESTRICTED_ENVS        :: V7_WIDTH_RESTRICTED_ENVS;

    _CS_LFS_CFLAGS                      :: LFS_CFLAGS;

    _CS_LFS_LDFLAGS                     :: LFS_LDFLAGS;

    _CS_LFS_LIBS                        :: LFS_LIBS;

    _CS_LFS_LINTFLAGS                   :: LFS_LINTFLAGS;

    _CS_LFS64_CFLAGS                    :: LFS64_CFLAGS;

    _CS_LFS64_LDFLAGS                   :: LFS64_LDFLAGS;

    _CS_LFS64_LIBS                      :: LFS64_LIBS;

    _CS_LFS64_LINTFLAGS                 :: LFS64_LINTFLAGS;

    _CS_XBS5_ILP32_OFF32_CFLAGS         :: XBS5_ILP32_OFF32_CFLAGS;

    _CS_XBS5_ILP32_OFF32_LDFLAGS        :: XBS5_ILP32_OFF32_LDFLAGS;

    _CS_XBS5_ILP32_OFF32_LIBS           :: XBS5_ILP32_OFF32_LIBS;

    _CS_XBS5_ILP32_OFF32_LINTFLAGS      :: XBS5_ILP32_OFF32_LINTFLAGS;

    _CS_XBS5_ILP32_OFFBIG_CFLAGS        :: XBS5_ILP32_OFFBIG_CFLAGS;

    _CS_XBS5_ILP32_OFFBIG_LDFLAGS       :: XBS5_ILP32_OFFBIG_LDFLAGS;

    _CS_XBS5_ILP32_OFFBIG_LIBS          :: XBS5_ILP32_OFFBIG_LIBS;

    _CS_XBS5_ILP32_OFFBIG_LINTFLAGS     :: XBS5_ILP32_OFFBIG_LINTFLAGS;

    _CS_XBS5_LP64_OFF64_CFLAGS          :: XBS5_LP64_OFF64_CFLAGS;

    _CS_XBS5_LP64_OFF64_LDFLAGS         :: XBS5_LP64_OFF64_LDFLAGS;

    _CS_XBS5_LP64_OFF64_LIBS            :: XBS5_LP64_OFF64_LIBS;

    _CS_XBS5_LP64_OFF64_LINTFLAGS       :: XBS5_LP64_OFF64_LINTFLAGS;

    _CS_XBS5_LPBIG_OFFBIG_CFLAGS        :: XBS5_LPBIG_OFFBIG_CFLAGS;

    _CS_XBS5_LPBIG_OFFBIG_LDFLAGS       :: XBS5_LPBIG_OFFBIG_LDFLAGS;

    _CS_XBS5_LPBIG_OFFBIG_LIBS          :: XBS5_LPBIG_OFFBIG_LIBS;

    _CS_XBS5_LPBIG_OFFBIG_LINTFLAGS     :: XBS5_LPBIG_OFFBIG_LINTFLAGS;

    _CS_POSIX_V6_ILP32_OFF32_CFLAGS     :: POSIX_V6_ILP32_OFF32_CFLAGS;

    _CS_POSIX_V6_ILP32_OFF32_LDFLAGS    :: POSIX_V6_ILP32_OFF32_LDFLAGS;

    _CS_POSIX_V6_ILP32_OFF32_LIBS       :: POSIX_V6_ILP32_OFF32_LIBS;

    _CS_POSIX_V6_ILP32_OFF32_LINTFLAGS  :: POSIX_V6_ILP32_OFF32_LINTFLAGS;

    _CS_POSIX_V6_ILP32_OFFBIG_CFLAGS    :: POSIX_V6_ILP32_OFFBIG_CFLAGS;

    _CS_POSIX_V6_ILP32_OFFBIG_LDFLAGS   :: POSIX_V6_ILP32_OFFBIG_LDFLAGS;

    _CS_POSIX_V6_ILP32_OFFBIG_LIBS      :: POSIX_V6_ILP32_OFFBIG_LIBS;

    _CS_POSIX_V6_ILP32_OFFBIG_LINTFLAGS :: POSIX_V6_ILP32_OFFBIG_LINTFLAGS;

    _CS_POSIX_V6_LP64_OFF64_CFLAGS      :: POSIX_V6_LP64_OFF64_CFLAGS;

    _CS_POSIX_V6_LP64_OFF64_LDFLAGS     :: POSIX_V6_LP64_OFF64_LDFLAGS;

    _CS_POSIX_V6_LP64_OFF64_LIBS        :: POSIX_V6_LP64_OFF64_LIBS;

    _CS_POSIX_V6_LP64_OFF64_LINTFLAGS   :: POSIX_V6_LP64_OFF64_LINTFLAGS;

    _CS_POSIX_V6_LPBIG_OFFBIG_CFLAGS    :: POSIX_V6_LPBIG_OFFBIG_CFLAGS;

    _CS_POSIX_V6_LPBIG_OFFBIG_LDFLAGS   :: POSIX_V6_LPBIG_OFFBIG_LDFLAGS;

    _CS_POSIX_V6_LPBIG_OFFBIG_LIBS      :: POSIX_V6_LPBIG_OFFBIG_LIBS;

    _CS_POSIX_V6_LPBIG_OFFBIG_LINTFLAGS :: POSIX_V6_LPBIG_OFFBIG_LINTFLAGS;

    _CS_POSIX_V7_ILP32_OFF32_CFLAGS     :: POSIX_V7_ILP32_OFF32_CFLAGS;

    _CS_POSIX_V7_ILP32_OFF32_LDFLAGS    :: POSIX_V7_ILP32_OFF32_LDFLAGS;

    _CS_POSIX_V7_ILP32_OFF32_LIBS       :: POSIX_V7_ILP32_OFF32_LIBS;

    _CS_POSIX_V7_ILP32_OFF32_LINTFLAGS  :: POSIX_V7_ILP32_OFF32_LINTFLAGS;

    _CS_POSIX_V7_ILP32_OFFBIG_CFLAGS    :: POSIX_V7_ILP32_OFFBIG_CFLAGS;

    _CS_POSIX_V7_ILP32_OFFBIG_LDFLAGS   :: POSIX_V7_ILP32_OFFBIG_LDFLAGS;

    _CS_POSIX_V7_ILP32_OFFBIG_LIBS      :: POSIX_V7_ILP32_OFFBIG_LIBS;

    _CS_POSIX_V7_ILP32_OFFBIG_LINTFLAGS :: POSIX_V7_ILP32_OFFBIG_LINTFLAGS;

    _CS_POSIX_V7_LP64_OFF64_CFLAGS      :: POSIX_V7_LP64_OFF64_CFLAGS;

    _CS_POSIX_V7_LP64_OFF64_LDFLAGS     :: POSIX_V7_LP64_OFF64_LDFLAGS;

    _CS_POSIX_V7_LP64_OFF64_LIBS        :: POSIX_V7_LP64_OFF64_LIBS;

    _CS_POSIX_V7_LP64_OFF64_LINTFLAGS   :: POSIX_V7_LP64_OFF64_LINTFLAGS;

    _CS_POSIX_V7_LPBIG_OFFBIG_CFLAGS    :: POSIX_V7_LPBIG_OFFBIG_CFLAGS;

    _CS_POSIX_V7_LPBIG_OFFBIG_LDFLAGS   :: POSIX_V7_LPBIG_OFFBIG_LDFLAGS;

    _CS_POSIX_V7_LPBIG_OFFBIG_LIBS      :: POSIX_V7_LPBIG_OFFBIG_LIBS;

    _CS_POSIX_V7_LPBIG_OFFBIG_LINTFLAGS :: POSIX_V7_LPBIG_OFFBIG_LINTFLAGS;

    _CS_V6_ENV                          :: V6_ENV;

    _CS_V7_ENV                          :: V7_ENV;
}
```
### `DIR`

Variant 1 (macOS, ARM64):

```jai
DIR :: struct {
    __dd_fd:    s32;
    __dd_loc:   s64;
    __dd_size:  s64;
    __dd_buf:   *u8;
    __dd_len:   s32;
    __dd_seek:  s64;
    __padding:  s64;
    __dd_flags: s32;
    __dd_lock:  _opaque_pthread_mutex_t;
    __dd_td:    *_telldir;
}
```
Variant 2 (Android, ARM64):

```jai
DIR :: struct {}
```
### `DT`

Variant 1 (Linux):

```jai
DT :: enum u8 {
    UNKNOWN :: 0;

    FIFO    :: 1;

    CHR     :: 2;

    DIR     :: 4;

    BLK     :: 6;

    REG     :: 8;

    LNK     :: 10;

    SOCK    :: 12;

    WHT     :: 14;
}
```
Variant 2 (macOS, ARM64):

```jai
DT :: enum u8 {
    UNKNOWN :: 0;
    FIFO    :: 1;
    CHR     :: 2;
    DIR     :: 4;
    BLK     :: 6;
    REG     :: 8;
    LNK     :: 10;
    SOCK    :: 12;
    WHT     :: 14;
}
```
Variant 3 (Android, ARM64):

```jai
DT :: enum u8 {
    UNKNOWN :: 0;

    FIFO    :: 1;

    CHR     :: 2;

    DIR     :: 4;

    BLK     :: 6;

    REG     :: 8;

    LNK     :: 10;

    SOCK    :: 12;
    WHT     :: 14;
}
```
### `Dl_info`

Variant 1 (Linux):

```jai
Dl_info :: struct {
    dli_fname: *u8;
    dli_fbase: *void;
    dli_sname: *u8;
    dli_saddr: *void;
}
```
Variant 2 (Android, ARM64):

```jai
Dl_info :: struct {

    dli_fname: *u8;

    dli_fbase: *void;

    dli_sname: *u8;

    dli_saddr: *void;
}
```
### `Dl_serinfo`

```jai
Dl_serinfo :: struct {
    dls_size: u64;
    dls_cnt:  u32;

    struct {
        __dls_serpath_pad: [1] Dl_serpath;

        #overlay (__dls_serpath_pad) dls_serpath: [0] Dl_serpath;
    }
}
```
### `Dl_serpath`

```jai
Dl_serpath :: struct {
    dls_name:  *u8;
    dls_flags: u32;
}
```
### `EFD`

```jai
EFD :: enum u32 {
    SEMAPHORE :: 1;

    CLOEXEC   :: 524288;

    NONBLOCK  :: 2048;

    EFD_SEMAPHORE :: SEMAPHORE;

    EFD_CLOEXEC   :: CLOEXEC;

    EFD_NONBLOCK  :: NONBLOCK;
}
```
### `FILE`

```jai
FILE :: struct {}
```
### `LIO`

```jai
LIO :: enum u32 {
    READ  :: 0;

    WRITE :: 1;

    NOP   :: 2;

    LIO_READ  :: READ;

    LIO_WRITE :: WRITE;

    LIO_NOP   :: NOP;
}
```
### `LIO_1`

```jai
LIO_1 :: enum u32 {
    WAIT   :: 0;

    NOWAIT :: 1;

    LIO_WAIT   :: WAIT;

    LIO_NOWAIT :: NOWAIT;
}
```
### `PC`

Variant 1 (Linux):

```jai
PC :: enum u32 {
    LINK_MAX           :: 0;

    MAX_CANON          :: 1;

    MAX_INPUT          :: 2;

    NAME_MAX           :: 3;

    PATH_MAX           :: 4;

    PIPE_BUF           :: 5;

    CHOWN_RESTRICTED   :: 6;

    NO_TRUNC           :: 7;

    VDISABLE           :: 8;

    SYNC_IO            :: 9;

    ASYNC_IO           :: 10;

    PRIO_IO            :: 11;

    SOCK_MAXBUF        :: 12;

    FILESIZEBITS       :: 13;

    REC_INCR_XFER_SIZE :: 14;

    REC_MAX_XFER_SIZE  :: 15;

    REC_MIN_XFER_SIZE  :: 16;

    REC_XFER_ALIGN     :: 17;

    ALLOC_SIZE_MIN     :: 18;

    SYMLINK_MAX        :: 19;

    _2_SYMLINKS        :: 20;

    _PC_LINK_MAX           :: LINK_MAX;

    _PC_MAX_CANON          :: MAX_CANON;

    _PC_MAX_INPUT          :: MAX_INPUT;

    _PC_NAME_MAX           :: NAME_MAX;

    _PC_PATH_MAX           :: PATH_MAX;

    _PC_PIPE_BUF           :: PIPE_BUF;

    _PC_CHOWN_RESTRICTED   :: CHOWN_RESTRICTED;

    _PC_NO_TRUNC           :: NO_TRUNC;

    _PC_VDISABLE           :: VDISABLE;

    _PC_SYNC_IO            :: SYNC_IO;

    _PC_ASYNC_IO           :: ASYNC_IO;

    _PC_PRIO_IO            :: PRIO_IO;

    _PC_SOCK_MAXBUF        :: SOCK_MAXBUF;

    _PC_FILESIZEBITS       :: FILESIZEBITS;

    _PC_REC_INCR_XFER_SIZE :: REC_INCR_XFER_SIZE;

    _PC_REC_MAX_XFER_SIZE  :: REC_MAX_XFER_SIZE;

    _PC_REC_MIN_XFER_SIZE  :: REC_MIN_XFER_SIZE;

    _PC_REC_XFER_ALIGN     :: REC_XFER_ALIGN;

    _PC_ALLOC_SIZE_MIN     :: ALLOC_SIZE_MIN;

    _PC_SYMLINK_MAX        :: SYMLINK_MAX;

    _PC_2_SYMLINKS         :: _2_SYMLINKS;
}
```
Variant 2 (macOS, ARM64):

```jai
PC :: enum u32 {
    LINK_MAX             :: 1;
    MAX_CANON            :: 2;
    MAX_INPUT            :: 3;
    NAME_MAX             :: 4;
    PATH_MAX             :: 5;
    PIPE_BUF             :: 6;
    CHOWN_RESTRICTED     :: 7;
    NO_TRUNC             :: 8;
    VDISABLE             :: 9;

    NAME_CHARS_MAX       :: 10;
    CASE_SENSITIVE       :: 11;
    CASE_PRESERVING      :: 12;
    EXTENDED_SECURITY_NP :: 13;
    AUTH_OPAQUE_NP       :: 14;

    _2_SYMLINKS          :: 15;
    ALLOC_SIZE_MIN       :: 16;
    ASYNC_IO             :: 17;
    FILESIZEBITS         :: 18;
    PRIO_IO              :: 19;
    REC_INCR_XFER_SIZE   :: 20;
    REC_MAX_XFER_SIZE    :: 21;
    REC_MIN_XFER_SIZE    :: 22;
    REC_XFER_ALIGN       :: 23;
    SYMLINK_MAX          :: 24;
    SYNC_IO              :: 25;
    XATTR_SIZE_BITS      :: 26;
    MIN_HOLE_SIZE        :: 27;
}
```
Variant 3 (Android, ARM64):

```jai
PC :: enum u32 {
    FILESIZEBITS       :: 0;
    LINK_MAX           :: 1;
    MAX_CANON          :: 2;
    MAX_INPUT          :: 3;
    NAME_MAX           :: 4;
    PATH_MAX           :: 5;
    PIPE_BUF           :: 6;
    _2_SYMLINKS        :: 7;
    ALLOC_SIZE_MIN     :: 8;
    REC_INCR_XFER_SIZE :: 9;
    REC_MAX_XFER_SIZE  :: 10;
    REC_MIN_XFER_SIZE  :: 11;
    REC_XFER_ALIGN     :: 12;
    SYMLINK_MAX        :: 13;
    CHOWN_RESTRICTED   :: 14;
    NO_TRUNC           :: 15;
    VDISABLE           :: 16;
    ASYNC_IO           :: 17;
    PRIO_IO            :: 18;
    SYNC_IO            :: 19;
}
```
### `PRIO`

Variant 1 (Linux):

```jai
PRIO :: enum u32 {
    PROCESS :: 0;

    PGRP    :: 1;

    USER    :: 2;
}
```
Variant 2 (macOS, ARM64):

```jai
PRIO :: enum s32 {
    PROCESS        :: 0;
    PGRP           :: 1;
    USER           :: 2;

    DARWIN_THREAD  :: 3;
    DARWIN_PROCESS :: 4;

    DARWIN_BG      :: 0x1000;

    DARWIN_NONUI   :: 0x1001;
}
```
Variant 3 (Android, ARM64):

```jai
PRIO :: enum s32 {
    PROCESS :: 0;
    PGRP    :: 1;
    USER    :: 2;
}
```
### `PTHREAD`

```jai
PTHREAD :: enum u32 {
    INHERIT_SCHED  :: 0;

    EXPLICIT_SCHED :: 1;

    PTHREAD_INHERIT_SCHED  :: INHERIT_SCHED;

    PTHREAD_EXPLICIT_SCHED :: EXPLICIT_SCHED;
}
```
### `PTHREAD_CANCEL`

```jai
PTHREAD_CANCEL :: enum u32 {
    ENABLE  :: 0;

    DISABLE :: 1;

    PTHREAD_CANCEL_ENABLE  :: ENABLE;

    PTHREAD_CANCEL_DISABLE :: DISABLE;
}
```
### `PTHREAD_CANCEL_1`

```jai
PTHREAD_CANCEL_1 :: enum u32 {
    DEFERRED     :: 0;

    ASYNCHRONOUS :: 1;

    PTHREAD_CANCEL_DEFERRED     :: DEFERRED;

    PTHREAD_CANCEL_ASYNCHRONOUS :: ASYNCHRONOUS;
}
```
### `PTHREAD_CREATE`

```jai
PTHREAD_CREATE :: enum u32 {
    JOINABLE :: 0;

    DETACHED :: 1;

    PTHREAD_CREATE_JOINABLE :: JOINABLE;

    PTHREAD_CREATE_DETACHED :: DETACHED;
}
```
### `PTHREAD_MUTEX`

Variant 1 (Linux):

```jai
PTHREAD_MUTEX :: enum u32 {
    TIMED_NP      :: 0;
    RECURSIVE_NP  :: 1;
    ERRORCHECK_NP :: 2;
    ADAPTIVE_NP   :: 3;

    NORMAL        :: 0;
    RECURSIVE     :: 1;
    ERRORCHECK    :: 2;
    DEFAULT       :: 0;

    FAST_NP       :: 0;

    PTHREAD_MUTEX_TIMED_NP      :: TIMED_NP;
    PTHREAD_MUTEX_RECURSIVE_NP  :: RECURSIVE_NP;
    PTHREAD_MUTEX_ERRORCHECK_NP :: ERRORCHECK_NP;
    PTHREAD_MUTEX_ADAPTIVE_NP   :: ADAPTIVE_NP;

    PTHREAD_MUTEX_NORMAL        :: NORMAL;
    PTHREAD_MUTEX_RECURSIVE     :: RECURSIVE;
    PTHREAD_MUTEX_ERRORCHECK    :: ERRORCHECK;
    PTHREAD_MUTEX_DEFAULT       :: DEFAULT;

    PTHREAD_MUTEX_FAST_NP       :: FAST_NP;
}
```
Variant 2 (Android, ARM64):

```jai
PTHREAD_MUTEX :: enum u32 {
    NORMAL        :: 0;
    RECURSIVE     :: 1;
    ERRORCHECK    :: 2;

    ERRORCHECK_NP :: 2;
    RECURSIVE_NP  :: 1;

    DEFAULT       :: 0;

    PTHREAD_MUTEX_NORMAL        :: NORMAL;
    PTHREAD_MUTEX_RECURSIVE     :: RECURSIVE;
    PTHREAD_MUTEX_ERRORCHECK    :: ERRORCHECK;

    PTHREAD_MUTEX_ERRORCHECK_NP :: ERRORCHECK_NP;
    PTHREAD_MUTEX_RECURSIVE_NP  :: RECURSIVE_NP;

    PTHREAD_MUTEX_DEFAULT       :: DEFAULT;
}
```
### `PTHREAD_MUTEX_1`

```jai
PTHREAD_MUTEX_1 :: enum u32 {
    STALLED    :: 0;
    STALLED_NP :: 0;
    ROBUST     :: 1;
    ROBUST_NP  :: 1;

    PTHREAD_MUTEX_STALLED    :: STALLED;
    PTHREAD_MUTEX_STALLED_NP :: STALLED_NP;
    PTHREAD_MUTEX_ROBUST     :: ROBUST;
    PTHREAD_MUTEX_ROBUST_NP  :: ROBUST_NP;
}
```
### `PTHREAD_PRIO`

```jai
PTHREAD_PRIO :: enum u32 {
    NONE    :: 0;
    INHERIT :: 1;
    PROTECT :: 2;

    PTHREAD_PRIO_NONE    :: NONE;
    PTHREAD_PRIO_INHERIT :: INHERIT;
    PTHREAD_PRIO_PROTECT :: PROTECT;
}
```
### `PTHREAD_PROCESS`

```jai
PTHREAD_PROCESS :: enum u32 {
    PRIVATE :: 0;

    SHARED  :: 1;

    PTHREAD_PROCESS_PRIVATE :: PRIVATE;

    PTHREAD_PROCESS_SHARED  :: SHARED;
}
```
### `PTHREAD_RWLOCK`

```jai
PTHREAD_RWLOCK :: enum u32 {
    PREFER_READER_NP              :: 0;
    PREFER_WRITER_NP              :: 1;
    PREFER_WRITER_NONRECURSIVE_NP :: 2;
    DEFAULT_NP                    :: 0;

    PTHREAD_RWLOCK_PREFER_READER_NP              :: PREFER_READER_NP;
    PTHREAD_RWLOCK_PREFER_WRITER_NP              :: PREFER_WRITER_NP;
    PTHREAD_RWLOCK_PREFER_WRITER_NONRECURSIVE_NP :: PREFER_WRITER_NONRECURSIVE_NP;
    PTHREAD_RWLOCK_DEFAULT_NP                    :: DEFAULT_NP;
}
```
### `PTHREAD_RWLOCK_PREFER`

```jai
PTHREAD_RWLOCK_PREFER :: enum u32 {
    READER_NP              :: 0;
    WRITER_NONRECURSIVE_NP :: 1;

    PTHREAD_RWLOCK_PREFER_READER_NP              :: READER_NP;
    PTHREAD_RWLOCK_PREFER_WRITER_NONRECURSIVE_NP :: WRITER_NONRECURSIVE_NP;
}
```
### `PTHREAD_SCOPE`

```jai
PTHREAD_SCOPE :: enum u32 {
    SYSTEM  :: 0;

    PROCESS :: 1;

    PTHREAD_SCOPE_SYSTEM  :: SYSTEM;

    PTHREAD_SCOPE_PROCESS :: PROCESS;
}
```
### `QOS_CLASS`

```jai
QOS_CLASS :: enum u32 {
    USER_INTERACTIVE :: 33;
    USER_INITIATED   :: 25;
    DEFAULT          :: 21;
    UTILITY          :: 17;
    BACKGROUND       :: 9;
    UNSPECIFIED      :: 0;
    QOS_CLASS_USER_INTERACTIVE :: USER_INTERACTIVE;
    QOS_CLASS_USER_INITIATED   :: USER_INITIATED;
    QOS_CLASS_DEFAULT          :: DEFAULT;
    QOS_CLASS_UTILITY          :: UTILITY;
    QOS_CLASS_BACKGROUND       :: BACKGROUND;
    QOS_CLASS_UNSPECIFIED      :: UNSPECIFIED;
}
```
### `REG`

Variant 1 (Linux):

```jai
REG :: enum u32 {
    R8      :: 0;

    R9      :: 1;

    R10     :: 2;

    R11     :: 3;

    R12     :: 4;

    R13     :: 5;

    R14     :: 6;

    R15     :: 7;

    RDI     :: 8;

    RSI     :: 9;

    RBP     :: 10;

    RBX     :: 11;

    RDX     :: 12;

    RAX     :: 13;

    RCX     :: 14;

    RSP     :: 15;

    RIP     :: 16;

    EFL     :: 17;

    CSGSFS  :: 18;

    ERR     :: 19;

    TRAPNO  :: 20;

    OLDMASK :: 21;

    CR2     :: 22;

    REG_R8      :: R8;

    REG_R9      :: R9;

    REG_R10     :: R10;

    REG_R11     :: R11;

    REG_R12     :: R12;

    REG_R13     :: R13;

    REG_R14     :: R14;

    REG_R15     :: R15;

    REG_RDI     :: RDI;

    REG_RSI     :: RSI;

    REG_RBP     :: RBP;

    REG_RBX     :: RBX;

    REG_RDX     :: RDX;

    REG_RAX     :: RAX;

    REG_RCX     :: RCX;

    REG_RSP     :: RSP;

    REG_RIP     :: RIP;

    REG_EFL     :: EFL;

    REG_CSGSFS  :: CSGSFS;

    REG_ERR     :: ERR;

    REG_TRAPNO  :: TRAPNO;

    REG_OLDMASK :: OLDMASK;

    REG_CR2     :: CR2;
}
```
Variant 2 (Android, x64):

```jai
REG :: enum u32 {
    REG_R8      :: 0;

    REG_R9      :: 1;

    REG_R10     :: 2;

    REG_R11     :: 3;

    REG_R12     :: 4;

    REG_R13     :: 5;

    REG_R14     :: 6;

    REG_R15     :: 7;

    REG_RDI     :: 8;

    REG_RSI     :: 9;

    REG_RBP     :: 10;

    REG_RBX     :: 11;

    REG_RDX     :: 12;

    REG_RAX     :: 13;

    REG_RCX     :: 14;

    REG_RSP     :: 15;

    REG_RIP     :: 16;

    REG_EFL     :: 17;

    REG_CSGSFS  :: 18;

    REG_ERR     :: 19;

    REG_TRAPNO  :: 20;

    REG_OLDMASK :: 21;

    REG_CR2     :: 22;

    NGREG       :: 23;
}
```
### `RLIMIT_RESOURCE`

Variant 1 (Linux):

```jai
RLIMIT_RESOURCE :: enum u32 {
    RLIMIT_CPU        :: 0;

    RLIMIT_FSIZE      :: 1;

    RLIMIT_DATA       :: 2;

    RLIMIT_STACK      :: 3;

    RLIMIT_CORE       :: 4;

    RLIMIT_RSS        :: 5;

    RLIMIT_NOFILE     :: 7;
    RLIMIT_OFILE      :: 7;

    RLIMIT_AS         :: 9;

    RLIMIT_NPROC      :: 6;

    RLIMIT_MEMLOCK    :: 8;

    RLIMIT_LOCKS      :: 10;

    RLIMIT_SIGPENDING :: 11;

    RLIMIT_MSGQUEUE   :: 12;

    RLIMIT_NICE       :: 13;

    RLIMIT_RTPRIO     :: 14;

    RLIMIT_RTTIME     :: 15;

    RLIMIT_NLIMITS    :: 16;
    RLIM_NLIMITS      :: 16;
}
```
Variant 2 (macOS, ARM64):

```jai
RLIMIT_RESOURCE :: enum s32 {
    CPU     :: 0;
    FSIZE   :: 1;
    DATA    :: 2;
    STACK   :: 3;
    CORE    :: 4;
    AS      :: 5;

    RSS     :: AS;
    MEMLOCK :: 6;
    NPROC   :: 7;

    NOFILE  :: 8;
}
```
Variant 3 (Android, ARM64):

```jai
RLIMIT_RESOURCE :: enum s32 {
    CPU        :: 0;
    FSIZE      :: 1;
    DATA       :: 2;
    STACK      :: 3;
    CORE       :: 4;

    RSS        :: 5;

    NPROC      :: 6;

    NOFILE     :: 7;

    MEMLOCK    :: 8;

    AS         :: 9;

    LOCKS      :: 10;
    SIGPENDING :: 11;
    MSGQUEUE   :: 12;
    NICE       :: 13;
    RTPRIO     :: 14;
    RTTIME     :: 15;
}
```
### `RTLD_DI`

```jai
RTLD_DI :: enum u32 {
    LMID        :: 1;

    LINKMAP     :: 2;

    CONFIGADDR  :: 3;

    SERINFO     :: 4;
    SERINFOSIZE :: 5;

    ORIGIN      :: 6;

    PROFILENAME :: 7;
    PROFILEOUT  :: 8;

    TLS_MODID   :: 9;

    TLS_DATA    :: 10;

    MAX         :: 10;

    RTLD_DI_LMID        :: LMID;

    RTLD_DI_LINKMAP     :: LINKMAP;

    RTLD_DI_CONFIGADDR  :: CONFIGADDR;

    RTLD_DI_SERINFO     :: SERINFO;
    RTLD_DI_SERINFOSIZE :: SERINFOSIZE;

    RTLD_DI_ORIGIN      :: ORIGIN;

    RTLD_DI_PROFILENAME :: PROFILENAME;
    RTLD_DI_PROFILEOUT  :: PROFILEOUT;

    RTLD_DI_TLS_MODID   :: TLS_MODID;

    RTLD_DI_TLS_DATA    :: TLS_DATA;

    RTLD_DI_MAX         :: MAX;
}
```
### `RTLD_DL`

```jai
RTLD_DL :: enum u32 {
    SYMENT  :: 1;

    LINKMAP :: 2;

    RTLD_DL_SYMENT  :: SYMENT;

    RTLD_DL_LINKMAP :: LINKMAP;
}
```
### `RUSAGE`

Variant 1 (Linux):

```jai
RUSAGE :: enum s32 {
    SELF     :: 0;

    CHILDREN :: -1;

    THREAD   :: 1;
}
```
Variant 2 (macOS, ARM64):

```jai
RUSAGE :: enum s32 {
    SELF     :: 0;
    CHILDREN :: -1;
}
```
Variant 3 (Android, ARM64):

```jai
RUSAGE :: enum s32 {
    SELF     :: 0;
    CHILDREN :: - 1;
    BOTH     :: - 2;
    THREAD   :: 1;
}
```
### `RUSAGE_INFO`

Variant 1 (macOS, ARM64):

```jai
RUSAGE_INFO :: enum {
    V0      :: 0;
    V1      :: 1;
    V2      :: 2;
    V3      :: 3;
    V4      :: 4;
    V5      :: 5;
    CURRENT :: V5;
}
```
Variant 2 (macOS, x64):

```jai
RUSAGE_INFO :: enum {
    V0      :: 0;
    V1      :: 1;
    V2      :: 2;
    V3      :: 3;
    V4      :: 4;
    CURRENT :: V4;
}
```
### `SFD`

```jai
SFD :: enum u32 {
    CLOEXEC  :: 524288;

    NONBLOCK :: 2048;

    SFD_CLOEXEC  :: CLOEXEC;

    SFD_NONBLOCK :: NONBLOCK;
}
```
### `TFD`

```jai
TFD :: enum u32 {
    CLOEXEC  :: 524288;

    NONBLOCK :: 2048;

    TFD_CLOEXEC  :: CLOEXEC;

    TFD_NONBLOCK :: NONBLOCK;
}
```
### `TFD_TIMER`

```jai
TFD_TIMER :: enum u32 {
    ABSTIME       :: 1;

    CANCEL_ON_SET :: 2;

    TFD_TIMER_ABSTIME       :: ABSTIME;

    TFD_TIMER_CANCEL_ON_SET :: CANCEL_ON_SET;
}
```
### `_G_fpos64_t`

```jai
_G_fpos64_t :: struct {
    __pos:   s64;
    __state: __mbstate_t;
}
```
### `_G_fpos_t`

```jai
_G_fpos_t :: struct {
    __pos:   s64;
    __state: __mbstate_t;
}
```
### `_IO_FILE`

```jai
_IO_FILE :: struct {}
```
### `_IO_cookie_io_functions_t`

```jai
_IO_cookie_io_functions_t :: struct {
    read:  cookie_read_function_t;
    write: cookie_write_function_t;
    seek:  cookie_seek_function_t;
    close: cookie_close_function_t;
}
```
### `_SC_definitions`

Variant 1 (Linux):

```jai
using _SC_definitions :: enum s32 {
    _SC_ARG_MAX                      :: 0;

    _SC_CHILD_MAX                    :: 1;

    _SC_CLK_TCK                      :: 2;

    _SC_NGROUPS_MAX                  :: 3;

    _SC_OPEN_MAX                     :: 4;

    _SC_STREAM_MAX                   :: 5;

    _SC_TZNAME_MAX                   :: 6;

    _SC_JOB_CONTROL                  :: 7;

    _SC_SAVED_IDS                    :: 8;

    _SC_REALTIME_SIGNALS             :: 9;

    _SC_PRIORITY_SCHEDULING          :: 10;

    _SC_TIMERS                       :: 11;

    _SC_ASYNCHRONOUS_IO              :: 12;

    _SC_PRIORITIZED_IO               :: 13;

    _SC_SYNCHRONIZED_IO              :: 14;

    _SC_FSYNC                        :: 15;

    _SC_MAPPED_FILES                 :: 16;

    _SC_MEMLOCK                      :: 17;

    _SC_MEMLOCK_RANGE                :: 18;

    _SC_MEMORY_PROTECTION            :: 19;

    _SC_MESSAGE_PASSING              :: 20;

    _SC_SEMAPHORES                   :: 21;

    _SC_SHARED_MEMORY_OBJECTS        :: 22;

    _SC_AIO_LISTIO_MAX               :: 23;

    _SC_AIO_MAX                      :: 24;

    _SC_AIO_PRIO_DELTA_MAX           :: 25;

    _SC_DELAYTIMER_MAX               :: 26;

    _SC_MQ_OPEN_MAX                  :: 27;

    _SC_MQ_PRIO_MAX                  :: 28;

    _SC_VERSION                      :: 29;

    _SC_PAGESIZE                     :: 30;

    _SC_RTSIG_MAX                    :: 31;

    _SC_SEM_NSEMS_MAX                :: 32;

    _SC_SEM_VALUE_MAX                :: 33;

    _SC_SIGQUEUE_MAX                 :: 34;

    _SC_TIMER_MAX                    :: 35;

    _SC_BC_BASE_MAX                  :: 36;

    _SC_BC_DIM_MAX                   :: 37;

    _SC_BC_SCALE_MAX                 :: 38;

    _SC_BC_STRING_MAX                :: 39;

    _SC_COLL_WEIGHTS_MAX             :: 40;

    _SC_EQUIV_CLASS_MAX              :: 41;

    _SC_EXPR_NEST_MAX                :: 42;

    _SC_LINE_MAX                     :: 43;

    _SC_RE_DUP_MAX                   :: 44;

    _SC_CHARCLASS_NAME_MAX           :: 45;

    _SC_2_VERSION                    :: 46;

    _SC_2_C_BIND                     :: 47;

    _SC_2_C_DEV                      :: 48;

    _SC_2_FORT_DEV                   :: 49;

    _SC_2_FORT_RUN                   :: 50;

    _SC_2_SW_DEV                     :: 51;

    _SC_2_LOCALEDEF                  :: 52;

    _SC_PII                          :: 53;

    _SC_PII_XTI                      :: 54;

    _SC_PII_SOCKET                   :: 55;

    _SC_PII_INTERNET                 :: 56;

    _SC_PII_OSI                      :: 57;

    _SC_POLL                         :: 58;

    _SC_SELECT                       :: 59;

    _SC_UIO_MAXIOV                   :: 60;

    _SC_IOV_MAX                      :: 60;

    _SC_PII_INTERNET_STREAM          :: 61;

    _SC_PII_INTERNET_DGRAM           :: 62;

    _SC_PII_OSI_COTS                 :: 63;

    _SC_PII_OSI_CLTS                 :: 64;

    _SC_PII_OSI_M                    :: 65;

    _SC_T_IOV_MAX                    :: 66;

    _SC_THREADS                      :: 67;

    _SC_THREAD_SAFE_FUNCTIONS        :: 68;

    _SC_GETGR_R_SIZE_MAX             :: 69;

    _SC_GETPW_R_SIZE_MAX             :: 70;

    _SC_LOGIN_NAME_MAX               :: 71;

    _SC_TTY_NAME_MAX                 :: 72;

    _SC_THREAD_DESTRUCTOR_ITERATIONS :: 73;

    _SC_THREAD_KEYS_MAX              :: 74;

    _SC_THREAD_STACK_MIN             :: 75;

    _SC_THREAD_THREADS_MAX           :: 76;

    _SC_THREAD_ATTR_STACKADDR        :: 77;

    _SC_THREAD_ATTR_STACKSIZE        :: 78;

    _SC_THREAD_PRIORITY_SCHEDULING   :: 79;

    _SC_THREAD_PRIO_INHERIT          :: 80;

    _SC_THREAD_PRIO_PROTECT          :: 81;

    _SC_THREAD_PROCESS_SHARED        :: 82;

    _SC_NPROCESSORS_CONF             :: 83;

    _SC_NPROCESSORS_ONLN             :: 84;

    _SC_PHYS_PAGES                   :: 85;

    _SC_AVPHYS_PAGES                 :: 86;

    _SC_ATEXIT_MAX                   :: 87;

    _SC_PASS_MAX                     :: 88;

    _SC_XOPEN_VERSION                :: 89;

    _SC_XOPEN_XCU_VERSION            :: 90;

    _SC_XOPEN_UNIX                   :: 91;

    _SC_XOPEN_CRYPT                  :: 92;

    _SC_XOPEN_ENH_I18N               :: 93;

    _SC_XOPEN_SHM                    :: 94;

    _SC_2_CHAR_TERM                  :: 95;

    _SC_2_C_VERSION                  :: 96;

    _SC_2_UPE                        :: 97;

    _SC_XOPEN_XPG2                   :: 98;

    _SC_XOPEN_XPG3                   :: 99;

    _SC_XOPEN_XPG4                   :: 100;

    _SC_CHAR_BIT                     :: 101;

    _SC_CHAR_MAX                     :: 102;

    _SC_CHAR_MIN                     :: 103;

    _SC_INT_MAX                      :: 104;

    _SC_INT_MIN                      :: 105;

    _SC_LONG_BIT                     :: 106;

    _SC_WORD_BIT                     :: 107;

    _SC_MB_LEN_MAX                   :: 108;

    _SC_NZERO                        :: 109;

    _SC_SSIZE_MAX                    :: 110;

    _SC_SCHAR_MAX                    :: 111;

    _SC_SCHAR_MIN                    :: 112;

    _SC_SHRT_MAX                     :: 113;

    _SC_SHRT_MIN                     :: 114;

    _SC_UCHAR_MAX                    :: 115;

    _SC_UINT_MAX                     :: 116;

    _SC_ULONG_MAX                    :: 117;

    _SC_USHRT_MAX                    :: 118;

    _SC_NL_ARGMAX                    :: 119;

    _SC_NL_LANGMAX                   :: 120;

    _SC_NL_MSGMAX                    :: 121;

    _SC_NL_NMAX                      :: 122;

    _SC_NL_SETMAX                    :: 123;

    _SC_NL_TEXTMAX                   :: 124;

    _SC_XBS5_ILP32_OFF32             :: 125;

    _SC_XBS5_ILP32_OFFBIG            :: 126;

    _SC_XBS5_LP64_OFF64              :: 127;

    _SC_XBS5_LPBIG_OFFBIG            :: 128;

    _SC_XOPEN_LEGACY                 :: 129;

    _SC_XOPEN_REALTIME               :: 130;

    _SC_XOPEN_REALTIME_THREADS       :: 131;

    _SC_ADVISORY_INFO                :: 132;

    _SC_BARRIERS                     :: 133;

    _SC_BASE                         :: 134;

    _SC_C_LANG_SUPPORT               :: 135;

    _SC_C_LANG_SUPPORT_R             :: 136;

    _SC_CLOCK_SELECTION              :: 137;

    _SC_CPUTIME                      :: 138;

    _SC_THREAD_CPUTIME               :: 139;

    _SC_DEVICE_IO                    :: 140;

    _SC_DEVICE_SPECIFIC              :: 141;

    _SC_DEVICE_SPECIFIC_R            :: 142;

    _SC_FD_MGMT                      :: 143;

    _SC_FIFO                         :: 144;

    _SC_PIPE                         :: 145;

    _SC_FILE_ATTRIBUTES              :: 146;

    _SC_FILE_LOCKING                 :: 147;

    _SC_FILE_SYSTEM                  :: 148;

    _SC_MONOTONIC_CLOCK              :: 149;

    _SC_MULTI_PROCESS                :: 150;

    _SC_SINGLE_PROCESS               :: 151;

    _SC_NETWORKING                   :: 152;

    _SC_READER_WRITER_LOCKS          :: 153;

    _SC_SPIN_LOCKS                   :: 154;

    _SC_REGEXP                       :: 155;

    _SC_REGEX_VERSION                :: 156;

    _SC_SHELL                        :: 157;

    _SC_SIGNALS                      :: 158;

    _SC_SPAWN                        :: 159;

    _SC_SPORADIC_SERVER              :: 160;

    _SC_THREAD_SPORADIC_SERVER       :: 161;

    _SC_SYSTEM_DATABASE              :: 162;

    _SC_SYSTEM_DATABASE_R            :: 163;

    _SC_TIMEOUTS                     :: 164;

    _SC_TYPED_MEMORY_OBJECTS         :: 165;

    _SC_USER_GROUPS                  :: 166;

    _SC_USER_GROUPS_R                :: 167;

    _SC_2_PBS                        :: 168;

    _SC_2_PBS_ACCOUNTING             :: 169;

    _SC_2_PBS_LOCATE                 :: 170;

    _SC_2_PBS_MESSAGE                :: 171;

    _SC_2_PBS_TRACK                  :: 172;

    _SC_SYMLOOP_MAX                  :: 173;

    _SC_STREAMS                      :: 174;

    _SC_2_PBS_CHECKPOINT             :: 175;

    _SC_V6_ILP32_OFF32               :: 176;

    _SC_V6_ILP32_OFFBIG              :: 177;

    _SC_V6_LP64_OFF64                :: 178;

    _SC_V6_LPBIG_OFFBIG              :: 179;

    _SC_HOST_NAME_MAX                :: 180;

    _SC_TRACE                        :: 181;

    _SC_TRACE_EVENT_FILTER           :: 182;

    _SC_TRACE_INHERIT                :: 183;

    _SC_TRACE_LOG                    :: 184;

    _SC_LEVEL1_ICACHE_SIZE           :: 185;

    _SC_LEVEL1_ICACHE_ASSOC          :: 186;

    _SC_LEVEL1_ICACHE_LINESIZE       :: 187;

    _SC_LEVEL1_DCACHE_SIZE           :: 188;

    _SC_LEVEL1_DCACHE_ASSOC          :: 189;

    _SC_LEVEL1_DCACHE_LINESIZE       :: 190;

    _SC_LEVEL2_CACHE_SIZE            :: 191;

    _SC_LEVEL2_CACHE_ASSOC           :: 192;

    _SC_LEVEL2_CACHE_LINESIZE        :: 193;

    _SC_LEVEL3_CACHE_SIZE            :: 194;

    _SC_LEVEL3_CACHE_ASSOC           :: 195;

    _SC_LEVEL3_CACHE_LINESIZE        :: 196;

    _SC_LEVEL4_CACHE_SIZE            :: 197;

    _SC_LEVEL4_CACHE_ASSOC           :: 198;

    _SC_LEVEL4_CACHE_LINESIZE        :: 199;

    _SC_IPV6                         :: 235;

    _SC_RAW_SOCKETS                  :: 236;

    _SC_V7_ILP32_OFF32               :: 237;

    _SC_V7_ILP32_OFFBIG              :: 238;

    _SC_V7_LP64_OFF64                :: 239;

    _SC_V7_LPBIG_OFFBIG              :: 240;

    _SC_SS_REPL_MAX                  :: 241;

    _SC_TRACE_EVENT_NAME_MAX         :: 242;

    _SC_TRACE_NAME_MAX               :: 243;

    _SC_TRACE_SYS_MAX                :: 244;

    _SC_TRACE_USER_EVENT_MAX         :: 245;

    _SC_XOPEN_STREAMS                :: 246;

    _SC_THREAD_ROBUST_PRIO_INHERIT   :: 247;

    _SC_THREAD_ROBUST_PRIO_PROTECT   :: 248;
}
```
Variant 2 (macOS, ARM64):

```jai
using _SC_definitions :: enum s32 {
    _SC_ARG_MAX                      :: 1;
    _SC_CHILD_MAX                    :: 2;
    _SC_CLK_TCK                      :: 3;
    _SC_NGROUPS_MAX                  :: 4;
    _SC_OPEN_MAX                     :: 5;
    _SC_JOB_CONTROL                  :: 6;
    _SC_SAVED_IDS                    :: 7;
    _SC_VERSION                      :: 8;
    _SC_BC_BASE_MAX                  :: 9;
    _SC_BC_DIM_MAX                   :: 10;
    _SC_BC_SCALE_MAX                 :: 11;
    _SC_BC_STRING_MAX                :: 12;
    _SC_COLL_WEIGHTS_MAX             :: 13;
    _SC_EXPR_NEST_MAX                :: 14;
    _SC_LINE_MAX                     :: 15;
    _SC_RE_DUP_MAX                   :: 16;
    _SC_2_VERSION                    :: 17;
    _SC_2_C_BIND                     :: 18;
    _SC_2_C_DEV                      :: 19;
    _SC_2_CHAR_TERM                  :: 20;
    _SC_2_FORT_DEV                   :: 21;
    _SC_2_FORT_RUN                   :: 22;
    _SC_2_LOCALEDEF                  :: 23;
    _SC_2_SW_DEV                     :: 24;
    _SC_2_UPE                        :: 25;
    _SC_STREAM_MAX                   :: 26;
    _SC_TZNAME_MAX                   :: 27;

    _SC_ASYNCHRONOUS_IO              :: 28;
    _SC_PAGESIZE                     :: 29;
    _SC_MEMLOCK                      :: 30;
    _SC_MEMLOCK_RANGE                :: 31;
    _SC_MEMORY_PROTECTION            :: 32;
    _SC_MESSAGE_PASSING              :: 33;
    _SC_PRIORITIZED_IO               :: 34;
    _SC_PRIORITY_SCHEDULING          :: 35;
    _SC_REALTIME_SIGNALS             :: 36;
    _SC_SEMAPHORES                   :: 37;
    _SC_FSYNC                        :: 38;
    _SC_SHARED_MEMORY_OBJECTS        :: 39;
    _SC_SYNCHRONIZED_IO              :: 40;
    _SC_TIMERS                       :: 41;
    _SC_AIO_LISTIO_MAX               :: 42;
    _SC_AIO_MAX                      :: 43;
    _SC_AIO_PRIO_DELTA_MAX           :: 44;
    _SC_DELAYTIMER_MAX               :: 45;
    _SC_MQ_OPEN_MAX                  :: 46;
    _SC_MAPPED_FILES                 :: 47;
    _SC_RTSIG_MAX                    :: 48;
    _SC_SEM_NSEMS_MAX                :: 49;
    _SC_SEM_VALUE_MAX                :: 50;
    _SC_SIGQUEUE_MAX                 :: 51;
    _SC_TIMER_MAX                    :: 52;

    _SC_NPROCESSORS_CONF             :: 57;
    _SC_NPROCESSORS_ONLN             :: 58;

    _SC_2_PBS                        :: 59;
    _SC_2_PBS_ACCOUNTING             :: 60;
    _SC_2_PBS_CHECKPOINT             :: 61;
    _SC_2_PBS_LOCATE                 :: 62;
    _SC_2_PBS_MESSAGE                :: 63;
    _SC_2_PBS_TRACK                  :: 64;
    _SC_ADVISORY_INFO                :: 65;
    _SC_BARRIERS                     :: 66;
    _SC_CLOCK_SELECTION              :: 67;
    _SC_CPUTIME                      :: 68;
    _SC_FILE_LOCKING                 :: 69;
    _SC_GETGR_R_SIZE_MAX             :: 70;
    _SC_GETPW_R_SIZE_MAX             :: 71;
    _SC_HOST_NAME_MAX                :: 72;
    _SC_LOGIN_NAME_MAX               :: 73;
    _SC_MONOTONIC_CLOCK              :: 74;
    _SC_MQ_PRIO_MAX                  :: 75;
    _SC_READER_WRITER_LOCKS          :: 76;
    _SC_REGEXP                       :: 77;
    _SC_SHELL                        :: 78;
    _SC_SPAWN                        :: 79;
    _SC_SPIN_LOCKS                   :: 80;
    _SC_SPORADIC_SERVER              :: 81;
    _SC_THREAD_ATTR_STACKADDR        :: 82;
    _SC_THREAD_ATTR_STACKSIZE        :: 83;
    _SC_THREAD_CPUTIME               :: 84;
    _SC_THREAD_DESTRUCTOR_ITERATIONS :: 85;
    _SC_THREAD_KEYS_MAX              :: 86;
    _SC_THREAD_PRIO_INHERIT          :: 87;
    _SC_THREAD_PRIO_PROTECT          :: 88;
    _SC_THREAD_PRIORITY_SCHEDULING   :: 89;
    _SC_THREAD_PROCESS_SHARED        :: 90;
    _SC_THREAD_SAFE_FUNCTIONS        :: 91;
    _SC_THREAD_SPORADIC_SERVER       :: 92;
    _SC_THREAD_STACK_MIN             :: 93;
    _SC_THREAD_THREADS_MAX           :: 94;
    _SC_TIMEOUTS                     :: 95;
    _SC_THREADS                      :: 96;
    _SC_TRACE                        :: 97;
    _SC_TRACE_EVENT_FILTER           :: 98;
    _SC_TRACE_INHERIT                :: 99;
    _SC_TRACE_LOG                    :: 100;
    _SC_TTY_NAME_MAX                 :: 101;
    _SC_TYPED_MEMORY_OBJECTS         :: 102;
    _SC_V6_ILP32_OFF32               :: 103;
    _SC_V6_ILP32_OFFBIG              :: 104;
    _SC_V6_LP64_OFF64                :: 105;
    _SC_V6_LPBIG_OFFBIG              :: 106;
    _SC_IPV6                         :: 118;
    _SC_RAW_SOCKETS                  :: 119;
    _SC_SYMLOOP_MAX                  :: 120;

    _SC_ATEXIT_MAX                   :: 107;
    _SC_IOV_MAX                      :: 56;
    _SC_PAGE_SIZE                    :: _SC_PAGESIZE;
    _SC_XOPEN_CRYPT                  :: 108;
    _SC_XOPEN_ENH_I18N               :: 109;
    _SC_XOPEN_LEGACY                 :: 110;
    _SC_XOPEN_REALTIME               :: 111;
    _SC_XOPEN_REALTIME_THREADS       :: 112;
    _SC_XOPEN_SHM                    :: 113;
    _SC_XOPEN_STREAMS                :: 114;
    _SC_XOPEN_UNIX                   :: 115;
    _SC_XOPEN_VERSION                :: 116;
    _SC_XOPEN_XCU_VERSION            :: 121;

    _SC_XBS5_ILP32_OFF32             :: 122;
    _SC_XBS5_ILP32_OFFBIG            :: 123;
    _SC_XBS5_LP64_OFF64              :: 124;
    _SC_XBS5_LPBIG_OFFBIG            :: 125;

    _SC_SS_REPL_MAX                  :: 126;
    _SC_TRACE_EVENT_NAME_MAX         :: 127;
    _SC_TRACE_NAME_MAX               :: 128;
    _SC_TRACE_SYS_MAX                :: 129;
    _SC_TRACE_USER_EVENT_MAX         :: 130;

    _SC_PASS_MAX                     :: 131;

    _SC_PHYS_PAGES                   :: 200;
}
```
Variant 3 (Android, ARM64):

```jai
using _SC_definitions :: enum s32 {
    _SC_ARG_MAX                      :: 0x0000;
    _SC_BC_BASE_MAX                  :: 0x0001;
    _SC_BC_DIM_MAX                   :: 0x0002;
    _SC_BC_SCALE_MAX                 :: 0x0003;
    _SC_BC_STRING_MAX                :: 0x0004;
    _SC_CHILD_MAX                    :: 0x0005;
    _SC_CLK_TCK                      :: 0x0006;
    _SC_COLL_WEIGHTS_MAX             :: 0x0007;
    _SC_EXPR_NEST_MAX                :: 0x0008;
    _SC_LINE_MAX                     :: 0x0009;
    _SC_NGROUPS_MAX                  :: 0x000a;
    _SC_OPEN_MAX                     :: 0x000b;
    _SC_PASS_MAX                     :: 0x000c;
    _SC_2_C_BIND                     :: 0x000d;
    _SC_2_C_DEV                      :: 0x000e;
    _SC_2_C_VERSION                  :: 0x000f;
    _SC_2_CHAR_TERM                  :: 0x0010;
    _SC_2_FORT_DEV                   :: 0x0011;
    _SC_2_FORT_RUN                   :: 0x0012;
    _SC_2_LOCALEDEF                  :: 0x0013;
    _SC_2_SW_DEV                     :: 0x0014;
    _SC_2_UPE                        :: 0x0015;
    _SC_2_VERSION                    :: 0x0016;
    _SC_JOB_CONTROL                  :: 0x0017;
    _SC_SAVED_IDS                    :: 0x0018;
    _SC_VERSION                      :: 0x0019;
    _SC_RE_DUP_MAX                   :: 0x001a;
    _SC_STREAM_MAX                   :: 0x001b;
    _SC_TZNAME_MAX                   :: 0x001c;
    _SC_XOPEN_CRYPT                  :: 0x001d;
    _SC_XOPEN_ENH_I18N               :: 0x001e;
    _SC_XOPEN_SHM                    :: 0x001f;
    _SC_XOPEN_VERSION                :: 0x0020;
    _SC_XOPEN_XCU_VERSION            :: 0x0021;
    _SC_XOPEN_REALTIME               :: 0x0022;
    _SC_XOPEN_REALTIME_THREADS       :: 0x0023;
    _SC_XOPEN_LEGACY                 :: 0x0024;
    _SC_ATEXIT_MAX                   :: 0x0025;
    _SC_IOV_MAX                      :: 0x0026;
    _SC_UIO_MAXIOV                   :: _SC_IOV_MAX;
    _SC_PAGESIZE                     :: 0x0027;
    _SC_PAGE_SIZE                    :: 0x0028;
    _SC_XOPEN_UNIX                   :: 0x0029;
    _SC_XBS5_ILP32_OFF32             :: 0x002a;
    _SC_XBS5_ILP32_OFFBIG            :: 0x002b;
    _SC_XBS5_LP64_OFF64              :: 0x002c;
    _SC_XBS5_LPBIG_OFFBIG            :: 0x002d;
    _SC_AIO_LISTIO_MAX               :: 0x002e;
    _SC_AIO_MAX                      :: 0x002f;
    _SC_AIO_PRIO_DELTA_MAX           :: 0x0030;
    _SC_DELAYTIMER_MAX               :: 0x0031;
    _SC_MQ_OPEN_MAX                  :: 0x0032;
    _SC_MQ_PRIO_MAX                  :: 0x0033;
    _SC_RTSIG_MAX                    :: 0x0034;
    _SC_SEM_NSEMS_MAX                :: 0x0035;
    _SC_SEM_VALUE_MAX                :: 0x0036;
    _SC_SIGQUEUE_MAX                 :: 0x0037;
    _SC_TIMER_MAX                    :: 0x0038;
    _SC_ASYNCHRONOUS_IO              :: 0x0039;
    _SC_FSYNC                        :: 0x003a;
    _SC_MAPPED_FILES                 :: 0x003b;
    _SC_MEMLOCK                      :: 0x003c;
    _SC_MEMLOCK_RANGE                :: 0x003d;
    _SC_MEMORY_PROTECTION            :: 0x003e;
    _SC_MESSAGE_PASSING              :: 0x003f;
    _SC_PRIORITIZED_IO               :: 0x0040;
    _SC_PRIORITY_SCHEDULING          :: 0x0041;
    _SC_REALTIME_SIGNALS             :: 0x0042;
    _SC_SEMAPHORES                   :: 0x0043;
    _SC_SHARED_MEMORY_OBJECTS        :: 0x0044;
    _SC_SYNCHRONIZED_IO              :: 0x0045;
    _SC_TIMERS                       :: 0x0046;
    _SC_GETGR_R_SIZE_MAX             :: 0x0047;
    _SC_GETPW_R_SIZE_MAX             :: 0x0048;
    _SC_LOGIN_NAME_MAX               :: 0x0049;
    _SC_THREAD_DESTRUCTOR_ITERATIONS :: 0x004a;
    _SC_THREAD_KEYS_MAX              :: 0x004b;
    _SC_THREAD_STACK_MIN             :: 0x004c;
    _SC_THREAD_THREADS_MAX           :: 0x004d;
    _SC_TTY_NAME_MAX                 :: 0x004e;

    _SC_THREADS                      :: 0x004f;
    _SC_THREAD_ATTR_STACKADDR        :: 0x0050;
    _SC_THREAD_ATTR_STACKSIZE        :: 0x0051;
    _SC_THREAD_PRIORITY_SCHEDULING   :: 0x0052;
    _SC_THREAD_PRIO_INHERIT          :: 0x0053;
    _SC_THREAD_PRIO_PROTECT          :: 0x0054;
    _SC_THREAD_SAFE_FUNCTIONS        :: 0x0055;

    _SC_NPROCESSORS_CONF             :: 0x0060;
    _SC_NPROCESSORS_ONLN             :: 0x0061;
    _SC_PHYS_PAGES                   :: 0x0062;
    _SC_AVPHYS_PAGES                 :: 0x0063;
    _SC_MONOTONIC_CLOCK              :: 0x0064;

    _SC_2_PBS                        :: 0x0065;
    _SC_2_PBS_ACCOUNTING             :: 0x0066;
    _SC_2_PBS_CHECKPOINT             :: 0x0067;
    _SC_2_PBS_LOCATE                 :: 0x0068;
    _SC_2_PBS_MESSAGE                :: 0x0069;
    _SC_2_PBS_TRACK                  :: 0x006a;
    _SC_ADVISORY_INFO                :: 0x006b;
    _SC_BARRIERS                     :: 0x006c;
    _SC_CLOCK_SELECTION              :: 0x006d;
    _SC_CPUTIME                      :: 0x006e;
    _SC_HOST_NAME_MAX                :: 0x006f;
    _SC_IPV6                         :: 0x0070;
    _SC_RAW_SOCKETS                  :: 0x0071;
    _SC_READER_WRITER_LOCKS          :: 0x0072;
    _SC_REGEXP                       :: 0x0073;
    _SC_SHELL                        :: 0x0074;
    _SC_SPAWN                        :: 0x0075;
    _SC_SPIN_LOCKS                   :: 0x0076;
    _SC_SPORADIC_SERVER              :: 0x0077;
    _SC_SS_REPL_MAX                  :: 0x0078;
    _SC_SYMLOOP_MAX                  :: 0x0079;
    _SC_THREAD_CPUTIME               :: 0x007a;
    _SC_THREAD_PROCESS_SHARED        :: 0x007b;
    _SC_THREAD_ROBUST_PRIO_INHERIT   :: 0x007c;
    _SC_THREAD_ROBUST_PRIO_PROTECT   :: 0x007d;
    _SC_THREAD_SPORADIC_SERVER       :: 0x007e;
    _SC_TIMEOUTS                     :: 0x007f;
    _SC_TRACE                        :: 0x0080;
    _SC_TRACE_EVENT_FILTER           :: 0x0081;
    _SC_TRACE_EVENT_NAME_MAX         :: 0x0082;
    _SC_TRACE_INHERIT                :: 0x0083;
    _SC_TRACE_LOG                    :: 0x0084;
    _SC_TRACE_NAME_MAX               :: 0x0085;
    _SC_TRACE_SYS_MAX                :: 0x0086;
    _SC_TRACE_USER_EVENT_MAX         :: 0x0087;
    _SC_TYPED_MEMORY_OBJECTS         :: 0x0088;
    _SC_V7_ILP32_OFF32               :: 0x0089;
    _SC_V7_ILP32_OFFBIG              :: 0x008a;
    _SC_V7_LP64_OFF64                :: 0x008b;
    _SC_V7_LPBIG_OFFBIG              :: 0x008c;
    _SC_XOPEN_STREAMS                :: 0x008d;
    _SC_XOPEN_UUCP                   :: 0x008e;

    _SC_LEVEL1_ICACHE_SIZE           :: 0x008f;
    _SC_LEVEL1_ICACHE_ASSOC          :: 0x0090;
    _SC_LEVEL1_ICACHE_LINESIZE       :: 0x0091;
    _SC_LEVEL1_DCACHE_SIZE           :: 0x0092;
    _SC_LEVEL1_DCACHE_ASSOC          :: 0x0093;
    _SC_LEVEL1_DCACHE_LINESIZE       :: 0x0094;
    _SC_LEVEL2_CACHE_SIZE            :: 0x0095;
    _SC_LEVEL2_CACHE_ASSOC           :: 0x0096;
    _SC_LEVEL2_CACHE_LINESIZE        :: 0x0097;
    _SC_LEVEL3_CACHE_SIZE            :: 0x0098;
    _SC_LEVEL3_CACHE_ASSOC           :: 0x0099;
    _SC_LEVEL3_CACHE_LINESIZE        :: 0x009a;
    _SC_LEVEL4_CACHE_SIZE            :: 0x009b;
    _SC_LEVEL4_CACHE_ASSOC           :: 0x009c;
    _SC_LEVEL4_CACHE_LINESIZE        :: 0x009d;
}
```
### `__arm_legacy_debug_state`

```jai
__arm_legacy_debug_state :: struct {
    __bvr: [16] u32;
    __bcr: [16] u32;
    __wvr: [16] u32;
    __wcr: [16] u32;
}
```
### `__arm_pagein_state`

```jai
__arm_pagein_state :: struct {
    __pagein_error: s32;
}
```
### `__darwin_arm_amx_state_v1`

```jai
__darwin_arm_amx_state_v1 :: struct {
    __x:               [8] [64] u8;
    __y:               [8] [64] u8;
    __z:               [64] [64] u8;
    __amx_state_t_el1: u64;

    _:                 [0] u8 #align 64;
}
```
### `__darwin_arm_cpmu_state64`

```jai
__darwin_arm_cpmu_state64 :: struct {
    __ctrs: [16] u64;
}
```
### `__darwin_arm_debug_state32`

```jai
__darwin_arm_debug_state32 :: struct {
    __bvr:       [16] u32;
    __bcr:       [16] u32;
    __wvr:       [16] u32;
    __wcr:       [16] u32;
    __mdscr_el1: u64;
}
```
### `__darwin_arm_debug_state64`

```jai
__darwin_arm_debug_state64 :: struct {
    __bvr:       [16] u64;
    __bcr:       [16] u64;
    __wvr:       [16] u64;
    __wcr:       [16] u64;
    __mdscr_el1: u64;
}
```
### `__darwin_arm_exception_state`

```jai
__darwin_arm_exception_state :: struct {
    __exception: u32;
    __fsr:       u32;
    __far:       u32;
}
```
### `__darwin_arm_exception_state64`

```jai
__darwin_arm_exception_state64 :: struct {
    __far:       u64;
    __esr:       u32;
    __exception: u32;
}
```
### `__darwin_arm_neon_state`

```jai
__darwin_arm_neon_state :: struct {
    __v:    [16] U128 #align 16;
    __fpsr: u32;
    __fpcr: u32;
}
```
### `__darwin_arm_neon_state64`

```jai
__darwin_arm_neon_state64 :: struct {
    __v:    [32] U128 #align 16;
    __fpsr: u32;
    __fpcr: u32;
}
```
### `__darwin_arm_thread_state`

```jai
__darwin_arm_thread_state :: struct {
    __r:    [13] u32;
    __sp:   u32;
    __lr:   u32;
    __pc:   u32;
    __cpsr: u32;
}
```
### `__darwin_arm_thread_state64`

```jai
__darwin_arm_thread_state64 :: struct {
    __x:    [29] u64;
    __fp:   u64;
    __lr:   u64;
    __sp:   u64;
    __pc:   u64;
    __cpsr: u32;
    __pad:  u32;
}
```
### `__darwin_arm_vfp_state`

```jai
__darwin_arm_vfp_state :: struct {
    __r:     [64] u32;
    __fpscr: u32;
}
```
### `__darwin_fp_control`

```jai
__darwin_fp_control :: struct {
    __bitfield: u16;

}
```
### `__darwin_fp_status`

```jai
__darwin_fp_status :: struct {
    __bitfield: u16;

}
```
### `__darwin_mcontext64`

Variant 1 (macOS, ARM64):

```jai
__darwin_mcontext64 :: struct {
    __es: __darwin_arm_exception_state64;
    __ss: __darwin_arm_thread_state64;
    __ns: __darwin_arm_neon_state64;
}
```
Variant 2 (macOS, x64):

```jai
__darwin_mcontext64 :: struct {
    __es: __darwin_x86_exception_state64;
    __ss: __darwin_x86_thread_state64;
    __fs: __darwin_x86_float_state64;
}
```
### `__darwin_mmst_reg`

```jai
__darwin_mmst_reg :: struct {
    __mmst_reg:  [10] u8;
    __mmst_rsrv: [6] u8;
}
```
### `__darwin_pthread_handler_rec`

```jai
__darwin_pthread_handler_rec :: struct {
    __routine: #type (unknown0: *void) -> void #c_call;
    __arg:     *void;
    __next:    *__darwin_pthread_handler_rec;
}
```
### `__darwin_x86_exception_state64`

```jai
__darwin_x86_exception_state64 :: struct {
    __trapno:     u16;
    __cpu:        u16;
    __err:        u32;
    __faultvaddr: u64;
}
```
### `__darwin_x86_float_state64`

```jai
__darwin_x86_float_state64 :: struct {
    __fpu_reserved:  [2] s32;
    __fpu_fcw:       __darwin_fp_control;
    __fpu_fsw:       __darwin_fp_status;
    __fpu_ftw:       u8;
    __fpu_rsrv1:     u8;
    __fpu_fop:       u16;

    __fpu_ip:        u32;
    __fpu_cs:        u16;

    __fpu_rsrv2:     u16;

    __fpu_dp:        u32;
    __fpu_ds:        u16;

    __fpu_rsrv3:     u16;
    __fpu_mxcsr:     u32;
    __fpu_mxcsrmask: u32;
    __fpu_stmm0:     __darwin_mmst_reg;
    __fpu_stmm1:     __darwin_mmst_reg;
    __fpu_stmm2:     __darwin_mmst_reg;
    __fpu_stmm3:     __darwin_mmst_reg;
    __fpu_stmm4:     __darwin_mmst_reg;
    __fpu_stmm5:     __darwin_mmst_reg;
    __fpu_stmm6:     __darwin_mmst_reg;
    __fpu_stmm7:     __darwin_mmst_reg;
    __fpu_xmm0:      __darwin_xmm_reg;
    __fpu_xmm1:      __darwin_xmm_reg;
    __fpu_xmm2:      __darwin_xmm_reg;
    __fpu_xmm3:      __darwin_xmm_reg;
    __fpu_xmm4:      __darwin_xmm_reg;
    __fpu_xmm5:      __darwin_xmm_reg;
    __fpu_xmm6:      __darwin_xmm_reg;
    __fpu_xmm7:      __darwin_xmm_reg;
    __fpu_xmm8:      __darwin_xmm_reg;
    __fpu_xmm9:      __darwin_xmm_reg;
    __fpu_xmm10:     __darwin_xmm_reg;
    __fpu_xmm11:     __darwin_xmm_reg;
    __fpu_xmm12:     __darwin_xmm_reg;
    __fpu_xmm13:     __darwin_xmm_reg;
    __fpu_xmm14:     __darwin_xmm_reg;
    __fpu_xmm15:     __darwin_xmm_reg;
    __fpu_rsrv4:     [96] u8;
    __fpu_reserved1: s32;
}
```
### `__darwin_x86_thread_state64`

```jai
__darwin_x86_thread_state64 :: struct {
    __rax:    u64;
    __rbx:    u64;
    __rcx:    u64;
    __rdx:    u64;
    __rdi:    u64;
    __rsi:    u64;
    __rbp:    u64;
    __rsp:    u64;
    __r8:     u64;
    __r9:     u64;
    __r10:    u64;
    __r11:    u64;
    __r12:    u64;
    __r13:    u64;
    __r14:    u64;
    __r15:    u64;
    __rip:    u64;
    __rflags: u64;
    __cs:     u64;
    __fs:     u64;
    __gs:     u64;
}
```
### `__darwin_xmm_reg`

```jai
__darwin_xmm_reg :: struct {
    __xmm_reg: [16] u8;
}
```
### `__dirstream`

```jai
__dirstream :: struct {}
```
### `__fsid_t`

```jai
__fsid_t :: struct {
    __val: [2] s32;
}
```
### `__itimer_which`

```jai
__itimer_which :: enum u32 {
    REAL    :: 0;

    VIRTUAL :: 1;

    PROF    :: 2;

    ITIMER_REAL    :: REAL;

    ITIMER_VIRTUAL :: VIRTUAL;

    ITIMER_PROF    :: PROF;
}
```
### `__jmp_buf_tag`

```jai
__jmp_buf_tag :: struct {
    __jmpbuf:         __jmp_buf;
    __mask_was_saved: s32;
    __saved_mask:     __sigset_t;
}
```
### `__kernel_fd_set`

```jai
__kernel_fd_set :: struct {
    fds_bits: [16] u64;
}
```
### `__kernel_fsid_t`

```jai
__kernel_fsid_t :: struct {
    val: [2] s32;
}
```
### `__kernel_sigaction`

Variant 1 (Android, ARM64):

```jai
__kernel_sigaction :: struct {
    sa_handler:  #type (unknown0: s32) -> void #c_call;
    sa_flags:    u64;

    sa_restorer: #type () -> void #c_call;

    sa_mask:     sigset_t;
}
```
Variant 2 (Android, x64):

```jai
__kernel_sigaction :: struct {
    sa_handler:  #type (unknown0: s32) -> void #c_call;
    sa_flags:    u64;
    sa_restorer: #type () -> void #c_call;
    sa_mask:     sigset_t;
}
```
### `__locale_struct`

```jai
__locale_struct :: struct {
    __locale_data :: struct {}
    __locales:       [13] *__locale_data;

    __ctype_b:       *u16;
    __ctype_tolower: *s32;
    __ctype_toupper: *s32;

    __names:         [13] *u8;
}
```
### `__locale_t`

```jai
__locale_t :: struct {}
```
### `__mbstate_t`

```jai
__mbstate_t :: struct {
    __count: s32;
    __value: struct {
        __wch: u32;

        #overlay (__wch) __wchb: [4] u8;
    };
}
```
### `__pid_type`

```jai
__pid_type :: enum u32 {
    TID  :: 0;
    PID  :: 1;
    PGRP :: 2;
    GID  :: 2;

    F_OWNER_TID  :: TID;
    F_OWNER_PID  :: PID;
    F_OWNER_PGRP :: PGRP;
    F_OWNER_GID  :: GID;
}
```
### `__pthread_cleanup_frame`

```jai
__pthread_cleanup_frame :: struct {
    __cancel_routine: #type (unknown0: *void) -> void #c_call;
    __cancel_arg:     *void;
    __do_it:          s32;
    __cancel_type:    s32;
}
```
### `__pthread_cleanup_t`

```jai
__pthread_cleanup_t :: struct {
    __cleanup_prev:    *__pthread_cleanup_t;
    __cleanup_routine: #type () -> void #c_call;
    __cleanup_arg:     *void;
}
```
### `__pthread_cond_s`

```jai
__pthread_cond_s :: struct {
    struct {
        __wseq: u64;

        #overlay (__wseq) __wseq32: struct {
            __low:  u32;
            __high: u32;
        };
    }

    struct {
        __g1_start: u64;

        #overlay (__g1_start) __g1_start32: struct {
            __low:  u32;
            __high: u32;
        };
    }

    __g_refs:       [2] u32;
    __g_size:       [2] u32;
    __g1_orig_size: u32;
    __wrefs:        u32;
    __g_signals:    [2] u32;
}
```
### `__pthread_internal_list`

```jai
__pthread_internal_list :: struct {
    __prev: *__pthread_internal_list;
    __next: *__pthread_internal_list;
}
```
### `__pthread_internal_slist`

```jai
__pthread_internal_slist :: struct {
    __next: *__pthread_internal_slist;
}
```
### `__pthread_mutex_s`

```jai
__pthread_mutex_s :: struct {
    __lock:    s32;
    __count:   u32;
    __owner:   s32;

    __nusers:  u32;

    __kind:    s32;

    __spins:   s16;
    __elision: s16;
    __list:    __pthread_internal_list;
}
```
### `__pthread_rwlock_arch_t`

```jai
__pthread_rwlock_arch_t :: struct {
    __readers:       u32;
    __writers:       u32;
    __wrphase_futex: u32;
    __writers_futex: u32;
    __pad3:          u32;
    __pad4:          u32;

    __cur_writer:    s32;
    __shared:        s32;
    __rwelision:     s8;

    __pad1:          [7] u8;

    __pad2:          u64;

    __flags:         u32;
}
```
### `__pthread_unwind_buf_t`

```jai
__pthread_unwind_buf_t :: struct {
    __cancel_jmp_buf: [1] struct {
        __cancel_jmp_buf: __jmp_buf;
        __mask_was_saved: s32;
    };

    __pad:            [4] *void;
}
```
### `__sFILE`

```jai
__sFILE :: struct {}
```
### `__shmid_ds_new`

```jai
__shmid_ds_new :: struct {
    shm_perm:     ipc_perm;
    shm_segsz:    u64 #align 4;
    shm_lpid:     pid_t;
    shm_cpid:     pid_t;
    shm_nattch:   shmatt_t;
    shm_atime:    time_t #align 4;
    shm_dtime:    time_t #align 4;
    shm_ctime:    time_t #align 4;
    shm_internal: *void #align 4;
}
```
### `__sifields`

```jai
__sifields :: struct {
    _sigchld: struct {
        _pid:    s32;
        _uid:    u32;
        _status: s32;
        _utime:  s64;
        _stime:  s64;
    };

    #overlay (_sigchld) _kill:     struct {
        _pid: s32;
        _uid: u32;
    };
    #overlay (_sigchld) _timer:    struct {
        _tid:         s32;
        _overrun:     s32;
        _sigval:      sigval_t;
        _sys_private: s32;
    };
    #overlay (_sigchld) _rt:       struct {
        _pid:    s32;
        _uid:    u32;
        _sigval: sigval_t;
    };
    #overlay (_sigchld) _sigfault: struct {
        _addr: *void;

        struct {
            _addr_bnd: struct {
                _dummy_bnd: [8] u8;
                _lower:     *void;
                _upper:     *void;
            };

            #overlay (_addr_bnd) _trapno:    s32;
            #overlay (_addr_bnd) _addr_lsb:  s16;
            #overlay (_addr_bnd) _addr_pkey: struct {
                _dummy_pkey: [8] u8;
                _pkey:       u32;
            };
            #overlay (_addr_bnd) _perf:      struct {
                _data:  u64;
                _type:  u32;
                _flags: u32;
            };
        }
    };
    #overlay (_sigchld) _sigpoll:  struct {
        _band: s64;
        _fd:   s32;
    };
    #overlay (_sigchld) _sigsys:   struct {
        _call_addr: *void;
        _syscall:   s32;
        _arch:      u32;
    };
}
```
### `__sigaction`

```jai
__sigaction :: struct {
    using __sigaction_u_: __sigaction_u;
    sa_tramp:             #type (unknown0: *void, unknown1: s32, unknown2: s32, unknown3: *siginfo_t, unknown4: *void) -> void #c_call;
    sa_mask:              sigset_t;
    sa_flags:             s32;
}
```
### `__sigaction_u`

```jai
__sigaction_u :: struct {
    sa_handler: #type (unknown0: s32) -> void #c_call;

    #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *__siginfo, unknown2: *void) -> void #c_call;
}
```
### `__siginfo`

```jai
__siginfo :: struct {
    si_signo:  s32;
    si_errno:  s32;
    si_code:   s32;
    si_pid:    pid_t;
    si_uid:    uid_t;
    si_status: s32;
    si_addr:   *void;
    si_value:  sigval;
    si_band:   s64;
    __pad:     [7] u64;
}
```
### `__sigset_t`

```jai
__sigset_t :: struct {
    __val: [16] u64;
}
```
### `_filesec`

```jai
_filesec :: struct {}
```
### `_fpstate`

```jai
_fpstate :: struct {
    cwd:               u16;
    swd:               u16;
    ftw:               u16;
    fop:               u16;
    rip:               u64;
    rdp:               u64;
    mxcsr:             u32;
    mxcr_mask:         u32;
    _st:               [8] _fpxreg;
    _xmm:              [16] _xmmreg;
    __glibc_reserved1: [24] u32;
}
```
### `_fpstate_64`

```jai
_fpstate_64 :: struct {
    cwd:        u16;
    swd:        u16;
    twd:        u16;
    fop:        u16;
    rip:        u64;
    rdp:        u64;
    mxcsr:      u32;
    mxcsr_mask: u32;
    st_space:   [32] u32;
    xmm_space:  [64] u32;
    reserved2:  [12] u32;
    struct {
        reserved3: [12] u32;

        #overlay (reserved3) sw_reserved: _fpx_sw_bytes;
    }
}
```
### `_fpx_sw_bytes`

```jai
_fpx_sw_bytes :: struct {
    magic1:        u32;
    extended_size: u32;
    xfeatures:     u64;
    xstate_size:   u32;
    padding:       [7] u32;
}
```
### `_fpxreg`

```jai
_fpxreg :: struct {
    significand:       [4] u16;
    exponent:          u16;
    __glibc_reserved1: [3] u16;
}
```
### `_libc_fpstate`

Variant 1 (Linux):

```jai
_libc_fpstate :: struct {
    cwd:               u16;
    swd:               u16;
    ftw:               u16;
    fop:               u16;
    rip:               u64;
    rdp:               u64;
    mxcsr:             u32;
    mxcr_mask:         u32;
    _st:               [8] _libc_fpxreg;
    _xmm:              [16] _libc_xmmreg;
    __glibc_reserved1: [24] u32;
}
```
Variant 2 (Android, x64):

```jai
_libc_fpstate :: struct {
    cwd:       u16;
    swd:       u16;
    ftw:       u16;
    fop:       u16;
    rip:       u64;
    rdp:       u64;
    mxcsr:     u32;
    mxcr_mask: u32;
    _st:       [8] _libc_fpxreg;
    _xmm:      [16] _libc_xmmreg;
    padding:   [24] u32;
}
```
### `_libc_fpxreg`

Variant 1 (Linux):

```jai
_libc_fpxreg :: struct {
    significand:       [4] u16;
    exponent:          u16;
    __glibc_reserved1: [3] u16;
}
```
Variant 2 (Android, x64):

```jai
_libc_fpxreg :: struct {
    significand: [4] u16;
    exponent:    u16;
    padding:     [3] u16;
}
```
### `_libc_xmmreg`

```jai
_libc_xmmreg :: struct {
    element: [4] u32;
}
```
### `_opaque_pthread_attr_t`

```jai
_opaque_pthread_attr_t :: struct {
    __sig:    s64;
    __opaque: [56] u8;
}
```
### `_opaque_pthread_cond_t`

```jai
_opaque_pthread_cond_t :: struct {
    __sig:    s64;
    __opaque: [40] u8;
}
```
### `_opaque_pthread_condattr_t`

```jai
_opaque_pthread_condattr_t :: struct {
    __sig:    s64;
    __opaque: [8] u8;
}
```
### `_opaque_pthread_mutex_t`

```jai
_opaque_pthread_mutex_t :: struct {
    __sig:    s64;
    __opaque: [56] u8;
}
```
### `_opaque_pthread_mutexattr_t`

```jai
_opaque_pthread_mutexattr_t :: struct {
    __sig:    s64;
    __opaque: [8] u8;
}
```
### `_opaque_pthread_once_t`

```jai
_opaque_pthread_once_t :: struct {
    __sig:    s64;
    __opaque: [8] u8;
}
```
### `_opaque_pthread_rwlock_t`

```jai
_opaque_pthread_rwlock_t :: struct {
    __sig:    s64;
    __opaque: [192] u8;
}
```
### `_opaque_pthread_rwlockattr_t`

```jai
_opaque_pthread_rwlockattr_t :: struct {
    __sig:    s64;
    __opaque: [16] u8;
}
```
### `_opaque_pthread_t`

```jai
_opaque_pthread_t :: struct {
    __sig:           s64;
    __cleanup_stack: *__darwin_pthread_handler_rec;
    __opaque:        [8176] u8;
}
```
### `_pcred`

```jai
_pcred :: struct {
    pc_lock:  [72] u8;
    pc_ucred: *ucred;
    p_ruid:   uid_t;
    p_svuid:  uid_t;
    p_rgid:   gid_t;
    p_svgid:  gid_t;
    p_refcnt: s32;
}
```
### `_pthread_cleanup_buffer`

```jai
_pthread_cleanup_buffer :: struct {
    __routine:    #type (unknown0: *void) -> void #c_call;
    __arg:        *void;
    __canceltype: s32;
    __prev:       *_pthread_cleanup_buffer;
}
```
### `_telldir`

```jai
_telldir :: struct {}
```
### `_ucred`

```jai
_ucred :: struct {
    cr_ref:     s32;
    cr_uid:     uid_t;
    cr_ngroups: s16;
    cr_groups:  [16] gid_t;
}
```
### `_xmmreg`

```jai
_xmmreg :: struct {
    element: [4] u32;
}
```
### `accessx_descriptor`

```jai
accessx_descriptor :: struct {
    ad_name_offset: u32;
    ad_flags:       s32;
    ad_pad:         [2] s32;
}
```
### `aiocb`

Variant 1 (Linux):

```jai
aiocb :: struct {
    aio_fildes:       s32;
    aio_lio_opcode:   s32;
    aio_reqprio:      s32;
    aio_buf:          *void;
    aio_nbytes:       u64;
    aio_sigevent:     sigevent;

    __next_prio:      *aiocb;
    __abs_prio:       s32;
    __policy:         s32;
    __error_code:     s32;
    __return_value:   s64;

    aio_offset:       s64;
    __pad:            [0] u8;

    __glibc_reserved: [32] u8;
}
```
Variant 2 (macOS, ARM64):

```jai
aiocb :: struct {
    aio_fildes:     s32;
    aio_offset:     s64;
    aio_buf:        *void;
    aio_nbytes:     u64;
    aio_reqprio:    s32;
    aio_sigevent:   sigevent;
    aio_lio_opcode: s32;
}
```
### `aiocb64`

```jai
aiocb64 :: struct {
    aio_fildes:       s32;
    aio_lio_opcode:   s32;
    aio_reqprio:      s32;
    aio_buf:          *void;
    aio_nbytes:       u64;
    aio_sigevent:     sigevent;

    __next_prio:      *aiocb;
    __abs_prio:       s32;
    __policy:         s32;
    __error_code:     s32;
    __return_value:   s64;

    aio_offset:       s64;
    __glibc_reserved: [32] u8;
}
```
### `aioinit`

```jai
aioinit :: struct {
    aio_threads:   s32;
    aio_num:       s32;
    aio_locks:     s32;
    aio_usedba:    s32;
    aio_debug:     s32;
    aio_numusers:  s32;

    aio_idle_time: s32;

    aio_reserved:  s32;
}
```
### `attribute_set`

```jai
attribute_set :: struct {
    commonattr: ATTR_CMN;
    volattr:    attrgroup_t;
    dirattr:    attrgroup_t;
    fileattr:   attrgroup_t;
    forkattr:   attrgroup_t;
}
```
### `attrlist`

```jai
attrlist :: struct {
    bitmapcount: u16;
    reserved:    u16;
    commonattr:  ATTR_CMN;
    volattr:     attrgroup_t;
    dirattr:     attrgroup_t;
    fileattr:    attrgroup_t;
    forkattr:    attrgroup_t;
}
```
### `attrreference`

```jai
attrreference :: struct {
    attr_dataoffset: s32;
    attr_length:     u32;
}
```
### `clockid_t`

Variant 1 (Linux):

```jai
clockid_t :: enum u32 {
    REALTIME           :: 0;

    MONOTONIC          :: 1;

    PROCESS_CPUTIME_ID :: 2;

    THREAD_CPUTIME_ID  :: 3;

    MONOTONIC_RAW      :: 4;

    REALTIME_COARSE    :: 5;

    MONOTONIC_COARSE   :: 6;

    BOOTTIME           :: 7;

    REALTIME_ALARM     :: 8;

    BOOTTIME_ALARM     :: 9;

    TAI                :: 11;

    CLOCK_REALTIME           :: REALTIME;

    CLOCK_MONOTONIC          :: MONOTONIC;

    CLOCK_PROCESS_CPUTIME_ID :: PROCESS_CPUTIME_ID;

    CLOCK_THREAD_CPUTIME_ID  :: THREAD_CPUTIME_ID;

    CLOCK_MONOTONIC_RAW      :: MONOTONIC_RAW;

    CLOCK_REALTIME_COARSE    :: REALTIME_COARSE;

    CLOCK_MONOTONIC_COARSE   :: MONOTONIC_COARSE;

    CLOCK_BOOTTIME           :: BOOTTIME;

    CLOCK_REALTIME_ALARM     :: REALTIME_ALARM;

    CLOCK_BOOTTIME_ALARM     :: BOOTTIME_ALARM;

    CLOCK_TAI                :: TAI;
}
```
Variant 2 (macOS, ARM64):

```jai
clockid_t :: enum u32 {
    REALTIME             :: 0;

    MONOTONIC            :: 6;

    MONOTONIC_RAW        :: 4;

    MONOTONIC_RAW_APPROX :: 5;

    UPTIME_RAW           :: 8;

    UPTIME_RAW_APPROX    :: 9;

    PROCESS_CPUTIME_ID   :: 12;

    THREAD_CPUTIME_ID    :: 16;

    _CLOCK_REALTIME             :: REALTIME;

    _CLOCK_MONOTONIC            :: MONOTONIC;

    _CLOCK_MONOTONIC_RAW        :: MONOTONIC_RAW;

    _CLOCK_MONOTONIC_RAW_APPROX :: MONOTONIC_RAW_APPROX;

    _CLOCK_UPTIME_RAW           :: UPTIME_RAW;

    _CLOCK_UPTIME_RAW_APPROX    :: UPTIME_RAW_APPROX;

    _CLOCK_PROCESS_CPUTIME_ID   :: PROCESS_CPUTIME_ID;

    _CLOCK_THREAD_CPUTIME_ID    :: THREAD_CPUTIME_ID;
}
```
Variant 3 (Android, ARM64):

```jai
clockid_t :: enum u32 {
    REALTIME           :: 0;
    MONOTONIC          :: 1;
    PROCESS_CPUTIME_ID :: 2;
    THREAD_CPUTIME_ID  :: 3;
    MONOTONIC_RAW      :: 4;
    REALTIME_COARSE    :: 5;
    MONOTONIC_COARSE   :: 6;
    BOOTTIME           :: 7;
    REALTIME_ALARM     :: 8;
    BOOTTIME_ALARM     :: 9;
    SGI_CYCLE          :: 10;
    TAI                :: 11;

    CLOCK_REALTIME           :: REALTIME;
    CLOCK_MONOTONIC          :: MONOTONIC;
    CLOCK_PROCESS_CPUTIME_ID :: PROCESS_CPUTIME_ID;
    CLOCK_THREAD_CPUTIME_ID  :: THREAD_CPUTIME_ID;
    CLOCK_MONOTONIC_RAW      :: MONOTONIC_RAW;
    CLOCK_REALTIME_COARSE    :: REALTIME_COARSE;
    CLOCK_MONOTONIC_COARSE   :: MONOTONIC_COARSE;
    CLOCK_BOOTTIME           :: BOOTTIME;
    CLOCK_REALTIME_ALARM     :: REALTIME_ALARM;
    CLOCK_BOOTTIME_ALARM     :: BOOTTIME_ALARM;
    CLOCK_SGI_CYCLE          :: SGI_CYCLE;
    CLOCK_TAI                :: TAI;
}
```
### `clockinfo`

```jai
clockinfo :: struct {
    hz:      s32;
    tick:    s32;
    tickadj: s32;
    stathz:  s32;
    profhz:  s32;
}
```
### `clone_args`

```jai
clone_args :: struct {
    flags:        u64;
    pidfd:        u64;
    child_tid:    u64;
    parent_tid:   u64;
    exit_signal:  u64;
    stack:        u64;
    stack_size:   u64;
    tls:          u64;
    set_tid:      u64;
    set_tid_size: u64;
    cgroup:       u64;
}
```
### `cpu_set_t`

Variant 1 (Linux):

```jai
cpu_set_t :: struct {
    __bits: [16] __cpu_mask;
}
```
Variant 2 (Android, ARM64):

```jai
cpu_set_t :: struct {
    __bits: [16] u64;
}
```
### `ctlname`

```jai
ctlname :: struct {
    ctl_name: *u8;
    ctl_type: s32;
}
```
### `dirent`

Variant 1 (Linux):

```jai
dirent :: struct {
    d_ino:    u64;
    d_off:    s64;

    d_reclen: u16;
    d_type:   DT;
    d_name:   [256] u8;
}
```
Variant 2 (macOS, ARM64):

```jai
dirent :: struct {
    d_ino:     u64;
    d_seekoff: u64;
    d_reclen:  u16;
    d_namlen:  u16;
    d_type:    DT;
    d_name:    [1024] u8;
}
```
Variant 3 (Android, ARM64):

```jai
dirent :: struct {
    d_ino:    ino_t;
    d_off:    s64;
    d_reclen: u16;
    d_type:   DT;
    d_name:   [256] u8;
}
```
### `dirent64`

Variant 1 (Linux):

```jai
dirent64 :: struct {
    d_ino:    u64;
    d_off:    s64;
    d_reclen: u16;
    d_type:   DT;
    d_name:   [256] u8;
}
```
Variant 2 (Android, ARM64):

```jai
dirent64 :: struct {
    d_ino:    ino_t;
    d_off:    s64;
    d_reclen: u16;
    d_type:   DT;
    d_name:   [256] u8;
}
```
### `diskextent`

```jai
diskextent :: struct {
    startblock: u32;
    blockcount: u32;
}
```
### `dl_info`

```jai
dl_info :: struct {
    dli_fname: *u8;
    dli_fbase: *void;
    dli_sname: *u8;
    dli_saddr: *void;
}
```
### `extern_proc`

Variant 1 (macOS, ARM64):

```jai
extern_proc :: struct {
    p_un:        struct {
        p_st1: struct {
            __p_forw: *proc;
            __p_back: *proc;
        };

        #overlay (p_st1) __p_starttime: timeval;
    };

    p_vmspace:   *vmspace;
    sigacts :: struct {}
    p_sigacts:   *sigacts;
    p_flag:      s32;
    p_stat:      u8;
    p_pid:       pid_t;
    p_oppid:     pid_t;
    p_dupfd:     s32;

    user_stack:  *u8;
    exit_thread: *void;
    p_debugger:  s32;
    sigwait:     s32;

    p_estcpu:    u32;
    p_cpticks:   s32;
    p_pctcpu:    fixpt_t;
    p_wchan:     *void;
    p_wmesg:     *u8;
    p_swtime:    u32;
    p_slptime:   u32;
    p_realtimer: itimerval;
    p_rtime:     timeval;
    p_uticks:    u64;
    p_sticks:    u64;
    p_iticks:    u64;
    p_traceflag: s32;
    vnode :: struct {}
    p_tracep:    *vnode;
    p_siglist:   s32;
    p_textvp:    *vnode;
    p_holdcnt:   s32;
    p_sigmask:   sigset_t;
    p_sigignore: sigset_t;
    p_sigcatch:  sigset_t;
    p_priority:  u8;
    p_usrpri:    u8;
    p_nice:      u8;
    p_comm:      [17] u8;
    p_pgrp:      *pgrp;
    user :: struct {}
    p_addr:      *user;
    p_xstat:     u16;
    p_acflag:    u16;
    rusage :: struct {}
    p_ru:        *rusage;
}
```
Variant 2 (macOS, x64):

```jai
extern_proc :: struct {
    p_un:        struct {
        p_st1: struct {
            __p_forw: *proc;
            __p_back: *proc;
        };

        #overlay (p_st1) __p_starttime: timeval;
    };

    p_vmspace:   *vmspace;
    sigacts :: struct {}
    p_sigacts:   *sigacts;
    p_flag:      s32;
    p_stat:      u8;
    p_pid:       pid_t;
    p_oppid:     pid_t;
    p_dupfd:     s32;

    user_stack:  *u8;
    exit_thread: *void;
    p_debugger:  s32;
    sigwait:     u32;

    p_estcpu:    u32;
    p_cpticks:   s32;
    p_pctcpu:    fixpt_t;
    p_wchan:     *void;
    p_wmesg:     *u8;
    p_swtime:    u32;
    p_slptime:   u32;
    p_realtimer: itimerval;
    p_rtime:     timeval;
    p_uticks:    u64;
    p_sticks:    u64;
    p_iticks:    u64;
    p_traceflag: s32;
    vnode :: struct {}
    p_tracep:    *vnode;
    p_siglist:   s32;
    p_textvp:    *vnode;
    p_holdcnt:   s32;
    p_sigmask:   sigset_t;
    p_sigignore: sigset_t;
    p_sigcatch:  sigset_t;
    p_priority:  u8;
    p_usrpri:    u8;
    p_nice:      u8;
    p_comm:      [17] u8;
    p_pgrp:      *pgrp;
    user :: struct {}
    p_addr:      *user;
    p_xstat:     u16;
    p_acflag:    u16;
    rusage :: struct {}
    p_ru:        *rusage;
}
```
### `f_owner_ex`

Variant 1 (Linux):

```jai
f_owner_ex :: struct {
    type: __pid_type;
    pid:  s32;
}
```
Variant 2 (Android, ARM64):

```jai
f_owner_ex :: struct {
    type: s32;
    pid:  s32;
}
```
### `fbootstraptransfer`

```jai
fbootstraptransfer :: struct {
    fbt_offset: s64;
    fbt_length: u64;
    fbt_buffer: *void;
}
```
### `fchecklv`

```jai
fchecklv :: struct {
    lv_file_start:         s64;
    lv_error_message_size: u64;
    lv_error_message:      *void;
}
```
### `fcodeblobs`

```jai
fcodeblobs :: struct {
    f_cd_hash:   *void;
    f_hash_size: u64;
    f_cd_buffer: *void;
    f_cd_size:   u64;
    f_out_size:  *u32;
    f_arch:      s32;
    __padding:   s32;
}
```
### `fgetsigsinfo`

```jai
fgetsigsinfo :: struct {
    fg_file_start:      s64;
    fg_info_request:    s32;
    fg_sig_is_platform: s32;
}
```
### `file_handle`

```jai
file_handle :: struct {
    handle_bytes: u32;
    handle_type:  s32;

    f_handle:     [0] u8;
}
```
### `filesec_property_t`

```jai
filesec_property_t :: enum u32 {
    OWNER         :: 1;
    GROUP         :: 2;
    UUID          :: 3;
    MODE          :: 4;
    ACL           :: 5;
    GRPUUID       :: 6;

    ACL_RAW       :: 100;
    ACL_ALLOCSIZE :: 101;

    FILESEC_OWNER         :: OWNER;
    FILESEC_GROUP         :: GROUP;
    FILESEC_UUID          :: UUID;
    FILESEC_MODE          :: MODE;
    FILESEC_ACL           :: ACL;
    FILESEC_GRPUUID       :: GRPUUID;

    FILESEC_ACL_RAW       :: ACL_RAW;
    FILESEC_ACL_ALLOCSIZE :: ACL_ALLOCSIZE;
}
```
### `flock`

Variant 1 (Linux):

```jai
flock :: struct {
    l_type:   s16;
    l_whence: s16;

    l_start:  s64;
    l_len:    s64;

    l_pid:    s32;
}
```
Variant 2 (macOS, ARM64):

```jai
flock :: struct {
    l_start:  s64;
    l_len:    s64;
    l_pid:    pid_t;
    l_type:   s16;
    l_whence: s16;
}
```
### `flock64`

```jai
flock64 :: struct {
    l_type:   s16;
    l_whence: s16;
    l_start:  s64;
    l_len:    s64;
    l_pid:    s32;
}
```
### `flocktimeout`

```jai
flocktimeout :: struct {
    fl:      flock;
    timeout: timespec;
}
```
### `fpunchhole`

```jai
fpunchhole :: struct {
    fp_flags:  u32;
    reserved:  u32;
    fp_offset: s64;
    fp_length: s64;
}
```
### `fsignatures`

Variant 1 (macOS, ARM64):

```jai
fsignatures :: struct {
    fs_file_start:       s64;
    fs_blob_start:       *void;
    fs_blob_size:        u64;

    fs_fsignatures_size: u64;
    fs_cdhash:           [20] u8;
    fs_hash_type:        s32;
}
```
Variant 2 (macOS, x64):

```jai
fsignatures :: struct {
    fs_file_start: s64;
    fs_blob_start: *void;
    fs_blob_size:  u64;
}
```
### `fspecread`

```jai
fspecread :: struct {
    fsr_flags:  u32;
    reserved:   u32;
    fsr_offset: s64;
    fsr_length: s64;
}
```
### `fssearchblock`

```jai
fssearchblock :: struct {
    returnattrs:         *attrlist;
    returnbuffer:        *void;
    returnbuffersize:    u64;
    maxmatches:          u64;
    timelimit:           timeval;
    searchparams1:       *void;
    sizeofsearchparams1: u64;
    searchparams2:       *void;
    sizeofsearchparams2: u64;
    searchattrs:         attrlist;
}
```
### `fstore`

```jai
fstore :: struct {
    fst_flags:      u32;
    fst_posmode:    s32;
    fst_offset:     s64;
    fst_length:     s64;
    fst_bytesalloc: s64;
}
```
### `fsupplement`

```jai
fsupplement :: struct {
    fs_file_start: s64;
    fs_blob_start: s64;
    fs_blob_size:  u64;
    fs_orig_fd:    s32;
}
```
### `ftrimactivefile`

```jai
ftrimactivefile :: struct {
    fta_offset: s64;
    fta_length: s64;
}
```
### `idtype_t`

```jai
idtype_t :: enum u32 {
    ALL  :: 0;
    PID  :: 1;
    PGID :: 2;

    P_ALL  :: ALL;
    P_PID  :: PID;
    P_PGID :: PGID;
}
```
### `image_offset`

```jai
image_offset :: struct {

    uuid:   uuid_t;

    offset: u32;
}
```
### `iovec`

Variant 1 (Linux):

```jai
iovec :: struct {
    iov_base: *void;
    iov_len:  u64;
}
```
Variant 2 (macOS, ARM64):

```jai
iovec :: struct {
    iov_base: *void;
    iov_len:  u64;
}
```
### `ipc_kludge`

```jai
ipc_kludge :: struct {
    msgbuf :: struct {}
    msgp:   *msgbuf;
    msgtyp: s64;
}
```
### `ipc_perm`

Variant 1 (Linux):

```jai
ipc_perm :: struct {
    __key:             s32;
    uid:               u32;
    gid:               u32;
    cuid:              u32;
    cgid:              u32;
    mode:              u32;
    __seq:             u16;
    __pad2:            u16;
    __glibc_reserved1: u64;
    __glibc_reserved2: u64;
}
```
Variant 2 (macOS, ARM64):

```jai
ipc_perm :: struct {
    uid:  uid_t;
    gid:  gid_t;
    cuid: uid_t;
    cgid: gid_t;
    mode: mode_t;
    _seq: u16;
    _key: key_t;
}
```
Variant 3 (macOS, x64):

```jai
ipc_perm :: struct {
    uid:  uid_t;
    gid:  gid_t;
    cuid: uid_t;
    cgid: gid_t;
    mode: mode_t;
    _seq: u16;
    _key: key_t;
}
```

The macOS x64 declaration has the same field sequence and types as the macOS
ARM64 declaration; it is a separately selected platform definition.
### `itimerspec`

```jai
itimerspec :: struct {
    it_interval: timespec;
    it_value:    timespec;
}
```
### `itimerval`

Variant 1 (Linux):

```jai
itimerval :: struct {

    it_interval: timeval;

    it_value:    timeval;
}
```
Variant 2 (macOS, ARM64):

```jai
itimerval :: struct {
    it_interval: timeval;
    it_value:    timeval;
}
```
Variant 3 (macOS, x64):

```jai
itimerval :: struct {
    it_interval: timeval;
    it_value:    timeval;
}
```
Variant 4 (Android, ARM64):

```jai
itimerval :: struct {
    it_interval: timeval;
    it_value:    timeval;
}
```
Variant 5 (Android, x64):

```jai
itimerval :: struct {
    it_interval: timeval;
    it_value:    timeval;
}
```
### `kinfo_proc`

The macOS ARM64 and x64 bindings use this same member sequence, including
`eproc.e_pcred`/`eproc.e_ucred` payload types and the four-element `e_spare`
array. `kp_proc.p_realtimer` is reached through the nested `extern_proc` field;
it is not a direct member of `kinfo_proc` or its nested `eproc`.

```jai
kinfo_proc :: struct {
    kp_proc:  extern_proc;
    eproc :: struct {
        e_paddr:   *proc;
        e_sess:    *session;
        e_pcred:   _pcred;
        e_ucred:   _ucred;
        e_vm:      vmspace;
        e_ppid:    pid_t;
        e_pgid:    pid_t;
        e_jobc:    s16;
        e_tdev:    dev_t;
        e_tpgid:   pid_t;
        e_tsess:   *session;

        e_wmesg:   [8] u8;
        e_xsize:   segsz_t;
        e_xrssize: s16;
        e_xccount: s16;
        e_xswrss:  s16;
        e_flag:    s32;

        e_login:   [12] u8;
        e_spare:   [4] s32;
    }

    kp_eproc: eproc;
}
```
### `loadavg`

```jai
loadavg :: struct {
    ldavg:  [3] fixpt_t;
    fscale: s64;
}
```
### `log2phys`

Variant 1 (macOS, ARM64):

```jai
log2phys :: struct {
    l2p_flags:       u32;

    l2p_contigbytes: s64 #align 4;

    l2p_devoffset:   s64 #align 4;
}
```
Variant 2 (macOS, x64):

```jai
log2phys :: struct {
    l2p_flags:       u32;
    l2p_contigbytes: s64 #align 4;

    l2p_devoffset:   s64 #align 4;
}
```
### `mcontext_t`

Variant 1 (Linux):

```jai
mcontext_t :: struct {
    gregs:       gregset_t;

    fpregs:      fpregset_t;
    __reserved1: [8] u64;
}
```
Variant 2 (Android, x64):

```jai
mcontext_t :: struct {
    gregs:       gregset_t;
    fpregs:      fpregset_t;
    __reserved1: [8] u64;
}
```
### `obstack`

```jai
obstack :: struct {}
```
### `ostat`

```jai
ostat :: struct {
    st_dev:       u16;
    st_ino:       ino_t;
    st_mode:      mode_t;
    st_nlink:     nlink_t;
    st_uid:       u16;
    st_gid:       u16;
    st_rdev:      u16;
    st_size:      s32;
    st_atimespec: timespec;
    st_mtimespec: timespec;
    st_ctimespec: timespec;
    st_blksize:   s32;
    st_blocks:    s32;
    st_flags:     u32;
    st_gen:       u32;
}
```
### `passwd`

Variant 1 (Linux):

```jai
passwd :: struct {
    pw_name:   *u8;

    pw_passwd: *u8;

    pw_uid:    u32;
    pw_gid:    u32;
    pw_gecos:  *u8;
    pw_dir:    *u8;
    pw_shell:  *u8;
}
```
Variant 2 (macOS, ARM64):

```jai
passwd :: struct {
    pw_name:   *u8;
    pw_passwd: *u8;
    pw_uid:    uid_t;
    pw_gid:    gid_t;
    pw_change: s64;
    pw_class:  *u8;
    pw_gecos:  *u8;
    pw_dir:    *u8;
    pw_shell:  *u8;
    pw_expire: s64;
}
```
Variant 3 (Android, ARM64):

```jai
passwd :: struct {
    pw_name:   *u8;
    pw_passwd: *u8;
    pw_uid:    uid_t;
    pw_gid:    gid_t;

    pw_gecos:  *u8;

    pw_dir:    *u8;
    pw_shell:  *u8;
}
```
### `pgrp`

```jai
pgrp :: struct {}
```
### `pollfd`

Variant 1 (Linux):

```jai
pollfd :: struct {
    fd:      s32;
    events:  s16;
    revents: s16;
}
```
Variant 2 (macOS, ARM64):

```jai
pollfd :: struct {
    fd:      s32;
    events:  s16;
    revents: s16;
}
```
### `proc`

```jai
proc :: struct {}
```
### `proc_rlimit_control_wakeupmon`

```jai
proc_rlimit_control_wakeupmon :: struct {
    wm_flags: u32;
    wm_rate:  s32;
}
```
### `pthread_attr_t`

Variant 1 (Linux):

```jai
pthread_attr_t :: struct {
    __size: [56] u8;

    #overlay (__size) __align: s64;
}
```
Variant 2 (Android, ARM64):

```jai
pthread_attr_t :: struct {
    flags:          u32;
    stack_base:     *void;
    stack_size:     u64;
    guard_size:     u64;
    sched_policy:   s32;
    sched_priority: s32;

    __reserved:     [16] u8;
}
```
### `pthread_barrier_t`

Variant 1 (Linux):

```jai
pthread_barrier_t :: struct {
    __size: [32] u8;

    #overlay (__size) __align: s64;
}
```
Variant 2 (Android, ARM64):

```jai
pthread_barrier_t :: struct {
    __private: [4] s64;
}
```
### `pthread_barrierattr_t`

```jai
pthread_barrierattr_t :: struct {
    __size: [4] u8;

    #overlay (__size) __align: s32;
}
```
### `pthread_cond_t`

Variant 1 (Linux):

```jai
pthread_cond_t :: struct {
    __data: __pthread_cond_s;

    #overlay (__data) __size:  [48] u8;
    #overlay (__data) __align: s64;
}
```
Variant 2 (Android, ARM64):

```jai
pthread_cond_t :: struct {
    __private: [12] s32;
}
```
### `pthread_condattr_t`

```jai
pthread_condattr_t :: struct {
    __size: [4] u8;

    #overlay (__size) __align: s32;
}
```
### `pthread_mutex_t`

Variant 1 (Linux):

```jai
pthread_mutex_t :: struct {
    __data: __pthread_mutex_s;

    #overlay (__data) __size:  [40] u8;
    #overlay (__data) __align: s64;
}
```
Variant 2 (Android, ARM64):

```jai
pthread_mutex_t :: struct {
    __private: [10] s32;
}
```
### `pthread_mutexattr_t`

```jai
pthread_mutexattr_t :: struct {
    __size: [4] u8;

    #overlay (__size) __align: s32;
}
```
### `pthread_override_s`

```jai
pthread_override_s :: struct {}
```
### `pthread_rwlock_t`

Variant 1 (Linux):

```jai
pthread_rwlock_t :: struct {
    __data: __pthread_rwlock_arch_t;

    #overlay (__data) __size:  [56] u8;
    #overlay (__data) __align: s64;
}
```
Variant 2 (Android, ARM64):

```jai
pthread_rwlock_t :: struct {
    __private: [14] s32;
}
```
### `pthread_rwlockattr_t`

```jai
pthread_rwlockattr_t :: struct {
    __size: [8] u8;

    #overlay (__size) __align: s64;
}
```
### `pthread_spinlock_t`

```jai
pthread_spinlock_t :: struct {
    __private: s64;
}
```
### `radvisory`

```jai
radvisory :: struct {
    ra_offset: s64;
    ra_count:  s32;
}
```
### `rlimit`

Variant 1 (Linux):

```jai
rlimit :: struct {
    rlim_cur: rlim_t;

    rlim_max: rlim_t;
}
```
Variant 2 (macOS, ARM64):

```jai
rlimit :: struct {
    rlim_cur: rlim_t;
    rlim_max: rlim_t;
}
```
Variant 3 (Android, ARM64):

```jai
rlimit :: struct {
    rlim_cur: u64;
    rlim_max: u64;
}
```
### `rlimit64`

Variant 1 (Linux):

```jai
rlimit64 :: struct {
    rlim_cur: rlim64_t;

    rlim_max: rlim64_t;
}
```
Variant 2 (Android, ARM64):

```jai
rlimit64 :: struct {
    rlim_cur: u64;
    rlim_max: u64;
}
```
### `rusage`

Variant 1 (Linux):

```jai
rusage :: struct {
    ru_utime: timeval;

    ru_stime: timeval;

    struct {
        ru_maxrss: s64;

        #overlay (ru_maxrss) __ru_maxrss_word: s64;
    }

    struct {
        ru_ixrss: s64;

        #overlay (ru_ixrss) __ru_ixrss_word: s64;
    }

    struct {
        ru_idrss: s64;

        #overlay (ru_idrss) __ru_idrss_word: s64;
    }

    struct {
        ru_isrss: s64;

        #overlay (ru_isrss) __ru_isrss_word: s64;
    }

    struct {
        ru_minflt: s64;

        #overlay (ru_minflt) __ru_minflt_word: s64;
    }

    struct {
        ru_majflt: s64;

        #overlay (ru_majflt) __ru_majflt_word: s64;
    }

    struct {
        ru_nswap: s64;

        #overlay (ru_nswap) __ru_nswap_word: s64;
    }

    struct {
        ru_inblock: s64;

        #overlay (ru_inblock) __ru_inblock_word: s64;
    }

    struct {
        ru_oublock: s64;

        #overlay (ru_oublock) __ru_oublock_word: s64;
    }

    struct {
        ru_msgsnd: s64;

        #overlay (ru_msgsnd) __ru_msgsnd_word: s64;
    }

    struct {
        ru_msgrcv: s64;

        #overlay (ru_msgrcv) __ru_msgrcv_word: s64;
    }

    struct {
        ru_nsignals: s64;

        #overlay (ru_nsignals) __ru_nsignals_word: s64;
    }

    struct {
        ru_nvcsw: s64;

        #overlay (ru_nvcsw) __ru_nvcsw_word: s64;
    }

    struct {
        ru_nivcsw: s64;

        #overlay (ru_nivcsw) __ru_nivcsw_word: s64;
    }
}
```
Variant 2 (macOS, ARM64):

```jai
rusage :: struct {
    ru_utime:    timeval;
    ru_stime:    timeval;

    ru_maxrss:   s64;

    ru_ixrss:    s64;
    ru_idrss:    s64;
    ru_isrss:    s64;
    ru_minflt:   s64;
    ru_majflt:   s64;
    ru_nswap:    s64;
    ru_inblock:  s64;
    ru_oublock:  s64;
    ru_msgsnd:   s64;
    ru_msgrcv:   s64;
    ru_nsignals: s64;
    ru_nvcsw:    s64;
    ru_nivcsw:   s64;
}
```
Variant 3 (Android, ARM64):

```jai
rusage :: struct {
    ru_utime:    timeval;
    ru_stime:    timeval;
    ru_maxrss:   s64;
    ru_ixrss:    s64;
    ru_idrss:    s64;
    ru_isrss:    s64;
    ru_minflt:   s64;
    ru_majflt:   s64;
    ru_nswap:    s64;
    ru_inblock:  s64;
    ru_oublock:  s64;
    ru_msgsnd:   s64;
    ru_msgrcv:   s64;
    ru_nsignals: s64;
    ru_nvcsw:    s64;
    ru_nivcsw:   s64;
}
```
### `rusage_info_v0`

```jai
rusage_info_v0 :: struct {
    ri_uuid:               [16] u8;
    ri_user_time:          u64;
    ri_system_time:        u64;
    ri_pkg_idle_wkups:     u64;
    ri_interrupt_wkups:    u64;
    ri_pageins:            u64;
    ri_wired_size:         u64;
    ri_resident_size:      u64;
    ri_phys_footprint:     u64;
    ri_proc_start_abstime: u64;
    ri_proc_exit_abstime:  u64;
}
```
### `rusage_info_v1`

```jai
rusage_info_v1 :: struct {
    ri_uuid:                  [16] u8;
    ri_user_time:             u64;
    ri_system_time:           u64;
    ri_pkg_idle_wkups:        u64;
    ri_interrupt_wkups:       u64;
    ri_pageins:               u64;
    ri_wired_size:            u64;
    ri_resident_size:         u64;
    ri_phys_footprint:        u64;
    ri_proc_start_abstime:    u64;
    ri_proc_exit_abstime:     u64;
    ri_child_user_time:       u64;
    ri_child_system_time:     u64;
    ri_child_pkg_idle_wkups:  u64;
    ri_child_interrupt_wkups: u64;
    ri_child_pageins:         u64;
    ri_child_elapsed_abstime: u64;
}
```
### `rusage_info_v2`

```jai
rusage_info_v2 :: struct {
    ri_uuid:                  [16] u8;
    ri_user_time:             u64;
    ri_system_time:           u64;
    ri_pkg_idle_wkups:        u64;
    ri_interrupt_wkups:       u64;
    ri_pageins:               u64;
    ri_wired_size:            u64;
    ri_resident_size:         u64;
    ri_phys_footprint:        u64;
    ri_proc_start_abstime:    u64;
    ri_proc_exit_abstime:     u64;
    ri_child_user_time:       u64;
    ri_child_system_time:     u64;
    ri_child_pkg_idle_wkups:  u64;
    ri_child_interrupt_wkups: u64;
    ri_child_pageins:         u64;
    ri_child_elapsed_abstime: u64;
    ri_diskio_bytesread:      u64;
    ri_diskio_byteswritten:   u64;
}
```
### `rusage_info_v3`

```jai
rusage_info_v3 :: struct {
    ri_uuid:                          [16] u8;
    ri_user_time:                     u64;
    ri_system_time:                   u64;
    ri_pkg_idle_wkups:                u64;
    ri_interrupt_wkups:               u64;
    ri_pageins:                       u64;
    ri_wired_size:                    u64;
    ri_resident_size:                 u64;
    ri_phys_footprint:                u64;
    ri_proc_start_abstime:            u64;
    ri_proc_exit_abstime:             u64;
    ri_child_user_time:               u64;
    ri_child_system_time:             u64;
    ri_child_pkg_idle_wkups:          u64;
    ri_child_interrupt_wkups:         u64;
    ri_child_pageins:                 u64;
    ri_child_elapsed_abstime:         u64;
    ri_diskio_bytesread:              u64;
    ri_diskio_byteswritten:           u64;
    ri_cpu_time_qos_default:          u64;
    ri_cpu_time_qos_maintenance:      u64;
    ri_cpu_time_qos_background:       u64;
    ri_cpu_time_qos_utility:          u64;
    ri_cpu_time_qos_legacy:           u64;
    ri_cpu_time_qos_user_initiated:   u64;
    ri_cpu_time_qos_user_interactive: u64;
    ri_billed_system_time:            u64;
    ri_serviced_system_time:          u64;
}
```
### `rusage_info_v4`

Variant 1 (macOS, ARM64):

```jai
rusage_info_v4 :: struct {
    ri_uuid:                          [16] u8;
    ri_user_time:                     u64;
    ri_system_time:                   u64;
    ri_pkg_idle_wkups:                u64;
    ri_interrupt_wkups:               u64;
    ri_pageins:                       u64;
    ri_wired_size:                    u64;
    ri_resident_size:                 u64;
    ri_phys_footprint:                u64;
    ri_proc_start_abstime:            u64;
    ri_proc_exit_abstime:             u64;
    ri_child_user_time:               u64;
    ri_child_system_time:             u64;
    ri_child_pkg_idle_wkups:          u64;
    ri_child_interrupt_wkups:         u64;
    ri_child_pageins:                 u64;
    ri_child_elapsed_abstime:         u64;
    ri_diskio_bytesread:              u64;
    ri_diskio_byteswritten:           u64;
    ri_cpu_time_qos_default:          u64;
    ri_cpu_time_qos_maintenance:      u64;
    ri_cpu_time_qos_background:       u64;
    ri_cpu_time_qos_utility:          u64;
    ri_cpu_time_qos_legacy:           u64;
    ri_cpu_time_qos_user_initiated:   u64;
    ri_cpu_time_qos_user_interactive: u64;
    ri_billed_system_time:            u64;
    ri_serviced_system_time:          u64;
    ri_logical_writes:                u64;
    ri_lifetime_max_phys_footprint:   u64;
    ri_instructions:                  u64;
    ri_cycles:                        u64;
    ri_billed_energy:                 u64;
    ri_serviced_energy:               u64;
    ri_interval_max_phys_footprint:   u64;
    ri_runnable_time:                 u64;
}
```
Variant 2 (macOS, x64):

```jai
rusage_info_v4 :: struct {
    ri_uuid:                          [16] u8;
    ri_user_time:                     u64;
    ri_system_time:                   u64;
    ri_pkg_idle_wkups:                u64;
    ri_interrupt_wkups:               u64;
    ri_pageins:                       u64;
    ri_wired_size:                    u64;
    ri_resident_size:                 u64;
    ri_phys_footprint:                u64;
    ri_proc_start_abstime:            u64;
    ri_proc_exit_abstime:             u64;
    ri_child_user_time:               u64;
    ri_child_system_time:             u64;
    ri_child_pkg_idle_wkups:          u64;
    ri_child_interrupt_wkups:         u64;
    ri_child_pageins:                 u64;
    ri_child_elapsed_abstime:         u64;
    ri_diskio_bytesread:              u64;
    ri_diskio_byteswritten:           u64;
    ri_cpu_time_qos_default:          u64;
    ri_cpu_time_qos_maintenance:      u64;
    ri_cpu_time_qos_background:       u64;
    ri_cpu_time_qos_utility:          u64;
    ri_cpu_time_qos_legacy:           u64;
    ri_cpu_time_qos_user_initiated:   u64;
    ri_cpu_time_qos_user_interactive: u64;
    ri_billed_system_time:            u64;
    ri_serviced_system_time:          u64;
    ri_logical_writes:                u64;
    ri_lifetime_max_phys_footprint:   u64;
    ri_instructions:                  u64;
    ri_cycles:                        u64;
    ri_billed_energy:                 u64;
    ri_serviced_energy:               u64;

    ri_unused:                        [2] u64;
}
```
### `rusage_info_v5`

```jai
rusage_info_v5 :: struct {
    ri_uuid:                          [16] u8;
    ri_user_time:                     u64;
    ri_system_time:                   u64;
    ri_pkg_idle_wkups:                u64;
    ri_interrupt_wkups:               u64;
    ri_pageins:                       u64;
    ri_wired_size:                    u64;
    ri_resident_size:                 u64;
    ri_phys_footprint:                u64;
    ri_proc_start_abstime:            u64;
    ri_proc_exit_abstime:             u64;
    ri_child_user_time:               u64;
    ri_child_system_time:             u64;
    ri_child_pkg_idle_wkups:          u64;
    ri_child_interrupt_wkups:         u64;
    ri_child_pageins:                 u64;
    ri_child_elapsed_abstime:         u64;
    ri_diskio_bytesread:              u64;
    ri_diskio_byteswritten:           u64;
    ri_cpu_time_qos_default:          u64;
    ri_cpu_time_qos_maintenance:      u64;
    ri_cpu_time_qos_background:       u64;
    ri_cpu_time_qos_utility:          u64;
    ri_cpu_time_qos_legacy:           u64;
    ri_cpu_time_qos_user_initiated:   u64;
    ri_cpu_time_qos_user_interactive: u64;
    ri_billed_system_time:            u64;
    ri_serviced_system_time:          u64;
    ri_logical_writes:                u64;
    ri_lifetime_max_phys_footprint:   u64;
    ri_instructions:                  u64;
    ri_cycles:                        u64;
    ri_billed_energy:                 u64;
    ri_serviced_energy:               u64;
    ri_interval_max_phys_footprint:   u64;
    ri_runnable_time:                 u64;
    ri_flags:                         u64;
}
```
### `sched_param`

Variant 1 (Linux):

```jai
sched_param :: struct {
    sched_priority: s32;
}
```
Variant 2 (macOS, ARM64):

```jai
sched_param :: struct {
    sched_priority: s32;
    __opaque:       [4] u8;
}
```
### `searchstate`

```jai
searchstate :: struct {
    ss_union_flags: u32 #align 1;
    ss_union_layer: u32 #align 1;
    ss_fsstate:     [548] u8;
}
```
### `sem_t`

Variant 1 (Linux):

```jai
sem_t :: struct {
    __size: [32] u8;

    #overlay (__size) __align: s64;
}
```
Variant 2 (Android, ARM64):

```jai
sem_t :: struct {
    count:      u32;

    __reserved: [3] s32;
}
```
### `session`

```jai
session :: struct {}
```
### `shm_info`

```jai
shm_info :: struct {
    used_ids:       s32;
    shm_tot:        u64;
    shm_rss:        u64;
    shm_swp:        u64;
    swap_attempts:  u64;
    swap_successes: u64;
}
```
### `shmid_ds`

```jai
shmid_ds :: struct {
    shm_perm:          ipc_perm;

    shm_segsz:         u64;

    shm_atime:         s64;
    shm_dtime:         s64;
    shm_ctime:         s64;

    shm_cpid:          s32;
    shm_lpid:          s32;
    shm_nattch:        shmatt_t;
    __glibc_reserved5: u64;
    __glibc_reserved6: u64;
}
```
### `shminfo`

Variant 1 (Linux):

```jai
shminfo :: struct {
    shmmax:            u64;
    shmmin:            u64;
    shmmni:            u64;
    shmseg:            u64;
    shmall:            u64;
    __glibc_reserved1: u64;
    __glibc_reserved2: u64;
    __glibc_reserved3: u64;
    __glibc_reserved4: u64;
}
```
Variant 2 (Android, ARM64):

```jai
shminfo :: struct {
    shmmax: s32;
    shmmin: s32;
    shmmni: s32;
    shmseg: s32;
    shmall: s32;
}
```
### `sigaction64_t`

```jai
sigaction64_t :: struct {
    sa_flags:    s32;
    struct {
        sa_handler: sighandler_t;

        #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo, unknown2: *void) -> void #c_call;
    }
    sa_mask:     sigset_t;
    sa_restorer: #type () -> void #c_call;
}
```
### `sigaction_t`

Variant 1 (Linux):

```jai
sigaction_t :: struct {
    using __sigaction_handler: struct {
        sa_handler: #type (unknown0: s32) -> void #c_call;

        #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo_t, unknown2: *void) -> void #c_call;
    };

    sa_mask:                   __sigset_t;

    sa_flags:                  s32;

    sa_restorer:               #type () -> void #c_call;
}
```
Variant 2 (macOS, ARM64):

```jai
sigaction_t :: struct {
    using __sigaction_u_: __sigaction_u;
    sa_mask:              sigset_t;
    sa_flags:             s32;
}
```
Variant 3 (Android, ARM64):

```jai
sigaction_t :: struct {
    sa_flags:    s32;
    struct {
        sa_handler: sighandler_t;

        #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo, unknown2: *void) -> void #c_call;
    }
    sa_mask:     sigset_t;
    sa_restorer: #type () -> void #c_call;
}
```
### `sigaltstack_t`

```jai
sigaltstack_t :: struct {
    ss_sp:    *void;
    ss_flags: s32;
    ss_size:  u64;
}
```
### `sigcontext`

Variant 1 (Linux):

```jai
sigcontext :: struct {
    r8:          u64;
    r9:          u64;
    r10:         u64;
    r11:         u64;
    r12:         u64;
    r13:         u64;
    r14:         u64;
    r15:         u64;
    rdi:         u64;
    rsi:         u64;
    rbp:         u64;
    rbx:         u64;
    rdx:         u64;
    rax:         u64;
    rcx:         u64;
    rsp:         u64;
    rip:         u64;
    eflags:      u64;
    cs:          u16;
    gs:          u16;
    fs:          u16;
    __pad0:      u16;
    err:         u64;
    trapno:      u64;
    oldmask:     u64;
    cr2:         u64;
    struct {
        fpstate: *_fpstate;

        #overlay (fpstate) __fpstate_word: u64;
    }

    __reserved1: [8] u64;
}
```
Variant 2 (Android, ARM64):

```jai
sigcontext :: struct {
    fault_address: u64;
    regs:          [31] u64;
    sp:            u64;
    pc:            u64;
    pstate:        u64;
    __reserved:    [4096] u8 #align 16;
}
```
Variant 3 (Android, x64):

```jai
sigcontext :: struct {
    r8:        u64;
    r9:        u64;
    r10:       u64;
    r11:       u64;
    r12:       u64;
    r13:       u64;
    r14:       u64;
    r15:       u64;
    rdi:       u64;
    rsi:       u64;
    rbp:       u64;
    rbx:       u64;
    rdx:       u64;
    rax:       u64;
    rcx:       u64;
    rsp:       u64;
    rip:       u64;
    eflags:    u64;
    cs:        u16;
    gs:        u16;
    fs:        u16;
    struct {
        ss: u16;

        #overlay (ss) __pad0: u16;
    }

    err:       u64;
    trapno:    u64;
    oldmask:   u64;
    cr2:       u64;
    fpstate:   *_fpstate_64;

    reserved1: [8] u64;
}
```
### `sigevent`

Variant 1 (Linux):

```jai
sigevent :: struct {
    sigev_value:  sigval;
    sigev_signo:  s32;
    sigev_notify: s32;

    _sigev_un:    struct {
        _pad: [12] s32;

        #overlay (_pad) _tid:          s32;
        #overlay (_pad) _sigev_thread: struct {
            _function:  #type (unknown0: sigval) -> void #c_call;
            _attribute: *pthread_attr_t;
        };
    };
}
```
Variant 2 (macOS, ARM64):

```jai
sigevent :: struct {
    sigev_notify:            s32;
    sigev_signo:             s32;
    sigev_value:             sigval;
    sigev_notify_function:   #type (unknown0: sigval) -> void #c_call;
    sigev_notify_attributes: *pthread_attr_t;
}
```
Variant 3 (Android, ARM64):

```jai
sigevent :: struct {
    sigev_value:  sigval_t;
    sigev_signo:  s32;
    sigev_notify: s32;
    _sigev_un:    struct {
        _pad: [12] s32;

        #overlay (_pad) _tid:          s32;
        #overlay (_pad) _sigev_thread: struct {
            _function:  #type (unknown0: sigval_t) -> void #c_call;
            _attribute: *void;
        };
    };
}
```
### `siginfo`

```jai
siginfo :: struct {
    struct {
        _si_pad: [32] s32;

        #overlay (_si_pad) using anon_struct: struct {
            si_signo:  s32;
            si_errno:  s32;
            si_code:   s32;
            _sifields: __sifields;
        };
    }
}
```
### `siginfo_t`

```jai
siginfo_t :: struct {
    si_signo:  s32;

    si_errno:  s32;

    si_code:   s32;

    __pad0:    s32;

    _sifields: struct {
        _pad: [28] s32;

        #overlay (_pad) _kill:     struct {
            si_pid: s32;
            si_uid: u32;
        };
        #overlay (_pad) _timer:    struct {
            si_tid:     s32;
            si_overrun: s32;
            si_sigval:  sigval;
        };
        #overlay (_pad) _rt:       struct {
            si_pid:    s32;
            si_uid:    u32;
            si_sigval: sigval;
        };
        #overlay (_pad) _sigchld:  struct {
            si_pid:    s32;
            si_uid:    u32;
            si_status: s32;
            si_utime:  s64;
            si_stime:  s64;
        };
        #overlay (_pad) _sigfault: struct {
            si_addr:     *void;

            si_addr_lsb: s16;
            _bounds:     struct {
                _addr_bnd: struct {
                    _lower: *void;
                    _upper: *void;
                };

                #overlay (_addr_bnd) _pkey: u32;
            };
        };
        #overlay (_pad) _sigpoll:  struct {
            si_band: s64;
            si_fd:   s32;
        };
        #overlay (_pad) _sigsys:   struct {
            _call_addr: *void;
            _syscall:   s32;
            _arch:      u32;
        };
    };
}
```
### `signalfd_siginfo`

```jai
signalfd_siginfo :: struct {
    ssi_signo:     u32;
    ssi_errno:     s32;
    ssi_code:      s32;
    ssi_pid:       u32;
    ssi_uid:       u32;
    ssi_fd:        s32;
    ssi_tid:       u32;
    ssi_band:      u32;
    ssi_overrun:   u32;
    ssi_trapno:    u32;
    ssi_status:    s32;
    ssi_int:       s32;
    ssi_ptr:       u64;
    ssi_utime:     u64;
    ssi_stime:     u64;
    ssi_addr:      u64;
    ssi_addr_lsb:  u16;
    __pad2:        u16;
    ssi_syscall:   s32;
    ssi_call_addr: u64;
    ssi_arch:      u32;
    __pad:         [28] u8;
}
```
### `sigset_t`

```jai
sigset_t :: struct {
    sig: [1] u64;
}
```
### `sigval`

```jai
sigval :: struct {
    sival_ptr: *void;

    #overlay (sival_ptr) sival_int: s32;
}
```
### `sigvec`

```jai
sigvec :: struct {
    sv_handler: #type (unknown0: s32) -> void #c_call;
    sv_mask:    s32;
    sv_flags:   s32;
}
```
### `stack_t`

Variant 1 (Linux):

```jai
stack_t :: struct {
    ss_sp:    *void;
    ss_flags: s32;
    ss_size:  u64;
}
```
Variant 2 (macOS, ARM64):

```jai
stack_t :: struct {
    ss_sp:    *void;
    ss_size:  u64;
    ss_flags: s32;
}
```
### `stat64_t`

Variant 1 (Linux):

```jai
stat64_t :: struct {
    st_dev:           u64;

    st_ino:           u64;
    st_nlink:         u64;
    st_mode:          u32;

    st_uid:           u32;
    st_gid:           u32;

    __pad0:           s32;
    st_rdev:          u64;
    st_size:          s64;

    st_blksize:       s64;
    st_blocks:        s64;

    st_atime:         timespec;
    st_mtime:         timespec;
    st_ctime:         timespec;

    __glibc_reserved: [3] s64;
}
```
Variant 2 (macOS, x64):

```jai
stat64_t :: struct {

    st_dev:       dev_t;

    st_mode:      mode_t;

    st_nlink:     nlink_t;

    st_ino:       u64;

    st_uid:       uid_t;

    st_gid:       gid_t;

    st_rdev:      dev_t;

    st_atime:     timespec;

    st_mtime:     timespec;

    st_ctime:     timespec;

    st_birthtime: timespec;

    st_size:      s64;

    st_blocks:    blkcnt_t;

    st_blksize:   blksize_t;

    st_flags:     u32;

    st_gen:       u32;

    st_lspare:    s32;

    st_qspare:    [2] s64;
}
```
Variant 3 (Android, ARM64):

```jai
stat64_t :: struct {
    st_dev:     dev_t;
    st_ino:     ino_t;
    st_mode:    mode_t;
    st_nlink:   nlink_t;
    st_uid:     uid_t;
    st_gid:     gid_t;
    st_rdev:    dev_t;
    __pad1:     u64;
    st_size:    s64;
    st_blksize: s32;
    __pad2:     s32;
    st_blocks:  s64;
    st_atime:   timespec;
    st_mtime:   timespec;
    st_ctime:   timespec;
    __unused4:  u32;
    __unused5:  u32;
}
```
Variant 4 (Android, x64):

```jai
stat64_t :: struct {
    st_dev:     dev_t;
    st_ino:     ino_t;
    st_nlink:   u64;
    st_mode:    mode_t;
    st_uid:     uid_t;
    st_gid:     gid_t;
    __pad0:     u32;
    st_rdev:    dev_t;
    st_size:    s64;
    st_blksize: s64;
    st_blocks:  s64;
    st_atime:   timespec;
    st_mtime:   timespec;
    st_ctime:   timespec;
    __pad3:     [3] s64;
}
```
### `stat_t`

Variant 1 (Linux):

```jai
stat_t :: struct {
    st_dev:           u64;

    st_ino:           u64;

    st_nlink:         u64;
    st_mode:          u32;

    st_uid:           u32;
    st_gid:           u32;

    __pad0:           s32;

    st_rdev:          u64;

    st_size:          s64;

    st_blksize:       s64;

    st_blocks:        s64;

    st_atime:         timespec;
    st_mtime:         timespec;
    st_ctime:         timespec;

    __glibc_reserved: [3] s64;
}
```
Variant 2 (macOS, ARM64):

```jai
stat_t :: struct {

    st_dev:       dev_t;

    st_mode:      mode_t;

    st_nlink:     nlink_t;

    st_ino:       u64;

    st_uid:       uid_t;

    st_gid:       gid_t;

    st_rdev:      dev_t;

    st_atime:     timespec;

    st_mtime:     timespec;

    st_ctime:     timespec;

    st_birthtime: timespec;

    st_size:      s64;

    st_blocks:    blkcnt_t;

    st_blksize:   blksize_t;

    st_flags:     u32;

    st_gen:       u32;

    st_lspare:    s32;

    st_qspare:    [2] s64;
}
```
Variant 3 (Android, ARM64):

```jai
stat_t :: struct {
    st_dev:     dev_t;
    st_ino:     ino_t;
    st_mode:    mode_t;
    st_nlink:   nlink_t;
    st_uid:     uid_t;
    st_gid:     gid_t;
    st_rdev:    dev_t;
    __pad1:     u64;
    st_size:    s64;
    st_blksize: s32;
    __pad2:     s32;
    st_blocks:  s64;
    st_atime:   timespec;
    st_mtime:   timespec;
    st_ctime:   timespec;
    __unused4:  u32;
    __unused5:  u32;
}
```
Variant 4 (Android, x64):

```jai
stat_t :: struct {
    st_dev:     dev_t;
    st_ino:     ino_t;
    st_nlink:   u64;
    st_mode:    mode_t;
    st_uid:     uid_t;
    st_gid:     gid_t;
    __pad0:     u32;
    st_rdev:    dev_t;
    st_size:    s64;
    st_blksize: s64;
    st_blocks:  s64;
    st_atime:   timespec;
    st_mtime:   timespec;
    st_ctime:   timespec;
    __pad3:     [3] s64;
}
```
### `statx_t`

Variant 1 (Linux):

```jai
statx_t :: struct {
    stx_mask:            u32;
    stx_blksize:         u32;
    stx_attributes:      u64;

    stx_nlink:           u32;
    stx_uid:             u32;
    stx_gid:             u32;
    stx_mode:            u16;
    __spare0:            [1] u16;

    stx_ino:             u64;
    stx_size:            u64;
    stx_blocks:          u64;
    stx_attributes_mask: u64;

    stx_atime:           statx_timestamp;
    stx_btime:           statx_timestamp;
    stx_ctime:           statx_timestamp;
    stx_mtime:           statx_timestamp;

    stx_rdev_major:      u32;
    stx_rdev_minor:      u32;
    stx_dev_major:       u32;
    stx_dev_minor:       u32;

    __spare2:            [14] u64;
}
```
Variant 2 (Android, ARM64):

```jai
statx_t :: struct {
    stx_mask:             u32;
    stx_blksize:          u32;
    stx_attributes:       u64;
    stx_nlink:            u32;
    stx_uid:              u32;
    stx_gid:              u32;
    stx_mode:             u16;
    __spare0:             [1] u16;
    stx_ino:              u64;
    stx_size:             u64;
    stx_blocks:           u64;
    stx_attributes_mask:  u64;
    stx_atime:            statx_timestamp;
    stx_btime:            statx_timestamp;
    stx_ctime:            statx_timestamp;
    stx_mtime:            statx_timestamp;
    stx_rdev_major:       u32;
    stx_rdev_minor:       u32;
    stx_dev_major:        u32;
    stx_dev_minor:        u32;
    stx_mnt_id:           u64;
    stx_dio_mem_align:    u32;
    stx_dio_offset_align: u32;
    __spare3:             [12] u64;
}
```
### `statx_timestamp`

```jai
statx_timestamp :: struct {
    tv_sec:     s64;
    tv_nsec:    u32;
    __reserved: s32;
}
```
### `timespec`

Variant 1 (Linux):

```jai
timespec :: struct {
    tv_sec:  s64;

    tv_nsec: s64;
}
```
Variant 2 (macOS, ARM64):

```jai
timespec :: struct {
    tv_sec:  s64;
    tv_nsec: s64;
}
```
Variant 3 (Android, ARM64):

```jai
timespec :: struct {
    tv_sec:  time_t;

    tv_nsec: s64;
}
```
### `timeval`

Variant 1 (Linux):

```jai
timeval :: struct {
    tv_sec:  s64;
    tv_usec: s64;
}
```
Variant 2 (macOS, ARM64):

```jai
timeval :: struct {
    tv_sec:  s64;
    tv_usec: s32;
}
```
### `timex`

```jai
timex :: struct {
    modes:      u32;
    offset:     s64;
    freq:       s64;
    maxerror:   s64;
    esterror:   s64;
    status:     s32;
    constant:   s64;
    precision:  s64;
    tolerance:  s64;
    time:       timeval;
    tick:       s64;
    ppsfreq:    s64;
    jitter:     s64;
    shift:      s32;
    stabil:     s64;
    jitcnt:     s64;
    calcnt:     s64;
    errcnt:     s64;
    stbcnt:     s64;

    tai:        s32;

    __bitfield: [44] u8;

}
```
### `timezone`

Variant 1 (Linux):

```jai
timezone :: struct {
    tz_minuteswest: s32;
    tz_dsttime:     s32;
}
```
Variant 2 (macOS, ARM64):

```jai
timezone :: struct {
    tz_minuteswest: s32;
    tz_dsttime:     s32;
}
```
Variant 3 (Android, ARM64):

```jai
timezone :: struct {
    tz_minuteswest: s32;
    tz_dsttime:     s32;
}
```
### `tm`

Variant 1 (Linux):

```jai
tm :: struct {
    tm_sec:    s32;
    tm_min:    s32;
    tm_hour:   s32;
    tm_mday:   s32;
    tm_mon:    s32;
    tm_year:   s32;
    tm_wday:   s32;
    tm_yday:   s32;
    tm_isdst:  s32;

    tm_gmtoff: s64;
    tm_zone:   *u8;
}
```
Variant 2 (macOS, ARM64):

```jai
tm :: struct {
    tm_sec:    s32;
    tm_min:    s32;
    tm_hour:   s32;
    tm_mday:   s32;
    tm_mon:    s32;
    tm_year:   s32;
    tm_wday:   s32;
    tm_yday:   s32;
    tm_isdst:  s32;
    tm_gmtoff: s64;
    tm_zone:   *u8;
}
```
Variant 3 (Android, ARM64):

```jai
tm :: struct {
    tm_sec:    s32;

    tm_min:    s32;

    tm_hour:   s32;

    tm_mday:   s32;

    tm_mon:    s32;

    tm_year:   s32;

    tm_wday:   s32;

    tm_yday:   s32;

    tm_isdst:  s32;

    tm_gmtoff: s64;

    tm_zone:   *u8;
}
```
### `ttysize`

```jai
ttysize :: struct {
    ts_lines: u16;
    ts_cols:  u16;
    ts_xxx:   u16;
    ts_yyy:   u16;
}
```
### `ucontext`

Variant 1 (Android, ARM64):

```jai
ucontext :: struct {
    uc_flags:    u64;
    uc_link:     *ucontext;
    uc_stack:    stack_t;
    struct {
        uc_sigmask: sigset_t;

        #overlay (uc_sigmask) uc_sigmask64: sigset64_t;
    }

    __padding:   [120] u8;
    uc_mcontext: mcontext_t;
}
```
Variant 2 (Android, x64):

```jai
ucontext :: struct {
    uc_flags:     u64;
    uc_link:      *ucontext;
    uc_stack:     stack_t;
    uc_mcontext:  mcontext_t;
    struct {
        uc_sigmask: sigset_t;

        #overlay (uc_sigmask) uc_sigmask64: sigset64_t;
    }

    __fpregs_mem: _libc_fpstate;
}
```
### `ucontext_t`

Variant 1 (Linux):

```jai
ucontext_t :: struct {
    uc_flags:     u64;
    uc_link:      *ucontext_t;
    uc_stack:     stack_t;
    uc_mcontext:  mcontext_t;
    uc_sigmask:   sigset_t;
    __fpregs_mem: _libc_fpstate;
    __ssp:        [4] u64;
}
```
Variant 2 (macOS, ARM64):

```jai
ucontext_t :: struct {
    uc_onstack:  s32;
    uc_sigmask:  u32;
    uc_stack:    stack_t;
    uc_link:     *ucontext_t;
    uc_mcsize:   u64;
    uc_mcontext: *__darwin_mcontext64;
}
```
### `ucred`

```jai
ucred :: struct {}
```
### `user`

```jai
user :: struct {
    regs:          user_regs_struct;
    u_fpvalid:     s32;
    pad0:          s32;
    i387:          user_fpregs_struct;
    u_tsize:       u64;
    u_dsize:       u64;
    u_ssize:       u64;
    start_code:    u64;
    start_stack:   u64;
    signal:        s64;
    reserved:      s32;
    pad1:          s32;
    u_ar0:         *user_regs_struct;
    u_fpstate:     *user_fpregs_struct;
    magic:         u64;
    u_comm:        [32] u8;
    u_debugreg:    [8] u64;
    error_code:    u64;
    fault_address: u64;
}
```
### `user_fpregs_struct`

```jai
user_fpregs_struct :: struct {
    cwd:       u16;
    swd:       u16;
    ftw:       u16;
    fop:       u16;
    rip:       u64;
    rdp:       u64;
    mxcsr:     u32;
    mxcr_mask: u32;
    st_space:  [32] u32;
    xmm_space: [64] u32;
    padding:   [24] u32;
}
```
### `user_fpsimd_struct`

```jai
user_fpsimd_struct :: struct {
    vregs: [32] U128 #align 16;
    fpsr:  u32;
    fpcr:  u32;
}
```
### `user_regs_struct`

Variant 1 (Android, ARM64):

```jai
user_regs_struct :: struct {
    regs:   [31] u64;
    sp:     u64;
    pc:     u64;
    pstate: u64;
}
```
Variant 2 (Android, x64):

```jai
user_regs_struct :: struct {
    r15:      u64;
    r14:      u64;
    r13:      u64;
    r12:      u64;
    rbp:      u64;
    rbx:      u64;
    r11:      u64;
    r10:      u64;
    r9:       u64;
    r8:       u64;
    rax:      u64;
    rcx:      u64;
    rdx:      u64;
    rsi:      u64;
    rdi:      u64;
    orig_rax: u64;
    rip:      u64;
    cs:       u64;
    eflags:   u64;
    rsp:      u64;
    ss:       u64;
    fs_base:  u64;
    gs_base:  u64;
    ds:       u64;
    es:       u64;
    fs:       u64;
    gs:       u64;
}
```
### `utsname`

Variant 1 (Linux):

```jai
utsname :: struct {

    sysname:    [65] u8;

    nodename:   [65] u8;

    release:    [65] u8;

    version:    [65] u8;

    machine:    [65] u8;

    domainname: [65] u8;
}
```
Variant 2 (macOS, ARM64):

```jai
utsname :: struct {
    sysname:  [256] u8;
    nodename: [256] u8;
    release:  [256] u8;
    version:  [256] u8;
    machine:  [256] u8;
}
```
Variant 3 (Android, ARM64):

```jai
utsname :: struct {

    sysname:    [65] u8;

    nodename:   [65] u8;

    release:    [65] u8;

    version:    [65] u8;

    machine:    [65] u8;

    domainname: [65] u8;
}
```
### `vmspace`

```jai
vmspace :: struct {
    dummy:  s32;
    dummy2: *u8;
    dummy3: [5] s32;
    dummy4: [3] *u8;
}
```
### `vol_attributes_attr`

```jai
vol_attributes_attr :: struct {
    validattr:  attribute_set_t;
    nativeattr: attribute_set_t;
}
```
### `vol_capabilities_attr`

```jai
vol_capabilities_attr :: struct {
    capabilities: vol_capabilities_set_t;
    valid:        vol_capabilities_set_t;
}
```
### `vtagtype`

Variant 1 (macOS, ARM64):

```jai
vtagtype :: enum u32 {
    NON      :: 0;

    UFS      :: 1;

    NFS      :: 2;
    MFS      :: 3;
    MSDOSFS  :: 4;
    LFS      :: 5;

    LOFS     :: 6;
    FDESC    :: 7;
    PORTAL   :: 8;
    NULL     :: 9;
    UMAP     :: 10;

    KERNFS   :: 11;
    PROCFS   :: 12;
    AFS      :: 13;
    ISOFS    :: 14;
    MOCKFS   :: 15;

    HFS      :: 16;
    ZFS      :: 17;
    DEVFS    :: 18;
    WEBDAV   :: 19;
    UDF      :: 20;

    AFP      :: 21;
    CDDA     :: 22;
    CIFS     :: 23;
    OTHER    :: 24;
    APFS     :: 25;

    LOCKERFS :: 26;
    BINDFS   :: 27;

    VT_NON      :: NON;

    VT_UFS      :: UFS;

    VT_NFS      :: NFS;
    VT_MFS      :: MFS;
    VT_MSDOSFS  :: MSDOSFS;
    VT_LFS      :: LFS;

    VT_LOFS     :: LOFS;
    VT_FDESC    :: FDESC;
    VT_PORTAL   :: PORTAL;
    VT_NULL     :: NULL;
    VT_UMAP     :: UMAP;

    VT_KERNFS   :: KERNFS;
    VT_PROCFS   :: PROCFS;
    VT_AFS      :: AFS;
    VT_ISOFS    :: ISOFS;
    VT_MOCKFS   :: MOCKFS;

    VT_HFS      :: HFS;
    VT_ZFS      :: ZFS;
    VT_DEVFS    :: DEVFS;
    VT_WEBDAV   :: WEBDAV;
    VT_UDF      :: UDF;

    VT_AFP      :: AFP;
    VT_CDDA     :: CDDA;
    VT_CIFS     :: CIFS;
    VT_OTHER    :: OTHER;
    VT_APFS     :: APFS;

    VT_LOCKERFS :: LOCKERFS;
    VT_BINDFS   :: BINDFS;
}
```
Variant 2 (macOS, x64):

```jai
vtagtype :: enum u32 {
    NON     :: 0;

    UFS     :: 1;

    NFS     :: 2;
    MFS     :: 3;
    MSDOSFS :: 4;
    LFS     :: 5;

    LOFS    :: 6;
    FDESC   :: 7;
    PORTAL  :: 8;
    NULL    :: 9;
    UMAP    :: 10;

    KERNFS  :: 11;
    PROCFS  :: 12;
    AFS     :: 13;
    ISOFS   :: 14;
    MOCKFS  :: 15;

    HFS     :: 16;
    ZFS     :: 17;
    DEVFS   :: 18;
    WEBDAV  :: 19;
    UDF     :: 20;

    AFP     :: 21;
    CDDA    :: 22;
    CIFS    :: 23;
    OTHER   :: 24;
    APFS    :: 25;

    VT_NON     :: NON;

    VT_UFS     :: UFS;

    VT_NFS     :: NFS;
    VT_MFS     :: MFS;
    VT_MSDOSFS :: MSDOSFS;
    VT_LFS     :: LFS;

    VT_LOFS    :: LOFS;
    VT_FDESC   :: FDESC;
    VT_PORTAL  :: PORTAL;
    VT_NULL    :: NULL;
    VT_UMAP    :: UMAP;

    VT_KERNFS  :: KERNFS;
    VT_PROCFS  :: PROCFS;
    VT_AFS     :: AFS;
    VT_ISOFS   :: ISOFS;
    VT_MOCKFS  :: MOCKFS;

    VT_HFS     :: HFS;
    VT_ZFS     :: ZFS;
    VT_DEVFS   :: DEVFS;
    VT_WEBDAV  :: WEBDAV;
    VT_UDF     :: UDF;

    VT_AFP     :: AFP;
    VT_CDDA    :: CDDA;
    VT_CIFS    :: CIFS;
    VT_OTHER   :: OTHER;
    VT_APFS    :: APFS;
}
```
### `vtype`

```jai
vtype :: enum u32 {
    NON  :: 0;

    REG  :: 1;
    DIR  :: 2;
    BLK  :: 3;
    CHR  :: 4;
    LNK  :: 5;

    SOCK :: 6;
    FIFO :: 7;
    BAD  :: 8;
    STR  :: 9;
    CPLX :: 10;

    VNON  :: NON;

    VREG  :: REG;
    VDIR  :: DIR;
    VBLK  :: BLK;
    VCHR  :: CHR;
    VLNK  :: LNK;

    VSOCK :: SOCK;
    VFIFO :: FIFO;
    VBAD  :: BAD;
    VSTR  :: STR;
    VCPLX :: CPLX;
}
```
### `wait_t`

```jai
wait_t :: struct {
    w_status: s32;

    #overlay (w_status) w_T: struct {
        __bitfield: u32;

    };
    #overlay (w_status) w_S: struct {
        __bitfield: u32;

    };
}
```
### `xsw_usage`

Variant 1 (macOS, ARM64):

```jai
xsw_usage :: struct {
    xsu_total:     u64;
    xsu_avail:     u64;
    xsu_used:      u64;
    xsu_pagesize:  u32;
    xsu_encrypted: s32;
}
```
Variant 2 (macOS, x64):

```jai
xsw_usage :: struct {
    xsu_total:     u64;
    xsu_avail:     u64;
    xsu_used:      u64;
    xsu_pagesize:  u32;
    xsu_encrypted: u32;
}
```
## Contracts and constraints

### Standard stream globals

`stdin`, `stdout`, and `stderr` are imported mutable external globals of type
`*FILE`: each denotes storage containing the current `FILE *` value, so reads
and assignments operate on the process-wide libc pointer variable. The foreign
binding must retain that lvalue/global identity; a procedure returning a
`FILE *`, a copied local pointer, or separate storage is not ABI-equivalent.
On Linux and Android the external identifiers are `stdin`, `stdout`, and
`stderr`. On macOS both ARM64 and x64, the public Jai identifiers map to the
SDK globals `__stdinp`, `__stdoutp`, and `__stderrp` respectively. The macOS
SDK declares these as `extern FILE *` globals and defines `stdin`/`stdout`/
`stderr` as preprocessor aliases to them. Jai's `#elsewhere libc "name"`
foreign-variable spelling provides the required external-name mapping; use
the C identifier shown above, leaving Mach-O symbol mangling to the linker.

### Linux call and architecture contract

On Linux, `USE_GNU` is a file-local compile-time constant set to `true`, not an exported identifier or import parameter. Consequently the conditional `ftello64` alias is absent. `stat`, `fstat`, and `lstat` are Jai wrappers returning the `s64` result of `syscall`; they preserve the libc `syscall` convention (nonnegative success, `-1` on failure with thread-local `errno` set), rather than returning a raw negative kernel errno. The output buffer is caller-owned and must point to a suitably sized `stat_t` for the selected ABI.

The Linux branch loads one OS-level binding set with no CPU-specific selection. Its syscall-number constants and records therefore have no declared ARM64 alternative in this surface. Do not treat the presence of an `OS == .LINUX` label as proof those Linux syscall numbers or layouts match ARM64. The macOS and Android branches do select ARM64 and x64 variants; the aggregate-shape entries retain those CPU labels. `__errno_location` is the public Jai name on both macOS CPUs and is linked to the Darwin SDK symbol `__error`.

This is a low-level collection of POSIX/Linux/Android/macOS ABI bindings selected by OS and CPU, including libc, pthread, resource, stdio, and syscall declarations. Most calls return native status/error values and use platform-specific structs, constants, syscall numbers, and layouts; callers must interpret `errno` immediately where required and must not assume one target’s constants/layout apply to another. Bindings cover multiple target architectures but are not a portable high-level wrapper; some declarations are generated or target-specific, and availability follows the selected OS/CPU files.
