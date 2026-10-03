// Generated only from pinned official Linux ARM64 UAPI C headers.
// Archive SHA-256: 460f484a244bdf3bfd5a683e6ee8dbde1c15b0489dbb65608969b6ece7c9751d

Kernel_Stat :: struct {
    st_dev: u64;
    st_ino: u64;
    st_mode: u32;
    st_nlink: u32;
    st_uid: u32;
    st_gid: u32;
    st_rdev: u64;
    __pad1: u64;
    st_size: s64;
    st_blksize: s32;
    __pad2: s32;
    st_blocks: s64;
    st_atime: s64;
    st_atime_nsec: u64;
    st_mtime: s64;
    st_mtime_nsec: u64;
    st_ctime: s64;
    st_ctime_nsec: u64;
    __unused4: u32;
    __unused5: u32;
}

SYS_FSTATAT :: 79;
SYS_FSTAT :: 80;
CURRENT_DIRECTORY :: -100;
NOFOLLOW :: 256;
