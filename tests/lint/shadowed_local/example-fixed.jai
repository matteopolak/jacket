#import "Basic";

clamp_count :: (count: int) -> int {
    if count > 10 {
        capped := 10;
        return capped;
    }

    return count;
}

main :: () {
    print("%\n", clamp_count(12));
}
