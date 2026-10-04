// Bodies that fail to compile for now (a declaration the metaprogram adds later) while the program
// also runs many compile-time calls must not be re-lowered over and over, and must compile once
// the declaration arrives. Every body calls a procedure whose polymorphic `$compare` default is a
// lambda, which must resolve to one instance rather than a new one per attempt.
#import "Basic";
#import "Compiler";

PROGRAM :: #string END
#import "Basic";

Find :: (arr: []$T, item: T, $compare: (T, T) -> bool = (a, b) => a == b) -> s64 {
    for arr  if compare(it, item) return it_index;
    return -1;
}

Square :: (x: s64) -> s64 { return x * x; }

#insert -> string {
    b: String_Builder;
    for 0..39 {
        print_to_builder(*b, "body_% :: () { n :: #run Square(%); arr: [4]s32; i := Find(arr, 1); v: Missing_Type; }\n", it, it);
    }
    print_to_builder(*b, "main :: () {\n");
    for 0..39  print_to_builder(*b, "    body_%();\n", it);
    print_to_builder(*b, "}\n");
    return builder_to_string(*b);
}
END

#run {
    w := compiler_create_workspace("t");
    opts := get_build_options(w);
    opts.output_type = .NO_OUTPUT;
    set_build_options(opts, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    failed := false;
    added := false;
    while true {
        msg := compiler_wait_for_message();
        if msg.kind == .PHASE && !added {
            phase := cast(*Message_Phase) msg;
            if phase.phase == .TYPECHECKED_ALL_WE_CAN {
                add_build_string("Missing_Type :: struct { x: s32; };", w);
                added = true;
            }
        }
        if msg.kind == .COMPLETE {
            failed = (cast(*Message_Complete) msg).error_code != .NONE;
            break;
        }
    }
    compiler_end_intercept(w);
    assert(added && !failed);
}

main :: () { print("ok\n"); }
