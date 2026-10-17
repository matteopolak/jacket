// Declaration order and storage dependencies are independent.
Packet :: struct {
    #overlay (payload) first: u64;
    #overlay (first) second: u32;
    tag: u32;
    payload: [2] u64;
    tail: u16;
}

Inner :: struct {
    prefix: u32;
    data: u64;
}

Promoted :: struct {
    #overlay (data) alias: u64;
    lead: u64;
    using inner: Inner;
    tail: u8;
}

Typed :: struct {
    #overlay (anchor) alias: type_of(anchor);
    anchor: u64;
    tail: u8;
}

Defaults :: struct {
    #overlay(anchor) alias: u64 = 17;
    anchor: u64;
}

Overrides :: struct {
    #overlay(anchor) alias: u64 = 17;
    anchor: u64 = 29;
}

TablePayload :: struct {
    count: u32;
    data: *void;
}

SdkShape :: struct {
    kind: u32;
    #overlay(payload) integers: [3] u32;
    payload: TablePayload;
    visibility: u32;
}

#assert size_of(SdkShape) == 32 && align_of(SdkShape) == 8;
#assert offset_of(SdkShape, "kind") == 0 && offset_of(SdkShape, "integers") == 8;
#assert offset_of(SdkShape, "payload") == 8 && offset_of(SdkShape, "visibility") == 24;

#assert offset_of(Packet, "first") == 8;
#assert offset_of(Packet, "second") == 8;
#assert offset_of(Packet, "tag") == 0;
#assert offset_of(Packet, "payload") == 8;
#assert offset_of(Packet, "tail") == 24;
#assert size_of(Packet) == 32;
#assert offset_of(Promoted, "alias") == 16;
#assert offset_of(Promoted, "tail") == 24;
#assert size_of(Promoted) == 32;
#assert offset_of(Typed, "tail") == 8;

#program_export "overlay_check"
overlay_check :: () -> s32 #c_call {
    value: Packet;
    value.payload[0] = 51;
    if value.first != 51 || value.second != 51 return 1;

    value.first = 73;
    if value.payload[0] != 73 || value.tag != 0 || value.tail != 0 return 2;

    promoted: Promoted;
    promoted.alias = 91;
    if promoted.inner.data != 91 || promoted.data != 91 || promoted.lead != 0 return 3;

    typed: Typed;
    typed.alias = 113;
    if typed.anchor != 113 || typed.tail != 0 return 4;

    defaults: Defaults;
    overrides: Overrides;

    if defaults.alias != 17 || defaults.anchor != 17 return 5;
    if overrides.alias != 29 || overrides.anchor != 29 return 6;

    return 0;
}

#assert overlay_check() == 0;
main :: () {}
