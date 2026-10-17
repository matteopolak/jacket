#import "Basic";

conditional :: (a: bool, b: bool, c: bool) -> s64 {
    if a {
        if b return 1;
    } else if c {
        return 2;
    } else {
        return 3;
    }

    return 4;
}

allocation :: () {
    p := alloc(8);
    p = realloc(p, 16, 8);
    free(p);
    text := copy_string("original");

    free(text);
}

main :: () {
    assert(conditional(true, false, false) == 4);
    allocation();
    print("ok\n");
}
