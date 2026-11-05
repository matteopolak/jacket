// The WASI host must allow an owned work write without exposing its caller's directory.
#import "Basic";
#import "File";

main :: () {
    if !write_entire_file("/work/registration-witness", "owned scratch") exit(1);
    print("registration scratch write passed\n");
}
