#import "Basic";
G :: #import "developer-tools-bitfields-generated";

main :: () {
    bits: G.Bits;

    assert(size_of(G.Bits) == 8 && align_of(G.Bits) == 4);
    G.initialize(*bits, 1, 9, 1, 7);
    assert(G.get_first(bits) == 1 && G.get_second(bits) == 9 && G.get_on(bits) && G.get_final(bits) == 7);
    bits.__bitfield[4] |= 0xa8;
    G.set_first(*bits, 6);
    G.set_second(*bits, 3);
    G.set_on(*bits, false);
    G.set_final(*bits, 2);
    assert(G.get_first(bits) == 6 && G.get_second(bits) == 3 && !G.get_on(bits) && G.get_final(bits) == 2);
    assert(G.read(*bits) == 542);
    assert((bits.__bitfield[4] & 0xf8) == 0xa8);
    scalar: G.Scalar;

    G.initialize_scalar(*scalar, 2, 19);
    assert(G.get_a(scalar) == 2 && G.get_b(scalar) == 19);
    G.set_a(*scalar, 7);
    assert(G.get_b(scalar) == 19);
    G.set_b(*scalar, 1);
    assert(G.read_scalar(*scalar) == 15);
    print("ok\n");
}
