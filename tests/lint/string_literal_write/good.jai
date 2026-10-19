#import "Basic";

main :: () {
    a := "hello";
    a = copy_string(a);
    a[0] = #char "j";

    b := copy_string("world");
    b[1] = #char "x";

    c := "abc";

    if a.count > 3 {
        c = copy_string(c);
    }

    c[0] = #char "z";

    d := "abc";
    e := *d;

    d[0] = #char "q";
    print("% % % % %\n", a, b, c, d, e);
}
