// rules: num.20
// A decimal constant must fit a signed type's range; only hex and binary constants may fill
// the sign bit.
main :: () {
    count: s16 = 40000;
}
