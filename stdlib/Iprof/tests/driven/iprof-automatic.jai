#import "Basic";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

nested :: (early: bool) {
    I.Automatic_Zone(0);
    assert(I.stack.zone == *I.zones[0]);
    if early return;

    I.Automatic_Zone_Guarded(1);
    assert(I.stack.zone == *I.zones[1] && I.stack.parent.zone == *I.zones[0]);
}

main :: () {
    I.init_runtime();
    I.zones[0].initted = true;
    I.zones[1].initted = true;
    I.zones[0].hash = 123;
    assert(I.hash(*I.zones[0], *I.dummy) == 123 + cast(u64) *I.dummy);
    node := I.createStackNode(*I.zones[0], *I.dummy);

    assert(node.zone == *I.zones[0] && node.parent == *I.dummy && node.recursion_depth == 0);
    I.insert_node(node);
    assert(I.zone_hash_count == 1);
    I.insert_node(node);
    assert(I.zone_hash_count == 2);
    I.init_zone_hash(512);
    assert(I.zone_hash_count == 0 && I.zone_hash.count == 512 && I.zone_hash_mask == 511);
    nested(true);
    assert(I.stack == *I.dummy && I.caches[0].total_entry_count == 1);
    nested(false);
    assert(I.stack == *I.dummy && I.caches[0].total_entry_count == 2);
    assert(I.caches[1].total_entry_count == 1);
    print("ok\n");
}
