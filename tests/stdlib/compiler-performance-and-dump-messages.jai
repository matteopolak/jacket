// PERFORMANCE_REPORT arrives once, right before COMPLETE, only when the metaprogram asked for it
// with DO_PERFORMANCE_REPORT_RUNS and/or _POLYMORPHS (the section it did not ask for is left
// empty); DEBUG_DUMP arrives for a procedure declared #dump, without any flag.
// rules: bo.14 bo.15
#import "Basic";
#import "Compiler";
#import "String";

TARGET :: #string JAI
#import "Basic";

twice :: (x: $T) -> T { return x + x; }
compute :: () -> int { return twice(20) + 2; }
traced :: (x: int) -> int #dump { return x + 1; }

VALUE :: #run compute();

main :: () {
    print("% % % %\n", VALUE, twice(1), twice(2.5), traced(1));
}
JAI

Kind :: type_of(Message.{}.kind);

Seen :: struct {
    kinds: [..] Kind;
    reports: [..] *Message_Performance_Report;
    dump_text: string;
}

collect :: (flags: Intercept_Flags) -> Seen {
    seen: Seen;
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, flags);
    add_build_string(TARGET, w);
    while true {
        message := compiler_wait_for_message();
        array_add(*seen.kinds, message.kind);
        if message.kind == .DEBUG_DUMP seen.dump_text = copy_string((cast(*Message_Debug_Dump) message).dump_text);
        if message.kind == .PERFORMANCE_REPORT array_add(*seen.reports, cast(*Message_Performance_Report) message);
        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    compiler_end_intercept(w);
    return seen;
}

count_of :: (seen: Seen, kind: Kind) -> int {
    n := 0;
    for seen.kinds if it == kind n += 1;
    return n;
}

check_dump :: (seen: Seen) {
    assert(count_of(seen, .DEBUG_DUMP) == 1, "dumps: %", count_of(seen, .DEBUG_DUMP));
    assert(contains(seen.dump_text, "traced"), "the dump names the procedure: %", seen.dump_text);
    // The dump comes before the program is built and written, which come before COMPLETE.
    dump_at, built_at := -1, -1;
    for seen.kinds {
        if it == .DEBUG_DUMP dump_at = it_index;
        if it == .PHASE && built_at < 0 && dump_at >= 0 built_at = it_index;
    }
    assert(dump_at >= 0 && dump_at < seen.kinds.count - 1);
}

#run {
    // Nothing asked for: no report, but the dump is still sent.
    plain := collect(0);
    assert(plain.reports.count == 0, "reports: %", plain.reports.count);
    check_dump(plain);

    both := collect(.DO_PERFORMANCE_REPORT_RUNS | .DO_PERFORMANCE_REPORT_POLYMORPHS);
    check_dump(both);
    assert(both.reports.count == 1 && count_of(both, .PERFORMANCE_REPORT) == 1);
    // Right before COMPLETE, after everything else.
    assert(both.kinds[both.kinds.count - 1] == .COMPLETE && both.kinds[both.kinds.count - 2] == .PERFORMANCE_REPORT);

    report := both.reports[0];
    assert(report.kind == .PERFORMANCE_REPORT);
    assert(report.time_report.run_directives >= 0 && report.time_report.bytecode_generating >= 0);
    assert(report.bytecode_report.num_instructions_total > 0 && report.bytecode_report.num_calls_total > 0);
    assert(report.bytecode_report.num_calls_inlined == 0 && report.bytecode_report.num_instructions_inlined == 0);

    runs := report.run_directive_report;
    assert(runs.reported && runs.records.count >= 1, "runs: %", runs.records.count);
    compute_run := false;
    for runs.records {
        assert(it.elapsed_time >= 0 && it.num_stalls >= 0);
        if it.callee && it.callee.name == "compute" compute_run = true;
    }
    assert(compute_run, "the run of compute is reported with its header");

    polys := report.polymorph_report;
    assert(polys.reported);
    assert(polys.num_solves_invoked >= 3 && polys.num_solves_used >= 2 && polys.num_solves_used <= polys.num_solves_invoked);
    twice_record: *Polymorph_Report.Record;
    for *polys.records if it.source && it.source.name == "twice" twice_record = it;
    assert(twice_record != null, "twice is reported as a polymorphic procedure");
    // twice(20) and twice(1) share an instance; twice(2.5) makes another.
    assert(twice_record.num_call_sites == 3, "call sites: %", twice_record.num_call_sites);
    assert(twice_record.num_deduplications_by_constants == 1, "deduplicated: %", twice_record.num_deduplications_by_constants);
    assert(twice_record.distinct_polymorphs.count == 2, "instances: %", twice_record.distinct_polymorphs.count);
    for twice_record.distinct_polymorphs {
        assert(it.polymorphed && it.polymorphed.name == "twice");
        assert(it.location.line_number > 0 && it.location.fully_pathed_filename.count > 0);
    }

    // Only the #run section asked for: the polymorph records are left out.
    runs_only := collect(.DO_PERFORMANCE_REPORT_RUNS);
    assert(runs_only.reports.count == 1);
    assert(runs_only.reports[0].run_directive_report.reported && !runs_only.reports[0].polymorph_report.reported);
    assert(runs_only.reports[0].polymorph_report.records.count == 0);

    polys_only := collect(.DO_PERFORMANCE_REPORT_POLYMORPHS);
    assert(polys_only.reports.count == 1);
    assert(!polys_only.reports[0].run_directive_report.reported && polys_only.reports[0].run_directive_report.records.count == 0);
    assert(polys_only.reports[0].polymorph_report.reported && polys_only.reports[0].polymorph_report.records.count > 0);
}

main :: () {
    print("ok\n");
}
