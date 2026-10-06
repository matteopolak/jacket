// Body probe for genuine Basic-qualified selected-protocol type identity.
Basic :: #import "Basic";
Protocol :: #import "Preload";
main :: () -> s64 {
    allocator: Basic.Allocator;
    plain: Protocol.Allocator = allocator;
    back: Basic.Allocator = plain;
    descriptor: Basic.Type_Info = .{ .INTEGER, 8 };
    raw: Protocol.Type_Info = descriptor;
    if raw.runtime_size != 8 return 1;
    integer: Basic.Type_Info_Integer;
    selected: Protocol.Type_Info_Integer = integer;
    wrapper: Basic.Any_Struct;
    selected_wrapper: Protocol.Any_Struct = wrapper;
    return 0;
}
