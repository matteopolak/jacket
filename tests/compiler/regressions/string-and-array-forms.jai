// Strings, fixed arrays, resizable arrays and array views.
#import "Basic";
#import "String";

MY_STRING :: #string DONE
This string has "quotes" and \backslashes\ with no escaping needed.
It can span two lines.
DONE

main :: () {
    // A string is a view of bytes: assignment copies the view, not the bytes.
    x: string;

    assert(x.count == 0 && x.data == null);
    x = "Hello";
    assert(x.count == 5);
    y := x;

    assert(y.data == x.data);

    // Subscripts give a u8.
    s := "Sailor";

    assert(s[0] == 83);
    assert(type_of(s[0]) == u8);
    assert(type_of(s.count) == s64);
    assert(type_of(s.data) == *u8);
    assert(size_of(string) == 16);

    // Substrings by editing the view.
    s.count = 4;
    assert(s == "Sail");
    s.count -= 1;
    s.data += 1;
    assert(s == "ail");

    // Literals carry a zero byte after the last character.
    lit := "Hello";

    assert(lit.data[lit.count] == 0);

    // #char.
    w: u8 = #char "W";

    assert(w == 87);
    assert((#char " ") == 32);
    commas := 0;
    for "a,b,,c" if it == #char "," commas += 1;

    assert(commas == 3);

    // sprint formats into a new string.
    n := 69105;
    message := sprint("There are % leaves in the pile. %", n, "For sure!!");

    assert(message == "There are 69105 leaves in the pile. For sure!!");
    free(message);

    builder: String_Builder;

    append(*builder, "One!");
    append(*builder, "Two!");
    print_to_builder(*builder, " ... number %, exclamation %.", 42, "wow");
    result := builder_to_string(*builder);

    assert(result == "One!Two! ... number 42, exclamation wow.");
    free(result);

    // Escape sequences.
    esc := "\e\n\r\t\"\\\0\x41\d066é";

    assert(esc[0] == 27 && esc[1] == 10 && esc[2] == 13 && esc[3] == 9);
    assert(esc[4] == #char "\"" && esc[5] == #char "\\" && esc[6] == 0);
    assert(esc[7] == 0x41 && esc[8] == 66);
    assert(esc[9] == 0xC3 && esc[10] == 0xA9 && esc.count == 11);
    assert("\U0001F600".count == 4);

    // Here strings keep their text verbatim, including the final newline.
    assert(MY_STRING.count > 0 && MY_STRING[MY_STRING.count - 1] == 10);
    assert(begins_with(MY_STRING, "This string has \"quotes\" and \\backslashes\\"));
    lines := 0;
    for MY_STRING if it == 10 lines += 1;

    assert(lines == 2);

    // Fixed arrays are zeroed and know their size.
    values: [4] int;
    for values assert(it == 0);

    values[0] = 10;
    values[3] = 40;
    assert(values.count == 4 && values.data == *values[0]);
    literal: [4] int = .[10, 20, 30, 40];

    assert(literal[2] == 30);
    literal[2] = 99;
    assert(literal[2] == 99);
    copy := literal;
    copy[0] = -1;
    assert(literal[0] == 10);

    // Array views.
    base: [5] int = .[10, 20, 30, 40, 50];
    view: [] int;
    view.data = base.data;
    view.count = base.count;
    assert(view[0] == 10 && view.count == 5);
    view[1] = 21;
    assert(base[1] == 21);

    sub: [] int;
    sub.data = base.data + 1;
    sub.count = 3;
    assert(sub[0] == 21 && sub[2] == 40);
    sub[0] = 22;
    assert(base[1] == 22);

    sum := 0;
    for sub sum += it;

    assert(sum == 22 + 30 + 40);

    // A fixed array converts to a view in calls.
    assert(total(base) == 10 + 22 + 30 + 40 + 50);

    // Resizable arrays.
    grown: [..] int;

    assert(grown.count == 0 && grown.allocated == 0);
    array_add(*grown, 10);
    array_add(*grown, 20);
    array_add(*grown, 30);
    assert(grown.count == 3 && grown.allocated >= 3);
    grown[0] = 99;
    assert(grown[0] == 99 && grown[1] == 20);

    names: [..] string;

    array_add(*names, "alpha");
    array_add(*names, "beta");
    extra := string.["gamma", "delta"];

    array_add(*names, ..extra);
    assert(names.count == 4 && names[2] == "gamma" && names[3] == "delta");
    array_add(*names, "x", "y");
    assert(names.count == 6 && names[5] == "y");

    array_ordered_remove_by_index(*grown, 0);
    assert(grown.count == 2 && grown[0] == 20 && grown[1] == 30);
    array_add(*grown, 40);
    array_unordered_remove_by_index(*grown, 0);
    assert(grown.count == 2 && grown[0] == 40 && grown[1] == 30);

    // A resizable array converts to a view that shares its storage.
    shared: [] int = grown;

    assert(shared.data == grown.data && shared.count == grown.count);
    shared[0] = 7;
    assert(grown[0] == 7);
    assert(total(grown) == 37);

    // Capacity does not shrink the count.
    big: [..] int;
    for 1..100 array_add(*big, it);

    assert(big.count == 100 && big.allocated >= 100);
    array_reset(*big);
    assert(big.count == 0);

    array_free(grown);
    array_free(names);
    array_free(big);

    print("ok\n");
}

total :: (values: [] int) -> int {
    sum := 0;
    for values sum += it;
    return sum;
}
