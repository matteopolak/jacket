// A one-character string constant converts to u8: comparisons and switch cases on bytes,
// arithmetic next to an integer, `u8` arguments and casts (sjorsdonkers/toml-jai's parser).
// rules: str.19
#import "Basic";

split_at :: (s: string, separator: u8) -> string {
    return "byte overload";
}

split_at :: (s: string, separator: string) -> string {
    return "string overload";
}

digit_value :: (c: u8) -> int {
    return c - "0";
}

main :: () {
    c: u8 = #char "'";
    assert(c == "'");
    hit := false;
    if c == {
        case "'";
            hit = true;
        case "x";
            hit = false;
    }
    assert(hit);

    assert(digit_value(#char "7") == 7);
    d: u8 = #char "b";
    assert(d - "a" + 10 == 11);
    assert("a" + 1 == #char "b");

    take_byte :: (b: u8) -> u8 {
        return b;
    }
    assert(take_byte(".") == #char ".");
    // A `string` overload is still the closer match.
    assert(split_at("a.b", ".") == "string overload");

    assert(cast(u8) "\u001F" == 31);
    total := 0;
    for cast(u8, "\u0000") ..cast(u8, "\u0003") total += 1;
    assert(total == 4);
    print("ok\n");
}
