#import "Basic";

// A pointer stored into a local whose address is taken writes the target's pointer width: the
// member after it keeps its value.
main :: () {
    record: struct {
        pointer: *u8;
        after: u32;
    } = .{ null, 0xAABBCCDD };

    keep := *record;

    record.pointer = null;
    print("% %\n", formatInt(record.after, base = 16), keep.pointer == null);
}
