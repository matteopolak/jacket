#import "Basic";

// A fixed-arity overload beats one that packs a variadic, even when the fixed one takes a
// constant parameter, so a call with no extra arguments can pick the cheaper specialisation.
describe :: ($text: string) -> string #modify {
    for 0..text.count - 1 if text[it] == #char "%" return false;
    return true;
} {
    return "fixed";
}

describe :: (text: string, values: ..Any) -> string {
    return "packed";
}

main :: () {
    print("%\n", describe("plain"));
    print("%\n", describe("with %"));
    print("%\n", describe("plain", 1));
}
