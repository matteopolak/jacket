// A procedure alias (`starts_with :: begins_with;` in String) joins the overload set
// of a same-named procedure in the importing module; an integer meeting a float in
// arithmetic becomes that float; String's replace reports its occurrence count.
// rules: proc.5
#import "Basic";
#import "String";

starts_with :: inline (s: string, ch: u8) -> bool {
    return s.count > 0 && s[0] == ch;
}

main :: () {
    assert(starts_with("-x", "-"));
    assert(starts_with("-x", #char "-"));

    top: s32 = 7;
    height: float = 2;
    lines := top / height;
    assert(type_of(lines) == float && lines == 3.5);
    big: s64 = 3;
    assert(big * 0.5 == 1.5 && height < top);

    text, occurrences := replace("a[0]b[0]", "[0]", "[1]");
    assert(text == "a[1]b[1]" && occurrences == 2);
    print("ok\n");
}
