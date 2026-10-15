M :: #import "meow_hash";

main :: () {
    #asm { pxor value:, value; }
    M.MeowHash(M.MeowDefaultSeed, 1, null, value);
}
