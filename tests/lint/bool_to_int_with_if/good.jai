#import "Basic";

main :: () {
    ready := true;
    count := 4;
    n := 0;

    a := cast(int) ready;
    b := ifx ready then 2 else 0;
    c := ifx ready then 1 else 3;
    d := ifx count > 9 then 1 else 3;
    e := ifx ready then 1.0 else 0.0;

    if ready n = 1;
    else n = 2;

    if ready {
        n = 1;
        print("on\n");
    } else {
        n = 0;
    }

    print("% % % % % %\n", a, b, c, d, e, n);
}
