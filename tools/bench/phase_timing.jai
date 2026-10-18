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

// settings: rewritten by phases.py
ENTRY :: "main.jai";
OUTPUT_PATH :: ".";
WRITES_EXECUTABLE :: false;
OPTIMIZED :: false;
RUN_MAIN :: false;
// end settings

started: Apollo_Time;

mark :: (name: string) {
    elapsed := to_float64_seconds(current_time_monotonic() - started);
    print("bench-phase: % %\n", name, elapsed);
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
    if OPTIMIZED set_optimization(*options, .OPTIMIZED);
    else set_optimization(*options, .DEBUG);

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
                case .POST_WRITE_EXECUTABLE; mark("post_write");
            }
        }

        if message.kind == .COMPLETE break;
    }

    mark("complete");
    compiler_end_intercept(w);
}

#run build();
