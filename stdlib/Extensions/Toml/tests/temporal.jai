#import "Basic";
#import "Extensions/Toml";

// Leap spellings are lossless format values, not a claim of historical instant validation.
main :: () {
    doc, error := parse_toml("utc = 2016-12-31T23:59:60.123456789012Z\nshifted = 2017-01-01T00:59:60+01:00\nlocal = 23:59:60\nlocal_date = 2016-12-31T23:59:60\nstructural = 2024-02-29T12:00:60Z\n");

    assert(error.kind == .NONE, error.message);
    defer free_toml(*doc);
    assert(toml_get(doc.root, "utc").temporal.second == 60);
    assert(toml_get(doc.root, "shifted").temporal.offset_minutes == 60);
    assert(toml_get(doc.root, "local").kind == .LOCAL_TIME);
    assert(toml_get(doc.root, "local_date").kind == .LOCAL_DATETIME);
    assert(toml_get(doc.root, "structural").temporal.second == 60);
    temporary_before := get_temporary_storage_mark();
    output, write_error := write_toml(doc.root);
    temporary_after := get_temporary_storage_mark();

    assert(temporary_before.total_bytes_occupied == temporary_after.total_bytes_occupied, "temporal writer consumed temporary storage");

    assert(write_error.kind == .NONE, write_error.message);
    defer free(output);
    again, again_error := parse_toml(output);

    assert(again_error.kind == .NONE, again_error.message);
    defer free_toml(*again);

    for doc.root.members {
        value := toml_get(again.root, it.key);
        assert(value.kind == it.value.kind && value.temporal.second == 60);
        assert(value.temporal.fraction == it.value.temporal.fraction);
        assert(value.temporal.offset_minutes == it.value.temporal.offset_minutes);
    }

    for string.["x = 23:59:61\n", "x = 2016-12-31T23:59:61Z\n", "x = 2016-12-31T23:59:61\n", "x = 12:34.5\n"] {
        rejected, failure := parse_toml(it);
        assert(failure.kind != .NONE && rejected.root == null);
    }

    check_fixed_fields();
    print("ok\n");
}

check_fixed_fields :: () {
    Case :: struct {
        input: string;
        expected: string;
    }

    for Case.[
        .{ "0000-01-01", "0000-01-01" },
        .{ "9999-12-31", "9999-12-31" },
        .{ "01:02", "01:02:00" },
        .{ "01:02:03.000001234567890123", "01:02:03.000001234567890123" },
        .{ "0000-01-01T01:02:03.000001234567890123-00:00", "0000-01-01T01:02:03.000001234567890123-00:00" },
        .{ "9999-12-31T23:59:60+23:59", "9999-12-31T23:59:60+23:59" },
        .{ "0000-01-01T00:00:00-23:59", "0000-01-01T00:00:00-23:59" },
        .{ "2001-02-03T04:05:06Z", "2001-02-03T04:05:06+00:00" },
        .{ "2001-02-03T04:05Z", "2001-02-03T04:05:00+00:00" },
        .{ "2001-02-03T04:05:06", "2001-02-03T04:05:06" },
    ] {
        doc, error := parse_toml(tprint("x=%", it.input));

        assert(error.kind == .NONE, error.message);
        expected := tprint("\"x\" = %\n", it.expected);
        text, write_error := write_toml(doc.root, .{ max_output_bytes = expected.count });

        assert(write_error.kind == .NONE && text == expected, text);
        free(text);
        _, short_error := write_toml(doc.root, .{ max_output_bytes = expected.count - 1 });

        assert(short_error.kind == .LIMIT);
        free_toml(*doc);
    }
}
