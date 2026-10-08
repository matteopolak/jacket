// rules: cast.35
// A cast of a constant checks its range at compile time; cast,trunc keeps the low bits.
main :: () {
    byte := cast(u8) 300;
}
