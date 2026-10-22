#import "Basic";
C :: #import "Compiler";

#assert IS_CROSS_COMPILING == (OS != BUILD_OS || CPU != BUILD_CPU);
#assert size_of(type_of(C.Build_Options.machine_options)) == MACHINE_OPTIONS_SIZE;

main :: () {
    options: C.Build_Options;
    assert(options.machine_options.count == MACHINE_OPTIONS_SIZE);
}
