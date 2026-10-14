// Original measured performance-message and nested record ABI regression.
#import "Basic";
C :: #import "Compiler";

#run {
    w := C.compiler_create_workspace("measured reports");
    options := C.get_build_options(w);
    options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.compiler_begin_intercept(w, .DO_PERFORMANCE_REPORT_RUNS | .DO_PERFORMANCE_REPORT_POLYMORPHS);
    C.add_build_string("#import \"Basic\"; identity :: (x: $T) -> T { return x; } work :: () { sum: u64; for i: 0..2000 sum += identity(cast(u64) i); assert(sum > 0); } #run work(); main :: () {}", w);
    reported := false;

    while true {
        message := C.compiler_wait_for_message();

        assert(message != null && message.workspace == w);

        if message.kind == .PERFORMANCE_REPORT {
            report := cast(*C.Message_Performance_Report) message;

            assert(!reported);
            reported = true;
            assert(report.run_directive_report.reported);
            assert(report.run_directive_report.records.count > 0);
            assert(report.time_report.run_directives > 0);
            assert(report.time_report.bytecode_generating >= 0);
            found_callee := false;

            for record: report.run_directive_report.records {
                assert(record.num_stalls >= 0 && record.elapsed_time >= 0);
                if record.callee != null found_callee = true;
            }

            assert(found_callee);
            assert(report.polymorph_report.reported);
            assert(report.polymorph_report.num_solves_invoked >= report.polymorph_report.num_solves_used);
            assert(report.polymorph_report.num_solves_used > 0);
            assert(report.polymorph_report.records.count > 0);

            for record: report.polymorph_report.records {
                assert(record.source != null && record.num_call_sites > 0);
                assert(record.distinct_polymorphs.count > 0);
                for instance: record.distinct_polymorphs assert(instance.polymorphed != null);
            }

            assert(report.bytecode_report.num_instructions_total > 0);
            assert(report.bytecode_report.num_calls_total > 0);
        }

        if message.kind == .COMPLETE {
            assert(reported && (cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    C.compiler_end_intercept(w);

    blank := C.compiler_create_workspace("unrequested run section");

    options = C.get_build_options(blank);
    options.output_type = .NO_OUTPUT;
    C.set_build_options(options, blank);
    C.compiler_begin_intercept(blank, .DO_PERFORMANCE_REPORT_POLYMORPHS);
    C.add_build_string("main :: () {}", blank);
    saw_blank := false;

    while true {
        message := C.compiler_wait_for_message();

        assert(message != null && message.workspace == blank);

        if message.kind == .PERFORMANCE_REPORT {
            report := cast(*C.Message_Performance_Report) message;

            assert(!report.run_directive_report.reported);
            assert(report.run_directive_report.records.count == 0);
            assert(report.polymorph_report.reported);
            saw_blank = true;
        }

        if message.kind == .COMPLETE {
            assert(saw_blank && (cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    C.compiler_end_intercept(blank);
}

main :: () {
    print("ok\n");
}
