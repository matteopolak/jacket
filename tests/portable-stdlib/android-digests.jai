// Reviewed Android/Toolchain digest signatures; standard SHA digest vectors.
#import "Basic";
A :: #import "Android/Toolchain";

as_hex :: (data: [] u8) -> string {
    result := alloc_string(data.count * 2);
    digits := "0123456789abcdef";

    for value, index: data {
        result[index * 2] = digits[value >> 4];
        result[index * 2 + 1] = digits[value & 15];
    }

    return result;
}

main :: () {
    one := A.sha1("");
    two := A.sha256("");
    text1 := as_hex(one);
    text2 := as_hex(two);

    assert(text1 == "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert(text2 == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    free(text1);
    free(text2);
    one = A.sha1("abc");
    two = A.sha256("abc");
    text1 = as_hex(one);
    text2 = as_hex(two);
    assert(text1 == "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert(text2 == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    free(text1);
    free(text2);
    print("ok\n");
}
