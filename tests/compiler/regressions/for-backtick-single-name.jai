// A backtick on one loop name in a `for_expansion` declares only that name in the caller:
// ``for value, `it_index: xs`` gives the inserted body `it_index`, and ``for `it, i: xs``
// gives it `it`. Neither leaks the unmarked name. The same holds when the loop itself runs
// over another `for_expansion`.
#import "Basic";

Letters :: struct {
    text: string;
}

for_expansion :: (letters: Letters, body: Code, flags: For_Flags) #expand {
    for byte, `it_index: cast([] u8) letters.text {
        `it := byte;
        #insert body;
    }
}

Doubled :: struct {
    values: [] int;
}

for_expansion :: (doubled: Doubled, body: Code, flags: For_Flags) #expand {
    for `it, i: doubled.values {
        `it_index := i * 2;
        #insert body;
    }
}

Words :: struct {
    letters: Letters;
}

// The inner loop is a custom expansion: its `it_index` comes from `Letters`' expansion.
for_expansion :: (words: Words, body: Code, flags: For_Flags) #expand {
    `it: u8;

    for letter, `it_index: words.letters {
        it = letter - #char "a";
        #insert body;
    }
}

main :: () {
    byte := 99;
    indices: [..] int;

    for Letters.{ "abc" } {
        assert(byte == 99);
        array_add(*indices, it_index);
        assert(it == "abc"[it_index]);
    }

    assert(indices.count == 3 && indices[0] == 0 && indices[2] == 2);

    i := -1;
    sum := 0;

    for Doubled.{ .[10, 20, 30] } {
        assert(i == -1);
        sum += it + it_index;
    }

    assert(sum == 60 + 0 + 2 + 4);

    letter := 99;
    total := 0;

    for Words.{ .{ "abc" } } {
        assert(letter == 99);
        total += it * 10 + it_index;
    }

    assert(total == 0 + 11 + 22);
    print("ok\n");
}
