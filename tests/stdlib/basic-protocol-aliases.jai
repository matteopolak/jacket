// Protocol types are provided by Preload; Basic does not add aliases for them.
// jailint: allow-file(unused_variable) -- the declarations only type-check public type identity.
Protocol :: #import "Preload";

main :: () -> s64 {
    allocator: Protocol.Allocator;
    plain: Protocol.Allocator = allocator;
    back: Protocol.Allocator = plain;
    descriptor: Protocol.Type_Info = .{ .INTEGER, 8 };
    raw: Protocol.Type_Info = descriptor;

    if raw.runtime_size != 8 return 1;

    integer: Protocol.Type_Info_Integer;
    selected: Protocol.Type_Info_Integer = integer;
    wrapper: Protocol.Any_Struct;
    selected_wrapper: Protocol.Any_Struct = wrapper;

    return 0;
}
