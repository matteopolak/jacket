// An aggregate parameter is the caller's value, read in place: a pointer to one of its members
// points into the caller's variable. Only `*p` of the whole parameter gives the procedure its
// own copy (aggregate-parameter-copy.jai).
// rules: proc.19
#import "Basic";

Tagged :: struct {
    kind: int;
    bytes: [16] u8;
}

outer: *Tagged;

payload :: (t: Tagged) -> *string {
    return cast(*string) t.bytes.data;
}

member_address :: (t: Tagged) -> bool {
    return *t.kind == *outer.kind && *t.bytes == *outer.bytes;
}

element_addresses :: (t: Tagged) -> bool {
    for *t.bytes if it != *outer.bytes[it_index] return false;
    return true;
}

reads_after_caller_write :: (t: Tagged) -> int {
    outer.kind = 9;
    return t.kind;
}

whole_address :: (t: Tagged) -> bool {
    p := *t;
    q := *t;

    p.kind = 5;
    outer.kind = 7;

    return p == q && t.kind == 5 && p != outer;
}

main :: () {
    t: Tagged;

    outer = *t;
    (cast(*string) t.bytes.data).* = "Hello";

    s := payload(t);

    assert(s.* == "Hello");
    assert(cast(*void) s == cast(*void) t.bytes.data);
    assert(member_address(t));
    assert(element_addresses(t));
    assert(reads_after_caller_write(t) == 9);

    t.kind = 1;
    assert(whole_address(t));
    assert(t.kind == 7);
}
