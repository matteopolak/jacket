// Independent compatibility wrappers over the public modern libc stat API.
// The declared stat_t layout is the supplied Linux x64 ABI contract.
#if OS == .LINUX {
    stat :: (path: *u8, buf: *stat_t) -> s64 {
        return cast(s64) sdk_stat(path, buf);
    }
    fstat :: (fd: s32, buf: *stat_t) -> s64 {
        return cast(s64) sdk_fstat(fd, buf);
    }
    lstat :: (path: *u8, buf: *stat_t) -> s64 {
        return cast(s64) sdk_lstat(path, buf);
    }

    #scope_file
    sdk_stat :: (path: *u8, buf: *stat_t) -> s32 #foreign sdk_libc "stat";
    sdk_fstat :: (fd: s32, buf: *stat_t) -> s32 #foreign sdk_libc "fstat";
    sdk_lstat :: (path: *u8, buf: *stat_t) -> s32 #foreign sdk_libc "lstat";
    sdk_libc :: #system_library "libc";
}
