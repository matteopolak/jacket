#import "Basic";

LIMIT :: 100;

main :: () {
    x := 150;
    f := 0.5;
    a := max(0, min(100, x));
    b := min(LIMIT, max(0, x));
    c := max(min(100, x), 0);
    d := max(0.0, min(1.0, f));

    if x < 0 {
        x = 0;
    } else if x > 100 {
        x = 100;
    }

    if x > LIMIT x = LIMIT;
    else if x <= 0 x = 0;

    e := ifx x < 0 then 0 else ifx x > 255 then 255 else x;
    g := ifx f > 1.0 then 1.0 else ifx f < 0.0 then 0.0 else f;

    print("% % % % % %\n", a, b, c, d, e, g);
}
