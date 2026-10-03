Bits :: union { first: u32; second: u32; }
main :: () -> int { bits: Bits; bits.first = 27; return cast(int) bits.second; }
