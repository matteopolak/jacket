// JSONTestSuite (nst/JSONTestSuite, MIT, data/JSONTestSuite/LICENSE): every `y_` case must be
// accepted, every `n_` case rejected, and the `i_` cases (the standard leaves them open) must
// not crash. The choices made for the `i_` cases are listed in docs/stdlib/json.md and pinned
// here, so a change in behavior is noticed.
//
//     jaic run stdlib/Extensions/Json/tests/suite.jai

#import "Basic";
#import "String";
#import "Extensions/Json";

#load "data/suite_cases.jai";

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;
    print("FAIL: %\n", what);
    failures += 1;
}

run_suite :: () {
    accepted_y := 0;
    rejected_n := 0;
    accepted_i := 0;
    rejected_i := 0;

    for SUITE_CASES {
        value, error := parse_json(it.text);
        accepted := error.kind == .NONE;

        if accepted {
            // What was accepted must survive a round trip through the encoder.
            again := to_json(value);
            second, error2 := parse_json(again);
            check(error2.kind == .NONE && to_json(second) == again, tprint("% does not round trip", it.name));
        }

        if begins_with(it.name, "y_") {
            check(accepted, tprint("% must be accepted: %", it.name, json_error_text(error)));
            if accepted accepted_y += 1;
        } else if begins_with(it.name, "n_") {
            check(!accepted, tprint("% must be rejected", it.name));
            if !accepted rejected_n += 1;
        } else {
            // Pinned: the untyped parser takes any number the grammar allows, whatever its size
            // (i_number_*); everything else open in the suite (lone surrogates, invalid UTF-8,
            // overlong forms, a byte order mark, nesting beyond the depth limit) is refused.
            check(accepted == begins_with(it.name, "i_number_"), tprint("% went the other way", it.name));
            if accepted accepted_i += 1;
            else rejected_i += 1;
        }
    }

    print("simd %: y accepted %/95, n rejected %/188, i accepted %, i rejected %\n", json_use_simd, accepted_y, rejected_n, accepted_i, rejected_i);
    check(accepted_y == 95 && rejected_n == 188 && accepted_i == 10 && rejected_i == 25, "case counts");
}

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;
        run_suite();
    }

    assert(failures == 0);
}
