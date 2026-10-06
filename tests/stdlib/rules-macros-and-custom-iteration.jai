// rules: macro.1 macro.2 macro.3 macro.4 macro.8 macro.9
// `#expand` macros: inline expansion, backtick access to the caller, Code arguments, and
// `for` over user types through for_expansion.
#import "Basic";

// macro.1, macro.2: the macro is expanded in place, so a backtick return leaves the procedure the macro was expanded into.
return_from_caller :: (v: int) #expand {
    if v > 2 {
        `return v * 10;
    }
}
outer :: (v: int) -> int {
    return_from_caller(v);
    return 1;
}

// macro.2: backtick names refer to the caller's locals.
bump_caller :: () #expand {
    `counter += 1;
}
exchange :: (a: *int, b: *int) #expand {
    t := a.*;
    a.* = b.*;
    b.* = t;
}

// macro.3: a Code argument is inserted where the macro says, as often as it says.
log: [..] string;
run_twice :: (c: Code) #expand {
    #insert c;
    #insert c;
}

// macro.4, macro.8: for over a struct finds its for_expansion; the macro gets the value's address
// (pointer parameter), the body and the flags, and declares `it` / `it_index`.
Bag :: struct {
    items: [3] int = .[1, 2, 3];
}
for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for i: 0..bag.items.count - 1 {
        `it := bag.items[i];
        `it_index := i;
        #insert body;
    }
}

// macro.9: a named expansion, selected with `for :walk`.
walk :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for < i: 0..bag.items.count - 1 {
        `it := bag.items[i] * 10;
        `it_index := i;
        #insert body;
    }
}

// macro.8: a by-value for_expansion receives a copy of the operand.
Countdown :: struct {
    from: int;
}
for_expansion :: (c: Countdown, body: Code, flags: For_Flags) #expand {
    n := c.from;
    while n > 0 {
        `it := n;
        `it_index := c.from - n;
        #insert body;
        n -= 1;
    }
}

main :: () {
    // macro.1
    assert(outer(5) == 50, "backtick return from the caller");
    assert(outer(1) == 1, "macro without the return path");

    // macro.2
    counter := 0;
    bump_caller();
    bump_caller();
    assert(counter == 2, "caller local through a backtick");
    x, y := 1, 2;
    exchange(*x, *y);
    assert(x == 2 && y == 1, "macro swapping through pointers");

    // macro.3
    run_twice(array_add(*log, "expr"));
    run_twice({
        array_add(*log, "block");
    });
    assert(log.count == 4 && log[0] == "expr" && log[3] == "block", "inserted Code");

    // macro.8
    b: Bag;
    s := 0;
    for b {
        s += it * 10 + it_index;
    }
    assert(s == 63, "for over a struct");
    seen := 0;
    for Countdown.{ 3 } {
        seen = seen * 10 + it;
    }
    assert(seen == 321, "by-value for_expansion");

    // macro.9
    walked: [..] int;
    for: walk v, i: b {
        array_add(*walked, v);
        array_add(*walked, i);
    }
    assert(walked.count == 6, "named expansion visits every item");
    assert(walked[0] == 30 && walked[1] == 2 && walked[4] == 10 && walked[5] == 0, "named expansion order");
    print("ok\n");
}
