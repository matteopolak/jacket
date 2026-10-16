#import "Basic";
libc :: #system_library "libc";
stdin: *void #elsewhere libc;
fgets :: (buffer: *u8, capacity: s32, stream: *void) -> *u8 #foreign libc;

main :: () {
    args := get_command_line_arguments();
    print("argc=% last=%\n", args.count, args[args.count - 1]);
    buffer: [64] u8;
    line := fgets(buffer.data, 64, stdin);
    print("got %", to_string(buffer.data));
    line = fgets(buffer.data, 64, stdin);
    print("then %\n", line == null);
}
