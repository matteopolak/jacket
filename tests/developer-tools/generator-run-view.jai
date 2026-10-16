// A conforming allocator can move a resizable buffer. The final public slice
// must survive subsequent tooling-queue growth while its Session remains live.
#import "Basic";
B :: #import "Bindings_Generator";

Probe :: struct {
    backing: Allocator;
    watched: *void;
    retired: bool;
}

allocate :: (mode: Allocator_Mode, bytes: s64, old_bytes: s64, memory: *void, data: *void) -> *void {
    probe := cast(*Probe) data;
    if probe.watched && memory == probe.watched {
        if mode == .FREE probe.retired = true;

        if mode == .RESIZE && bytes > 0 {
            replacement := probe.backing.proc(.ALLOCATE, bytes, 0, null, probe.backing.data);
            if !replacement return null;

            memcpy(replacement, memory, min(bytes, old_bytes));
            probe.backing.proc(.FREE, 0, old_bytes, memory, probe.backing.data);
            probe.retired = true;

            return replacement;
        }
    }

    return probe.backing.proc(mode, bytes, old_bytes, memory, probe.backing.data);
}

main :: () {
    push_context, defer_pop;
    probe: Probe;
    probe.backing = context.allocator;
    context.allocator = .{ allocate, *probe };
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-run-view.h", #filepath));
    defer array_free(options.source_files);
    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    ok := B.run_generator(*generator, options);

    assert(ok && generator.flattened_global_scope_members.count == 1);
    view := generator.flattened_global_scope_members;

    probe.watched = view.data;
    for 0..128 array_add(*generator.declarations_for_global_scope, view[0]);

    // Check retirement before touching the potentially dangling public slice.
    assert(!probe.retired);
    assert(view[0].name == "StableView");
    probe.watched = null;
    array_free(generator.declarations_for_global_scope);
    print("view ok\n");
}
