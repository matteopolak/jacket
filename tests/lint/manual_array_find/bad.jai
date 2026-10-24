#import "Basic";

Item :: struct {
    id: int;
}

has :: (names: [] string, wanted: string) -> bool {
    found := false;

    for names {
        if it == wanted {
            found = true;
            break;
        }
    }

    return found;
}

position :: (values: [] int, wanted: int) -> int {
    index := -1;

    for v: values {
        if wanted == v {
            index = it_index;
            break;
        }
    }

    return index;
}

both :: (values: [..] u8, wanted: u8) -> bool, int {
    found := false;
    where := 0;

    for values if it == wanted {
        found = true;
        where = it_index;
        break;
    }

    return found, where;
}

main :: () {
    print("%\n", has(.["a", "b"], "b"));
}
