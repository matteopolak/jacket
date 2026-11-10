#import "Basic";
#import "String";
#import "File";
#import "Unicode";
#import "Extensions/Environment";
#import "Extensions/Unicode_Text";
#import "Extensions/Testing";

hex :: (s: string) -> u32 {
    n: u32;

    for s {
        digit := ifx it <= #char "9" then it - #char "0" else it - #char "A" + 10;
        n = n * 16 + digit;
    }

    return n;
}

encode :: (builder: *String_Builder, cp: u32) {
    buffer: [4] u8;
    encoded := string.{ 4, buffer.data };
    character_utf32_to_utf8(cp, *encoded);
    append(builder, encoded);
}

codepoints :: (field: string) -> string {
    b: String_Builder;
    tokens := split(trim(field), #char " ");

    defer free(tokens.data);
    for tokens if it.count encode(*b, hex(it));

    return builder_to_string(*b);
}

suite :: (name: string) -> string {
    base := output_directory();

    text, ok := read_entire_file(tprint("%/unicode-text-conformance/%", base, name));

    expect(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");

    return text;
}

normalization_conformance :: () {
    text := suite("NormalizationTest.txt");

    defer free(text);
    lines := split(text, #char "\n");

    defer free(lines.data);
    cases := 0;

    for line: lines {
        _, body, _ := split_from_left(line, #char "#");
        body = trim(body);
        if !body.count || body[0] == #char "@" continue;

        fields := split(body, #char ";");

        defer free(fields.data);
        columns: [5] string;
        for i: 0..4 columns[i] = codepoints(fields[i]);

        defer for columns free(it);

        for f: 0..3 {
            form := cast(Normalization_Form) f;

            for c: 0..4 {
                expected := 0;
                if f == 0 expected = ifx c < 3 then 1 else 3;
                if f == 1 expected = ifx c < 3 then 2 else 4;
                if f == 2 expected = 3;
                if f == 3 expected = 4;

                b: String_Builder;

                defer free_buffers(*b);
                error := normalize(columns[c], form, *b);

                require(error.kind == .NONE);
                actual := builder_to_string(*b);

                defer free(actual);
                if actual != columns[expected] print("normalization line %, form %, col %\n", line, f, c);

                require(actual == columns[expected]);
                yes, qc_error := is_normalized(columns[c], form);
                if yes != (columns[c] == columns[expected]) print("QC line %, form %, col %, yes %\n", line, f, c, yes);

                require(qc_error.kind == .NONE && yes == (columns[c] == columns[expected]));
            }
        }

        cases += 1;
    }

    require(cases == 20034);
} @test

break_conformance :: (name: string, kind: Segmentation, expected_cases: int) {
    text := suite(name);

    defer free(text);
    lines := split(text, #char "\n");

    defer free(lines.data);
    cases := 0;

    for line: lines {
        _, body, _ := split_from_left(line, #char "#");
        body = trim(body);
        if !body.count continue;

        tokens := split(body, #char " ");

        defer free(tokens.data);
        b: String_Builder;

        defer free_buffers(*b);
        expected: [..] int;

        defer array_free(expected);

        for tokens {
            if it == "÷" array_add(*expected, builder_string_length(*b));
            else if it != "×" && it.count encode(*b, hex(it));
        }

        input := builder_to_string(*b);

        defer free(input);
        iterator := segments(input, kind);
        found := 1;

        while true {
            start, end, more := next_segment(*iterator);
            if !more break;

            require(found < expected.count);
            if start != expected[found - 1] || end != expected[found] print("%: % at % got %..% expected %..%\n", name, line, found, start, end, expected[found - 1], expected[found]);

            require(start == expected[found - 1] && end == expected[found]);

            if kind == .GRAPHEME {
                reverse, reverse_error := previous_grapheme(input, end);
                require(reverse_error.kind == .NONE && reverse == start);
            }

            found += 1;
        }

        require(iterator.error.kind == .NONE && found == expected.count);
        cases += 1;
    }

    require(cases == expected_cases);
}

grapheme_conformance :: () {
    break_conformance("GraphemeBreakTest.txt", .GRAPHEME, 766);
} @test
word_conformance :: () {
    break_conformance("WordBreakTest.txt", .WORD, 1944);
} @test
sentence_conformance :: () {
    break_conformance("SentenceBreakTest.txt", .SENTENCE, 512);
} @test

normalization_unlisted_scalars :: () {
    text := suite("NormalizationTest.txt");

    defer free(text);
    listed: [..] bool;

    array_resize(*listed, 0x110000);
    defer array_free(listed);
    lines := split(text, #char "\n");

    defer free(lines.data);
    part_one := false;

    for line: lines {
        if starts_with(line, "@Part1") {
            part_one = true;
            continue;
        }

        if starts_with(line, "@Part") part_one = false;
        if !part_one || !line.count || line[0] == #char "#" continue;

        _, field, _ := split_from_left(line, #char ";");
        listed[hex(trim(field))] = true;
    }

    // NUL separators prevent adjacent scalar values from interacting during normalization.
    for block: 0..271 {
        b: String_Builder;

        defer free_buffers(*b);

        for cp: block * 4096..(block + 1) * 4096 - 1 {
            if listed[cp] || (cp >= 0xD800 && cp <= 0xDFFF) continue;
            encode(*b, cast(u32) cp);
            append(*b, #char "\0");
        }

        input := builder_to_string(*b);

        defer free(input);

        for f: 0..3 {
            form := cast(Normalization_Form) f;
            error := normalize(input, form, *b);

            require(error.kind == .NONE);
            output := builder_to_string(*b);

            defer free(output);
            require(output == input);
            yes, qc_error := is_normalized(input, form);

            require(yes && qc_error.kind == .NONE);
        }
    }
} @test
