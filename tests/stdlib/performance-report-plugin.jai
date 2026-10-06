// Performance_Report is a metaprogram plugin; jaic cannot load plugins, so drive its
// callbacks by hand: the intercept flags, the "nothing received" path, and a full
// report built from synthetic records (including null nodes and baked constants).
#import "Basic";
#import "Compiler";
Performance_Report :: #import "Performance_Report";

main :: () {
    plugin := Performance_Report.get_plugin();
    assert(plugin.before_intercept != null);
    assert(plugin.message != null);

    flags: Intercept_Flags;
    plugin.before_intercept(plugin, *flags);
    assert((flags & .DO_PERFORMANCE_REPORT_RUNS) != 0);
    assert((flags & .DO_PERFORMANCE_REPORT_POLYMORPHS) != 0);

    complete: Message_Complete;
    complete.kind = .COMPLETE;
    plugin.message(plugin, *complete);

    // A polymorph instantiated with N=3.
    three: Code_Literal;
    three.value_type = .NUMBER;
    three._s64 = 3;
    constant: Code_Declaration;
    constant.name = "N";
    constant.expression = *three;
    members: [1] *Code_Scope_Entry;
    members[0] = cast(*Code_Scope_Entry) *constant;
    constants: Code_Block;
    constants.members = members;
    instance: Code_Procedure_Header;
    instance.name = "sum";
    instance.constants_block = *constants;

    runs: [2] Run_Directive_Report.Record;
    runs[0].elapsed_time = 0.25;
    runs[1].elapsed_time = 1.5;
    runs[1].num_stalls = 2;

    distinct: [2] Polymorph_Report.Distinct_Polymorph;
    distinct[0].polymorphed = *instance;
    distinct[0].location = .{ "poly.jai", 7, 3 };
    polys: [2] Polymorph_Report.Record;
    polys[0].num_call_sites = 1;
    polys[1].source = *instance;
    polys[1].num_call_sites = 4;
    polys[1].num_deduplications_by_constants = 2;
    polys[1].distinct_polymorphs = distinct;

    report: Message_Performance_Report;
    report.kind = .PERFORMANCE_REPORT;
    report.time_report.run_directives = 0.5;
    report.bytecode_report.num_calls_total = 10;
    report.run_directive_report.reported = true;
    report.run_directive_report.records = runs;
    report.polymorph_report.reported = true;
    report.polymorph_report.num_solves_invoked = 5;
    report.polymorph_report.records = polys;

    plugin.message(plugin, *report);
    plugin.message(plugin, *complete);

    // The records are the compiler's; sorting must not reorder them.
    assert(runs[0].elapsed_time == 0.25);
    assert(polys[0].num_call_sites == 1);
    free(plugin);
    print("ok\n");
}
