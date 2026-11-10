// The `memcpy` intrinsic is a move on every backend (LLVM `memmove`, the interpreter's `ptr::copy`),
// so overlapping ranges are safe and a copy runs at the machine's speed rather than byte by byte.
move_bytes :: (destination: *void, source: *void, count: s64) {
    if destination == source || count == 0 return;

    memcpy(destination, source, count);
}
