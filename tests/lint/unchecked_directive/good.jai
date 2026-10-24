#import "Basic";

// Every index comes from `for items`, so it is in range.
sum :: (items: [] int) -> int #no_abc {
    total := 0;
    for items total += it;
    return total;
}

bump :: (n: int) -> int {
    x := n;

    #if CPU == .X64 {
        // One add; the compiler does not fuse it with the loop below.
        #asm { add x, 1; }
    }

    return x;
}

checked :: (items: [] int) -> int {
    total := 0;
    for items total += it;
    return total;
}

main :: () {
    values := int.[1, 2, 3];

    #no_abc { // the index is a constant inside the array
        print("%\n", values[1]);
    }

    for 0..2 #no_aoc { // small counts cannot overflow
        print("%\n", it);
    }

    print("% % %\n", sum(values), bump(1), checked(values));
}
