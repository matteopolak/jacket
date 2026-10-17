#import "Basic";

main :: () {
    for exponent: 1..2046 {
        bits := ((cast(u64) exponent) << 52);
        value := (cast(*float64) *bits).*;
        print("%,%,%\n", 64, exponent, formatFloat(value, mode = .SHORTEST));
    }

    for exponent: 1..254 {
        bits := ((cast(u32) exponent) << 23);
        value := (cast(*float32) *bits).*;
        print("%,%,%\n", 32, exponent, formatFloat(value, mode = .SHORTEST));
    }
}
