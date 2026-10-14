// Byte-oriented search, split, token parsing, and line-ending behavior.
#import "Basic";
String :: #import "String";

main :: () {
    found, tail := String.contains("copper/blue", "per/");
    assert(found && tail == "blue");

    fields := String.split(";a;;", #char ";");
    assert(fields.count == 4);
    assert(fields[0] == "" && fields[1] == "a" && fields[2] == "" && fields[3] == "");
    assert(String.split("", #char ",").count == 1);
    assert(String.DEFAULT_DELIMITERS == ";=()" && String.DEFAULT_SPACES == " \t\r\n");

    letters: [5] u8 = .[77, 105, 88, 101, 68];
    mutable: string;
    mutable.data = letters.data;
    mutable.count = letters.count;
    String.to_lower_in_place(mutable);
    assert(mutable == "mixed");

    input := "   name  =value";
    token, ok := String.parse_token(*input);
    assert(ok && token == "name" && input == "  =value");
    token, ok = String.parse_token(*input);
    assert(ok && token == "=" && input == "value");

    normalized := String.normalize_line_endings("one\r\ntwo\rthree");
    assert(normalized == "one\ntwo\rthree");
    print("ok\n");
}
