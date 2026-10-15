#import "Basic";

braced_else :: (a: bool, b: bool) -> s64 {
    if a {
        return 1;
    } else {
        if b return 2;
    }

    return 3;
}

chained_else :: (a: bool, b: bool) -> s64 {
    if a {
        return 1;
    } else if b return 2;

    return 3;
}

switch_like :: (n: s64) -> s64 {
    if n == {
        case 1; return 1;
        case 2; return 2;
        case; return 3;
    }
}

ifx_points :: (a: bool) -> s64 {
    return ifx a then 1 else 2;
}

static_points :: () -> s64 {
    #if true {
        return 1;
    } else {
        return 2;
    }
}

main :: () {
    assert(static_points() == 1);
    assert(switch_like(8) == 3 && ifx_points(false) == 2);
    assert(braced_else(false, true) == 2 && chained_else(false, false) == 3);
}
