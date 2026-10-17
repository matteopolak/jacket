// Reviewed pause macro; native opcode comparison is performed by the runner.
#import "Basic";
M :: #import "Machine_X64";

#assert size_of(M.Machine_Options_X86) == 44;

main :: () {
    for 0..255 M.pause();
    print("ok\n");
}
