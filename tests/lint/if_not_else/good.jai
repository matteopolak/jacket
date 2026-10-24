#import "Basic";

main :: () {
    ready := false;
    a := 3;
    b := 4;

    if ready {
        print("going\n");
    } else {
        print("waiting\n");
    }

    if !ready {
        print("waiting\n");
    }

    if !ready {
        print("waiting\n");
    } else if a == b {
        print("same\n");
    } else {
        print("going\n");
    }

    if a == b {
        print("same\n");
    } else {
        print("differ\n");
    }

    label := ifx ready then "go" else "wait";
    maybe := ifx !ready then 1;

    print("% %\n", label, maybe);
}
