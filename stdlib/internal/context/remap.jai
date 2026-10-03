#module_parameters(VERBOSE := true);

Remap_Context :: () #expand {
    mapped := remap_context();
    `push_context, defer_pop mapped;
}

remap_context :: () -> (result: #Context) {
    if context.context_info == type_info(#Context) return context;
    return remap_context(*context, context.context_info);
}

remap_context :: (foreign_context: *void, foreign_type: *Type_Info_Struct) -> (result: #Context) {
    result: #Context;
    result.context_info = type_info(#Context);

    if !foreign_context || !valid_base(foreign_type) {
        diagnostic("Remap_Context: foreign context requires a leading base struct\n");
        return result;
    }

    plan := get_plan(foreign_type);
    entry := plan.entries;

    while entry {
        memcpy(cast(*u8) *result + entry.destination, cast(*u8) foreign_context + entry.source, entry.bytes);
        entry = entry.next;
    }

    result.context_info = type_info(#Context);

    return result;
}

#scope_file
#import "Basic";

Copy_Entry :: struct {
    source: s64;
    destination: s64;
    bytes: s64;
    next: *Copy_Entry;
}

Copy_Plan :: struct {
    foreign_type: *Type_Info_Struct;
    entries: *Copy_Entry;
    next: *Copy_Plan;
}

plans: *Copy_Plan;
plan_lock: s32;

diagnostic :: (message: string) {
    #if VERBOSE context.logger(message, context.logger_data, Log_Info.{ common_flags = .WARNING });
}

valid_base :: (info: *Type_Info_Struct) -> bool {
    if !info || info.type != .STRUCT || info.members.count == 0 return false;
    first := info.members[0];
    return first.name == "base" && stored(first) && first.type && first.type.type == .STRUCT && valid_extent(first, info.runtime_size);
}

stored :: (member: Type_Info_Struct_Member) -> bool {
    return (member.flags & (.CONSTANT | .OVERLAY)) == 0;
}

valid_extent :: (member: Type_Info_Struct_Member, size: s64) -> bool {
    if !member.type || member.type.runtime_size < 0 || member.offset_in_bytes < 0 || member.offset_in_bytes > size return false;
    return member.type.runtime_size <= size - member.offset_in_bytes;
}

find_destination :: (info: *Type_Info_Struct, name: string) -> (offset: s64, bytes: s64, found: bool) {
    for member: info.members {
        if stored(member) && member.name == name && valid_extent(member, info.runtime_size)
            return member.offset_in_bytes, member.type.runtime_size, true;
    }

    if valid_base(info) {
        base := info.members[0];
        nested := cast(*Type_Info_Struct) base.type;

        for member: nested.members {
            if stored(member) && member.name == name && valid_extent(member, nested.runtime_size)
                return base.offset_in_bytes + member.offset_in_bytes, member.type.runtime_size, true;
        }
    }

    return 0, 0, false;
}

cache_allocate :: ($T: Type) -> *T {
    defaults: #Context;
    allocator := defaults.allocator;
    memory := cast(*T) allocator.proc(.ALLOCATE, size_of(T), 0, null, allocator.data);

    assert(memory != null, "context mapping cache allocation failed");
    memory.* = .{};

    return memory;
}

add_fields :: (plan: *Copy_Plan, source: *Type_Info_Struct, base_offset: s64, skip_base: bool) {
    own := type_info(#Context);

    for member: source.members {
        if !stored(member) || (skip_base && member.name == "base") continue;

        if !valid_extent(member, source.runtime_size) {
            diagnostic("Remap_Context: invalid source member extent\n");
            continue;
        }

        offset, bytes, found := find_destination(own, member.name);
        if !found || bytes != member.type.runtime_size continue;

        entry := cache_allocate(Copy_Entry);
        entry.source = base_offset + member.offset_in_bytes;
        entry.destination = offset;
        entry.bytes = bytes;
        entry.next = plan.entries;
        plan.entries = entry;
    }
}

get_plan :: (foreign_type: *Type_Info_Struct) -> *Copy_Plan {
    acquired := compare_and_swap(*plan_lock, cast(s32) 0, cast(s32) 1);
    while !acquired acquired = compare_and_swap(*plan_lock, cast(s32) 0, cast(s32) 1);

    defer {
        released := compare_and_swap(*plan_lock, cast(s32) 1, cast(s32) 0);
        assert(released);
    }

    cursor := plans;
    while cursor {
        if cursor.foreign_type == foreign_type return cursor;
        cursor = cursor.next;
    }

    plan := cache_allocate(Copy_Plan);
    plan.foreign_type = foreign_type;
    base := foreign_type.members[0];

    add_fields(plan, cast(*Type_Info_Struct) base.type, base.offset_in_bytes, false);
    add_fields(plan, foreign_type, 0, true);
    plan.next = plans;
    plans = plan;

    return plan;
}
