// Checks the rules of docs/language/strings-and-literals.md.
// rules: str.1 str.2 str.3 str.4 str.6 str.7 str.8 str.9 str.10 str.11 str.12 str.13 str.14 str.15 str.17 str.18
#import "Basic";

bytes_are :: (s: string, expected: [] u8) -> bool {
    if s.count != expected.count return false;
    for expected if s[it_index] != it return false;
    return true;
}

first_three :: (p: *u8) -> [3] u8 {
    out: [3] u8;
    out[0] = p[0];
    out[1] = p[1];
    out[2] = p[2];
    return out;
}

HERE :: #string DONE
  indented "quotes" \n stay
DONEX is not the end
DONE

CRLF :: #string,cr DONE
a
b
DONE

PERCENT :: #string,\% DONE
x\%y
DONE

main :: () {
    // str.1: a string is a count and a data pointer.
    assert(size_of(string) == 16, "str.1 size");
    probe := "probe";
    #assert type_of(probe.count) == s64;
    #assert type_of(probe.data) == *u8;
    assert((cast(*s64) *probe).* == 5, "str.1 count first");
    assert((cast(**u8) (cast(*u8) *probe + 8)).* == probe.data, "str.1 data second");

    // str.2: arbitrary bytes, including NUL and invalid UTF-8.
    raw := "a\0\xff\xfeb";
    assert(raw.count == 5 && raw[1] == 0 && raw[2] == 0xff && raw[4] == #char "b", "str.2");

    // str.3: the documented example.
    s := "a\tb\x41\d066é\0z";
    assert(bytes_are(s, .[97, 9, 98, 65, 66, 195, 169, 0, 122]), "str.3");

    // str.4: every escape.
    assert(bytes_are("\n\r\t\0\e\a\b\f\v", .[10, 13, 9, 0, 27, 7, 8, 12, 11]), "str.4 controls");
    assert(bytes_are("\\\"\'\%", .[92, 34, 39, 0x1f]), "str.4 quotes");
    assert(bytes_are("\x7a\d255", .[0x7a, 255]), "str.4 numeric");
    assert(bytes_are(" ​\U0001F600", .[0xc2, 0xa0, 0xe2, 0x80, 0x8b, 0xf0, 0x9f, 0x98, 0x80]), "str.4 unicode");

    // str.6: #char is the byte value.
    assert(#char "A" == 65 && #char "\n" == 10, "str.6");

    // str.7, str.8, str.9: here-string body, terminator and indentation.
    assert(HERE == "  indented \"quotes\" \\n stay\nDONEX is not the end\n", "str.7 str.8 str.9");

    // str.10, str.11: line endings.
    assert(CRLF == "a\r\nb\r\n", "str.11");
    for HERE assert(it != #char "\r", "str.10");

    // str.12: #string,\% makes \% the 0x1f byte.
    assert(bytes_are(PERCENT, .[#char "x", 0x1f, #char "y", 10]), "str.12");

    // str.13: comparison by length and bytes.
    assert(!("x\0y" == "x\0z"), "str.13 equal");
    assert("abc" != "ab\0", "str.13 not equal");
    assert("ab" == "ab" && "ab" != "abc", "str.13 length");

    // str.14, str.15: a literal as *u8 gets a trailing NUL; embedded NULs survive.
    c: *u8 = "hi";
    assert(c[0] == #char "h" && c[1] == #char "i" && c[2] == 0, "str.14");
    seen := first_three("A\0B");
    assert(seen[0] == #char "A" && seen[1] == 0 && seen[2] == #char "B", "str.15");

    // str.17: advance drops leading bytes.
    view := "hello";
    advance(*view, 2);
    assert(view == "llo", "str.17");

    // str.18: iteration yields u8 bytes.
    total := 0;
    for ch: "hey" {
        #assert type_of(ch) == u8;
        total += ch;
    }
    assert(total == #char "h" + #char "e" + #char "y", "str.18");

    print("ok\n");
}
