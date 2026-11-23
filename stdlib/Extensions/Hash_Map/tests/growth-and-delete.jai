// Ten thousand integer keys through every growth step: lookups, overwrites, deletes of half the keys,
// reinsertion, iteration and clear, each checked against values computed from the keys.
#import "Basic";
Map :: #import "Extensions/Hash_Map";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

main :: () {
    m: Map.Map(s64, s64);

    defer Map.deinit(*m);

    N :: 10000;

    expect(m.count == 0 && m.capacity == 0, "zero value is empty and unallocated");
    expect(Map.get(*m, 1, -1) == -1 && !Map.contains(*m, 1), "lookups on the empty map");

    for i: 0..N - 1 {
        _, replaced := Map.insert(*m, i, i * i);
        expect(!replaced, tprint("insert % reported a replacement", i));
    }

    expect(m.count == N, tprint("count after inserts is %", m.count));
    expect(m.capacity >= N && (m.capacity & (m.capacity - 1)) == 0, tprint("capacity % is a power of two above count", m.capacity));

    for i: 0..N - 1 expect(Map.get(*m, i, -1) == i * i, tprint("get % after growth", i));

    expect(Map.find(*m, N) == null, "find of a missing key is null");

    // Overwrite returns the previous value.
    previous, replaced := Map.insert(*m, 7, 1);

    expect(replaced && previous == 49, "overwrite returns the previous value");
    expect(m.count == N, "overwrite does not change count");

    // Pointers from find write through.
    slot := Map.find(*m, 8);

    expect(slot != null, "find an existing key");
    slot.* += 1000;
    expect(Map.get(*m, 8) == 1064, "write through find pointer");

    // Delete every odd key.
    removed := 0;

    for i: 0..N - 1 {
        if i & 1 == 0 continue;

        value, ok := Map.delete(*m, i);

        expect(ok, tprint("delete % found the key", i));
        // Key 7 was overwritten with 1 above; every other odd key still holds its square.
        expect(value == ifx i == 7 then 1 else i * i, tprint("delete % returned its value", i));
        if ok removed += 1;
    }

    expect(removed == N / 2, tprint("removed % odd keys", removed));
    expect(m.count == N / 2, tprint("count after deletes is %", m.count));

    for i: 0..N - 1 expect(Map.contains(*m, i) == ((i & 1) == 0), tprint("contains % after deletes", i));

    _, deleted_again := Map.delete(*m, 1);

    expect(!deleted_again, "deleting a missing key reports nothing removed");

    // Sum of squares of the even keys, closed form: 4 * sum of j^2 for j in 0..4999. Key 8 was
    // raised by 1000 through a find pointer.
    expected_sum: s64 = 4 * (4999 * 5000 * 9999 / 6) + 1000;
    sum: s64 = 0;
    seen := 0;

    for value, key: m {
        sum += value;
        seen += 1;
        expect(key & 1 == 0, "iteration yields only even keys");
    }

    expect(seen == N / 2, tprint("iteration visited % entries", seen));
    expect(sum == expected_sum, tprint("sum of values is %, expected %", sum, expected_sum));

    // Reinsert the odd keys: the map is back to N entries.
    for i: 0..N - 1 if i & 1 Map.insert(*m, i, i * i);

    expect(m.count == N, tprint("count after reinsertion is %", m.count));

    // Clear empties the map and it accepts inserts again.
    Map.clear(*m);
    expect(m.count == 0 && !Map.contains(*m, 2), "clear empties the map");
    Map.insert(*m, 2, 4);
    expect(m.count == 1 && Map.get(*m, 2) == 4, "insert after clear");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
