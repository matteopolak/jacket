#import "Basic";

visit :: (user_data: $T, proc: (n: int, user_data: T)) {
    proc(3, user_data);
}
store :: (x: $T) -> T {
    return x;
}

total := 0;

main :: () {
    visit(null, (n: int, d: *void) {
        total += n;
    });
    assert(total == 3);
    p := store(null);
    assert(type_of(p) == *void && p == null);
    print("ok\n");
}
