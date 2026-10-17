// The borrowed crash trace contains the real fault PC; no allocation in callback.
D :: #import "Debug";

oracle :: #foreign_library "../../../../target/developer_tools_debug_lifecycle";
raise_trap :: () -> s32 #foreign oracle "original_debug_raise_trap";

on_crash :: (trace: [] string) {
    if trace.count != 1 || trace[0].count != size_of(*void) * 2 + 2 D.abort(.Failure);
    if trace[0][0] != #char "0" || trace[0][1] != #char "x" D.abort(.Failure);

    nonzero := false;
    for character, index: trace[0] if index >= 2 && character != #char "0" nonzero = true;
    if !nonzero D.abort(.Failure);

    D.output("original crash callback\n");
}

main :: () {
    D.crash_handler = on_crash;
    D.enable_signal_handler();
    raised := raise_trap();
    if raised == 0 D.abort(.Failure);

    D.abort(.Failure);
}
