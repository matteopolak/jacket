// Same imports/runtime/output, without invoking the public spin hint.
#import "Basic";
M :: #import "Machine_X64";

#assert size_of(M.Machine_Options_X86) == 44;

main :: () {
    print("ok\n");
}
