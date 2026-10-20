// Builds one program in a workspace of its own and prints when each compiler phase message arrives.
// Every compiler runs this same metaprogram, so the phase columns of a comparison come from one method.
// `phases.py` rewrites the settings below for each run and turns the lines into stage buckets.
//
//   bench-phase: <name> <seconds since this metaprogram started>
//
// Names: parsed, typechecked (every TYPECHECKED_ALL_WE_CAN, so the last one counts), built,
// pre_write, post_write, complete.
#import "Basic";
#import "Compiler";
#import "Process";
#import "String";
#load "link_command.jai";

// settings: rewritten by phases.py
ENTRY :: "main.jai";
OUTPUT_PATH :: ".";
WRITES_EXECUTABLE :: false;
OPTIMIZED :: false;
RUN_MAIN :: false;
IMPORT_DIRS :: string.[];
// end settings

started: Apollo_Time;

mark :: (name: string) {
    elapsed, _ := to_float64_seconds(current_time_monotonic() - started);
    print("bench-phase: % %\n", name, elapsed);
}

// Links with the system linker (`link_command.jai`) whichever compiler wrote the objects, and prints how long
// that took as `bench-phase: link <seconds>`: a duration, unlike the other lines, which are times since the start.
link :: (w: Workspace, phase: *Message_Phase) {
    mark("ready_link");

    inputs: Link_Inputs;
    inputs.output = phase.executable_name;

    objects: [..] string;

    array_add(*objects, ..phase.compiler_generated_object_files);
    array_add(*objects, ..phase.support_object_files);
    inputs.objects = objects;

    libraries: [..] string;

    array_add(*libraries, ..phase.system_libraries);
    array_add(*libraries, ..phase.user_libraries);
    inputs.libraries = libraries;

    #if OS == .MACOS {
        inputs.sdk_path = ask_xcrun("--show-sdk-path");
        inputs.sdk_version = ask_xcrun("--show-sdk-version");
    }

    began := current_time_monotonic();
    result := run_command(..link_arguments(inputs));

    print("bench-phase: link %\n", to_float64_seconds(current_time_monotonic() - began));

    phase.linker_exit_code = result.exit_code;
    compiler_custom_link_command_is_complete(w);
}

ask_xcrun :: (question: string) -> string {
    _, output := run_command("xcrun", question, capture_and_return_output = true);
    return trim(output);
}

build :: () {
    started = current_time_monotonic();
    set_build_options_dc(.{ do_output = false });

    w := compiler_create_workspace("bench");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    if WRITES_EXECUTABLE options.output_type = .EXECUTABLE;

    options.output_path = OUTPUT_PATH;
    options.output_executable_name = "program";
    options.intermediate_path = OUTPUT_PATH;
    options.text_output_flags = 0; // no link line or workspace stats in the compared output
    if WRITES_EXECUTABLE options.use_custom_link_command = LINKER.count > 0;

    if OPTIMIZED set_optimization(*options, .OPTIMIZED);
    else set_optimization(*options, .DEBUG);

    if IMPORT_DIRS.count > 0 {
        import_path: [..] string;
        array_add(*import_path, ..options.import_path);
        for IMPORT_DIRS array_add(*import_path, it);

        options.import_path = import_path;
    }

    set_build_options(options, w);

    compiler_begin_intercept(w);
    add_build_file(ENTRY, w);
    if RUN_MAIN add_build_string("#run main();", w);

    while true {
        message := compiler_wait_for_message();
        if message.kind == .PHASE {
            phase := cast(*Message_Phase) message;
            if phase.phase == {
                case .ALL_SOURCE_CODE_PARSED; mark("parsed");
                case .TYPECHECKED_ALL_WE_CAN; mark("typechecked");
                case .ALL_TARGET_CODE_BUILT; mark("built");
                case .PRE_WRITE_EXECUTABLE; mark("pre_write");
                case .READY_FOR_CUSTOM_LINK_COMMAND; link(w, phase);
                case .POST_WRITE_EXECUTABLE; mark("post_write");
            }
        }

        if message.kind == .COMPLETE break;
    }

    mark("complete");
    compiler_end_intercept(w);
}

#run build();
