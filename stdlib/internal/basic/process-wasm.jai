exit :: (status: s32) {
    Wasi.exit(status);
}

sleep_milliseconds :: (milliseconds: s32) {
    assert(milliseconds >= 0);
    requested: Wasi.timespec;
    requested.tv_sec = milliseconds / 1000;
    requested.tv_nsec = cast(s64) (milliseconds % 1000) * 1000000;
    Wasi.nanosleep(*requested, null);
}

#scope_file
Wasi :: #import "internal/files/wasi-libc";
