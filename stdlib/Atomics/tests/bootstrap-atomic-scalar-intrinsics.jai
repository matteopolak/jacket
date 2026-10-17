// Original real-prelude scalar intrinsic contract, with no local declarations.
#import "Basic";

Token :: enum u32 {
    FIRST :: 7;
    SECOND :: 29;
}

first :: () -> int #no_context {
    return 13;
}

second :: () -> int #no_context {
    return 47;
}

main :: () {
    value: u64;

    atomic_store(*value, 0xfedcba9876543210);
    assert(atomic_load(*value) == 0xfedcba9876543210);
    assert(atomic_exchange(*value, 29) == 0xfedcba9876543210);
    success, previous := compare_and_swap(*value, 29, 47);

    assert(success && previous == 29 && value == 47);
    token: Token = .FIRST;

    assert(atomic_exchange(*token, .SECOND) == .FIRST);
    assert(atomic_load(*token) == .SECOND);
    procedure: () -> int #no_context = first;
    old := atomic_exchange(*procedure, second);

    assert(old() == 13 && atomic_load(*procedure)() == 47);
    print("ok\n");
}
