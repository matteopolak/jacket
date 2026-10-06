// Macros are procedures inlined at the call site, with access to the caller's scope.

// A `Code` argument is a block of the caller's code; `#insert` pastes it in.
repeat :: (times: int, body: Code) #expand {
    for 1..times #insert body;
}

// A backtick reaches into the caller: `total below is the caller's variable,
// and `defer schedules code for the end of the caller's scope.
tally_up :: (amount: int) #expand {
    `total += amount;
}

stopwatch :: (label: string) #expand {
    start := `steps;
    `defer print("  % took % steps\n", label, `steps - start);
}

// Macros can return from their caller: a tiny "try" for error codes.
check :: (ok: bool, message: string) #expand {
    if !ok {
        print("  check failed: %\n", message);
        `return false;
    }
}

launch_sequence :: (fuel: int) -> bool {
    check(fuel > 0, "no fuel");
    check(fuel > 50, "not enough fuel for orbit");
    print("  liftoff with % fuel!\n", fuel);
    return true;
}

// A custom `for` loop: define for_expansion for your type and `for` just works.
// This one walks a Collatz sequence: halve if even, 3n + 1 if odd, until 1.
Collatz :: struct {
    start: int;
}

for_expansion :: (sequence: *Collatz, body: Code, flags: For_Flags) #expand {
    n := sequence.start;
    index := 0;
    while true {
        `it := n;
        `it_index := index;
        #insert body;
        if n == 1 break;
        n = ifx n % 2 == 0 then n / 2 else 3 * n + 1;
        index += 1;
    }
}

// A linked list with its own iterator, so callers never touch `next`.
Chain :: struct {
    label: string;
    next: *Chain;
}

for_expansion :: (chain: *Chain, body: Code, flags: For_Flags) #expand {
    link := chain;
    index := 0;
    while link {
        `it := link;
        `it_index := index;
        #insert body;
        link = link.next;
        index += 1;
    }
}

tour_macros :: () {
    print("  ");
    repeat(3, #code {
        print("hip ");
    });
    print("hooray!\n");

    total := 0;
    tally_up(10);
    tally_up(32);
    print("  total after two macro calls: %\n", total);

    steps := 0;
    {
        stopwatch("counting to 1000");
        for 1..1000 steps += 1;
    }

    launch_sequence(0);
    launch_sequence(20);
    launch_sequence(90);

    print("  Collatz from 6:");
    six := Collatz.{ 6 };
    for value: six print(" %", value);
    print("\n");
    longest_start, longest_length := 0, 0;
    for start: 1..30 {
        length := 0;
        sequence := Collatz.{ start };
        for sequence length = it_index + 1;
        if length > longest_length longest_start, longest_length = start, length;
    }
    print("  Below 31, % takes the longest path: % numbers.\n", longest_start, longest_length);

    c := Chain.{ "charlie", null };
    b := Chain.{ "bravo", *c };
    a := Chain.{ "alpha", *b };
    print("  chain:");
    for link: a print(" %:%", it_index, link.label);
    print("\n");
}
