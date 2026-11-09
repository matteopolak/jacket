#import "Basic";
Files_Time :: #import "File";
Environment_Time :: #import "Extensions/Environment";
Conformance_Time :: #import "Extensions/Time";

vector_number :: (data: string, cursor: *s64, width: s64) -> s64 {
    assert(cursor.* + width <= data.count);
    value: u64;

    for 1..width {
        value = (value << 8) | data[cursor.*];
        cursor.* += 1;
    }

    if width == 4 return cast(s64) cast,no_check(s32) cast(u32) value;

    return cast,no_check(s64) value;
}

time_conformance :: () {
    root := Environment_Time.environment_variable("JACKET_OUTPUT_DIR");
    if root.count == 0 root = tprint("%../../../../target", #filepath);

    data, ok := Files_Time.read_entire_file(tprint("%/time-conformance/vectors.bin", root));

    assert(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");
    defer free(data);
    cursor: s64;
    zones := vector_number(data, *cursor, 4);
    cases: s64;

    for 1..zones {
        length := vector_number(data, *cursor, 2);
        name := string.{ length, data.data + cursor };

        cursor += length;
        zone, error := Conformance_Time.time_zone_get(name, directory = tprint("%/time-conformance/zoneinfo", root));

        assert(error.kind == .NONE, tprint("%: % at %", name, error.message, error.offset));
        defer Conformance_Time.time_zone_free(*zone);
        count := vector_number(data, *cursor, 4);
        for 1..count {
            seconds := vector_number(data, *cursor, 8);
            expected := vector_number(data, *cursor, 4);
            n := vector_number(data, *cursor, 1);
            abbreviation := string.{ n, data.data + cursor };

            cursor += n;
            instant, e := Conformance_Time.timestamp_from_seconds(seconds);

            assert(e.kind == .NONE);
            actual := Conformance_Time.offset_at(zone, instant);

            assert(actual.seconds == expected, tprint("% at %: % != %", name, seconds, actual.seconds, expected));
            assert(Conformance_Time.abbreviation_at(zone, instant) == abbreviation, tprint("% abbreviation at %", name, seconds));
            cases += 1;
        }

        resolutions := vector_number(data, *cursor, 4);

        for 1..resolutions {
            wall_second := vector_number(data, *cursor, 8);
            early := vector_number(data, *cursor, 8);
            late := vector_number(data, *cursor, 8);
            gap := vector_number(data, *cursor, 1) != 0;
            raw, _ := Conformance_Time.timestamp_from_seconds(wall_second);
            wall := Conformance_Time.to_civil(Conformance_Time.time_zone_utc(), raw);

            for mode: Conformance_Time.Disambiguation.[.COMPATIBLE, .EARLIER, .LATER, .REJECT] {
                got, e := Conformance_Time.to_timestamp(zone, wall, mode);
                if mode == .REJECT assert(e.kind == (ifx gap then .GAP else .FOLD), tprint("% reject %", name, wall_second));
                else {
                    expected := ifx mode == .LATER || (mode == .COMPATIBLE && gap) then late else early;
                    assert(e.kind == .NONE && got.seconds == expected, tprint("% resolution % %: % != % (%)", name, wall_second, mode, got.seconds, expected, e.message));
                }
            }
        }
    }

    assert(cursor == data.count && zones > 590 && cases > 100000);
    print("% zones, % offsets and abbreviations\n", zones, cases);
} @test @timeout(180)
