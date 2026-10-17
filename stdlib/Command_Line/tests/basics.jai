#import "Basic";
C :: #import "Command_Line";

Kind :: enum u8 {
    OAK :: 1;
    ASH :: 4;
}

Options :: struct {
    enabled: bool;
    kind: Kind = .OAK;
    sizes: [3] s32 = .[1, 2, 3];
    threshold: float64 = 0.5;
}

main :: () {
    ok, value, set, free_args := C.parse_arguments(Options, args = .["--sizes", "7", "8", "9", "--kind", "ASH", "--", "-free", "tail"], flags = C.Default_Argument_Flags | .DOUBLE_DASH_REQUIRED);

    assert(ok && value.sizes[0] == 7 && value.sizes[2] == 9 && value.kind == .ASH);
    assert(set.sizes && set.kind && !set.enabled && !set.threshold);
    assert(free_args.count == 2 && free_args[0] == "-free" && free_args[1] == "tail");
    array_free(free_args);
    ok, value, set, free_args = C.parse_arguments(Options, args = .["-kind", "OAK", "-kind", "ASH"]);
    assert(!ok);
    array_free(free_args);
    ok, value, set, free_args = C.parse_arguments(Options, args = .["-sizes", "7", "8"]);
    assert(!ok);
    array_free(free_args);
    ok, value, set, free_args = C.parse_arguments(Options, args = .["-threshold", "1.25"]);
    assert(ok && value.threshold == 1.25 && set.threshold);
    array_free(free_args);
    ok, value, set, free_args = C.parse_arguments(Options, args = .["loose"], flags = cast(C.Argument_Flags) 0);
    assert(!ok);
    array_free(free_args);
    ok, value, set, free_args = C.parse_arguments(Options);
    assert(ok && !set.enabled && !set.kind && !set.sizes && free_args.count == 0);
    array_free(free_args);
    args := get_command_line_arguments();

    assert(args.count >= 1 && args[0].count > 0);
    array_free(args);
    print("ok\n");
}
