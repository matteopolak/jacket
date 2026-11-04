#import "Basic";
make_number :: () -> int { return 42; }
freeze_bytes :: (value: $T) -> [] u8 {
    bytes: [size_of(T)] u8;
    memcpy(*bytes, *value, size_of(T));
    return bytes;
}
FROZEN :: #run freeze_bytes(make_number());
main :: () { print("%", FROZEN); }
