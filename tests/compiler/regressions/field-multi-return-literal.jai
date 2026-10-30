// A procedure literal stored in a struct-literal field may return several values.
// rules: proc.13
#import "Basic";

Pad :: struct {
    id: int;
}

Handler :: struct {
    proc: (data: *u8) -> bool, Pad;
    other: int;
}

make :: () -> Handler {
    return Handler.{
        proc = (data: *u8) -> bool, Pad {
            return true, .{ 1 };
        },
        other = 3,
    };
}

main :: () {
    h := make();
    ok, pad := h.proc(null);
    assert(ok && pad.id == 1 && h.other == 3);
}
