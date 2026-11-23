// A foreign context keeps the allocator and thread index it was given, and allocations through that allocator work.
#import "Basic";
R :: #import "Remap_Context"(VERBOSE = false);

#add_context remap_number: s64 = 7;

Foreign_Base :: struct {
    thread_index: u32;
    allocator: Allocator;
}

Foreign_Context :: struct {
    base: Foreign_Base;
    remap_number: s64;
}

Other_Context :: struct {
    base: Foreign_Base;
    remap_number: s64;
    unknown_field: [4] u8;
}

main :: () {
    foreign: Foreign_Context;
    foreign.base.thread_index = 3;
    foreign.base.allocator = temp;
    foreign.remap_number = 12;

    result := R.remap_context(*foreign, type_info(Foreign_Context));

    print("thread=% number=% allocator_proc_kept=% allocator_data_kept=%\n", result.thread_index, result.remap_number, result.allocator.proc == temp.proc, result.allocator.data == temp.data);
    assert(result.thread_index == 3 && result.remap_number == 12);
    assert(result.allocator.proc == temp.proc && result.allocator.data == temp.data);
    assert(result.context_info == type_info(#Context));

    // Memory from the kept allocator comes from the temporary storage, so it is valid until the reset.
    value: *s64;

    push_context {
        context.allocator = result.allocator;
        value = New(s64);
    }

    value.* = 77;
    print("value through kept allocator=%\n", value.*);
    assert(value.* == 77);

    // A second foreign struct with an extra field: the extra field is ignored, the allocator is still kept.
    other: Other_Context;
    other.base.allocator = temp;
    other.remap_number = -4;
    other_result := R.remap_context(*other, type_info(Other_Context));

    print("other number=% allocator_kept=%\n", other_result.remap_number, other_result.allocator.proc == temp.proc);
    assert(other_result.remap_number == -4 && other_result.allocator.proc == temp.proc);

    // With no foreign value, remap_context() keeps the allocator of the active context.
    push_context {
        context.allocator = temp;
        active := R.remap_context();
        print("active allocator kept=% context_type_ok=%\n", active.allocator.proc == temp.proc, active.context_info == type_info(#Context));
        assert(active.allocator.proc == temp.proc && active.context_info == type_info(#Context));
    }

    print("ok\n");
}
