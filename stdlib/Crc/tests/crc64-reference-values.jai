// crc64 matches the CRC-64/WE check value and the bit-by-bit reference for short strings.
#import "Basic";
Crc :: #import "Crc";

main :: () {
    // The check value of CRC-64/WE, the parameter set this polynomial, init and final XOR define.
    assert(Crc.crc64("123456789") == 0x62EC59E3F1A4F00A);
    assert(Crc.crc64("") == 0);
    assert(Crc.crc64("a") == 0xCE73F427ACC0A99A);
    assert(Crc.crc64("ab") == 0x8DD7F7A401A658A);
    assert(Crc.crc64("ba") == 0x7C33C210E75D69B7);
    assert(Crc.crc64("The quick brown fox jumps over the lazy dog") == 0xBCD8BB366D256116);

    print("ok\n");
}
