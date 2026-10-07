// Close to the machine: inline x64 assembly with #asm, and C's `long double`.

#import "Extensions/Long_Double";

tour_machine :: () {
    // #asm blocks name Jai variables directly. jaic lowers each instruction
    // to plain operations, so the same block runs natively and in the browser.
    word: u64 = 0x0123_4567_89ab_cdef;
    #asm {
        mov swapped:, word;
        bswap swapped;
        mov word, swapped;
    }
    print("bswap reverses the bytes: 0x%\n", formatInt(word, base = 16));

    bits: u64 = 0b1011_0000_1111;
    ones, leading: u64;
    #asm {
        popcnt ones, bits;
        lzcnt leading, bits;
    }
    print("0b% has % bits set and % leading zeros\n", formatInt(bits, base = 2), ones, leading);

    // `mul` gives the full 128-bit product in two registers: high and low.
    high: u64;
    low: u64 = 0xffff_ffff_ffff_ffff;
    three: u64 = 3;
    #asm { mul high, low, three; }
    print("(2^64 - 1) * 3 = % * 2^64 + %\n", high, low);

    // Long_Double is C's `long double` for the target. On WebAssembly it is a
    // 128-bit float with a 113-bit mantissa, done in software by the compiler.
    // Where it is the same as float64 (Apple arm64, Windows) both rows match.
    print("Long_Double is % bytes; wider than float64: %\n",
        size_of(Long_Double), LONG_DOUBLE_IS_WIDE);
    print("  sqrt(2) as float64:     ");
    print_digits(square_root(cast(float64) 2), 34);
    print("  sqrt(2) as Long_Double: ");
    print_digits(square_root(cast(Long_Double) 2), 34);
    print("  the true value:         1.414213562373095048801688724209698\n");
}

// Newton's method, for any float type.
square_root :: (x: $T) -> T {
    guess := x;
    for 1..60 guess = (guess + x / guess) / 2;
    return guess;
}

// Prints a number in [1, 10) digit by digit, beyond what `print` shows.
print_digits :: (x: $T, count: int) {
    whole := cast(int) x;
    print("%.", whole);
    rest := x - cast(T) whole;
    for 1..count - 1 {
        rest *= 10;
        digit := cast(int) rest;
        print("%", digit);
        rest -= cast(T) digit;
    }
    print("\n");
}
