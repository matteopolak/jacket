// A build program of a cross build: it starts a process (which needs the host's process
// support, not the target's) and drives a workspace through the `Compiler` module.
#import "Basic";
#import "Compiler";
#import "Process";

build :: () {
    #if OS == .WINDOWS {
        result, output := run_command("cmd", "/c", "echo", "from the build program", capture_and_return_output = true);
    } else {
        result, output := run_command("echo", "from the build program", capture_and_return_output = true);
    }

    assert(result.exit_code == 0, "the process failed");
    assert(OS == BUILD_OS && CPU == BUILD_CPU, "compile-time code sees the machine it runs on");

    w := compiler_create_workspace("child");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string("main :: () {}", w);

    while true {
        message := compiler_wait_for_message();
        if !message continue;
        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(w);
}

#run build();

main :: () {}
