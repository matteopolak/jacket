#import "Basic";
C :: #import "Compiler";
R :: #import "Windows_Resources";

#assert OS == .WINDOWS;
#run check_console();

check_console :: () {
    before := C.get_build_options();

    R.disable_runtime_console();
    after := C.get_build_options();
    count := before.additional_linker_arguments.count;

    assert(after.additional_linker_arguments.count == count + 2);
    for argument, index: before.additional_linker_arguments assert(after.additional_linker_arguments[index] == argument);

    assert(after.additional_linker_arguments[count] == "/SUBSYSTEM:windows");
    assert(after.additional_linker_arguments[count + 1] == "/ENTRY:mainCRTStartup");
}

main :: () {}
