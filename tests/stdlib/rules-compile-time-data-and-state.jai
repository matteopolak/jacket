// rules: ctdata.1 ctdata.2 ctdata.3 ctdata.4 ctdata.6 ctdata.7 ctdata.8 ctdata.9 ctdata.11 ctdata.12 ctdata.13 ctdata.14
// Pins the claims of docs/metaprogramming/compile-time-data-and-state.md.
#import "Basic";
#import "Compiler";

// ctdata.1: scalars, strings and types computed by #run become constants.
seven :: () -> int {
    return 3 + 4;
}

greeting :: () -> string {
    return sprint("hello %", 5);
}

pick_type :: () -> Type {
    return float64;
}

SEVEN :: #run seven();
GREETING :: #run greeting();
PICKED :: #run pick_type();
#assert SEVEN == 7;
#assert PICKED == float64;

// ctdata.2: a pointer into a global stays a pointer to that global.
anchor: [4] int;

point_at_anchor :: () -> *int {
    return *anchor[2];
}

ANCHOR_PTR :: #run point_at_anchor();

// ctdata.3: heap memory reached from the result is copied along, recursively.
Link :: struct {
    value: int;
    next: *Link;
}

build_chain :: () -> *Link {
    first := New(Link);
    first.value = 10;
    first.next = New(Link);
    first.next.value = 20;
    first.next.next = New(Link);
    first.next.next.value = 30;
    return first;
}

CHAIN :: #run build_chain();

// ctdata.4: small integers cast to pointers keep their numeric value.
small_pointer :: () -> *u8 {
    return cast(*u8) 0x40;
}

SMALL_PTR :: #run small_pointer();

// ctdata.14: a sign-extended handle (top bit set) cast to a pointer keeps its value too.
Opaque_Handle :: *struct {
    unused: u8;
};

TOP_BIT_HANDLE :: cast(Opaque_Handle) (cast(s64) (cast,trunc(s32) 0x80000000));

// ctdata.6: union members are copied as raw bytes.
Either :: union {
    as_pointer: *int;
    as_number: s64;
}

Holder :: struct {
    tag: int;
    payload: Either;
}

make_holder :: () -> Holder {
    h: Holder;
    h.tag = 2;
    h.payload.as_number = 0x1234_5678;
    return h;
}

HOLDER :: #run make_holder();

// ctdata.7: the doc's example (a struct holding a string and an array built with array_add).
Item :: struct {
    name: string;
    tags: [] int;
}

make_item :: () -> Item {
    arr: [..] int;
    for 1..4 array_add(*arr, it * it);
    return .{ "sq", arr };
}

TABLE :: #run make_item();

// ctdata.8 / ctdata.9 / ctdata.11: compile-time writes to globals are undone before main,
// except for #no_reset globals and initializers that are themselves #run results.
counter := 5;
#no_reset keep := 0;

#run {
    counter = 99;
    keep = 7;
}

calls := 0;

counted :: () -> int {
    calls += 1;
    return 64;
}

initialized := #run counted();

Marker_Type :: struct {
    a: u32;
    b: u16;
}

marker_global: Marker_Type;

main :: () {
    // ctdata.1
    assert(SEVEN == 7, "ctdata.1: int constant");
    assert(GREETING == "hello 5", "ctdata.1: string constant");
    assert(PICKED == float64, "ctdata.1: Type constant");

    // ctdata.2
    assert(ANCHOR_PTR == *anchor[2], "ctdata.2: pointer into a global");
    anchor[2] = 55;
    assert(ANCHOR_PTR.* == 55, "ctdata.2: the pointer aliases the live global");

    // ctdata.3
    assert(CHAIN.value == 10, "ctdata.3: first node");
    assert(CHAIN.next.value == 20, "ctdata.3: second node");
    assert(CHAIN.next.next.value == 30, "ctdata.3: third node");
    assert(CHAIN.next.next.next == null, "ctdata.3: chain ends");

    // ctdata.4
    assert(cast(u64) SMALL_PTR == 0x40, "ctdata.4: small pointer value kept");

    // ctdata.14
    assert(cast(u64) TOP_BIT_HANDLE == 0xFFFF_FFFF_8000_0000, "ctdata.14: sign-extended handle kept");

    // ctdata.6
    assert(HOLDER.tag == 2, "ctdata.6: plain field");
    assert(HOLDER.payload.as_number == 0x1234_5678, "ctdata.6: union bytes kept as written");

    // ctdata.7
    assert(TABLE.name == "sq", "ctdata.7: name");
    assert(TABLE.tags.count == 4, "ctdata.7: tags count");
    assert(TABLE.tags[0] == 1 && TABLE.tags[1] == 4 && TABLE.tags[2] == 9 && TABLE.tags[3] == 16,
           "ctdata.7: tags");
    assert(tprint("% %", TABLE.name, TABLE.tags) == "sq [1, 4, 9, 16]", "ctdata.7: printed form");

    // ctdata.8 / ctdata.11
    assert(counter == 5, "ctdata.8: compile-time write to a global is undone");
    assert(keep == 7, "ctdata.11: #no_reset global keeps the compile-time write");
    assert(tprint("counter = %, keep = %", counter, keep) == "counter = 5, keep = 7", "ctdata.11");

    // ctdata.9
    assert(initialized == 64, "ctdata.9: #run initializer keeps its value");
    assert(calls == 0, "ctdata.9: the side effect of the #run is undone");

    // ctdata.12: every type the program uses has a descriptor in the type table.
    info := get_runtime_info();
    assert(info.type_table.count > 0, "ctdata.12: the type table is filled");
    found := false;
    for info.type_table if it == type_info(Marker_Type) found = true;
    assert(found, "ctdata.12: user struct is in the type table");
    found_int := false;
    for info.type_table if it == type_info(int) found_int = true;
    assert(found_int, "ctdata.12: int is in the type table");

    // ctdata.13: globals live inside one of the reported data segments.
    assert(info.global_data_info != null, "ctdata.13: global data info exists");
    segments := info.global_data_info.segment_info;
    assert(segments.count > 0, "ctdata.13: there are data segments");
    address := cast(u64) *marker_global;
    inside := false;
    for segments {
        start := cast(u64) it.data.data;
        if address >= start && address < start + cast(u64) it.data.count inside = true;
    }
    assert(inside, "ctdata.13: a global lies inside a reported segment");
}
