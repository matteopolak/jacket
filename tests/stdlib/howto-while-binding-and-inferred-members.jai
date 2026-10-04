#import "Basic";

// `.MEMBER == x`, enum-typed struct parameters, and `while name := value`.

Stuff :: enum { FIRST; SECOND; THIRD; }

source: string;

shorten :: () -> string {
    if source.count == 0 return "";
    source.count -= 1;
    return source;
}

main :: () {
    x := Stuff.SECOND;
    assert(!(.FIRST == x));
    assert(.SECOND == x);
    assert(.THIRD != x);

    Ticket :: struct(_order: Stuff = .FIRST) { order := _order; }
    a: Ticket;
    b: Ticket(.SECOND);
    assert(a.order == .FIRST);
    assert(b.order == .SECOND);

    source = "hello";
    total := 0;
    skipped := 0;
    while s := shorten() {
        if s.count == 3 {
            skipped += 1;
            continue s;
        }
        total += s.count;
    }
    assert(total == 4 + 2 + 1);
    assert(skipped == 1);
}
