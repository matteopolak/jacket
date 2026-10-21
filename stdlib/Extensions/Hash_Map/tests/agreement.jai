// The vector and the plain-loop group scans must agree, and both must agree with Hash_Table, over
// a long random mix of inserts, overwrites, removals and lookups. The two Hash_Map instances run the
// same algorithm, so their control bytes have to match exactly.
//
//     jaic stdlib/Extensions/Hash_Map/tests/agreement.jai -interp
#import "Basic";
Vector :: #import "Extensions/Hash_Map";
Scalar :: #import "Extensions/Hash_Map"(SCALAR_GROUPS = true);
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
    fast: Vector.Map(s64, s64);
    slow: Scalar.Map(s64, s64);
    model: Reference.Table(s64, s64);

    defer Vector.deinit(*fast);
    defer Scalar.deinit(*slow);
    defer Reference.deinit(*model);

    for 0..operations - 1 {
        key := cast(s64) (next_random() % cast(u64) key_space);
        action := next_random() % 10;

        if action < 4 {
            a, x := Vector.insert(*fast, key, it);
            b, y := Scalar.insert(*slow, key, it);
            c, z := Reference.table_find(*model, key);

            Reference.table_set(*model, key, it);
            if a != b || x != y || x != c || (x && a != z) check(false, "insert outcome");
        } else if action < 6 {
            a, added_fast := Vector.entry(*fast, key);
            b, added_slow := Scalar.entry(*slow, key);
            exists := Reference.table_contains(*model, key);

            if added_fast != added_slow || added_fast == exists check(false, "entry outcome");
            if added_fast && (a.* != 0 || b.* != 0) check(false, "entry slot is zeroed");

            a.* = it;
            b.* = it;
            Reference.table_set(*model, key, it);
        } else if action < 8 {
            x, a := Vector.delete(*fast, key);
            y, b := Scalar.delete(*slow, key);
            c, z := Reference.table_remove(*model, key);
            if a != b || a != c || x != y || x != z check(false, "delete outcome");
        } else {
            a := Vector.find(*fast, key);
            b := Scalar.find(*slow, key);
            c, z := Reference.table_find(*model, key);
            got := Vector.get(*fast, key, default = -1);

            if (a != null) != (b != null) || (a != null) != c check(false, "find outcome");
            if a && (a.* != b.* || a.* != z) check(false, "find value");
            if got != ifx c then z else -1 check(false, "get outcome");
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
        expected := Vector.find(*fast, key);
        if !expected || expected.* != value check(false, "scalar entry matches");
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
