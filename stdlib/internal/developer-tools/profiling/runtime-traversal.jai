traverse :: (func: (z: *Zone_Profiling_Data)) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(func != null, "Iprof: traversal requires a callback");
    if !profile_ready return;

    // Nodes are persistent; table storage may be replaced during callbacks.
    push_allocator(profile_owner);
    snapshot: [..] *Zone_Profiling_Data;

    defer array_free(snapshot);
    for node: zone_hash if node array_add(*snapshot, node);
    for node: snapshot func(node);
}

#scope_file
#import "Basic";
