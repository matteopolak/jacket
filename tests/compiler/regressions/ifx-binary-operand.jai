// An `ifx` of untyped numbers next to an unsigned operand takes that operand's type, unlike one
// assigned to an unsigned target (focus's Input module turns a shifted letter into its key code
// with `char - (ifx lower then 32 else 0)`).
#import "Basic";

fold :: (char: u16) -> u16 {
    return char - (ifx char >= #char "a" && char <= #char "z" then 32 else 0);
}

main :: () {
    wide: u64 = 8;
    on := true;

    assert(fold(#char "a") == #char "A");
    assert(fold(#char "Q") == #char "Q");
    assert(wide == (ifx on then 8 else 4));
    assert(wide + (ifx on then 2 else 1) == 10);
    print("ok\n");
}
