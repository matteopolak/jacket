// NFC, NFD, NFKC and NFKD of known strings (composition, reordering, Hangul, compatibility), is_normalized,
// and the errors for invalid UTF-8 and for limits.
#import "Basic";
UT :: #import "Extensions/Unicode_Text";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// UTF-8 text for a list of code points.
cps :: (values: [] u32) -> string {
    out: [..] u8;

    for values {
        cp := it;
        if cp < 0x80 {
            array_add(*out, cast(u8) cp);
        } else if cp < 0x800 {
            array_add(*out, cast(u8) (0xC0 | (cp >> 6)));
            array_add(*out, cast(u8) (0x80 | (cp & 0x3F)));
        } else if cp < 0x10000 {
            array_add(*out, cast(u8) (0xE0 | (cp >> 12)));
            array_add(*out, cast(u8) (0x80 | ((cp >> 6) & 0x3F)));
            array_add(*out, cast(u8) (0x80 | (cp & 0x3F)));
        } else {
            array_add(*out, cast(u8) (0xF0 | (cp >> 18)));
            array_add(*out, cast(u8) (0x80 | ((cp >> 12) & 0x3F)));
            array_add(*out, cast(u8) (0x80 | ((cp >> 6) & 0x3F)));
            array_add(*out, cast(u8) (0x80 | (cp & 0x3F)));
        }
    }

    return .{ out.count, out.data };
}

// The normalized text and the error from normalize.
normalized :: (text: string, form: UT.Normalization_Form, options: UT.Unicode_Text_Options = .{}) -> string, UT.Unicode_Text_Error {
    builder: String_Builder;
    error := UT.normalize(text, form, *builder, options);
    return builder_to_string(*builder), error;
}

main :: () {
    // Composition and decomposition of e with acute accent.
    composed, e1 := normalized(cps(.[0x65, 0x301]), .NFC);

    expect(e1.kind == .NONE && composed == cps(.[0xE9]), "NFC composes e and combining acute to U+00E9");

    decomposed, e2 := normalized(cps(.[0xE9]), .NFD);

    expect(e2.kind == .NONE && decomposed == cps(.[0x65, 0x301]), "NFD splits U+00E9");

    // The angstrom sign is a canonical singleton for A with ring above.
    ring, e3 := normalized(cps(.[0x212B]), .NFC);

    expect(e3.kind == .NONE && ring == cps(.[0xC5]), "NFC maps U+212B to U+00C5");

    ring_decomposed, e4 := normalized(cps(.[0xC5]), .NFD);

    expect(e4.kind == .NONE && ring_decomposed == cps(.[0x41, 0x30A]), "NFD of U+00C5 is A and combining ring");

    // Hangul syllable GA (U+AC01) splits into a leading, vowel and trailing jamo.
    hangul, e5 := normalized(cps(.[0x1100, 0x1161, 0x11A8]), .NFC);

    expect(e5.kind == .NONE && hangul == cps(.[0xAC01]), "NFC composes Hangul jamo");

    hangul_decomposed, e6 := normalized(cps(.[0xAC01]), .NFD);

    expect(e6.kind == .NONE && hangul_decomposed == cps(.[0x1100, 0x1161, 0x11A8]), "NFD splits Hangul");

    // Two marks on a: a circumflex (U+0302, class 230) and dot below (U+0323, class 220). Canonical
    // order puts the dot first; the result composes to U+1EAD.
    marks, e7 := normalized(cps(.[0x61, 0x302, 0x323]), .NFC);

    expect(e7.kind == .NONE && marks == cps(.[0x1EAD]), "NFC composes a with two marks to U+1EAD");

    reordered, e8 := normalized(cps(.[0x61, 0x302, 0x323]), .NFD);

    expect(e8.kind == .NONE && reordered == cps(.[0x61, 0x323, 0x302]), "NFD reorders combining marks by class");

    // Compatibility forms: the fi ligature and the circled digit one.
    canonical_fi, e9 := normalized(cps(.[0xFB01]), .NFC);

    expect(e9.kind == .NONE && canonical_fi == cps(.[0xFB01]), "NFC keeps the fi ligature");

    compat_fi, e10 := normalized(cps(.[0xFB01]), .NFKC);

    expect(e10.kind == .NONE && compat_fi == "fi", "NFKC maps the fi ligature to fi");

    compat_fi_d, e11 := normalized(cps(.[0xFB01]), .NFKD);

    expect(e11.kind == .NONE && compat_fi_d == "fi", "NFKD maps the fi ligature to fi");

    circled, e12 := normalized(cps(.[0x2460]), .NFKD);

    expect(e12.kind == .NONE && circled == "1", "NFKD maps circled one to 1");

    circled_canonical, e13 := normalized(cps(.[0x2460]), .NFD);

    expect(e13.kind == .NONE && circled_canonical == cps(.[0x2460]), "NFD keeps circled one");

    // is_normalized agrees with normalize.
    ok_c, e14 := UT.is_normalized(cps(.[0xE9]), .NFC);

    expect(e14.kind == .NONE && ok_c, "U+00E9 is NFC");

    ok_d, e15 := UT.is_normalized(cps(.[0x65, 0x301]), .NFC);

    expect(e15.kind == .NONE && !ok_d, "e plus combining acute is not NFC");

    ok_nfd, e16 := UT.is_normalized(cps(.[0x65, 0x301]), .NFD);

    expect(e16.kind == .NONE && ok_nfd, "e plus combining acute is NFD");

    // Invalid UTF-8: 'a' then 0xFF. The error points at the bad byte and the builder stays empty.
    bad := string.{ 2, cast(*u8) alloc(2) };
    bad[0] = #char "a";
    bad[1] = 0xFF;
    builder: String_Builder;
    bad_error := UT.normalize(bad, .NFC, *builder);

    expect(bad_error.kind == .INVALID_UTF8 && bad_error.offset == 1, tprint("invalid UTF-8: kind % offset %", bad_error.kind, bad_error.offset));
    expect(builder_string_length(*builder) == 0, "builder is unchanged after invalid input");

    // Limits: input size and output size.
    small_input := UT.Unicode_Text_Options.{ max_size = 4 };
    _, input_error := normalized("abcdefgh", .NFC, small_input);

    expect(input_error.kind == .LIMIT, "input over max_size is a LIMIT error");

    small_output := UT.Unicode_Text_Options.{ max_output = 1 };
    _, output_error := normalized(cps(.[0x65, 0x301]), .NFC, small_output);

    expect(output_error.kind == .LIMIT, "output over max_output is a LIMIT error");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
