#import "Basic";

main :: () {
    ready := false;
    a := 3;
    b := 4;

    if !ready {
        print("waiting\n");
    } else {
        print("going\n");
    }

    if a != b {
        print("differ\n");
    } else {
        print("same\n");
    }

    if !(a > b && ready) print("x\n");
    else print("y\n");

    label := ifx !ready then "wait" else "go";
    size := ifx a != b then 1 else 2;

    print("% %\n", label, size);
}
