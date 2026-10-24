#import "Basic";

clamp_count :: (count: int) -> int {
    if count > 10 {
        limit := 10;
        return limit;
    }

    return count;
}

grid_sum :: (rows: [][] int) -> int {
    total := 0;

    // Loop variables are scoped to their loop on purpose, named or not.
    for rows {
        for it {
            total += it;
        }
    }

    for row, r: rows {
        for row, c: row {
            total += row * (r + c);
        }
    }

    return total;
}

siblings :: (flag: bool) -> int {
    if flag {
        value := 1;
        return value;
    }

    {
        value := 2;
        return value;
    }
}

later :: () -> int {
    {
        value := 1;
        print("%\n", value);
    }

    value := 2;
    _ := 3;

    return value;
}

outer :: () -> int {
    count := 1;

    // A nested procedure cannot see the locals around it.
    inner :: () -> int {
        count := 2;
        return count;
    }

    return count + inner();
}

main :: () {
    print("%\n", clamp_count(12) + siblings(true) + later() + outer());
    print("%\n", grid_sum(.[.[1, 2], .[3]]));
}
