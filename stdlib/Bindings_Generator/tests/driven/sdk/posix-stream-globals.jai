P :: #import "POSIX";

probe :: #library,system "posix_stream_probe";
native_address :: (index: u32) -> **P.FILE #foreign probe "binding_stream_address";
native_value :: (index: u32) -> *P.FILE #foreign probe "binding_stream_value";
native_assign :: (index: u32, value: *P.FILE) #foreign probe "binding_stream_assign";
renamed_cell: s32 #elsewhere probe "binding_stream_control_cell";
native_cell_address :: () -> *s32 #foreign probe "binding_stream_cell_address";
libc :: #library,system "c";
abort :: () #foreign libc;

main :: () {
    input := P.stdin;
    output := P.stdout;
    error := P.stderr;
    valid := input != null && output != null && error != null;
    valid = valid && *P.stdin == native_address(0);
    valid = valid && *P.stdout == native_address(1);
    valid = valid && *P.stderr == native_address(2);
    valid = valid && native_value(0) == input;
    valid = valid && native_value(1) == output;
    valid = valid && native_value(2) == error;

    // Mutate the SDK's actual pointer cells, without doing IO or closing a FILE.
    P.stdin = output;
    P.stdout = error;
    P.stderr = input;
    valid = valid && native_value(0) == output;
    valid = valid && native_value(1) == error;
    valid = valid && native_value(2) == input;
    native_assign(0, error);
    native_assign(1, input);
    native_assign(2, output);
    valid = valid && P.stdin == error;
    valid = valid && P.stdout == input;
    valid = valid && P.stderr == output;
    // Restore all process globals before reporting any assertion failure.
    native_assign(0, input);
    native_assign(1, output);
    native_assign(2, error);
    valid = valid && P.stdin == input && P.stdout == output && P.stderr == error;

    saved := renamed_cell;

    valid = valid && *renamed_cell == native_cell_address();
    renamed_cell = 37;
    valid = valid && <<native_cell_address() == 37;
    <<native_cell_address() = 83;
    valid = valid && renamed_cell == 83;
    renamed_cell = saved;
    if !valid abort();
}
