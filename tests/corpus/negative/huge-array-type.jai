// Found by fuzzing: the size of this type overflowed 64 bits and crashed the compiler.
Grid :: [1 << 40] [1 << 30] u8;

main :: () {
    g: Grid;
}
