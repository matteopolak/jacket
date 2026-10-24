#import "Basic";

clamp_count :: (count: int) -> int {
    if count > 10 {
        count := 10;
        return count;
    }

    return count;
}

total_of :: (xs: [] int) -> int {
    total := 0;

    for xs {
        total := it * 2;
        print("%\n", total);
    }

    {
        total := 7;
        print("%\n", total);
    }

    return total;
}

pick :: (n: int) -> int {
    result := 0;

    if n == {
        case 1;
            result := 5;

            print("%\n", result);
        case;
            result = 2;
    }

    return result;
}

main :: () {
    print("%\n", clamp_count(12) + total_of(.[1, 2]) + pick(1));
}
