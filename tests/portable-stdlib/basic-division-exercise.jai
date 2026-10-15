#import "Basic";

#assert CPU == .X64;

calls: int;
samples: [9] u64 = .[3, 99, 23, 1, 5, 7, 0xffffffffffffffff, 0xffffffffffffffff, 0];

draw :: () -> u64 {
    assert(calls < samples.count);
    result := samples[calls];
    calls += 1;
    return result;
}

main :: () {
    test_unsigned_division();
    assert(calls == 0);
    test_unsigned_division(random_iterations = 3, random_get = draw);
    assert(calls == 9);
    test_unsigned_division(random_iterations = -1, random_get = draw);
    assert(calls == 9);
    print("ok\n");
}
