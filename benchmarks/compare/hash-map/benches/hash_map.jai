// Hash_Map against Hash_Table (the standard library's table: open addressing, linear probing,
// 70% load) at 1k, 100k and 1M entries. One iteration is one pass of the workload over every
// entry; the report divides by the entries (`set_items`) to give a time per operation.
//
//   insert          a fresh table, every key inserted (growth included)
//   hit / miss      lookups of keys in / not in a table built before the timer starts
//   churn           a sliding window: the oldest key leaves as a new one arrives (remove + insert)
//   iterate         the sum of every value
//   string_insert   as insert, with string keys that live in one buffer
//   string_hit      lookups of string keys in a shuffled order
//
// 666rayen999--hash_map is not measured: it does not give correct answers (see docs/extensions/hash-map.md).
#import "Basic";
#import "String";
#import "Extensions/Testing";
Linear :: #import "Hash_Table";
Fast :: #import "Extensions/Hash_Map";

SIZES :: s64.[1_000, 100_000, 1_000_000];

Data :: struct {
    keys: [] s64;
    order: [] s64;
    misses: [] s64;
    ring: [] s64;
    strings: [] string;
    shuffled: [] string;
    ready: bool;
}

datasets: [3] Data;
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

shuffle :: (items: [] $T) {
    for < 1..items.count - 1 {
        other := cast(s64) (next_random() % cast(u64) (it + 1));
        swap := items[it];
        items[it] = items[other];
        items[other] = swap;
    }
}

dataset :: (index: int) -> *Data {
    data := *datasets[index];

    if data.ready return data;

    size := SIZES[index];
    data.keys = make_keys(size);
    data.order = NewArray(size, s64);
    memcpy(data.order.data, data.keys.data, size * size_of(s64));
    shuffle(data.order);
    data.misses = make_keys(size);
    data.ring = make_keys(2 * size);

    // The first half of the ring is the table's first window.
    memcpy(data.ring.data, data.keys.data, size * size_of(s64));

    builder: String_Builder;
    for 0..size - 1 print_to_builder(*builder, "key-%-%\n", it, next_random() % 100000);

    text := builder_to_string(*builder);
    lines := split(text, "\n");
    data.strings = array_view(lines, 0, size);
    data.shuffled = NewArray(size, string);
    memcpy(data.shuffled.data, data.strings.data, size * size_of(string));
    shuffle(data.shuffled);
    data.ready = true;

    return data;
}

// The three maps behind one spelling.
create :: ($T: Type) -> *T {
    return New(T);
}

set :: (table: *Linear.Table($K, $V), key: K, value: V) {
    Linear.table_set(table, key, value);
}

set :: (table: *Fast.Map($K, $V), key: K, value: V) {
    Fast.insert(table, key, value);
}

find :: (table: *Linear.Table($K, $V), key: K) -> *V {
    return Linear.table_find_pointer(table, key);
}

find :: (table: *Fast.Map($K, $V), key: K) -> *V {
    return Fast.find(table, key);
}

remove_key :: (table: *Linear.Table($K, $V), key: K) {
    Linear.table_remove(table, key);
}

remove_key :: (table: *Fast.Map($K, $V), key: K) {
    Fast.delete(table, key);
}

destroy :: (table: *Linear.Table($K, $V)) {
    Linear.deinit(table);
    free(table);
}

destroy :: (table: *Fast.Map($K, $V)) {
    Fast.deinit(table);
    free(table);
}

insert_int :: ($T: Type, b: *Bench, data: *Data) {
    set_items(b, data.keys.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        table := create(T);
        for key: data.keys set(table, key, key);
        black_box(table.count);
        destroy(table);
    }
}

hit_int :: ($T: Type, b: *Bench, data: *Data) {
    table := create(T);

    for key: data.keys set(table, key, key);
    set_items(b, data.keys.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        sum: s64;
        for key: data.order sum += find(table, key).*;
        black_box(sum);
    }

    destroy(table);
}

miss_int :: ($T: Type, b: *Bench, data: *Data) {
    table := create(T);

    for key: data.keys set(table, key, key);
    set_items(b, data.keys.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        found := 0;
        for key: data.misses if find(table, key) found += 1;
        black_box(found);
    }

    destroy(table);
}

churn_int :: ($T: Type, b: *Bench, data: *Data) {
    table := create(T);
    size := data.keys.count;
    ring := data.ring.count;

    for key: data.keys set(table, key, key);
    set_items(b, size);
    reset_timer(b);

    position := 0;

    for 0..b.iterations - 1 {
        for 0..size - 1 {
            remove_key(table, data.ring[position % ring]);
            set(table, data.ring[(position + size) % ring], position);
            position += 1;
        }

        black_box(table.count);
    }

    destroy(table);
}

iterate_int :: ($T: Type, b: *Bench, data: *Data) {
    table := create(T);

    for key: data.keys set(table, key, key);
    set_items(b, data.keys.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        sum: s64;
        for value: table sum += value;
        black_box(sum);
    }

    destroy(table);
}

insert_string :: ($T: Type, b: *Bench, data: *Data) {
    set_items(b, data.strings.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        table := create(T);
        for key: data.strings set(table, key, key.count);
        black_box(table.count);
        destroy(table);
    }
}

hit_string :: ($T: Type, b: *Bench, data: *Data) {
    table := create(T);

    for key: data.strings set(table, key, key.count);
    set_items(b, data.strings.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        sum: s64;
        for key: data.shuffled sum += find(table, key).*;
        black_box(sum);
    }

    destroy(table);
}

insert_1k__hash_table :: (b: *Bench) {
    insert_int(Linear.Table(s64, s64), b, dataset(0));
} @bench

insert_1k__hash_map :: (b: *Bench) {
    insert_int(Fast.Map(s64, s64), b, dataset(0));
} @bench

insert_100k__hash_table :: (b: *Bench) {
    insert_int(Linear.Table(s64, s64), b, dataset(1));
} @bench

insert_100k__hash_map :: (b: *Bench) {
    insert_int(Fast.Map(s64, s64), b, dataset(1));
} @bench

insert_1m__hash_table :: (b: *Bench) {
    insert_int(Linear.Table(s64, s64), b, dataset(2));
} @bench

insert_1m__hash_map :: (b: *Bench) {
    insert_int(Fast.Map(s64, s64), b, dataset(2));
} @bench

hit_1k__hash_table :: (b: *Bench) {
    hit_int(Linear.Table(s64, s64), b, dataset(0));
} @bench

hit_1k__hash_map :: (b: *Bench) {
    hit_int(Fast.Map(s64, s64), b, dataset(0));
} @bench

hit_100k__hash_table :: (b: *Bench) {
    hit_int(Linear.Table(s64, s64), b, dataset(1));
} @bench

hit_100k__hash_map :: (b: *Bench) {
    hit_int(Fast.Map(s64, s64), b, dataset(1));
} @bench

hit_1m__hash_table :: (b: *Bench) {
    hit_int(Linear.Table(s64, s64), b, dataset(2));
} @bench

hit_1m__hash_map :: (b: *Bench) {
    hit_int(Fast.Map(s64, s64), b, dataset(2));
} @bench

miss_1k__hash_table :: (b: *Bench) {
    miss_int(Linear.Table(s64, s64), b, dataset(0));
} @bench

miss_1k__hash_map :: (b: *Bench) {
    miss_int(Fast.Map(s64, s64), b, dataset(0));
} @bench

miss_100k__hash_table :: (b: *Bench) {
    miss_int(Linear.Table(s64, s64), b, dataset(1));
} @bench

miss_100k__hash_map :: (b: *Bench) {
    miss_int(Fast.Map(s64, s64), b, dataset(1));
} @bench

miss_1m__hash_table :: (b: *Bench) {
    miss_int(Linear.Table(s64, s64), b, dataset(2));
} @bench

miss_1m__hash_map :: (b: *Bench) {
    miss_int(Fast.Map(s64, s64), b, dataset(2));
} @bench

churn_1k__hash_table :: (b: *Bench) {
    churn_int(Linear.Table(s64, s64), b, dataset(0));
} @bench

churn_1k__hash_map :: (b: *Bench) {
    churn_int(Fast.Map(s64, s64), b, dataset(0));
} @bench

churn_100k__hash_table :: (b: *Bench) {
    churn_int(Linear.Table(s64, s64), b, dataset(1));
} @bench

churn_100k__hash_map :: (b: *Bench) {
    churn_int(Fast.Map(s64, s64), b, dataset(1));
} @bench

churn_1m__hash_table :: (b: *Bench) {
    churn_int(Linear.Table(s64, s64), b, dataset(2));
} @bench

churn_1m__hash_map :: (b: *Bench) {
    churn_int(Fast.Map(s64, s64), b, dataset(2));
} @bench

iterate_1k__hash_table :: (b: *Bench) {
    iterate_int(Linear.Table(s64, s64), b, dataset(0));
} @bench

iterate_1k__hash_map :: (b: *Bench) {
    iterate_int(Fast.Map(s64, s64), b, dataset(0));
} @bench

iterate_100k__hash_table :: (b: *Bench) {
    iterate_int(Linear.Table(s64, s64), b, dataset(1));
} @bench

iterate_100k__hash_map :: (b: *Bench) {
    iterate_int(Fast.Map(s64, s64), b, dataset(1));
} @bench

iterate_1m__hash_table :: (b: *Bench) {
    iterate_int(Linear.Table(s64, s64), b, dataset(2));
} @bench

iterate_1m__hash_map :: (b: *Bench) {
    iterate_int(Fast.Map(s64, s64), b, dataset(2));
} @bench

string_insert_1k__hash_table :: (b: *Bench) {
    insert_string(Linear.Table(string, s64), b, dataset(0));
} @bench

string_insert_1k__hash_map :: (b: *Bench) {
    insert_string(Fast.Map(string, s64), b, dataset(0));
} @bench

string_insert_100k__hash_table :: (b: *Bench) {
    insert_string(Linear.Table(string, s64), b, dataset(1));
} @bench

string_insert_100k__hash_map :: (b: *Bench) {
    insert_string(Fast.Map(string, s64), b, dataset(1));
} @bench

string_insert_1m__hash_table :: (b: *Bench) {
    insert_string(Linear.Table(string, s64), b, dataset(2));
} @bench

string_insert_1m__hash_map :: (b: *Bench) {
    insert_string(Fast.Map(string, s64), b, dataset(2));
} @bench

string_hit_1k__hash_table :: (b: *Bench) {
    hit_string(Linear.Table(string, s64), b, dataset(0));
} @bench

string_hit_1k__hash_map :: (b: *Bench) {
    hit_string(Fast.Map(string, s64), b, dataset(0));
} @bench

string_hit_100k__hash_table :: (b: *Bench) {
    hit_string(Linear.Table(string, s64), b, dataset(1));
} @bench

string_hit_100k__hash_map :: (b: *Bench) {
    hit_string(Fast.Map(string, s64), b, dataset(1));
} @bench

string_hit_1m__hash_table :: (b: *Bench) {
    hit_string(Linear.Table(string, s64), b, dataset(2));
} @bench

string_hit_1m__hash_map :: (b: *Bench) {
    hit_string(Fast.Map(string, s64), b, dataset(2));
} @bench
