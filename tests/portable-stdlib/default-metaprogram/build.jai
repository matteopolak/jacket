// Real standard-library build entry; arguments follow the compiler's standalone '-'.
D :: #import "Default_Metaprogram";
C :: #import "Compiler";
using #import "Basic";

run_build :: () {
    parent := C.get_build_options();

    D.build();
    after := C.get_build_options();

    assert(after.output_type == parent.output_type);
    assert(after.output_executable_name == parent.output_executable_name);
    assert(after.output_path == parent.output_path);
    assert(after.write_added_strings == parent.write_added_strings);
    assert(after.append_executable_filename_extension == parent.append_executable_filename_extension);
}
