// A 64-bit CRC of `s` (polynomial 0x42f0e1eba9ea3693, bits fed high first, start and end value all
// ones). It detects accidental corruption; it is not a cryptographic digest.
//
// ```jai
// assert(crc64("jacket") == crc64("jacket"));
// assert(crc64("jacket") != crc64("jackel"));
// ```
crc64 :: (s: string) -> u64 #no_aoc {
    crc: u64 = 0xffffffffffffffff;

    for byte: s {
        crc ^= (cast(u64) byte) << 56;

        for 0..7 {
            top := (crc >> 63) != 0;
            crc <<= 1;
            if top crc ^= 0x42f0e1eba9ea3693;
        }
    }

    return crc ^ 0xffffffffffffffff;
}
