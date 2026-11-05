// The WASI host must allow an owned work write without exposing its caller's directory.
#import "Basic";
#import "File";

main :: () {
    args := get_command_line_arguments();
    folder := ifx args.count == 2 then args[1] else "/work";
    if !write_entire_file(tprint("%/registration-witness", folder), "owned scratch") exit(1);
    print("registration scratch write passed\n");
}
