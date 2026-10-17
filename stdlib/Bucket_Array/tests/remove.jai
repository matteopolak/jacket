#import "Basic";
B :: #import "Bucket_Array";

Initialized :: struct {
    value: s64 = 71;
    other: s64;
}

Fixture :: struct(always_pointer := false) {
    array: B.Bucket_Array(s64, 3, always_pointer);
    locators: [12] B.Bucket_Locator;
    pointers: [12] *s64;
    expected: [10] s64;
}

prepare :: ($always_pointer: bool) -> Fixture(always_pointer) {
    fixture: Fixture(always_pointer);

    for value: 0..11 {
        locator, pointer := B.bucket_array_add(*fixture.array, value);
        fixture.locators[value] = locator;
        fixture.pointers[value] = pointer;
    }

    B.bucket_array_remove(*fixture.array, fixture.locators[1]);
    B.bucket_array_remove(*fixture.array, fixture.locators[7]);
    count: s64;

    for bucket: fixture.array.all_buckets for slot : 0..2 if bucket.occupied[slot] {
        fixture.expected[count] = bucket.data[slot];
        count += 1;
    }

    if count != 10 || fixture.array.count != 10 exit(2);

    return fixture;
}

finish :: (fixture: *$T/Fixture, visits: s64) {
    array := *fixture.array;
    if visits != 10 || array.count != 4 exit(3);

    for value: 0..11 {
        locator := fixture.locators[value];
        bucket := array.all_buckets[locator.bucket_index];
        retained := (value % 2) != 0 && value != 1 && value != 7;

        if bucket.occupied[locator.slot_index] != retained exit(4);

        if retained {
            if B.bucket_array_find_pointer(array, locator) != fixture.pointers[value] exit(5);
            if B.bucket_array_find(array, locator) != value exit(6);
        }
    }

    for bucket: array.all_buckets {
        occupied: s64;
        listed: s64;

        for state, slot: bucket.occupied {
            if state occupied += 1;
            else if bucket.lowest_maybe_not_occupied > slot exit(7);
        }

        if occupied != bucket.count exit(8);
        for candidate: array.unfull_buckets if candidate == bucket listed += 1;
        if listed != (ifx bucket.count < 3 then 1 else 0) exit(9);
    }

    previous_buckets := array.all_buckets.count;

    for value: 100..107 {
        locator, pointer := B.bucket_array_add(array, value);
        if pointer.* != value || B.bucket_array_find_pointer(array, locator) != pointer exit(10);
    }

    if array.count != 12 || array.all_buckets.count != previous_buckets || array.unfull_buckets.count != 0 exit(11);

    for value: s64.[3, 5, 9, 11] {
        if B.bucket_array_find_pointer(array, fixture.locators[value]) != fixture.pointers[value] exit(12);
        if fixture.pointers[value].* != value exit(13);
    }

    B.bucket_array_reset(array);
    if array.count != 0 || array.all_buckets.count != 0 || array.unfull_buckets.count != 0 exit(14);
}

forward_value :: () {
    fixture := prepare(false);
    visits: s64;

    for value, index: fixture.array {
        if index != visits || value != fixture.expected[visits] exit(15);
        if (value % 2) == 0 remove value;
        visits += 1;
    }

    finish(*fixture, visits);
}

reverse_value :: () {
    fixture := prepare(false);
    visits: s64;

    for < value, index: fixture.array {
        if index != 9 - visits || value != fixture.expected[9 - visits] exit(16);
        if (value % 2) == 0 remove value;
        visits += 1;
    }

    finish(*fixture, visits);
}

forward_pointer :: () {
    fixture := prepare(false);
    visits: s64;

    for *value, index: fixture.array {
        if index != visits || value.* != fixture.expected[visits] exit(17);
        if (value.* % 2) == 0 remove value;
        visits += 1;
    }

    finish(*fixture, visits);
}

reverse_pointer :: () {
    fixture := prepare(false);
    visits: s64;

    for < *value, index: fixture.array {
        if index != 9 - visits || value.* != fixture.expected[9 - visits] exit(18);
        if (value.* % 2) == 0 remove value;
        visits += 1;
    }

    finish(*fixture, visits);
}

always_pointer :: ($reverse: bool) {
    fixture := prepare(true);
    visits: s64;

    #if reverse {
        for < value, index: fixture.array {
            if index != 9 - visits || value.* != fixture.expected[9 - visits] exit(19);
            if (value.* % 2) == 0 remove value;
            visits += 1;
        }
    } else {
        for value, index: fixture.array {
            if index != visits || value.* != fixture.expected[visits] exit(20);
            if (value.* % 2) == 0 remove value;
            visits += 1;
        }
    }

    finish(*fixture, visits);
}

main :: () {
    if type_info(B.Bucket_Array(s64, 3)).name != "Bucket_Array" exit(21);
    if type_info(B.Bucket(s64, 3)).name != "Bucket" || type_info(B.Bucket_Locator).name != "Bucket_Locator" exit(22);

    initialized: B.Bucket_Array(Initialized, 2);
    first, first_locator := B.find_and_occupy_empty_slot(*initialized);

    if first.value != 71 || first.other != 0 exit(23);

    second, second_locator := B.find_and_occupy_empty_slot(*initialized, false);
    second.* = .{ 93, 17 };
    if B.bucket_array_find(*initialized, first_locator).value != 71 exit(24);
    if B.bucket_array_find(*initialized, second_locator).value != 93 exit(25);

    B.bucket_array_reset(*initialized);
    forward_value();
    reverse_value();
    forward_pointer();
    reverse_pointer();
    always_pointer(false);
    always_pointer(true);
    write_string("ok\n");
}
