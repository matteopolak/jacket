// rules: lambda.1 lambda.2 lambda.3 lambda.6
// Anonymous procedures and lambdas (more in lang-lambdas.jai).
#import "Basic";

n: int;
total: int;

twice :: (body: () -> (), times: int) {
    for 1..times body();
}

apply_pair :: (a: int, b: int, f: (int, int) -> int) -> int {
    return f(a, b);
}

scale :: (x) => x * 3;

main :: () {
    // lambda.6: a block-bodied anonymous procedure runs its body on every call.
    twice(() {
        n += 1;
    }, 3);
    assert(n == 3, "block-bodied procedure literal");

    // lambda.1: a procedure literal with a body, `(x) => expr` and bare `x => expr`.
    square := (x: int) -> int {
        return x * x;
    };
    assert(square(7) == 49, "procedure literal with a body");
    assert(apply_pair(4, 5, (a, b) => a - b) == -1, "parenthesised lambda");
    twice(() => {
        total += 10;
    }, 2);
    assert(total == 20, "lambda with no parameters");

    // lambda.2: parameter types come from the expected procedure type.
    halve: (float64) -> float64 = x => x / 2;
    assert(halve(5) == 2.5, "lambda typed from its declaration");

    // lambda.3: a lambda bound with `::` is polymorphic; each use gets its own instance.
    assert(scale(4) == 12, "int instance");
    assert(scale(0.5) == 1.5, "float instance");
    assert(type_of(scale(cast(u8) 2)) == u8, "u8 instance keeps its type");
    print("ok\n");
}
