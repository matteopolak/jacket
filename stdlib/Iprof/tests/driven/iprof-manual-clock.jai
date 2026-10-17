#import "Basic";
#import "File";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";

sleep_for_measurement :: () {
    #if OS == .WINDOWS W.Sleep(10);
    else {
        duration: P.timespec;
        duration.tv_nsec = 10_000_000;
        assert(P.nanosleep(*duration, null) == 0);
    }
}

reference_seconds :: () -> float64 {
    #if OS == .WINDOWS {
        frequency, ticks: W.LARGE_INTEGER;
        assert(W.QueryPerformanceFrequency(*frequency) && W.QueryPerformanceCounter(*ticks));
        return cast(float64) ticks.QuadPart / cast(float64) frequency.QuadPart;
    } else {
        time: P.timespec;
        assert(P.clock_gettime(.MONOTONIC, *time) == 0);
        return cast(float64) time.tv_sec + cast(float64) time.tv_nsec / 1_000_000_000.0;
    }
}

main :: () {
    I.init_runtime();
    assert(I.stack == *I.dummy && I.timestamps_to_seconds > 0);
    frequency := I.get_value(*I.integer_timestamps_per_second);

    assert(frequency > 0 && frequency * I.timestamps_to_seconds > 0.99999 && frequency * I.timestamps_to_seconds < 1.00001);
    root := I.stack;

    I.init_runtime();
    assert(I.stack == root);
    I.zones[0].name = "original outer";
    I.zones[0].initted = true;
    I.zones[1].name = "original inner";
    I.zones[1].initted = true;
    outer_before := reference_seconds();

    I.zone_begin(0);
    outer := I.stack;
    inner_before := reference_seconds();

    sleep_for_measurement();
    I.zone_begin(1);
    inner := I.stack;

    sleep_for_measurement();
    I.zone_end();
    assert(I.stack == outer);
    sleep_for_measurement();
    I.zone_begin(0);
    recursive := I.stack;

    assert(I.stack == recursive && recursive.parent == outer && recursive.recursion_depth == 1);
    sleep_for_measurement();
    I.zone_end();
    inner_after := reference_seconds();

    I.zone_end();
    outer_after := reference_seconds();

    assert(I.stack == root && outer.parent == root && inner.parent == outer);
    assert(outer.total_entry_count == 1 && inner.total_entry_count == 1 && recursive.total_entry_count == 1);
    assert(outer.total_self_ticks > 0 && inner.total_hier_ticks > 0 && recursive.total_self_ticks > 0);
    assert(outer.total_hier_ticks == outer.total_self_ticks + inner.total_hier_ticks + recursive.total_hier_ticks);
    assert(cast(float64) outer.total_hier_ticks * I.timestamps_to_seconds >= 0.025);
    measured := cast(float64) outer.total_hier_ticks * I.timestamps_to_seconds;

    assert(measured + 0.00001 >= inner_after - inner_before);
    assert(measured <= outer_after - outer_before + 0.00001);
    I.zones[1000].name = "original persistent node";
    I.zones[1000].initted = true;
    persistent: *I.Zone_Profiling_Data;

    {
        push_allocator(temp);
        I.zone_begin(1000);
        persistent = I.stack;
        I.zone_end();
    }

    reset_temporary_storage();
    scratch := talloc(16384);

    memset(scratch, #char "X", 16384);
    I.zone_begin(1000);
    assert(I.stack == persistent && persistent.total_entry_count == 2);
    I.zone_end();
    before := I.zone_hash_count;

    for index: 2..321 {
        I.zones[index].name = "original growth zone";
        I.zones[index].initted = true;
        I.zone_begin(index);
        I.zone_end();
    }

    assert(I.zone_hash_count == before + 320);
    I.zone_begin(0);
    assert(I.stack == outer);
    I.zone_end();
    assert(outer.total_entry_count == 2 && I.stack == root);
    assert(I.caches[0] == outer && I.zone_hash_count == before + 320);
    direct := I.Stack_Push(*I.zones[0]);

    assert(direct == outer && I.stack == direct);
    I.zone_end();
    assert(outer.total_entry_count == 3 && I.stack == root);
    report: I.Report;

    array_add(*report.records, I.Report_Row.{ name = outer.zone.name, self_time = cast(float64) outer.total_self_ticks * I.timestamps_to_seconds * 1000, hier_time = cast(float64) outer.total_hier_ticks * I.timestamps_to_seconds * 1000, calls = cast(float64) outer.total_entry_count, zone = outer.zone });
    array_add(*report.records, I.Report_Row.{ name = inner.zone.name, self_time = cast(float64) inner.total_self_ticks * I.timestamps_to_seconds * 1000, hier_time = cast(float64) inner.total_hier_ticks * I.timestamps_to_seconds * 1000, calls = cast(float64) inner.total_entry_count, zone = inner.zone });
    defer array_free(report.records);
    filename := tprint("%../../../../target/developer-tools-iprof-measured.csv", #filepath);

    I.save_csv_report(*report, filename);
    csv, csv_ok := read_entire_file(filename);

    defer free(csv);
    assert(csv_ok && contains(csv, "name,self (ms),hier (ms),calls") && contains(csv, "original outer") && contains(csv, "original inner"));
    print("ok\n");
}

#scope_file
#import "String";
