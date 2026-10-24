#import "Basic";

main :: () {
    t := 0.0;

    while t < 1.0 {
        print("%\n", t);
        t += 0.1;
    }

    remaining: float64 = 10;
    dt := 0.25;

    while 0 < remaining {
        remaining = remaining - dt;
    }
}
