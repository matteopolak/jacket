// Independent interpretation of the platform mode and wait status contracts.
IFTODT :: (mode: mode_t) -> mode_t { return (mode >> 12) & 15; }
DTTOIF :: (dirtype: mode_t) -> mode_t { return dirtype << 12; }
S_ISDIR :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFDIR; }
S_ISCHR :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFCHR; }
S_ISBLK :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFBLK; }
S_ISREG :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFREG; }
S_ISFIFO :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFIFO; }
S_ISLNK :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFLNK; }
S_ISSOCK :: (mode: mode_t) -> bool { return (mode & S_IFMT) == S_IFSOCK; }

WEXITSTATUS :: (status: s32) -> s32 { return (status >> 8) & 255; }
WTERMSIG :: (status: s32) -> s32 { return status & 127; }
WIFEXITED :: (status: s32) -> bool { return WTERMSIG(status) == 0; }
WCOREDUMP :: (status: s32) -> s32 { return status & 128; }

#if OS == .MACOS {
    WSTATUS :: (status: s32) -> s32 { return WTERMSIG(status); }
    WSTOPSIG :: (status: s32) -> s32 { return status >> 8; }
    WIFSIGNALED :: (status: s32) -> bool {
        signal := WTERMSIG(status);
        return signal > 0 && signal < 127;
    }
    WIFSTOPPED :: (status: s32) -> bool {
        return WTERMSIG(status) == 127 && WSTOPSIG(status) != 19;
    }
    WIFCONTINUED :: (status: s32) -> bool {
        return WTERMSIG(status) == 127 && WSTOPSIG(status) == 19;
    }
} else {
    WSTOPSIG :: (status: s32) -> s32 { return WEXITSTATUS(status); }
    WIFSIGNALED :: (status: s32) -> bool {
        signal := WTERMSIG(status);
        return signal > 0 && signal < 127;
    }
    WIFSTOPPED :: (status: s32) -> bool { return (status & 255) == 127; }
    WIFCONTINUED :: (status: s32) -> bool { return status == 65535; }
}
