#import "Basic";
D :: #import "Debug";

main :: () {
    value: s64 = 19;

    assert(D.is_valid_pointer(*value));
    assert(!D.is_valid_pointer(null));
    assert(!D.is_valid_pointer(cast(*void) 1));
    assert(D.translate_symbols == D.backtrace_symbols);
    addresses: [128] *void;
    count := D.backtrace(addresses);

    assert(count > 0 && count <= addresses.count);
    names := D.backtrace_symbols(addresses, count);

    defer D.free_backtrace(names);
    assert(names.count == count);
    for name: names assert(name.count > 0);

    trace := D.backtrace();

    defer D.free_backtrace(trace);
    assert(trace.count > 0);
    assert(D.backtrace(addresses, 4096) == 0);
    assert(!D.is_debugger_present());
    assert(!D.breakpoint());
    D.output("native debug output\n");
    print("ok\n");
}
