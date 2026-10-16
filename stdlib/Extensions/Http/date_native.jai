// Wall-clock date adapter. Loaded with the existing native HTTP client/server surface.
// Portable explicit-seconds formatting and parsing remain in date.jai.

#scope_export

// The date for the current time (see http_date).
http_date_now :: (cache: *Date_Cache) -> string #no_aoc {
    now := current_time_consensus();
    epoch := APOLLO_TIME_FOR_JAN_1_1970;
    elapsed: Apollo_Time;
    elapsed.low = now.low - epoch.low;
    borrow: u64 = ifx now.low < epoch.low then 1 else 0;

    elapsed.high = now.high - epoch.high - cast(s64) borrow;
    seconds, ok := to_seconds(elapsed);

    assert(ok, "HTTP date seconds exceed the supported range");

    return http_date(cache, seconds);
}
