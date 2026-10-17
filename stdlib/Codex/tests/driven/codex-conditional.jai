#import "Basic";
#import "Compiler";
C :: #import "Codex";

#run {
    w := compiler_create_workspace("Original conditional nesting");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_file(tprint("%codex-conditional-input.jai", #filepath), w);
    braced, chained, switched, ifx_scored, static_scored: bool;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            if tc.expression.header.name == "braced_else" {
                metrics := C.collect_metrics(tc);
                assert(metrics.maximum_if_depth == 2 && metrics.if_points == 6, "braced depth %, points %", metrics.maximum_if_depth, metrics.if_points);
                braced = true;
            }

            if tc.expression.header.name == "static_points" {
                metrics := C.collect_metrics(tc);
                assert(metrics.if_points == 3 && metrics.maximum_if_depth == 1, "static points %, depth %", metrics.if_points, metrics.maximum_if_depth);
                static_scored = true;
            }

            if tc.expression.header.name == "switch_like" {
                metrics := C.collect_metrics(tc);
                assert(metrics.if_points == 3 && metrics.maximum_if_depth == 1, "switch points %, depth %", metrics.if_points, metrics.maximum_if_depth);
                switched = true;
            }

            if tc.expression.header.name == "ifx_points" {
                metrics := C.collect_metrics(tc);
                assert(metrics.if_points == 2 && metrics.maximum_if_depth == 1);
                ifx_scored = true;
            }

            if tc.expression.header.name == "chained_else" {
                metrics := C.collect_metrics(tc);
                assert(metrics.maximum_if_depth == 1 && metrics.if_points == 6);
                chained = true;
            }
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(braced && chained && switched && ifx_scored && static_scored);
}

main :: () {
    print("ok\n");
}
