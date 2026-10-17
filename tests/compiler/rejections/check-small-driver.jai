#import "Compiler";
C :: #import "Check";

#run {
    workspace := compiler_create_workspace("Original C integer promotion check");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.os_target = .WINDOWS;
    options.cpu_target = .X64;
    set_build_options(options, workspace);
    compiler_begin_intercept(workspace);
    add_build_file("check-small-input.jai", workspace);
    settings: C.C_Porting_Check_Settings;
    settings.skip_modules = true;

    while true {
        message := compiler_wait_for_message();
        if !message break;
        C.check_c_porting(message, settings);
        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(workspace);
}

main :: () {}
