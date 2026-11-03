#import "Basic";

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

main :: () {
    print("%\n", has(.["ada", "grace"], "grace"));
}
