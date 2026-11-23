// String keys for word counting through entry, and a custom hash and equality that ignore ASCII case.
#import "Basic";
#import "String";
Map :: #import "Extensions/Hash_Map";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// FNV-1a over the lowercased bytes.
hash_ignore_case :: (text: string) -> u32 {
    h: u32 = 2166136261;

    for text {
        c := it;
        if c >= #char "A" && c <= #char "Z" c += 32;
        h = (h ^ cast(u32) c) * 16777619;
    }

    return h;
}

equal_ignore_case :: (a: string, b: string) -> bool {
    if a.count != b.count return false;

    for 0..a.count - 1 {
        x := a[it];
        y := b[it];

        if x >= #char "A" && x <= #char "Z" x += 32;
        if y >= #char "A" && y <= #char "Z" y += 32;
        if x != y return false;
    }

    return true;
}

main :: () {
    // Word counting: `entry` adds a zero value for a new key and returns a pointer to it.
    counts: Map.Map(string, int);

    defer Map.deinit(*counts);

    text := "the cat and the hat and the bat";
    words := split(text, " ");

    for word: words {
        count, added := Map.entry(*counts, word);
        count.* += 1;
        if added expect(Map.get(*counts, word, -1) == 1, "a new entry starts at zero then gets 1");
    }

    expect(counts.count == 5, tprint("distinct words: %", counts.count));
    expect(Map.get(*counts, "the") == 3, "the occurs 3 times");
    expect(Map.get(*counts, "and") == 2, "and occurs 2 times");
    expect(Map.get(*counts, "cat") == 1, "cat occurs once");
    expect(Map.get(*counts, "dog", 0) == 0, "missing word takes the default");
    expect(!Map.contains(*counts, "The"), "string keys are case sensitive by default");

    total := 0;
    for value, key: counts total += value;

    expect(total == words.count, tprint("counts sum to % words, got %", words.count, total));

    // Strings that share a prefix or are equal in length stay separate.
    _, a_replaced := Map.insert(*counts, "ab", 1);
    _, b_replaced := Map.insert(*counts, "ba", 2);

    expect(!a_replaced && !b_replaced, "new string keys are not reported as replaced");
    expect(Map.get(*counts, "ab") == 1 && Map.get(*counts, "ba") == 2, "anagram keys stay separate");

    // Custom hash and equality: "Key" and "key" are one entry.
    folded: Map.Map(string, int, hash_ignore_case, equal_ignore_case);

    defer Map.deinit(*folded);

    Map.insert(*folded, "Key", 1);
    expect(Map.get(*folded, "key") == 1, "lookup ignores ASCII case");
    previous, replaced := Map.insert(*folded, "KEY", 2);

    expect(replaced && previous == 1, "insert under another case replaces the entry");
    expect(folded.count == 1, tprint("case variants share one entry: %", folded.count));
    expect(!Map.contains(*folded, "keys"), "a longer key is absent");

    // Many keys through growth, then reserve and clear.
    big: Map.Map(string, int);

    defer Map.deinit(*big);

    Map.reserve(*big, 500);
    for i: 0..499 Map.insert(*big, tprint("item-%", i), i);

    expect(big.count == 500, tprint("500 string keys: %", big.count));
    expect(Map.get(*big, "item-0") == 0 && Map.get(*big, "item-499") == 499, "string keys after reserve");
    expect(Map.get(*big, "item-500", -1) == -1, "missing string key after growth");

    Map.clear(*big);
    expect(big.count == 0 && !Map.contains(*big, "item-0"), "clear removes string keys");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
