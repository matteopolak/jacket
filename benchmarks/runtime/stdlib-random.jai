// Random: raw values, floats in [0, 1) and ranges, from a fixed seed.
#import "Basic";
#import "Random";

main :: () {
    checksum: u64 = 1469598103934665603;

    random_seed(1234);
    sum: float64;

    for 0..50_000_000 - 1 {
        checksum = (checksum ^ random_get()) * 1099511628211;
        sum += random_get_zero_to_one();
        sum += random_get_within_range(-3, 3);
    }

    print("% %\n", checksum, cast(s64) sum);
}
