put :: (builder: *String_Builder, x: $T) #modify {
    k := type_info(T).type;
    return k == .INTEGER || k == .FLOAT || k == .BOOL || k == .ENUM, "Codex put requires a scalar";
} {
    append(builder, cast(*u8) *x, size_of(T));
}

get :: (s: *string, x: *$T) #modify {
    k := type_info(T).type;
    return k == .INTEGER || k == .FLOAT || k == .BOOL || k == .ENUM, "Codex get requires a scalar";
} {
    assert(s != null && x != null && s.count >= size_of(T), "Codex: short scalar input");
    memcpy(x, s.data, size_of(T));
    s.data += size_of(T);
    s.count -= size_of(T);
}

put :: (builder: *String_Builder, x: string) {
    put(builder, x.count);
    append(builder, x);
}

get :: (s: *string, result: *string, error: *bool) {
    assert(s != null && result != null && error != null);
    count: s64;

    if s.count < size_of(s64) {
        error.* = true;
        return;
    }

    get(s, *count);

    if count < 0 || count > s.count {
        error.* = true;
        return;
    }

    data := cast(*u8) alloc(count);
    if !data && count > 0 {
        error.* = true;
        return;
    }

    memcpy(data, s.data, count);
    result.* = .{ count, data };
    s.data += count;
    s.count -= count;
}

put_short_string :: (builder: *String_Builder, s: string, $count_type: Type, $zero_terminated := false) {
    #assert count_type == s8 || count_type == u8 || count_type == s16 || count_type == u16 || count_type == s32 || count_type == u32;
    count := cast(count_type) s.count;

    put(builder, count);
    append(builder, s);
    #if zero_terminated append(builder, cast(u8) 0);
}

get_short_string :: (input: *string, $count_type: Type, error: *bool, $allocate := true, $zero_terminated := false) -> string {
    #assert count_type == s8 || count_type == u8 || count_type == s16 || count_type == u16 || count_type == s32 || count_type == u32;
    assert(input != null && error != null);

    if input.count < size_of(count_type) {
        error.* = true;
        return .{};
    }

    count: count_type;

    get(input, *count);
    size := cast(s64) count;
    if size < 0 || size > input.count {
        error.* = true;
        return .{};
    }

    #if zero_terminated {
        if size == input.count || input.data[size] != 0 {
            error.* = true;
            return .{};
        }
    }

    result: string = .{ size, input.data };

    #if allocate {
        data := cast(*u8) alloc(size + (ifx zero_terminated then 1 else 0));
        if !data && (size > 0 || zero_terminated) {
            error.* = true;
            return .{};
        }

        memcpy(data, input.data, size);
        #if zero_terminated data[size] = 0;
        result.data = data;
    }

    input.data += size;
    input.count -= size;

    #if zero_terminated {
        input.data += 1;
        input.count -= 1;
    }

    return result;
}

#scope_file
#import "Basic";
