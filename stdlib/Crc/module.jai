// A 64-bit CRC checksum: `crc64` for detecting corrupted data.
//
// ```jai
// assert(crc64("jacket") == crc64("jacket"));
// ```

crc64 :: Impl.crc64;

#scope_file
Impl :: #import "internal/codecs/crc";
