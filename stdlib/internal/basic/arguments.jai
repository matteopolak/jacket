// The program's command-line arguments, starting with its own path. The slice is allocated; free it with `free` when
// done with it. For parsing options, use `Command_Line`.
//
// ```jai,no_run
// args := get_command_line_arguments();
// defer free(args.data);
// for args print("%\n", it);
// ```
get_command_line_arguments :: () -> [] string {
    count := __command_line_arguments.count;
    if count <= 0 return .{};

    result: [] string = .{ count, cast(*string) alloc(count * size_of(string)) };

    assert(result.data != null);
    for i: 0..count - 1 result[i] = to_string(__command_line_arguments[i]);

    return result;
}
