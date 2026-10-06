// Arrays come in three flavours: fixed-size, views, and resizable.

tour_arrays :: () {
    // Fixed size: the count is part of the type and the data lives inline.
    squares: [8] int;
    for 0..squares.count - 1 squares[it] = it * it;
    print("squares:        %\n", squares);

    // An array literal. `.[ ]` infers its element type.
    primes := int.[2, 3, 5, 7, 11, 13];
    // A view (`[] T`) is a pointer and a count. Every array converts to one.
    print("sum of primes:  %\n", sum(primes));
    print("sum of squares: %\n", sum(squares));
    // Views can point into the middle of another array.
    middle: [] int = primes;
    middle.data += 2;
    middle.count = 3;
    print("middle primes:  %\n", middle);

    // Resizable arrays grow as you add to them.
    sieve := sieve_of_eratosthenes(60);
    defer array_free(sieve);
    print("primes below 60 (%): %\n", sieve.count, sieve);

    // `it` is the element and `it_index` its position; name them if you like.
    words := string.["zero", "one", "two", "three"];
    for word, position: words {
        if position == 0 continue;
        print("% ", tprint("%=%", position, word));
    }
    print("\n");

    // `for <` walks backwards, `for *` gives pointers so you can change elements.
    for < words print("% ", it);
    print("\n");
    numbers: [..] int;
    defer array_free(numbers);
    array_add(*numbers, 4, 8, 15, 16, 23, 42);
    for *numbers it.* *= 10;
    print("times ten:      %\n", numbers);

    // `remove it` drops the current element mid-loop without skipping the next one.
    for numbers if it % 20 != 0 remove it;
    print("multiples of 20 (in some order): %\n", numbers);

    found, index := array_find(numbers, 160);
    print("160 is in there: % (at index %)\n", found, index);
}

sum :: (values: [] int) -> int {
    total := 0;
    for values total += it;
    return total;
}

sieve_of_eratosthenes :: (limit: int) -> [..] int {
    composite := NewArray(limit, bool);
    defer array_free(composite);
    result: [..] int;
    for n: 2..limit - 1 {
        if composite[n] continue;
        array_add(*result, n);
        multiple := n * n;
        while multiple < limit {
            composite[multiple] = true;
            multiple += n;
        }
    }
    return result;
}
