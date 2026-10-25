#import "Basic";

Key :: struct {
    packed: u32;
}

main :: () {
    key := Key.{ 0x04030201 };
    bytes := (*key).([4] u8);

    assert(size_of(type_of(bytes)) == 4, "a pointer to a value cast to an array of its size reads the value as that array");
    assert(bytes[0] == 1 && bytes[3] == 4, "the bytes are the value's");
    print("ok\n");
}
