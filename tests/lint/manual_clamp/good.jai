#import "Basic";

main :: () {
    x := 150;
    lo := 0;
    hi := 100;
    f := 0.5;

    a := clamp(x, 0, 100);
    b := max(lo, min(hi, x)); // the bounds are not constants
    c := max(0, x);
    d := min(100, max(x, lo)); // only one bound is constant
    e := max(0, min(100, x) + 1);

    if x < 0 {
        x = 0;
    } else if x > 100 {
        x = 99; // not the bound
    }

    if x < 0 {
        x = 0;
    } else if x > 100 {
        x = 100;
    } else {
        x += 1;
    }

    if x < 0 {
        x = 0;
        print("low\n");
    } else if x > 100 {
        x = 100;
    }

    if x < 100 {
        x = 100;
    } else if x > 0 {
        x = 0; // the bounds are swapped
    }

    g := ifx x < 0 then 0 else ifx x > 255 then 254 else x;
    h := ifx f < 0.0 then 0.0 else f;

    print("% % % % % % %\n", a, b, c, d, e, g, h);
}
