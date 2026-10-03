// Two separable integer passes adapted from the licensed public C pl_mpeg
// implementation. Coefficients use the decoder's premultiplied scale.
video_idct :: (block: *s32) {
    B.assert(block != null);

    for pass: 0..1 {
        for line: 0..7 {
            stride: s64 = 1;
            base := line * 8;

            if pass == 0 {
                stride = 8;
                base = line;
            }

            b1 := cast(s64) block[base + 4 * stride];
            b3 := cast(s64) block[base + 2 * stride] + block[base + 6 * stride];
            b4 := cast(s64) block[base + 5 * stride] - block[base + 3 * stride];
            tmp1 := cast(s64) block[base + stride] + block[base + 7 * stride];
            tmp2 := cast(s64) block[base + 3 * stride] + block[base + 5 * stride];
            b6 := cast(s64) block[base + stride] - block[base + 7 * stride];
            b7 := tmp1 + tmp2;
            m0 := cast(s64) block[base];
            x4 := ((b6 * 473 - b4 * 196 + 128) >> 8) - b7;
            x0 := x4 - (((tmp1 - tmp2) * 362 + 128) >> 8);
            x1 := m0 - b1;
            x2 := (((cast(s64) block[base + 2 * stride] - block[base + 6 * stride]) * 362 + 128) >> 8) - b3;
            x3 := m0 + b1;
            y3 := x1 + x2;
            y4 := x3 + b3;
            y5 := x1 - x2;
            y6 := x3 - b3;
            y7 := -x0 - ((b4 * 473 + b6 * 196 + 128) >> 8);
            values := s64.[b7 + y4, x4 + y3, y5 - x0, y6 - y7, y6 + y7, x0 + y5, y3 - x4, y4 - b7];

            for index: 0..7 {
                value := values[index];
                if pass == 1 value = (value + 128) >> 8;
                B.assert(value >= -2147483648 && value <= 2147483647, "MPEG IDCT coefficient overflow");
                block[base + index * stride] = cast(s32) value;
            }
        }
    }
}

#scope_file
B :: #import "Basic";
