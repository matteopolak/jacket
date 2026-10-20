// The vector and the plain-loop group scans must agree, and both must agree with Hash_Table, over
// a long random mix of inserts, overwrites, removals and lookups. The two Swiss instances run the
// same algorithm, so their control bytes have to match exactly.
//
//     jaic stdlib/Extensions/Swiss_Table/tests/agreement.jai -interp
#import "Basic";
Vector :: #import "Extensions/Swiss_Table";
Scalar :: #import "Extensions/Swiss_Table"(SCALAR_GROUPS = true);
Reference :: #import "Hash_Table";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    if failures < 20 print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

state: u64 = 0x2545F4914F6CDD1D;

next_random :: () -> u64 #no_aoc {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

run :: (operations: s64, key_space: s64) {
    fast: Vector.Table(s64, s64);
    slow: Scalar.Table(s64, s64);
    model: Reference.Table(s64, s64);

    defer Vector.deinit(*fast);
    defer Scalar.deinit(*slow);
    defer Reference.deinit(*model);

    for 0..operations - 1 {
        key := cast(s64) (next_random() % cast(u64) key_space);
        action := next_random() % 10;

        if action < 4 {
            Vector.table_set(*fast, key, it);
            Scalar.table_set(*slow, key, it);
            Reference.table_set(*model, key, it);
        } else if action < 6 {
            a, _ := Vector.table_add(*fast, key, it);
            b, _ := Scalar.table_add(*slow, key, it);
            exists := Reference.table_contains(*model, key);

            if !exists Reference.table_add(*model, key, it);
            if a != b || a == exists check(false, "add outcome");
        } else if action < 8 {
            a, x := Vector.table_remove(*fast, key);
            b, y := Scalar.table_remove(*slow, key);
            c, z := Reference.table_remove(*model, key);
            if a != b || a != c || x != y || x != z check(false, "remove outcome");
        } else {
            a, x := Vector.table_find(*fast, key);
            b, y := Scalar.table_find(*slow, key);
            c, z := Reference.table_find(*model, key);
            if a != b || a != c || x != y || x != z check(false, "find outcome");
        }

        if fast.count != slow.count || fast.count != model.count {
            check(false, "counts diverged");
            return;
        }
    }

    check(fast.capacity == slow.capacity, "same capacity");
    check(fast.growth_left == slow.growth_left, "same growth_left");
    check(memcmp(fast.control, slow.control, fast.capacity) == 0, "same control bytes");

    // Each entry of the vector table, visited exactly once, is in the model with the same value.
    visited := 0;

    for value, key: fast {
        visited += 1;
        ok, expected := Reference.table_find(*model, key);
        if !ok || expected != value check(false, "entry matches the model");
    }

    check(visited == model.count, "iteration count matches the model");

    visited = 0;

    for value, key: slow {
        visited += 1;
        ok, expected := Vector.table_find(*fast, key);
        if !ok || expected != value check(false, "scalar entry matches");
    }

    check(visited == fast.count, "scalar iteration count");
}

main :: () {
    run(20000, 64);
    run(60000, 1000);
    run(60000, 1000000);

    if failures == 0 print("ok\n");
    else exit(1);
}
