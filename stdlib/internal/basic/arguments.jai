get_command_line_arguments :: () -> [] string {
    count := __command_line_arguments.count;
    if count <= 0 return .{};

    result: [] string = .{ count, cast(*string) alloc(count * size_of(string)) };

    assert(result.data != null);
    for i: 0..count - 1 result[i] = to_string(__command_line_arguments[i]);

    return result;
}
