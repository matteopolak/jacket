// List and safely extract one archive; differential.py --bench supplies identical Python work.
#import "Basic";
#import "File";
#import "Extensions/Archive";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4);
    bytes, ok := read_entire_file(args[1]);

    assert(ok);
    defer free(bytes);
    format := ifx args[2] == "zip" then Archive_Format.ZIP else Archive_Format.TAR;
    start, _ := to_float64_seconds(current_time_monotonic());
    a, error := read_archive(cast([] u8) bytes, format);

    assert(error.kind == .NONE, error.message);
    indexed, _ := to_float64_seconds(current_time_monotonic());

    defer free_archive(*a);
    error = extract(*a, args[3]);
    assert(error.kind == .NONE, error.message);
    done, _ := to_float64_seconds(current_time_monotonic());

    print("% %\n", indexed - start, done - indexed);
}
