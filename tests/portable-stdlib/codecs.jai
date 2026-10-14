#import "Basic";
B :: #import "Base64";
C :: #import "Crc";
P :: #import "PCG";

main :: () {
    B.test_base64_module();
    inputs: [7] string = .["", "f", "fo", "foo", "foob", "fooba", "foobar"];
    outputs: [7] string = .["", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy"];

    for input, i: inputs {
        encoded := B.base64_encode(input);

        assert(encoded == outputs[i]);
        decoded, ok := B.base64_decode(encoded);

        assert(ok && decoded == input);
        free(encoded);
        free(decoded);
    }

    partial, valid := B.base64_decode("TWE!rest");

    assert(!valid && partial == "Ma");
    free(partial);
    partial, valid = B.base64_decode("TQ\r==");
    assert(!valid && partial == "M");
    free(partial);
    assert(C.crc64("123456789") == 0x62ec59e3f1a4f00a);
    state: u64;

    P.random_seed(*state, 42);
    expected: [5] u32 = .[0x75830bbd, 0x0e6dfdb2, 0xce19afdf, 0xd8cfe2c3, 0x012b061e];
    for value: expected assert(P.random_get(*state) == value);

    print("ok\n");
}
