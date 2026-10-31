// Appending raw bytes to a growing buffer.

#scope_module

// Appends `count` bytes from `source` to `to`, doubling its room when it is full.
append_bytes :: (to: *[..] u8, source: *u8, count: int) {
    if count <= 0 return;

    needed := to.count + count;

    if needed > to.allocated {
        grown := to.allocated * 2;
        if grown < needed grown = needed;
        if grown < 512 grown = 512;

        array_reserve(to, grown);
    }

    memcpy(to.data + to.count, source, count);
    to.count = needed;
}
