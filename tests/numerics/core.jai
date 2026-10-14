#import "Basic";
M :: #import "Math";
H :: #import "Float16";
S :: #import "Sloppy_Math";
C :: #import "Color_Interpolation";

main :: () {
    assert(M.frac(-1.25) == 0.75);
    assert(M.sqrt(cast(float64) 9) == 3);
    v := M.xyz(0.01, 0.0, 0.0);
    n := M.normalize(*v, epsilon = 0.001);

    assert(M.abs(n - 0.01) < 0.000001 && v.x == 1);
    a := M.Matrix3_Identity;
    a._33 = 0.001;
    ok, b := M.inverse(a);

    assert(ok && M.abs(b._33 - 1000) < 0.001);
    a._33 = 0;
    ok, b = M.inverse(a);
    assert(!ok && b._11 == a._11 && b._33 == 0);
    z: M.Matrix4;
    ok4, inv4 := M.inverse(z);

    assert(!ok4 && inv4._11 == 1 && inv4._44 == 1);
    assert(type_info(M.Vector3).name == "Vector3");
    assert(type_info(H.Float16).name == "Float16");

    for i: 0..65535 {
        h := cast(H.Float16) i;
        f := H.float16_to_float32(h);
        back := H.float32_to_float16(f);

        if (i & 0x7c00) == 0x7c00 && (i & 0x3ff) != 0 {
            assert(M.isnan(f));
            assert((cast(u16) back & 0x7c00) == 0x7c00 && (cast(u16) back & 0x3ff) != 0);
            assert((cast(u16) back & 0x8000) == (cast(u16) i & 0x8000));
        } else assert(cast(u16) back == cast(u16) i);
    }

    for i: 0..0x7bfe {
        a := H.float16_to_float32(cast(H.Float16) i);
        b := H.float16_to_float32(cast(H.Float16) (i + 1));
        midpoint := (a + b) / 2;
        expected := i + (i & 1);

        assert(cast(u16) H.float32_to_float16(midpoint) == cast(u16) expected);
        assert(cast(u16) H.float32_to_float16(-midpoint) == (cast(u16) expected | 0x8000));
    }

    assert(cast(u16) H.float32_to_float16(65520.0) == 0x7c00);
    assert(cast(u16) H.float32_to_float16(1.00048828125) == 0x3c00);
    assert(cast(u16) H.float32_to_float16(1.00146484375) == 0x3c02);
    assert(S.values_are_close(M.xy(0), M.xy(0.0001)));
    r := C.hsv_to_rgb(M.xyz(120, 1, 1));

    assert(r.x == 0 && r.y == 1 && r.z == 0);
    print("numerics core ok\n");
}
