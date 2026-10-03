#assert OS == .LINUX && CPU == .ARM64, "This private kernel ABI requires Linux ARM64";

#load "abi.jai";

#assert size_of(Kernel_Stat) == 128;

read_path :: (path: *u8, result: *Kernel_Stat, no_follow: bool) -> s64 {
    flags: s32 = ifx no_follow then NOFOLLOW else 0;
    return native_syscall(SYS_FSTATAT, cast(s32) CURRENT_DIRECTORY, path, result, flags);
}

read_fd :: (fd: s32, result: *Kernel_Stat) -> s64 {
    return native_syscall(SYS_FSTAT, fd, result);
}

#scope_file
libc :: #library,system "c";
native_syscall :: (number: s64, args: ..Any) -> s64 #foreign libc "syscall";
