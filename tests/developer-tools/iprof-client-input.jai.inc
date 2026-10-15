// Owned source fragment; compiled/linted with the actual generated context setup.
#import "Basic";

main :: () {
    assert(context._Iprof.IMPORT_MODE == .CLIENT);
    assert(context._Iprof.MANUAL_MODE == Original_Expected_Manual);
    assert(context._Iprof.USE_GRAPHICS == Original_Expected_Graphics);
    assert(context._Iprof.DO_HISTORY == Original_Expected_History);
    #insert context._Iprof.prepend_main;
    assert(context._Iprof.stack == *context._Iprof.dummy);
    zone: context._Iprof.Zone;
    zone.name = "original generated client setup";
    zone.initted = true;
    node := context._Iprof.Stack_Push(*zone);

    context._Iprof.zone_end();
    assert(node.zone == *zone && node.total_entry_count == 1);
    print("ok\n");
}
