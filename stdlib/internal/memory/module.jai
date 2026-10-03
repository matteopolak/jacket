move_bytes :: (destination: *void, source: *void, count: s64) {
    dest := cast(*u8) destination;
    src := cast(*u8) source;

    if dest == src || count == 0 return;

    if cast(u64) dest > cast(u64) src {
        i := count;
        while i > 0 {
            i -= 1;
            dest[i] = src[i];
        }
    } else {
        i: s64 = 0;
        while i < count {
            dest[i] = src[i];
            i += 1;
        }
    }
}
