#import "Basic";
#import "File";

#import "Extensions/Csv";
#import "Extensions/Json";
#import "Extensions/Environment";
#import "Extensions/Testing";

// Every csv-spectrum case. location_coordinates expects one object rather than an array of them.
spectrum :: () {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    cases := string.["comma_in_quotes", "empty", "empty_crlf", "escaped_quotes", "json", "location_coordinates", "newlines", "newlines_crlf", "quotes_and_newlines", "simple", "simple_crlf", "utf8"];
    for name: cases {
        data, found := read_entire_file(tprint("%/csv-conformance/csvs/%.csv", base, name));

        require(found, "run PYTHONPATH=tools python3 tools/portable/build_portable.py data");
        defer free(data);
        json, got := read_entire_file(tprint("%/csv-conformance/json/%.json", base, name));

        require(got, "run PYTHONPATH=tools python3 tools/portable/build_portable.py data");
        defer free(json);
        expected, error := from_json(Json_Value, json);

        require(error.kind == .NONE);
        defer free_json(*expected);
        reader := csv_reader(cast([] u8) data, .{ header = true, lax_quotes = true, require_equal_width = true });

        defer csv_reader_free(*reader);
        objects: [] Json_Value = expected.items;
        if expected.kind == .OBJECT objects = .{ 1, *expected };
        else require(expected.kind == .ARRAY);

        for *object: objects {
            row, ok, e := csv_next(*reader);

            require(ok && e.kind == .NONE);
            expect_equal(row.count, object.members.count);

            for header: reader.header {
                value := json_get(object, header);
                require(value != null && value.kind == .STRING);
                expect_equal(row[it_index], value.text, tprint("%: %", name, header));
            }
        }

        _, present, e := csv_next(*reader);

        expect(!present && e.kind == .NONE);
    }
} @test
