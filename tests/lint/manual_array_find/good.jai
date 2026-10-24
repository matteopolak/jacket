#import "Basic";

Item :: struct {
    id: int;
}

lookup :: (names: [] string, items: [] Item, wanted: string, key: int) -> bool {
    found := false;

    // Already the library's way.
    ok, at := array_find(names, wanted);

    // A member comparison needs more than `array_find(items, key)`.
    for items {
        if it.id == key {
            found = true;
            break;
        }
    }

    // More work than recording the hit.
    for names {
        if it == wanted {
            print("%\n", it);
            found = true;
            break;
        }
    }

    // Keeps looking.
    count := 0;

    for names {
        if it == wanted {
            count += 1;
        }
    }

    // An else branch.
    for names {
        if it == wanted {
            found = true;
            break;
        } else {
            count += 1;
        }
    }

    return found || ok || at > count;
}

main :: () {
    print("%\n", lookup(.["a", "b"], .[], "b", 1));
}
