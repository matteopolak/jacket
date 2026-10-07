#import "Basic";

find_a :: (s: string, sub: string, start_index := 0) -> s64 {
    return start_index + 100;
}

find_b :: (s: string, sub: string, start_index := 0) -> s64 {
    return start_index + 200;
}

find_b :: (s: string, sub: u8) -> s64 {
    return -1;
}

main :: () {
    whole := true;
    g: (s: string, substring: string, start_index := 0) -> s64;
    g = ifx whole then find_b else find_a;
    assert(g("x", "y", start_index = 4) == 204);
    whole = false;
    g = ifx whole then find_b else find_a;
    assert(g("x", "y", 1) == 101);
    assert(g("x", "y") == 100);
    assert(g(substring = "y", s = "x", start_index = 2) == 102);
    print("ok\n");
}

inferred :: () {
    whole := false;
    h := ifx whole then find_a else find_b;
    assert(h("x", "y") == 200);
    k := ifx whole then find_b else find_a;
    assert(k("x", "y", 5) == 105);
}

#run inferred();
