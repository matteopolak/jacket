#import "Basic";

main :: () {
    n := 3;
    if n > 2 {
        print("big\n");
    } else {
        print("small\n");
    }
    if n > 0 {} else {}
    #if OS == .WINDOWS {
        print("x\n");
    } else {
        print("x\n");
    }
    print("%\n", ifx n > 0 then "pos" else "neg");
    print("%\n", sign(n));
}

// The last case is spelled out before the default on purpose.
sign :: (n: int) -> int {
    if n > 0 {
        return 1;
    } else if n == 0 {
        return 0;
    } else {
        return 0;
    }
}
