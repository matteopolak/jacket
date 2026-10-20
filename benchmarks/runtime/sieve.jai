// Byte stores and branches over a large array. Prints the number of primes.
#import "Basic";

main :: () {
    limit :: 30_000_000;
    composite := NewArray(limit + 1, bool);

    count := 0;

    for 2..limit {
        if composite[it] continue;

        count += 1;
        multiple := it * it;
        while multiple <= limit {
            composite[multiple] = true;
            multiple += it;
        }
    }

    print("%\n", count);
}
