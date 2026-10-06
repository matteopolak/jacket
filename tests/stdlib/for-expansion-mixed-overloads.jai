// rules: macro.12
#import "Basic";

Bag :: struct {
    items: [3] int = .[1, 2, 3];
}
Other :: struct {
    x: int;
}

for_expansion :: (o: Other, body: Code, flags: For_Flags) #expand {}
for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for i: 0..bag.items.count - 1 {
        `it := bag.items[i];
        `it_index := i;
        #insert body;
    }
}

main :: () {
    b: Bag;
    sum := 0;
    for b sum += it;
    assert(sum == 6);
    print("ok\n");
}
