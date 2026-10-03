// Diagnostic output owns an immutable native counter capture while formatting.
#scope_file
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = rpmalloc_config_t, STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = !USE_RAW || Statistics_Settings.FIRST_CLASS_HEAPS, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);
Statistics_Basic :: #import "Basic";
Statistics_Settings :: #import "internal/bindings/rpmalloc-build";
Statistics_Frames :: #import "internal/bindings/callback-context";
Statistics_Bridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = !USE_RAW || Statistics_Settings.FIRST_CLASS_HEAPS, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);

#scope_export
#if !USE_RAW {
    #if ENABLE_STATISTICS {
        dump_statistics :: (builder: *$T) {
            #assert T == Statistics_Basic.String_Builder, "dump_statistics requires Basic.String_Builder";

            previous := Statistics_Frames.enter(cast(*void) *context);

            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);
            snapshot := Statistics_Bridge.snapshot_create();

            Domain.leave(*domain_scope);
            Statistics_Frames.leave(previous);

            if !snapshot {
                builder.failed = true;
                return;
            }

            defer Statistics_Bridge.snapshot_release(snapshot);
            if !Statistics_Basic.print_to_builder(builder, "rpmalloc allocation/span statistics\n") return;

            count := Statistics_Bridge.snapshot_count(snapshot);
            for heap: 0..cast(s64) count - 1 {
                id := cast(s64) Statistics_Bridge.snapshot_header(snapshot, cast(u64) heap, 0);
                owner := Statistics_Bridge.snapshot_header(snapshot, cast(u64) heap, 1);
                full := Statistics_Bridge.snapshot_header(snapshot, cast(u64) heap, 2);
                reserved := Statistics_Bridge.snapshot_header(snapshot, cast(u64) heap, 3);
                to_global := cast(s64) Statistics_Bridge.snapshot_header(snapshot, cast(u64) heap, 4);
                from_global := cast(s64) Statistics_Bridge.snapshot_header(snapshot, cast(u64) heap, 5);

                if !Statistics_Basic.print_to_builder(builder, "heap % owner % full % reserved % to-global % from-global %\n", id, owner, full, reserved, to_global, from_global) return;

                spans := Statistics_Bridge.snapshot_counters(snapshot, cast(u64) heap, 0);

                for index: 0..62 {
                    row := spans + index * 10;
                    occupied: bool;

                    for field: 0..9 if row[field] != 0 occupied = true;
                    if !occupied continue;
                    if !Statistics_Basic.print_to_builder(builder, " span % current % high % deferred % to-global % from-global % to-cache % from-cache % to-reserve % from-reserve % maps %\n", index + 1, row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8], row[9]) return;
                }

                sizes := Statistics_Bridge.snapshot_counters(snapshot, cast(u64) heap, 1);

                for index: 0..126 {
                    row := sizes + index * 11;
                    occupied: bool;

                    for field: 0..10 if row[field] != 0 occupied = true;
                    if !occupied continue;
                    if !Statistics_Basic.print_to_builder(builder, " size % current % peak % total % freed % spans % span-peak % to-cache % from-cache % from-reserve % maps % unused %\n", index, row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8], row[9], row[10]) return;
                }
            }

            global := Statistics_Bridge.snapshot_global(snapshot);

            if !Statistics_Basic.print_to_builder(builder, "global mapped % peak % cached % huge % huge-peak % mapped-total % unmapped-total %\n", global.mapped, global.mapped_peak, global.cached, global.huge_alloc, global.huge_alloc_peak, global.mapped_total, global.unmapped_total) return;
        }
    } else {
        // Explicit reviewed empty expansion when statistics are disabled.
        dump_statistics :: (builder: *void) #expand {}
    }
}
