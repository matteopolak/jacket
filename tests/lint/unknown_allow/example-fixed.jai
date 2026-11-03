#import "Basic";

// jailint: allow(float_equality)
is_zero :: (x: float) -> bool {
    return x == 0.0;
}

main :: () {
    print("%\n", is_zero(0.0));
}
