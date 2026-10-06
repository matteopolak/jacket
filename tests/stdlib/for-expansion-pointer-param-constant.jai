#import "Basic";
#import "Unicode";

// `for :utf8_iter X` takes a `*string` parameter; a constant (non-addressable) string
// is spilled to a temporary and its address passed.
main :: () {
    S :: "héllo";
    count := 0;
    for: utf8_iter S {
        count += 1;
    }
    assert(count == 5, "got % chars", count);

    s := "abc";
    sum := 0;
    for: utf8_iter s {
        sum += it_index;
    }
    assert(sum == 3, "got %", sum);
}
