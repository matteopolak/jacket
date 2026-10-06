// Naming a for_expansion's index (even `_`) hides its `it_index` from the body,
// so an enclosing `it_index` stays visible.
#import "Basic";

Node :: struct {
    next: *Node;
    v: int;
}

for_expansion :: (list: *$T, body: Code, flags: For_Flags) #expand
#modify {
    return T == Node;
}
{
    `it := list;
    `it_index := 0;
    while it {
        next := it.next;
        defer {
            it = next;
            it_index += 1;
        }
        #insert body;
    }
}

Info :: struct {
    total: int;
}

Holder :: struct {
    head: *Node;
}

for_expansion :: (h: *Holder, body: Code, flags: For_Flags) #expand {
    `it_index: Info;
    for n, _: h.head {
        it_index.total += n.v;
    }
    `it := it_index.total;
    #insert body;
}

main :: () {
    a := Node.{ null, 1 };
    b := Node.{ *a, 2 };
    h := Holder.{ *b };
    for h if it != 3 exit(1);
    count := 0;
    for n, i: h.head count += i;
    if count != 1 exit(2);
    print("ok\n");
}
