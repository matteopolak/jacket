#import "Basic";
Compiler :: #import "Compiler";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

visits: s64;
expanded: bool;
seen: [2048] bool;

inspect :: (node: *I.Zone_Profiling_Data) {
    assert(node != null && node.zone != null && node != *I.dummy);
    index := (cast(s64) node.zone - cast(s64) *I.zones[0]) / size_of(I.Zone);

    assert(index >= 0 && index < I.zones.count && !seen[index]);
    assert(node.total_entry_count > 0);
    seen[index] = true;
    visits += 1;

    if !expanded {
        expanded = true;

        // Force a table replacement while visiting; the original nodes survive.
        for i: 2..201 {
            I.zones[i].initted = true;
            I.zone_begin(i);
            I.zone_end();
        }
    }
}

count :: (node: *I.Zone_Profiling_Data) {
    assert(node != null && node.zone != null);
    visits += 1;
}

main :: () {
    source: I.Zone_Info;

    assert(type_of(source.load) == *Compiler.Message_File);
    assert(type_of(source.name) == string && type_of(source.line) == s64);
    I.traverse(count);
    assert(visits == 0);
    I.init_runtime();
    I.zones[0].initted = true;
    I.zones[1].initted = true;
    I.zone_begin(0);
    outer := I.stack;

    I.zone_begin(1);
    I.zone_end();
    I.zone_end();
    I.traverse(inspect);
    assert(visits == 2 && seen[0] && seen[1]);
    assert(I.caches[0] == outer && I.zone_hash_count == 202);
    visits = 0;
    I.traverse(count);
    assert(visits == 202);
    print("ok\n");
}
