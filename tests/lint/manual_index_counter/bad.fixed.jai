#import "Basic";

show :: (names: [] string) {

    for names {
        print("% %\n", it_index, it);
    }
}

number :: (lines: [] string) {

    for lines {
        if !it continue;
        print("%: %\n", it_index, it);
    }
}

// A string's bytes are counted by `it_index` too.
letters :: (word: string) {

    for word {
        print("% %\n", it_index, it);
    }
}

// A `case` body and the arm of a `#if` are statement lists, not blocks.
by_case :: (names: [] string, mode: int) {
    if mode == {
        case 1;

            for names {
                print("% %\n", it_index, it);
            }
    }
}

by_static_if :: (names: [] string) {
    #if true {

        for names {
            print("% %\n", it_index, it);
        }
    }
}

main :: () {
    by_case(.["a"], 1);
    by_static_if(.["a"]);
    letters("abc");
    show(.["a", "b"]);
    number(.["x", "", "y"]);
}
