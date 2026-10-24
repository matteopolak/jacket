#import "Basic";

main :: () {
    // Counting with an integer.
    for i: 0..9 {
        t := i * 0.1;
        print("%\n", t);
    }

    // An integer counter next to a float value.
    count := 0;
    total := 0.0;

    while count < 10 {
        total += 0.1;
        count += 1;
    }

    // A convergence loop is not counting.
    x := 2.0;
    err := 1.0;

    while err > 0.0001 {
        next := (x + 2.0 / x) / 2;
        err = next - x;
        x = next;
    }

    // The step depends on the variable.
    y := 1.0;

    while y < 100 {
        y *= 2;
    }

    z := 1.0;

    while z < 100 {
        z += z;
    }

    print("% % % % %\n", total, x, err, y, z);
}
