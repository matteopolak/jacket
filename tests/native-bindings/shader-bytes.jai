// Independent owned-byte-copy safety. This tests no fabricated shader compiler.
#import "Basic";
Bytes :: #import "internal/bindings/shader-bytes";

main :: () {
    guarded: struct {
        before: u64 = 0x716;
        bytes: [5] u8 = .[1, 0, 2, 255, 0];
        after: u64 = 0x918;
    };

    copied := Bytes.copy(guarded.bytes.data, guarded.bytes.count);

    assert(copied.count == guarded.bytes.count);
    for byte, i: guarded.bytes assert(copied[i] == byte);

    guarded.bytes[0] = 73;
    assert(copied[0] == 1);
    assert(guarded.before == 0x716 && guarded.after == 0x918);
    free(copied);
    empty := Bytes.copy(null, 0);

    assert(empty.count == 0);
    free(empty);
}
