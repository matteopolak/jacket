#import "Basic";
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

Leak_Json :: #import "Extensions/Json";

Leak_Record :: struct {
    day: Leak_Time.Date; @json(with=time_json)
    time_json_encode :: Leak_Time.time_json_encode;
    time_json_decode :: Leak_Time.time_json_decode;
}

time_json_allocation_balance :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        for text: string.["{\"day\":\"2024-02-29\"}", "{\"day\":\"2024-02-30\"}", "{\"day\":\"2024-02-\\u00329\"}", "{\"day\":\"2024-02-29\",\"extra\":["] {
            v, e := Leak_Json.from_json(Leak_Record, text);
            if e.kind == .NONE {
                s := Leak_Json.to_json(v);
                free(s);
                Leak_Json.free_json(*v);
            } else Leak_Json.free_json(*e);
        }
    }

    assert(tally.outstanding == 0);
} @test
