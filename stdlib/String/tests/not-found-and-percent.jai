// Not-found results of the splits differ by overload, `%%` is checked but left unconsumed by
// `scan`, and substring search finds matches from any start offset.
#import "Basic";
S :: #import "String";

main :: () {
    found, left, right := S.split_from_right("abc", cast(u8) 124);

    assert(!found && left == "" && right == "abc");
    found, left, right = S.split_from_right("abc", "|");
    assert(!found && left == "abc" && right == "");
    found, left, right = S.split_from_left("abc", cast(u8) 124);
    assert(!found && left == "abc" && right == "");
    found, left, right = S.split_from_right("a|b|c", cast(u8) 124);
    assert(found && left == "a|b" && right == "c");

    assert(S.find_index_from_left("abcabc", "bc") == 1);
    assert(S.find_index_from_left("abcabc", "bc", 2) == 4);
    assert(S.find_index_from_left("abcabc", "cx") == -1);
    assert(S.find_index_from_left("abcabc", cast(u8) 99, -5) == 2);

    ok, values := S.scan("%%", "%");

    assert(!ok && values.count == 0);
    ok, values = S.scan("%%", "x");
    assert(!ok);

    assert(S.scan2("%", "%%"));
    print("ok\n");
}
