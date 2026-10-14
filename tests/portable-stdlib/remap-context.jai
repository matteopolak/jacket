#import "Basic";
R :: #import "Remap_Context"(VERBOSE = false);

#add_context remap_number: s64 = 7;
#add_context remap_different_size: s64 = 99;
#add_context remap_pointer: *s64;

Foreign_Base :: struct {
    thread_index: u32;
    allocator: Allocator;
}

Foreign_Context :: struct {
    base: Foreign_Base;
    remap_number: s64;
    remap_different_size: u8;
    remap_pointer: *s64;
    unmatched: s64;
}

main :: () {
    marker: s64 = 123;
    foreign: Foreign_Context;
    foreign.base.thread_index = 81;
    foreign.base.allocator = context.allocator;
    foreign.remap_number = 42;
    foreign.remap_different_size = 1;
    foreign.remap_pointer = *marker;
    result := R.remap_context(*foreign, type_info(Foreign_Context));

    assert(result.thread_index == 81 && result.remap_number == 42);
    assert(result.remap_different_size == 99 && result.remap_pointer == *marker);
    assert(result.context_info == type_info(#Context));
    assert(result.allocator.proc == context.allocator.proc);
    foreign.remap_number = -17;
    result = R.remap_context(*foreign, type_info(Foreign_Context));
    assert(result.remap_number == -17);
    context.remap_number = 55;
    result = R.remap_context();
    assert(result.remap_number == 55 && result.context_info == type_info(#Context));
    print("ok\n");
}
