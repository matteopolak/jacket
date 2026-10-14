#import "Basic";
M :: #import "Memory"(with_candidates = true, with_unwraps = true);

main :: () {
    source: [160] u8;
    target: [160] u8;

    for *byte, i: source byte.* = cast(u8) i;

    M.copy_128(target.data, source.data);
    assert(target[15] == 15 && target[16] == 0);
    M.copy_256(target.data, source.data);
    assert(target[31] == 31 && target[32] == 0);
    M.sse_copy_1024(target.data, source.data);
    assert(target[127] == 127 && target[128] == 0);
    M.copy_1_u64(target.data, source.data, 160);
    assert(target[159] == 159);
    M.has_avx = .NO;
    assert(M.has_avx == .NO);
    print("ok\n");
}
