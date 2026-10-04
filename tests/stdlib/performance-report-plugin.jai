// Performance_Report is a metaprogram plugin; check that it typechecks and wires its callbacks.
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

    // A COMPLETE message with no report received prints the "not initialized" notice.
    complete: Message_Complete;
    complete.kind = .COMPLETE;
    plugin.message(plugin, *complete);
    free(plugin);
    print("ok\n");
}
