// Run by test_link_command.py: prints the link argument list for fixed inputs.
#import "Basic";
#import "Compiler";
#load "link_command.jai";

#run {
    set_build_options_dc(.{ do_output = false });
    inputs: Link_Inputs;
    inputs.objects = .["a.o", "b.o"];
    inputs.libraries = .["-lm"];
    inputs.output = "out/program";
    inputs.sdk_path = "/sdk";
    inputs.sdk_version = "99.0";
    for link_arguments(inputs) print("%\n", it);
}
