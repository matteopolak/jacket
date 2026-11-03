#import "Basic";

count_sums :: (a: [] int, b: [] int, c: [] int, target: int) -> int {
    hits := 0;

    for x: a {
        if x <= 0 continue;

        for y: b {
            if y <= 0 continue;

            for z: c {
                if z <= 0 continue;
                if x + y + z == target hits += 1;
            }
        }
    }

    return hits;
}

main :: () {
    print("%\n", count_sums(.[1, 2], .[3, 4], .[5, 6], 9));
}
