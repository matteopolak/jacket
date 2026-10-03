// Direct MPEG synthesis matrix. Kept separate from entropy/sample decoding so a
// future verified factorization can replace it without changing the public API.
audio_matrix_transform :: (s: [32][3] s32, ss: s32, d: *float, dp: s32) {
    B.assert(ss >= 0 && ss < 3 && d != null && dp >= 0 && dp <= 2147483583);

    for row: 0..63 {
        sum: float64;
        for col: 0..31 sum += cast(float64) s[col][ss] * SYNTHESIS_MATRIX[row][col];
        d[dp + row] = cast(float) sum;
    }
}

#scope_file
B :: #import "Basic";
Math :: #import "Math";

SYNTHESIS_MATRIX :: #run make_matrix();

make_matrix :: () -> [64][32] float64 {
    result: [64][32] float64;

    for row: 0..63 for col : 0..31 {
        result[row][col] = Math.cos(cast(float64) ((16 + row) * (2 * col + 1)) * Math.PI64 / 64.0);
    }

    return result;
}
