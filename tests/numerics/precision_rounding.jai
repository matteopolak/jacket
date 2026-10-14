#import "Basic";
M :: #import "Math";
C :: #import "Color_Interpolation";
H :: #import "Float16";

main :: () {
    assert(cast(u16) H.float32_to_float16(cast(float32) 1.00048828125) == 0x3c00);
    assert(cast(u16) H.float32_to_float16(cast(float32) 1.00146484375) == 0x3c02);
    assert(M.half_to_float(1) == cast(float32) 5.9604644775390625e-8);

    for c: .[M.xyz(0, 0, 0), M.xyz(1, 1, 1), M.xyz(1, 0, 0), M.xyz(0.2, 0.4, 0.7)] {
        back := C.oklab_to_linear_srgb(C.linear_srgb_to_oklab(c));
        assert(M.distance(back, c) < 0.000001);
    }

    print("explicit precision rounding ok\n");
}
