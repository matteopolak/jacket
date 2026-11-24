// Generated UUIDs: version 4 and 7 values, the RFC variant, version 7 ordering and timestamp, and
// name-based version 3 and 5 against published test vectors.
#import "Basic";
U :: #import "Extensions/Uuid";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// The DNS namespace from RFC 9562, Appendix B.
DNS :: "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

main :: () {
    // Version 4: random, version nibble 4, RFC variant.
    previous: U.Uuid;

    for i: 0..999 {
        id, error := U.uuid_v4();

        expect(error.kind == .NONE, "uuid_v4 succeeds");
        expect(U.uuid_version(id) == 4, tprint("uuid_v4 % has version %", i, U.uuid_version(id)));
        expect(U.uuid_variant(id) == .RFC, "uuid_v4 has the RFC variant");
        expect(i == 0 || !U.uuid_equal(id, previous), "consecutive uuid_v4 values differ");
        previous = id;
    }

    // Version 7: time ordered, version nibble 7, RFC variant. The 48-bit prefix is the Unix time in
    // milliseconds; 1577836800000 is 2020-01-01 and 4102444800000 is 2100-01-01.
    last: U.Uuid;

    for i: 0..999 {
        id, error := U.uuid_v7();

        expect(error.kind == .NONE, tprint("uuid_v7 % succeeds: %", i, error.message));
        expect(U.uuid_version(id) == 7, tprint("uuid_v7 % has version %", i, U.uuid_version(id)));
        expect(U.uuid_variant(id) == .RFC, "uuid_v7 has the RFC variant");

        if i > 0 expect(U.uuid_compare(last, id) == -1, tprint("uuid_v7 % is above the one before it", i));

        last = id;
    }

    millis: u64 = 0;
    for 0..5 millis = (millis << 8) | cast(u64) last.bytes[it];

    expect(millis > 1577836800000 && millis < 4102444800000, tprint("v7 timestamp % ms is between 2020 and 2100", millis));

    // Version 5 (SHA-1) and version 3 (MD5) of "python.org" in the DNS namespace.
    namespace, parse_error := U.parse_uuid(DNS);

    expect(parse_error.kind == .NONE, "DNS namespace parses");

    v5 := U.uuid_v5(namespace, "python.org");

    expect(U.format_uuid(v5) == "886313e1-3b8a-5372-9b90-0c9aee199e5d", tprint("uuid_v5 gives %", U.format_uuid(v5)));
    expect(U.uuid_version(v5) == 5 && U.uuid_variant(v5) == .RFC, "uuid_v5 version and variant");

    v3 := U.uuid_v3(namespace, "python.org");

    expect(U.format_uuid(v3) == "6fa459ea-ee8a-3ca4-894e-db77e160355e", tprint("uuid_v3 gives %", U.format_uuid(v3)));
    expect(U.uuid_version(v3) == 3 && U.uuid_variant(v3) == .RFC, "uuid_v3 version and variant");

    // Names are exact bytes: a different case is a different UUID.
    expect(!U.uuid_equal(U.uuid_v5(namespace, "Python.org"), v5), "uuid_v5 is case sensitive");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
