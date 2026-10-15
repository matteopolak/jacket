// Original generic context-constant/module-alias mechanism; no name special case.
#import "Basic";
Original_Module :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

#add_context _OriginalTools :: Original_Module;

main :: () {
    context._OriginalTools.init_runtime();
    context._OriginalTools.zones[0].name = "original context alias";
    context._OriginalTools.zones[0].initted = true;
    context._OriginalTools.zone_begin(0);
    node := context._OriginalTools.stack;

    context._OriginalTools.zone_end();
    assert(node.total_entry_count == 1 && node.zone == *context._OriginalTools.zones[0]);
    print("ok\n");
}
