#import "Basic";
M :: #import "Machine_X64";
C :: #import "Compiler";

main :: () {
    options: C.Build_Options;

    memset(options.machine_options.data, 0xa5, 128);
    machine := M.get_machine_options_x86(*options);

    assert(cast(*void) machine == cast(*void) options.machine_options.data);
    M.enable_feature(*machine.features.leaves, .SSE2);
    assert(M.check_feature(M.get_machine_options_x86(*options).features.leaves, .SSE2));
    M.disable_feature(*machine.features.leaves, .SSE2);
    assert(!M.check_feature(M.get_machine_options_x86(*options).features.leaves, .SSE2));
    assert((options.machine_options[3] & 4) == 0);
    for i: 44..127 assert(options.machine_options[i] == 0xa5);

    options.machine_options[0] = 0x7b;
    assert((machine.features.leaves[0] & 255) == 0x7b);
    print("ok\n");
}
