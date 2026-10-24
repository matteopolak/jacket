#import "Basic";

sum :: (items: [] int) -> int #no_abc {
    total := 0;
    for items total += it;
    return total;
}

bump :: (n: int) -> int {
    x := n;

    #if CPU == .X64 {
        #asm { add x, 1; }
    }

    return x;
}

main :: () {
    values := int.[1, 2, 3];

    #no_abc {
        print("%\n", values[1]);
    }

    for 0..2 #no_aoc {
        print("%\n", it);
    }

    print("% %\n", sum(values), bump(1));
}
