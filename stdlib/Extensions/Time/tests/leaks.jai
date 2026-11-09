#load "../../test-support/counting-allocator.jai";
Leak_Time :: #import "Extensions/Time";

time_zone_allocation_balance :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        zone, error := Leak_Time.time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");

        assert(error.kind == .NONE);
        other := Leak_Time.time_zone_retain(zone);

        Leak_Time.time_zone_free(*zone);
        Leak_Time.time_zone_free(*other);

        for text: string.["", "EST5EDT,", "EST26"] {
            _, e := Leak_Time.time_zone_from_posix(text);
            assert(e.kind != .NONE);
        }
    }

    assert(tally.outstanding == 0 && tally.allocations > 0);
} @test
