#import "Basic";
Cache_Time :: #import "Extensions/Time";
Cache_Thread :: #import "Thread";
Cache_Env :: #import "Extensions/Environment";

Cache_Input :: struct {
    directory: string;
}

cache_reader :: (thread: *Cache_Thread.Thread) -> s64 {
    input := cast(*Cache_Input) thread.data;

    for 1..64 {
        zone, e := Cache_Time.time_zone_get("America/New_York", directory = input.directory);

        assert(e.kind == .NONE);
        instant, _ := Cache_Time.timestamp_from_seconds(1710055800);

        assert(Cache_Time.offset_at(zone, instant).seconds == -14400);
        other := Cache_Time.time_zone_retain(zone);

        Cache_Time.time_zone_free(*zone);
        Cache_Time.time_zone_free(*other);
    }

    return 0;
}

concurrent_zone_cache :: () {
    root := Cache_Env.environment_variable("JACKET_OUTPUT_DIR");
    if root.count == 0 root = tprint("%../../../../target", #filepath);

    input := Cache_Input.{ tprint("%/time-conformance/zoneinfo", root) };
    threads: [4] Cache_Thread.Thread;

    for *thread: threads {
        assert(Cache_Thread.thread_init(thread, cache_reader));
        thread.data = *input;
        Cache_Thread.thread_start(thread);
    }

    for *thread: threads {
        assert(Cache_Thread.thread_is_done(thread, -1));
        Cache_Thread.thread_deinit(thread);
    }
} @test
