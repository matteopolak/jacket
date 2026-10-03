MAX_DEPTH :: 100;
MAX_HASH_SIZE :: 65536;
INIT_HASH_SIZE :: 256;
MAX_PROFILING_ZONES_FOR_MANUAL_MODE :: 2048;

#if MANUAL_MODE zones: [MAX_PROFILING_ZONES_FOR_MANUAL_MODE] Zone;
else zones: [] Zone;
caches: [] *Zone_Profiling_Data;
dummy: Zone_Profiling_Data;
dummy2: Zone_Profiling_Data;
stack: *Zone_Profiling_Data;
zone_hash: [..] *Zone_Profiling_Data;
zone_hash_count: s64 = 0;
zone_hash_mask: u64 = 0;
timestamps_to_seconds: float64;
integer_timestamps_per_second: History_Scalar;

init_runtime :: () {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if profile_ready return;

    profile_owner = context.default_allocator;
    Profile_Pool.set_allocators(*profile_pool, profile_owner);
    frequency, frequency_ok := profile_clock_frequency();

    assert(frequency_ok && frequency > 0, "Iprof: profiling clock initialization failed");
    initial, initial_ok := profile_clock_ticks();

    assert(initial_ok, "Iprof: profiling clock initialization failed");
    profile_last_tick = initial;
    last_integer_timestamp = initial;
    profile_nominal_seconds = 1.0 / cast(float64) frequency;
    last_update_time = cast(float64) initial * profile_nominal_seconds;
    timestamps_to_seconds = 1.0 / cast(float64) frequency;
    value := cast(float64) frequency;

    integer_timestamps_per_second.values = .[value, value, value];
    dummy = .{};
    dummy.t_self_start = initial;
    stack = *dummy;
    assert(zones.count >= 0);
    push_allocator(profile_owner);
    array_resize(*caches, zones.count);
    profile_resize_hash(INIT_HASH_SIZE);
    profile_ready = true;
}

Stack_Push :: (zone: *Zone) -> *Zone_Profiling_Data {
    return profile_push(zone, null);
}

zone_begin :: inline (zone_index: s64) #expand {
    profile_zone_begin(zone_index);
}

zone_end :: inline () {
    profile_zone_end();
}

hash :: (z: *Zone, s: *Zone_Profiling_Data) -> u64 #no_aoc {
    assert(z != null);
    return z.hash + cast(u64) s;
}

createStackNode :: (zone: *Zone, parent: *Zone_Profiling_Data) -> *Zone_Profiling_Data {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if !profile_ready init_runtime();

    assert(zone != null);
    depth := count_recursion_depth(parent, zone);

    assert(depth >= 0, "Iprof: invalid profiling parent chain");
    push_allocator(profile_owner);
    node := cast(*Zone_Profiling_Data) Profile_Pool.get(*profile_pool, size_of(Zone_Profiling_Data));

    assert(node != null, "Iprof: profiling pool allocation failed");
    node.* = .{};
    node.zone = zone;
    node.parent = parent;
    node.recursion_depth = depth;

    return node;
}

insert_node :: (q: *Zone_Profiling_Data) #no_aoc {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(q != null && q.zone != null && zone_hash.count > 0);
    if zone_hash_count >= zone_hash.count / 2 && zone_hash.count < MAX_HASH_SIZE profile_resize_hash(zone_hash.count * 2);

    assert(zone_hash_count < zone_hash.count, "Iprof: profiling node limit reached");
    profile_insert_slot(q);
}

init_zone_hash :: (size: s64) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if !profile_ready init_runtime();

    assert(size > 0 && size <= MAX_HASH_SIZE && (size & (size - 1)) == 0);
    assert(profile_activation_count == 0, "Iprof: cannot reset a live profiling stack");
    push_allocator(profile_owner);
    array_free(zone_hash);
    zone_hash = .{};
    zone_hash_count = 0;
    profile_resize_hash(size);
    for *cache: caches cache.* = null;
}

#scope_module
#import "Basic";

Profile_Activation :: struct {
    node: *Zone_Profiling_Data;
    hierarchical_start: s64;
}

Profile_Pool :: #import "Pool";

profile_pool: Profile_Pool.Pool;
profile_ready: bool;
profile_owner: Allocator;
profile_last_tick: s64;
profile_activation_count: s64;
profile_activations: [MAX_DEPTH] Profile_Activation;

profile_now :: () -> s64 {
    ticks, ok := profile_clock_ticks();
    assert(ok && ticks >= profile_last_tick, "Iprof: profiling clock failed or moved backward");
    profile_last_tick = ticks;
    return ticks;
}

profile_key_hash :: (zone: *Zone, parent: *Zone_Profiling_Data) -> u64 #no_aoc {
    return hash(zone, parent);
}

profile_find_node :: (zone: *Zone, parent: *Zone_Profiling_Data) -> *Zone_Profiling_Data {
    index := profile_key_hash(zone, parent) & zone_hash_mask;

    for 0..zone_hash.count - 1 {
        node := zone_hash[index];
        if !node return null;
        if node.zone == zone && node.parent == parent return node;
        index = (index + 1) & zone_hash_mask;
    }

    return null;
}

profile_insert_slot :: (node: *Zone_Profiling_Data) {
    index := profile_key_hash(node.zone, node.parent) & zone_hash_mask;

    for 0..zone_hash.count - 1 {
        if !zone_hash[index] {
            zone_hash[index] = node;
            zone_hash_count += 1;
            return;
        }

        index = (index + 1) & zone_hash_mask;
    }

    assert(false, "Iprof: profiling hash table is full");
}

profile_resize_hash :: (size: s64) {
    assert(size > 0 && size <= MAX_HASH_SIZE && (size & (size - 1)) == 0);
    push_allocator(profile_owner);
    replacement: [..] *Zone_Profiling_Data;

    array_resize(*replacement, size);
    old := zone_hash;

    zone_hash = replacement;
    zone_hash_mask = cast(u64) (size - 1);
    zone_hash_count = 0;
    for node: old if node profile_insert_slot(node);

    array_free(old);
}

profile_allocate_node :: (zone: *Zone, parent: *Zone_Profiling_Data) -> *Zone_Profiling_Data {
    node := createStackNode(zone, parent);
    insert_node(node);
    return node;
}

profile_push :: (zone: *Zone, cached: *Zone_Profiling_Data) -> *Zone_Profiling_Data {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if !profile_ready init_runtime();

    assert(zone != null && zone.initted, "Iprof: profiling zone is not initialized");
    assert(profile_activation_count < MAX_DEPTH, "Iprof: maximum profiling depth exceeded");
    parent := stack;

    assert(parent != null, "Iprof: profiling stack was modified");
    if profile_activation_count == 0 assert(parent == *dummy, "Iprof: profiling stack was modified");
    else assert(parent == profile_activations[profile_activation_count - 1].node, "Iprof: profiling stack was modified");

    now := profile_now();

    assert(parent.t_self_start <= now);
    parent.total_self_ticks += now - parent.t_self_start;
    node := cached;
    if !node || node.zone != zone || node.parent != parent node = profile_find_node(zone, parent);
    if !node node = profile_allocate_node(zone, parent);

    assert(node.total_entry_count < 0xffffffff, "Iprof: profiling call count overflow");
    node.total_entry_count += 1;
    node.t_self_start = now;
    profile_activations[profile_activation_count] = .{ node, now };
    profile_activation_count += 1;
    stack = node;

    return node;
}

profile_zone_begin :: (zone_index: s64) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if !profile_ready init_runtime();

    assert(zone_index >= 0 && zone_index < zones.count && caches.count == zones.count, "Iprof: invalid or resized profiling zone array (index %, zones %, caches %)", zone_index, zones.count, caches.count);
    node := profile_push(*zones[zone_index], caches[zone_index]);

    caches[zone_index] = node;
}

profile_zone_end :: () {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(profile_ready && profile_activation_count > 0, "Iprof: zone_end without an active zone");
    activation := profile_activations[profile_activation_count - 1];

    assert(stack == activation.node && stack.parent != null, "Iprof: profiling stack was modified");
    now := profile_now();

    assert(now >= stack.t_self_start && now >= activation.hierarchical_start);
    stack.total_self_ticks += now - stack.t_self_start;
    stack.total_hier_ticks += now - activation.hierarchical_start;
    stack = stack.parent;
    stack.t_self_start = now;
    profile_activation_count -= 1;
}
