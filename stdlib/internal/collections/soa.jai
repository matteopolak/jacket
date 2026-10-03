Soa :: struct(T: Type, N: s64) #modify {
    return (cast(*Type_Info) T).type == .STRUCT && N >= 0, "Soa requires a struct and nonnegative extent";
} {
    #insert #run make_fields(T);
}

operator [] :: (soa: $S/Soa, i: int) -> S.T {
    assert(i >= 0 && i < S.N);
    result: S.T;
    #insert #run make_access(S.T, false);
    return result;
}

operator []= :: (array: *$S/Soa, index: int, value: S.T) {
    assert(index >= 0 && index < S.N);
    #insert #run make_access(S.T, true);
}

for_expansion :: (array: *$S/Soa, body: Code, flags: For_Flags) #expand {
    assert((flags & ~For_Flags.POINTER) == 0, "reverse Soa iteration is unsupported");

    for index: 0..S.N - 1 {
        value := array.*[index];

        `it_index := index;

        #if (flags & .POINTER) != 0 {
            defer array.*[index] = value;
            `it := *value;
            #insert body;
        } else {
            `it := value;
            #insert body;
        }
    }
}

example :: () {
    values: Soa(Example_Item, 2);
    values[0] = Example_Item.{ 17, 34 };
    item := values[0];
    assert(item.x + item.y == 51);
}

#scope_file
#import "Basic";

Example_Item :: struct {
    x: int;
    y: int;
}

stored :: (member: Type_Info_Struct_Member) -> bool {
    return (member.flags & (.CONSTANT | .IMPORTED | .OVERLAY)) == 0;
}

selected :: (info: *Type_Info_Struct, index: s64) -> bool {
    member := info.members[index];
    if !stored(member) return false;

    assert(member.type.runtime_size >= 0 && member.offset_in_bytes >= 0);
    start := member.offset_in_bytes;
    end := start + member.type.runtime_size;

    for previous: 0..index - 1 {
        other := info.members[previous];
        if !stored(other) continue;
        if start < other.offset_in_bytes + other.type.runtime_size && other.offset_in_bytes < end return false;
    }

    return true;
}

make_fields :: ($T: Type) -> string {
    info := cast(*Type_Info_Struct) type_info(T);
    builder: String_Builder;

    defer free_buffers(*builder);

    for member, index: info.members if selected(info, index) {
        append(*builder, member.name);
        append(*builder, ": [N] type_of(T.");
        append(*builder, member.name);
        append(*builder, ");\n");
    }

    return builder_to_string(*builder);
}

make_access :: ($T: Type, write: bool) -> string {
    info := cast(*Type_Info_Struct) type_info(T);
    builder: String_Builder;

    defer free_buffers(*builder);

    for member, index: info.members if selected(info, index) {
        if write {
            append(*builder, "array.");
            append(*builder, member.name);
            append(*builder, "[index] = value.");
            append(*builder, member.name);
            append(*builder, ";\n");
        } else {
            append(*builder, "result.");
            append(*builder, member.name);
            append(*builder, " = soa.");
            append(*builder, member.name);
            append(*builder, "[i];\n");
        }
    }

    return builder_to_string(*builder);
}
