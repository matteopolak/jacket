// Not-found results of the right-hand splits agree across overloads, `%%` matches a percent
// sign in both scanners, and a negative start index searches from the beginning.
#import "Basic";
S :: #import "String";

main :: () {
    found, left, right := S.split_from_right("abc", cast(u8) 124);

    assert(!found && left == "abc" && right == "");
    found, left, right = S.split_from_right("abc", "|");
    assert(!found && left == "abc" && right == "");
    found, left, right = S.split_from_left("abc", cast(u8) 124);
    assert(!found && left == "abc" && right == "");
    found, left, right = S.split_from_right("a|b|c", cast(u8) 124);
    assert(found && left == "a|b" && right == "c");

    assert(S.find_index_from_left("abcabc", "bc", -5) == 1);
    assert(S.find_index_from_left("abcabc", "bc", 2) == 4);
    assert(S.find_index_from_left("abcabc", cast(u8) 99, -5) == 2);

    ok, values := S.scan("%%", "%");

    assert(ok && values.count == 0);
    ok, values = S.scan("100%%", "100%");
    assert(ok);
    ok, values = S.scan("%d%%", "50%");
    assert(ok && values.count == 1);
    ok, values = S.scan("%%", "x");
    assert(!ok);

    assert(S.scan2("%", "%%"));
    number: int;

    assert(S.scan2("50%", "%d%%", *number) && number == 50);
    assert(!S.scan2("50", "%d%%", *number));
    print("ok\n");
}
