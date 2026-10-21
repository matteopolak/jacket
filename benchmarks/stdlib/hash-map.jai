// Hash_Map against Hash_Table: insert, lookup hits and misses, delete-heavy churn, string keys
// and full iteration, at three sizes. Each workload runs RUNS times on fresh tables; the table
// printed is the median in nanoseconds per operation.
//
//     jaic benchmarks/stdlib/hash-map.jai -O2 -exe hash-map-bench && ./hash-map-bench
#import "Basic";
#import "String";
Chained :: #import "Hash_Table";
Fast :: #import "Extensions/Hash_Map";

RUNS :: 7;
OPERATIONS_PER_RUN :: 3_000_000;

// The two maps behind one spelling.
set :: (table: *Chained.Table($K, $V), key: K, value: V) {
    Chained.table_set(table, key, value);
}

set :: (table: *Fast.Map($K, $V), key: K, value: V) {
    Fast.insert(table, key, value);
}

find :: (table: *Chained.Table($K, $V), key: K) -> *V {
    return Chained.table_find_pointer(table, key);
}

find :: (table: *Fast.Map($K, $V), key: K) -> *V {
    return Fast.find(table, key);
}

remove_key :: (table: *Chained.Table($K, $V), key: K) {
    Chained.table_remove(table, key);
}

remove_key :: (table: *Fast.Map($K, $V), key: K) {
    Fast.delete(table, key);
}

release :: (table: *Chained.Table($K, $V)) {
    Chained.deinit(table);
}

release :: (table: *Fast.Map($K, $V)) {
    Fast.deinit(table);
}

state: u64 = 0x9E3779B97F4A7C15;

next_random :: () -> u64 #no_aoc {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

make_keys :: (count: s64) -> [] s64 {
    keys := NewArray(count, s64);
    for 0..count - 1 keys[it] = cast(s64) (next_random() >> 2);
    return keys;
}

shuffled :: (keys: [] s64) -> [] s64 {
    result := NewArray(keys.count, s64);

    memcpy(result.data, keys.data, keys.count * size_of(s64));

    for < 1..result.count - 1 {
        other := cast(s64) (next_random() % cast(u64) (it + 1));
        swap := result[it];
        result[it] = result[other];
        result[other] = swap;
    }

    return result;
}

median :: (samples: [] float64) -> float64 {
    sorted := NewArray(samples.count, float64);

    defer array_free(sorted);
    memcpy(sorted.data, samples.data, samples.count * size_of(float64));

    for 1..sorted.count - 1 {
        value := sorted[it];
        j := it;

        while j > 0 && sorted[j - 1] > value {
            sorted[j] = sorted[j - 1];
            j -= 1;
        }

        sorted[j] = value;
    }

    return sorted[sorted.count / 2];
}

checksum: s64;

// ---- workloads: each returns nanoseconds per operation for one run ----

insert_int :: ($T: Type, keys: [] s64) -> float64 {
    repeats := max(1, OPERATIONS_PER_RUN / keys.count);
    start := seconds_since_init();

    for 1..repeats {
        table: T;
        for key: keys set(*table, key, key);
        checksum += table.count;
        release(*table);
    }

    return (seconds_since_init() - start) * 1e9 / cast(float64) (repeats * keys.count);
}

lookup_hit :: ($T: Type, keys: [] s64, order: [] s64) -> float64 {
    table: T;

    defer release(*table);
    for key: keys set(*table, key, key);

    repeats := max(1, OPERATIONS_PER_RUN / keys.count);
    start := seconds_since_init();
    sum: s64;

    for 1..repeats for key: order {
        pointer := find(*table, key);
        sum += pointer.*;
    }

    elapsed := seconds_since_init() - start;

    checksum += sum;

    return elapsed * 1e9 / cast(float64) (repeats * keys.count);
}

lookup_miss :: ($T: Type, keys: [] s64, misses: [] s64) -> float64 {
    table: T;

    defer release(*table);
    for key: keys set(*table, key, key);

    repeats := max(1, OPERATIONS_PER_RUN / keys.count);
    start := seconds_since_init();
    found := 0;

    for 1..repeats for key: misses if find(*table, key) found += 1;

    elapsed := seconds_since_init() - start;

    checksum += found;

    return elapsed * 1e9 / cast(float64) (repeats * keys.count);
}

// A sliding window: the oldest key leaves as a new one arrives, so the table stays at
// keys.count entries while deleted slots pile up. One operation is a remove plus an insert.
churn :: ($T: Type, keys: [] s64, extra: [] s64) -> float64 {
    table: T;

    defer release(*table);
    for key: keys set(*table, key, key);

    start := seconds_since_init();
    steps := min(extra.count, max(OPERATIONS_PER_RUN, keys.count));

    for step: 0..steps - 1 {
        oldest := ifx step < keys.count then keys[step] else extra[step - keys.count];
        remove_key(*table, oldest);
        set(*table, extra[step], step);
    }

    elapsed := seconds_since_init() - start;

    checksum += table.count;

    return elapsed * 1e9 / cast(float64) steps;
}

iterate :: ($T: Type, keys: [] s64) -> float64 {
    table: T;

    defer release(*table);
    for key: keys set(*table, key, key);

    repeats := max(1, OPERATIONS_PER_RUN / keys.count);
    start := seconds_since_init();
    sum: s64;

    for 1..repeats for value: table sum += value;

    elapsed := seconds_since_init() - start;

    checksum += sum;

    return elapsed * 1e9 / cast(float64) (repeats * table.count);
}

insert_string :: ($T: Type, keys: [] string) -> float64 {
    repeats := max(1, OPERATIONS_PER_RUN / 4 / keys.count);
    start := seconds_since_init();

    for 1..repeats {
        table: T;
        for key: keys set(*table, key, key.count);
        checksum += table.count;
        release(*table);
    }

    return (seconds_since_init() - start) * 1e9 / cast(float64) (repeats * keys.count);
}

lookup_string :: ($T: Type, keys: [] string, order: [] string) -> float64 {
    table: T;

    defer release(*table);
    for key: keys set(*table, key, key.count);

    repeats := max(1, OPERATIONS_PER_RUN / 4 / keys.count);
    start := seconds_since_init();
    sum: s64;

    for 1..repeats for key: order sum += find(*table, key).*;

    elapsed := seconds_since_init() - start;

    checksum += sum;

    return elapsed * 1e9 / cast(float64) (repeats * keys.count);
}

report :: (name: string, size: s64, chained: [] float64, fast: [] float64) {
    a := median(chained);
    b := median(fast);
    row(name, tprint("%", size), number(a), number(b), number(a / b));
}

number :: (value: float64) -> string {
    return tprint("%", FormatFloat.{ value = value, trailing_width = 2, zero_removal = .NO });
}

pad :: (text: string, width: s64) -> string {
    if text.count >= width return text;

    result := alloc_string(width);

    memset(result.data, #char " ", width);
    memcpy(result.data, text.data, text.count);

    return result;
}

row :: (name: string, size: string, a: string, b: string, c: string) {
    print("%%%%%\n", pad(name, 20), pad(size, 10), pad(a, 20), pad(b, 20), c);
}

Int_Case :: enum {
    INSERT;
    HIT;
    MISS;
    CHURN;
    ITERATE;
}

main :: () {
    row("workload", "entries", "Hash_Table ns/op", "Hash_Map ns/op", "speedup");

    ChainedInt :: Chained.Table(s64, s64);
    FastInt :: Fast.Map(s64, s64);
    ChainedString :: Chained.Table(string, s64);
    FastString :: Fast.Map(string, s64);

    sizes := s64.[1_000, 100_000, 1_000_000];

    for size: sizes {
        keys := make_keys(size);
        order := shuffled(keys);
        misses := make_keys(size);
        extra := make_keys(max(OPERATIONS_PER_RUN, size));

        for kind: Int_Case.INSERT..Int_Case.ITERATE {
            a: [RUNS] float64;
            b: [RUNS] float64;

            for run: 0..RUNS - 1 {
                // Alternate who goes first, so neither always gets the warmer caches.
                if kind == {
                    case .INSERT;
                        if run % 2 == 0 {
                            a[run] = insert_int(ChainedInt, keys);
                            b[run] = insert_int(FastInt, keys);
                        } else {
                            b[run] = insert_int(FastInt, keys);
                            a[run] = insert_int(ChainedInt, keys);
                        }
                    case .HIT;
                        if run % 2 == 0 {
                            a[run] = lookup_hit(ChainedInt, keys, order);
                            b[run] = lookup_hit(FastInt, keys, order);
                        } else {
                            b[run] = lookup_hit(FastInt, keys, order);
                            a[run] = lookup_hit(ChainedInt, keys, order);
                        }
                    case .MISS;
                        if run % 2 == 0 {
                            a[run] = lookup_miss(ChainedInt, keys, misses);
                            b[run] = lookup_miss(FastInt, keys, misses);
                        } else {
                            b[run] = lookup_miss(FastInt, keys, misses);
                            a[run] = lookup_miss(ChainedInt, keys, misses);
                        }
                    case .CHURN;
                        if run % 2 == 0 {
                            a[run] = churn(ChainedInt, keys, extra);
                            b[run] = churn(FastInt, keys, extra);
                        } else {
                            b[run] = churn(FastInt, keys, extra);
                            a[run] = churn(ChainedInt, keys, extra);
                        }
                    case .ITERATE;
                        if run % 2 == 0 {
                            a[run] = iterate(ChainedInt, keys);
                            b[run] = iterate(FastInt, keys);
                        } else {
                            b[run] = iterate(FastInt, keys);
                            a[run] = iterate(ChainedInt, keys);
                        }
                }
            }

            names := string.["int insert", "int lookup hit", "int lookup miss", "int remove+insert", "int iterate"];

            report(names[cast(s64) kind], size, a, b);
        }

        array_free(keys);
        array_free(order);
        array_free(misses);
        array_free(extra);

        // Strings: the keys live in one buffer, as they would after parsing a file.
        builder: String_Builder;
        for 0..size - 1 print_to_builder(*builder, "key-%-%\n", it, next_random() % 100000);

        text := builder_to_string(*builder);
        lines := split(text, "\n");
        strings := array_view(lines, 0, size);
        shuffled_strings := NewArray(size, string);

        memcpy(shuffled_strings.data, strings.data, size * size_of(string));

        for < 1..size - 1 {
            other := cast(s64) (next_random() % cast(u64) (it + 1));
            swap := shuffled_strings[it];
            shuffled_strings[it] = shuffled_strings[other];
            shuffled_strings[other] = swap;
        }

        a: [RUNS] float64;
        b: [RUNS] float64;

        for run: 0..RUNS - 1 {
            if run % 2 == 0 {
                a[run] = insert_string(ChainedString, strings);
                b[run] = insert_string(FastString, strings);
            } else {
                b[run] = insert_string(FastString, strings);
                a[run] = insert_string(ChainedString, strings);
            }
        }

        report("string insert", size, a, b);

        for run: 0..RUNS - 1 {
            if run % 2 == 0 {
                a[run] = lookup_string(ChainedString, strings, shuffled_strings);
                b[run] = lookup_string(FastString, strings, shuffled_strings);
            } else {
                b[run] = lookup_string(FastString, strings, shuffled_strings);
                a[run] = lookup_string(ChainedString, strings, shuffled_strings);
            }
        }

        report("string lookup hit", size, a, b);
    }

    print("(checksum %)\n", checksum);
}
