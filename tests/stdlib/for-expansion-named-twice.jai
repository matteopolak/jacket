// A for_expansion macro that inserts the loop body twice, used with a named iterator
// (chess-jai's `for :gen_moves mov: chess`, which walks tactical moves, then quiet ones).
#import "Basic";

twice :: (n: int, body: Code, flags: For_Flags) #expand {
    `it := 0;
    `it_index := 0;
    i := 0;
    while i < n {
        it = i;
        it_index = i;
        #insert body;
        i += 1;
    }
    i = 0;
    while i < n {
        it = 10 + i;
        it_index = i;
        #insert body;
        i += 1;
    }
}

main :: () {
    total, indices := 0, 0;
    for: twice value, position: 3 {
        total += value;
        indices += position;
    }
    assert(total == 36);
    assert(indices == 6);
    print("ok\n");
}
