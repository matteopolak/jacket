// from_json over a Json_Source: a document that arrives in pieces decodes to exactly what the same
// text decodes to, however it is cut, and a source that fails or runs past the size limit is an
// error. Json has no IO of its own, so the pieces here come from a string.
//
//     jaic run stdlib/Extensions/Json/tests/streaming.jai

#import "Basic";
#import "Extensions/Json";

Person :: struct {
    name: string;
    age: int;
    tags: [..] string;
}

TEXT :: "{\"name\": \"Zo\\u00eb \\\"Z\\\" \xc3\xa9\", \"age\": 31, \"tags\": [\"a\", \"bb\", \"ccc\"]}";

Pieces :: struct {
    text: string;
    at: int;
    step: int;
    // Fail with this text once `at` reaches `fail_at` (negative: never).
    fail_at := -1;
    reads: int;
}

next_piece :: (data: *void, buffer: [] u8) -> s64, string {
    pieces := cast(*Pieces) data;

    pieces.reads += 1;

    if pieces.fail_at >= 0 && pieces.at >= pieces.fail_at return 0, "the source broke";

    count := min(min(pieces.step, buffer.count), pieces.text.count - pieces.at);

    memcpy(buffer.data, pieces.text.data + pieces.at, count);

    pieces.at += count;

    return count, "";
}

source_of :: (pieces: *Pieces) -> Json_Source {
    return .{ pieces, next_piece };
}

check_person :: (person: Person, what: string) {
    assert(person.name == "Zo\xc3\xab \"Z\" \xc3\xa9" && person.age == 31 && person.tags.count == 3 && person.tags[2] == "ccc", "%: % % %", what, person.name, person.age, person.tags.count);
}

main :: () {
    // Every piece size from one byte up decodes the same, scalar and vector paths.
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        for step: 1..TEXT.count + 3 {
            pieces := Pieces.{ text = TEXT, step = step };

            person, error := from_json(Person, source_of(*pieces));

            assert(error.kind == .NONE, "step %: %", step, json_error_text(error));

            check_person(person, tprint("step %", step));
            free_json(*person);
        }
    }

    // A document larger than the buffer's first size takes several refills.
    big: [..] u8;

    defer array_free(big);

    append_text :: (buffer: *[..] u8, text: string) {
        for text array_add(buffer, it);
    }

    append_text(*big, "[");

    for 0..9_999 {
        if it > 0 append_text(*big, ",");

        append_text(*big, tprint("%", it));
    }

    append_text(*big, "]");

    pieces := Pieces.{ text = string.{ big.count, big.data }, step = 1000 };

    numbers, numbers_error := from_json([] int, source_of(*pieces));

    assert(numbers_error.kind == .NONE && numbers.count == 10_000 && numbers[9_999] == 9_999, "a long document: %", json_error_text(numbers_error));

    free_json(*numbers);

    // Errors are the ones the text gives, with the same position.
    bad := "{\"name\": \"x\", \"age\": true}";

    pieces = Pieces.{ text = bad, step = 3 };

    _, streamed := from_json(Person, source_of(*pieces));
    _, direct := from_json(Person, bad);

    assert(streamed.kind == .TYPE && streamed.kind == direct.kind && streamed.offset == direct.offset && streamed.line == direct.line && streamed.column == direct.column && streamed.path == direct.path, "same error: %", json_error_text(streamed));

    free_json(*streamed);
    free_json(*direct);

    // A source that fails half way is a .READ error carrying its text, and nothing leaks.
    pieces = Pieces.{ text = TEXT, step = 4, fail_at = 20 };

    _, broken := from_json(Person, source_of(*pieces));

    assert(broken.kind == .READ && broken.message == "the source broke", "a failing source: % %", broken.kind, broken.message);

    free_json(*broken);

    // The size limit stops the reading.
    pieces = Pieces.{ text = TEXT, step = 4 };

    _, limited := from_json(Person, source_of(*pieces), Json_Options.{ max_size = 16 });

    assert(limited.kind == .LIMIT, "the size limit: %", json_error_text(limited));
    assert(pieces.at <= 16 + 4, "the source was read to % of % bytes after passing the limit", pieces.at, TEXT.count);

    free_json(*limited);

    // An empty source is a syntax error.
    pieces = Pieces.{ text = "", step = 4 };

    _, empty := from_json(Person, source_of(*pieces));

    assert(empty.kind == .SYNTAX, "an empty source: %", json_error_text(empty));

    free_json(*empty);

    print("ok\n");
}
