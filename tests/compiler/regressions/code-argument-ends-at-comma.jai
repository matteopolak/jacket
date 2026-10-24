// A `#code` argument ends at the next comma of the argument list, so named arguments can follow
// it; a braced `#code { }` still holds a multi-value assignment.
#import "Basic";

count_args :: ($a: Code, $limit := 0, $b: Code = #code 0) -> s64 #expand {
    return limit;
}

// Outside an argument list the comma still belongs to the assignment.
PAIR :: #code a, b = 1, 2;

code_count :: (codes: ..Code) -> s64 {
    return codes.count;
}

main :: () {
    x := 1;

    assert(count_args(#code x == 1, limit = 3) == 3);
    assert(count_args(#code x = 2, b = #code x == 1) == 0);
    assert(count_args(#code {
        x, y := 1, 2;
    }, limit = 5) == 5);

    assert(code_count(PAIR) == 1);

    print("ok\n");
}
