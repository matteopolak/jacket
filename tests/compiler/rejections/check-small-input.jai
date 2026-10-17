#program_export "small_sum"
small_sum :: (a: u8, b: u8) -> u8 #c_call {
    return a + b;
}

main :: () {}
