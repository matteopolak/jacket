// Byte locations inside quoted literals and here-strings.
//
//     jaic stdlib/Extensions/String_Literal/tests/locations.jai -interp

#import "Basic";
#import "Compiler";
String_Literal :: #import "Extensions/String_Literal";

where :: (code: Code, byte: int) -> string {
    literal := cast(*Code_Literal) compiler_get_nodes(code);
    location, ok := String_Literal.location_of_byte(literal, byte);
    return tprint("%:%:%", ok, location.line_number, location.character_number);
}

PLAIN_CODE :: #code "SELECT id FROM users";
ESCAPED_CODE :: #code "a\tb\x41\u00e9 \"q\" z";
EIGHT_CODE :: #code "\U0001F600!";

HERE_CODE :: #code #string SQL
SELECT 1
  FROM nowhere
SQL;

PERCENT_CODE :: #code #string,\% END
a\%b
END;

PLAIN :: #run where(PLAIN_CODE, 7);
ESCAPED :: #run where(ESCAPED_CODE, 11);
EIGHT :: #run where(EIGHT_CODE, 4);
END_AT :: #run where(PLAIN_CODE, 20);
HERE :: #run where(HERE_CODE, 12);
PERCENT :: #run where(PERCENT_CODE, 2);
OUTSIDE :: #run where(PLAIN_CODE, 99);

main :: () {
    expect("plain", PLAIN, "true:15:29");
    // a, \t, b, \x41, \u00e9 (two bytes), space, \", q, \", space, z: byte 11 is `z`.
    expect("escaped", ESCAPED, "true:16:45");
    expect("eight", EIGHT, "true:17:32");
    expect("end", END_AT, "true:15:42");
    expect("here", HERE, "true:21:4");
    expect("percent", PERCENT, "true:25:4");
    expect("outside", OUTSIDE, "false:15:21");
    if failures exit(1);

    print("ok\n");
}

failures := 0;

expect :: (what: string, got: string, wanted: string) {
    if got == wanted return;
    print("FAIL %: got %, wanted %\n", what, got, wanted);
    failures += 1;
}
