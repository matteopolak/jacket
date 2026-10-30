// A `#assert` whose condition holds a braced body needs no `;` after it or its message.
// rules: note.13
#import "Basic";

round_down :: (n: s64, $base: s64) -> s64 {
    #assert #run -> bool {
        value := cast(s64) base;
        return value == 1 || value == 2;
    } "base must be 1 or 2"

    mask := ~(base - 1);

    return n & mask;
}

main :: () {
    #assert #run -> bool {
        return true;
    }

    assert(round_down(5, 2) == 4);
    print("ok\n");
}
