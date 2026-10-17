#import "Basic";
#import "Compiler";

#run {
    w := compiler_create_workspace("Retained member bodies");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;

    #if #exists(CHILD_OUTPUT) {
        options.output_type = .EXECUTABLE;
        options.output_executable_name = CHILD_OUTPUT;
        // Exercise procedure modification independently of the externally owned crash archive.
        options.backtrace_on_crash = .OFF;
        #if #exists(CHILD_OPTIMIZED) options.llvm_options.bitcode_optimization_setting = .O2;
        else options.llvm_options.bitcode_optimization_setting = .O0;
    }

    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_string("R :: struct { touch :: (x:int)->int {return x+1;} } main :: () {assert(R.touch(8)==9);} #import \"Basic\";", w);
    members: int;

    while true {
        message := compiler_wait_for_message();

        assert(message != null);

        if message.kind == .TYPECHECKED for typed: (cast(*Message_Typechecked) message).procedure_bodies {
            body := typed.expression;
            if !body.enclosing_load || !body.enclosing_load.enclosing_import continue;
            if body.enclosing_load.enclosing_import.module_type != .MAIN_PROGRAM continue;
            if body.header.name == "main" continue;

            assert(body.header.body_or_null == body);
            compiler_modify_procedure(w, body);
            add_build_string("#run {assert(R.touch(8)==9);}", w);
            members += 1;
        }

        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(members == 1);
}

main :: () {}
