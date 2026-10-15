// Original reproducer for a test-packet construction error, not a compiler bug.
// The shift binds inside the cast operand, so signed32 shifts before widening.
// The result carries sign-extended upper bits instead of a zero-extended native
// word. The exact-packet assertion fails; message tests use a typed u64 local
// before shifting instead. Compiler arithmetic/check semantics are unchanged.
#import "Basic";

main :: () {
    word: s32 = 65416;
    packed := (cast(u64) word << 16);
    print("packed=%, expected=%\n", packed, 0xff880000);
    assert(packed == 0xff880000);
}
