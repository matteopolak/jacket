#import "Basic";

Counter :: struct {
    hits: int;
}

main :: () {
    ready := true;
    count := 4;
    c: Counter;

    a := ifx ready then 1 else 0;
    b := ifx count > 3 then 0 else 1;
    small: u8 = ifx ready then 1 else 0;

    if ready n = 1;
    else n = 0;

    if count > 2 {
        c.hits = 0;
    } else {
        c.hits = 1;
    }

    print("% % % % %\n", a, b, small, n, c);
}

n: int;
