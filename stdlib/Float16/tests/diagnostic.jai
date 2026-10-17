#import "Basic";
M :: #import "Math";
H :: #import "Float16";

main :: () {
    bits: union {
        f: float;
        u: u32;
    };

    bits.f = H.float16_to_float32(cast(H.Float16) 0x8000);
    print("decoded negative zero bits %\n", bits.u);
    print("encoded literal negative zero %\n", cast(u16) H.float32_to_float16(-0.0));
    print("sqrt64 equals3 %, sqrt32 equals3 %\n", M.sqrt(cast(float64) 9) == 3, M.sqrt(cast(float32) 9) == 3);

    for i: 0..65535 {
        f := H.float16_to_float32(cast(H.Float16) i);
        h := H.float32_to_float16(f);

        if (i & 0x7c00) == 0x7c00 && (i & 0x3ff) != 0 continue;

        if cast(u16) h != cast(u16) i {
            print("mismatch % % %\n", i, cast(s64) (f * 16777216), cast(u16) h);
            return;
        }
    }

    print("all half patterns ok\n");
}
