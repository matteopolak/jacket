// A procedure declared inside a procedure hides the module-level procedures of its name, also
// when it is used as a value.
// rules: scope.20
#import "Basic";
#import "Math";

apply :: (callback: (float) -> float, x: float) -> float {
    return callback(x);
}

main :: () {
    sin :: (x: float) -> float {
        return x + 41;
    }

    assert(apply(sin, 1.0) == 42);
    assert(sin(1.0) == 42);
    print("ok\n");
}
