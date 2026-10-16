// Public build options are LINK syntax on Windows. The compiler selects the
// equivalent arguments for the actual target linker and compiler driver.
#import "Compiler";

#run {
    if OS == .WINDOWS {
        options := get_build_options();
        options.additional_linker_arguments = string.[
            "/SUBSYSTEM:windows",
            "/ENTRY:mainCRTStartup",
        ];
        set_build_options(options);
    }
}

main :: () -> int {
    return 0;
}
