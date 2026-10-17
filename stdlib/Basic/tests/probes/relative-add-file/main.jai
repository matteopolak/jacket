// Invoke run_relative_file with direct/helper/explicit-location under #run,stallable.
// Reviewed caller-location rule: direct/explicit-location pass; helper rejects.
C :: #import "Compiler";
#load "helper/insertion.jai";

run_relative_file :: (mode: string) {
    w := C.compiler_create_workspace("relative filename control");
    options := C.get_build_options(w);
    options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.set_build_options_dc(.{ do_output = false, write_added_strings = false });
    if mode == "direct" C.add_build_file("child.jai", w);
    else if mode == "helper" add_relative("child.jai", w);
    else if mode == "explicit-location" add_at_location("child.jai", w, #caller_location);
    else C.compiler_report("Expected direct or helper.");
}
