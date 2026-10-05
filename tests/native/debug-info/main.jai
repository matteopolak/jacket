// Native debug information fixture (crates/jaic-cli/tests/debug_info.rs): a multi-file
// program with locals of many types and a polymorphic procedure. Keep line numbers stable.
#import "Basic";
#load "shapes.jai";

counter: int = 7;

main :: () {
    a: int = 42;
    b: float64 = 3.5;
    flag := true;
    name := "hello";
    v := Vec2.{1, 2};
    c := Color.GREEN;
    arr: [4] s32 = .[1, 2, 3, 4];
    ints := int.[10, 20, 30];
    list: [..] int;
    array_add(*list, 5);
    n2: Node;
    n1 := Node.{1, *n2};
    p := *a;
    w := scale(v, 2);
    s := total_of(ints);
    f := total_of(float.[1.5, 2.5]);
    if a > 10 {
        inner := a + 1;
        print("% %\n", inner, s);  // line 27
    }
    counter += 1;
    print("% % % % % % % % % % %\n", b, flag, name, w, c, arr, n1.value, f, p.*, list, counter);
}
