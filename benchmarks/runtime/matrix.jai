// Integer and float arithmetic with indexing in nested loops. Prints a checksum.
#import "Basic";

N :: 300;

main :: () {
    a := NewArray(N * N, float64);
    b := NewArray(N * N, float64);
    c := NewArray(N * N, float64);

    for i: 0..N - 1 for j : 0..N - 1 {
        a[i * N + j] = cast(float64) ((i * 7 + j * 3) % 11);
        b[i * N + j] = cast(float64) ((i * 5 + j) % 13);
    }

    for round: 1..4 {
        for i: 0..N - 1 for j : 0..N - 1 {
            sum: float64;
            for k: 0..N - 1 sum += a[i * N + k] * b[k * N + j];
            c[i * N + j] = sum / cast(float64) round;
        }

        a, c = c, a;
    }

    total: float64;
    for a total += it;

    print("%\n", cast(s64) (total / 1000.0));
}
