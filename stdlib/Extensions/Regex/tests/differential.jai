// Matches random patterns against random texts and compares every match and capture span with
// what Rust's regex crate found (data/differential_cases.jai, from tools/gen_regex_corpus.py).
//
//     jaic run stdlib/Extensions/Regex/tests/differential.jai

#import "Basic";
Regex :: #import "Extensions/Regex";

#load "data/differential_cases.jai";

Case :: struct {
    pattern: string;
    text: string;
    expected: string; // "E" if it must not compile; else per match `s-e,s-e,...` (x: no
    // span), separated by spaces, then `|`
}

// The same notation as the expected column.
describe :: (pattern: string, text: string) -> string {
    re, ok := Regex.compile(pattern);
    if !ok return "E";

    defer Regex.free_regex(*re);
    builder: String_Builder;

    for caps: Regex.captures_all(*re, text) {
        if it_index append(*builder, " ");

        for 0..caps.slots.count / 2 - 1 {
            if it append(*builder, ",");
            s, e := Regex.span(caps, it);
            if s < 0 append(*builder, "x");
            else print_to_builder(*builder, "%-%", s, e);
        }
    }

    append(*builder, "|");

    return builder_to_string(*builder,, temp);
}

main :: () {
    failures := 0;

    for CASES {
        got := describe(it.pattern, it.text);
        if got != it.expected {
            failures += 1;
            if failures <= 20 print("FAIL: % in %\n    want %\n    got  %\n", it.pattern, it.text, it.expected, got);
        }

        reset_temporary_storage();
    }

    if failures {
        print("% of % differential cases failed\n", failures, CASES.count);
        exit(1);
    }
}
