// Preserve the reviewed Linux public record/constants. ARM64 uses a complete
// private kernel record and syscall definitions, then explicitly marshals it.
#if OS == .LINUX {
    stat :: (path: *u8, buf: *stat_t) -> s64 {
        #if CPU == .ARM64 {
            if !buf return Kernel.read_path(path, null, false);

            native: Kernel.Kernel_Stat;
            result := Kernel.read_path(path, *native, false);

            if result == 0 binding_copy_kernel_stat(buf, native);

            return result;
        } else {
            return syscall(SYS_stat, path, buf);
        }
    }

    lstat :: (path: *u8, buf: *stat_t) -> s64 {
        #if CPU == .ARM64 {
            if !buf return Kernel.read_path(path, null, true);

            native: Kernel.Kernel_Stat;
            result := Kernel.read_path(path, *native, true);

            if result == 0 binding_copy_kernel_stat(buf, native);

            return result;
        } else {
            return syscall(SYS_lstat, path, buf);
        }
    }

    fstat :: (fd: s32, buf: *stat_t) -> s64 {
        #if CPU == .ARM64 {
            if !buf return Kernel.read_fd(fd, null);

            native: Kernel.Kernel_Stat;
            result := Kernel.read_fd(fd, *native);

            if result == 0 binding_copy_kernel_stat(buf, native);

            return result;
        } else {
            return syscall(SYS_fstat, fd, buf);
        }
    }
}

#scope_file
#if OS == .LINUX && CPU == .ARM64 {
    Kernel :: #import "internal/bindings/linux-arm64-kernel";

    binding_copy_kernel_stat :: (destination: *stat_t, native: Kernel.Kernel_Stat) {
        converted: stat_t;
        converted.st_dev = native.st_dev;
        converted.st_ino = native.st_ino;
        converted.st_nlink = cast(u64) native.st_nlink;
        converted.st_mode = native.st_mode;
        converted.st_uid = native.st_uid;
        converted.st_gid = native.st_gid;
        converted.st_rdev = native.st_rdev;
        converted.st_size = native.st_size;
        converted.st_blksize = cast(s64) native.st_blksize;
        converted.st_blocks = native.st_blocks;
        converted.st_atime.tv_sec = native.st_atime;
        converted.st_atime.tv_nsec = cast(s64) native.st_atime_nsec;
        converted.st_mtime.tv_sec = native.st_mtime;
        converted.st_mtime.tv_nsec = cast(s64) native.st_mtime_nsec;
        converted.st_ctime.tv_sec = native.st_ctime;
        converted.st_ctime.tv_nsec = cast(s64) native.st_ctime_nsec;
        <<destination = converted;
    }
}
