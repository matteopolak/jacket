#import "Basic";

twice :: (value: $T) -> $T {
    return value + value;
}

sum_through :: (last: s64) -> s64 {
    answer := 0;
    for i: 0..last answer += i;
    return answer;
}

COMPILE_TIME_SUM :: #run sum_through(4);
#assert(COMPILE_TIME_SUM == 10);
#assert(type_of(twice(cast(u8) 6)) == u8);

main :: () {
    #if COMPILE_TIME_SUM == 10 {
        selected := 4;
    } else {
        selected := 0;
    }

    assert(twice(cast(s64) 9) == 18 && selected == 4,
           "polymorphic calls and compile-time evaluation");
    print("ok\n");
}
