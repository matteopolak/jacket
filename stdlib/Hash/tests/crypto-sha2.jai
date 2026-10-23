#import "Basic";
S :: #import "internal/crypto/sha";

hex :: (bytes: [] u8) -> string {
    digits := "0123456789abcdef";
    builder: String_Builder;

    for bytes {
        append(*builder, digits[it >> 4]);
        append(*builder, digits[it & 15]);
    }

    return builder_to_string(*builder);
}

main :: () {
    long := "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";

    assert(long.count == 112);

    a := S.sha512("abc");

    assert(hex(a) == "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f");
    a = S.sha512("");
    assert(hex(a) == "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");
    a = S.sha512(long);
    assert(hex(a) == "8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909");

    b := S.sha384("abc");

    assert(hex(b) == "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7");
    b = S.sha384("");
    assert(hex(b) == "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b");
    b = S.sha384(long);
    assert(hex(b) == "09330c33f71147e83d192fc782cd1b4753111b173b3b05d22fa08086e3b0f712fcc7c71a557e2db966c3e9fa91746039");

    print("ok\n");
}
