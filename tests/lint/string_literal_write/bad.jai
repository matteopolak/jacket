#import "Basic";

main :: () {
    a := "hello";
    a[0] = #char "j";

    b: string = "world";
    b[1] += 1;
    b.data[2] = #char "x";
    b.data.* = #char "W";

    print("% %\n", a, b);
}
