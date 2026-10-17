P :: #import "POSIX";

libc :: #library,system "c";
abort :: () #foreign libc;
process_id :: () -> s32 #foreign libc "getpid";
make_path :: (buffer: *u8, capacity: u64, format: *u8, args: ..Any) -> s32 #foreign libc "snprintf";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    #if OS == .LINUX && CPU == .X64 {
        #assert size_of(P.stat_t) == 144;
        #assert offset_of(P.stat_t, "st_mode") == 24;
        #assert P.SYS_stat == 4 && P.SYS_fstat == 5 && P.SYS_lstat == 6;
    } else #if OS == .LINUX {
        // The generic kernel layout arm64 uses.
        #assert size_of(P.stat_t) == 128;
        #assert offset_of(P.stat_t, "st_mode") == 16;
    }

    pid := process_id();
    name_buffer, link_buffer, target_buffer: [128] u8;
    name_count := make_path(name_buffer.data, 128, ".work/posix-stat-%d.txt", pid);
    link_count := make_path(link_buffer.data, 128, ".work/posix-stat-%d-link", pid);
    target_count := make_path(target_buffer.data, 128, "posix-stat-%d.txt", pid);

    check(name_count > 0 && name_count < 128 && link_count > 0 && link_count < 128 && target_count > 0 && target_count < 128);
    name: string = .{ data = name_buffer.data, count = name_count };
    fd := P.open(name.data, P.O_RDWR | P.O_CREAT | P.O_TRUNC, cast(P.mode_t) (P.S_IRUSR | P.S_IWUSR));

    check(fd >= 0);
    defer check(P.close(fd) == 0);
    defer check(P.unlink(name.data) == 0);
    bytes := "native file status";

    check(P.write(fd, bytes.data, cast(u64) bytes.count) == bytes.count);
    status, by_fd: P.stat_t;

    check(P.stat(name.data, *status) == 0);
    check(status.st_size == bytes.count && P.S_ISREG(cast(P.mode_t) status.st_mode));
    check(status.st_nlink >= 1);
    check(P.fstat(fd, *by_fd) == 0);
    check(by_fd.st_size == status.st_size && by_fd.st_ino == status.st_ino && by_fd.st_dev == status.st_dev);

    link: string = .{ data = link_buffer.data, count = link_count };

    check(P.symlink(target_buffer.data, link.data) == 0);
    defer check(P.unlink(link.data) == 0);
    symbolic, followed: P.stat_t;

    check(P.lstat(link.data, *symbolic) == 0);
    check(P.S_ISLNK(cast(P.mode_t) symbolic.st_mode));
    check(P.stat(link.data, *followed) == 0);
    check(P.S_ISREG(cast(P.mode_t) followed.st_mode) && followed.st_size == bytes.count);

    unchanged: P.stat_t;
    unchanged.st_size = 0x1234_5678;
    check(P.stat(".work/no-such-stat-file", *unchanged) == -1);
    check(P.errno() == P.ENOENT && unchanged.st_size == 0x1234_5678);
    check(P.fstat(-1, *unchanged) == -1);
    check(P.errno() == P.EBADF && unchanged.st_size == 0x1234_5678);
    check(P.stat(name.data, null) == -1 && P.errno() == P.EFAULT);
}
