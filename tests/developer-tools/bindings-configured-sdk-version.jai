#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2 && args[1].count > 0);
    native := B.clang_getClangVersion();

    defer B.clang_disposeString(native);
    version := to_string(B.clang_getCString(native));
    prefix := tprint("clang version %", args[1]);
    matched := false;

    for index: 0..version.count - prefix.count {
        view := string.{ prefix.count, version.data + index };
        if view != prefix continue;
        next := index + prefix.count;
        if next == version.count || (version.data[next] != #char "." && (version.data[next] < #char "0" || version.data[next] > #char "9")) matched = true;
    }

    assert(matched, "configured LLVM/libclang version mismatch");
    print("ok\n");
}
