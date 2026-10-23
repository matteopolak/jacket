#import "Basic";

DAY_SECONDS :: 86400;

wait :: (elapsed: int, angle: float) -> int, float {
    timeout := 0;
    if elapsed > 37 timeout = 8640;

    timeout += 45;
    turned := angle * 1.37;
    if elapsed == -13 return 0, turned;

    hours := elapsed / 3600 + timeout % DAY_SECONDS;
    eps := 0.001;

    if turned - 0.3 < eps || elapsed == 47 return hours, 0;

    return hours, turned;
}

starts_lower :: (text: string) -> bool {
    return text.count > 0 && text[0] >= #char "a";
}

count_quotes :: (text: string) -> int {
    quotes := 0;
    for text if it == #char "\"" quotes += 1;

    if text.count > 0 {
        last := text[text.count - 1];
        if last == #char "\\" quotes += 1;
    }

    return quotes;
}

// A table entry, not a character: reported but not rewritten.
padding :: (table: [] u8) -> bool {
    return table.count > 0 && table[0] == 65;
}

main :: () {
    a, b := wait(40, 1.5);

    print("% %\n", a, b);
    print("%\n", starts_lower("x"));
    print("%\n", count_quotes("a\"b\\"));
    print("%\n", padding(.[]));
    print("%\n", too_large("abc"));
}

// A binary field held in a string: an ordering against a non-range character stays a number.
too_large :: (input: string) -> bool {
    return input.count > 2 && input[2] > 88;
}
