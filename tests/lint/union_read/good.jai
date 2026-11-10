#import "Basic";

Value_Kind :: enum {
    INTEGER;
    TEXT;
}

Value :: struct {
    kind: Value_Kind;
    union {
        integer: int;
        text: string;
    }
}

Bits :: union {
    f: float32;
    u: u32;
}

// Inside a branch on the tag.
integer_of :: (v: Value) -> int {
    if v.kind == .INTEGER return v.integer;
    return 0;
}

// In a case of a switch on the tag.
describe :: (v: *Value) -> string {
    if #complete v.kind == {
        case .INTEGER; return tprint("%", v.integer);
        case .TEXT; return v.text;
    }

    return "";
}

// In `ifx` and after `&&`.
length :: (v: Value) -> int {
    if v.kind == .TEXT && v.text.count > 0 return v.text.count;
    return ifx v.kind == .INTEGER then v.integer else 0;
}

// A predicate that looks at the tag counts as the check.
is_text :: (v: Value) -> bool {
    return v.kind == .TEXT;
}

text_if_any :: (v: Value) -> string {
    if is_text(v) return v.text;
    return "";
}

// After an early exit or an assert on the tag.
text_or_empty :: (v: Value) -> string {
    if v.kind != .TEXT return "";
    return v.text;
}

asserted :: (v: Value) -> string {
    assert(v.kind == .TEXT);
    return v.text;
}

// Written here, then read: the live member is in view (a pun is deliberate).
bits_of :: (f: float32) -> u32 {
    b: Bits;
    b.f = f;
    return b.u;
}

// A C-style event: every member starts with the same fields.
Any_Event :: struct {
    type: s32;
    window: u64;
}

Key_Event :: struct {
    type: s32;
    window: u64;
    key: u32;
}

Event :: union {
    type: s32;
    any: Any_Event;
    key: Key_Event;
    pad: [8] s64;
}

next_event :: (e: *Event) {
    e.type = 1;
}

// What every member starts with reads the same whichever was written, and serves as the tag.
window_of :: (e: Event) -> u64 {
    return e.any.window;
}

checked_key :: (e: *Event) -> u32 {
    if e.type == 1 return e.key.key;
    return 0;
}

// Filled by a call it was passed to.
next_key :: () -> u32 {
    e: Event;
    next_event(*e);
    return e.key.key;
}

// Writes and addresses are not reads.
set :: (v: *Value) {
    v.kind = .TEXT;
    v.text = "set";
    p := *v.integer;
    p.* = 0;
    v.integer += 1;
}

main :: () {
    v: Value;

    set(*v);
    print("% % % %\n", integer_of(v), describe(*v), length(v), text_or_empty(v));
    e: Event;

    next_event(*e);
    print("% % % %\n", asserted(v), bits_of(1), window_of(e), text_if_any(v));
    print("% %\n", checked_key(*e), next_key());
}
