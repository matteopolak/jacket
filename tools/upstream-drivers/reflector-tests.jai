// Builds n00bmind/reflector's test executable (bin/test) through the project's own unotest
// workspace helpers. The project's build.jai also builds a Windows-only benchmark, so the sweep
// copies this driver next to it instead (case `reflector-tests` in tools/upstream-cases.json).
#import "Basic";
#import "Compiler";
#import "unotest";
#load "module.jai";

#run {
    set_build_options_dc(.{ do_output = false });
    options := get_build_options();
    options.output_path = "bin";
    options.intermediate_path = "bin";
    w := BeginTestWorkspace("Tests", options);
    AddAllStrings(w);
    add_build_file("src/test.jai", w);
    EndTestWorkspace(w);
}
