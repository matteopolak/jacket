// llhttp's request and response fixtures (nodejs/llhttp, MIT, data/llhttp/LICENSE) run through
// the codec: the same messages must give the same heads, bodies and errors, except where the
// codec is stricter on purpose. Each such case carries its reason (`deviation`, from
// tools/http_deviations.json); a case that differs without one fails, and so does a listed
// deviation that no longer differs. Fixtures that ran with an llhttp leniency flag may differ
// (the codec has no lenient mode). Every case also has to give one result however its input is cut.
//
//     jaic run stdlib/Extensions/Http/tests/llhttp.jai

#load "lib/support.jai";
#load "data/llhttp_cases.jai";

// The expected log in the form `observe` writes: body runs merged, error reasons dropped, and
// for the llhttp_finish cases the outcome of ending the input.
canonical :: (case_: Llhttp_Case) -> string {
    builder: String_Builder;
    body: String_Builder;
    have_body := false;

    lines := split(case_.expected, "\n");

    for line: lines {
        if line.count == 0 continue;

        if line[0] == #char "B" {
            append(*body, slice(line, 2, line.count - 2));
            have_body = true;

            continue;
        }

        if have_body {
            append(*builder, "B\x01");
            append(*builder, builder_to_string(*body));
            append(*builder, "\n");
            reset(*body);
            have_body = false;
        }

        if line[0] == #char "E" {
            append(*builder, "E\n");
        } else if line[0] == #char "H" || line[0] == #char "T" {
            // llhttp keeps blanks around a value in the value; RFC 9110 5.5 says they are not part of it.
            append(*builder, trim_right(line, " \t"));
            append(*builder, "\n");
        } else {
            append(*builder, line);
            append(*builder, "\n");
        }
    }

    if have_body {
        append(*builder, "B\x01");
        append(*builder, builder_to_string(*body));
        append(*builder, "\n");
    }

    // llhttp_finish returning SAFE_WITH_CB completes the message in the callback.
    if case_.finish == 1 append(*builder, "C\n");
    if case_.finish == 2 append(*builder, "E\n");

    return builder_to_string(*builder);
}

main :: () {
    matched := 0;
    deviated := 0;
    lenient := 0;

    for LLHTTP_CASES {
        run := default_run(ifx it.response then Parser_Kind.RESPONSE else Parser_Kind.REQUEST);
        run.eof = it.finish >= 0;

        whole := observe_every_cut(run, it.input, it.id);
        expected := canonical(it);

        if whole == expected {
            matched += 1;

            check(it.deviation.count == 0, tprint("% is listed as a deviation (%) but matches llhttp", it.id, it.deviation));

            continue;
        }

        if it.deviation.count > 0 {
            deviated += 1;

            continue;
        }

        if it.lenient.count > 0 {
            lenient += 1;

            continue;
        }

        check(false, tprint("% (%)\n  input: %\n  llhttp: %\n  ours:   %", it.id, it.name, it.input, replace(expected, "\n", " | "), replace(whole, "\n", " | ")));
    }

    print("% cases: % match llhttp, % deviate on purpose, % need a lenient mode\n", LLHTTP_CASES.count, matched, deviated, lenient);

    finish();
}
