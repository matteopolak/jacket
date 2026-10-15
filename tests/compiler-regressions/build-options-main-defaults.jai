// Reviewed Build_Options defaults and explicit output metadata round trip.
// Automatic main filename/path/.build spelling is not a reviewed API promise.
#import "Basic";
#import "Compiler";

#run {
    own := get_build_options();

    assert(own.os_target == OS && own.cpu_target == CPU);
    w := compiler_create_workspace("output metadata");
    other := get_build_options(w);
    defaults: Build_Options;

    assert(other.output_executable_name == defaults.output_executable_name);
    assert(other.output_path == defaults.output_path);
    assert(other.intermediate_path == defaults.intermediate_path);
    other.output_type = .NO_OUTPUT;
    other.output_executable_name = "chosen-output";
    other.output_path = "chosen-directory/";
    other.intermediate_path = "chosen-intermediate/";
    set_build_options(other, w);
    back := get_build_options(w);

    assert(back.output_executable_name == other.output_executable_name);
    assert(back.output_path == other.output_path && back.intermediate_path == other.intermediate_path);
    compiler_destroy_workspace(w);
}

main :: () {
    print("ok\n");
}
