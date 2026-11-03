#import "Basic";

clamp_count :: (count: int) -> int {
    if count > 10 {
        count := 10;
        return count;
    }

    return count;
}

main :: () {
    print("%\n", clamp_count(12));
}
