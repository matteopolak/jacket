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

// Nothing says the text is the live member.
text_of :: (v: *Value) -> string {
    return v.text;
}

// The check is on another value.
second :: (a: Value, b: Value) -> int {
    if a.kind == .INTEGER return b.integer;
    return 0;
}

// A union passed in: which member was written is decided elsewhere.
float_of :: (b: Bits) -> float32 {
    return b.f;
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

// The key is not part of what every event starts with.
key_of :: (e: *Event) -> u32 {
    return e.key.key;
}

// Read through a member path.
Holder :: struct {
    value: Value;
}

held_text :: (h: Holder) -> string {
    return h.value.text;
}

main :: () {
    v: Value;
    v.kind = .TEXT;
    v.text = "hi";
    print("% % %\n", text_of(*v), second(v, v), float_of(.{ u = 1 }));
    e: Event;
    e.type = 1;
    print("% %\n", held_text(.{ value = v }), key_of(*e));
}
