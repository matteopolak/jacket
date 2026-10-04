#import "Basic";

Flags :: enum_flags u32 { A; B; C; }

pick :: (s: string, flags := Flags.A) -> u32 { return cast(u32) flags; }
pick :: (n: int, reversed := false) -> u32 { return 99; }

main :: () {
    assert(pick("x", .A | .C) == 5);
    assert(pick("x", (.A | .B | .C) & ~.B) == 5);
    assert(pick(3) == 99);
    print("ok\n");
}
