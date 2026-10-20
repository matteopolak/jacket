// A by-value `for` copies scalar elements into `it` (so `it` keeps the removed value after
// `remove`) and names aggregate elements in place (so `*it` is the element and `it` shows the
// element that moved in after `remove`).
// rules: flow.29
#import "Basic";

Post :: struct {
    id: int;
    tag: int;
}

Wrapper :: struct {
    value: int;
}

live_blocks: int;

make_post :: (id: int) -> *Post {
    p := New(Post);
    p.id = id;
    live_blocks += 1;
    return p;
}

drop_post :: (p: *Post) {
    live_blocks -= 1;
    free(p);
}

collect :: (values: [] int) -> string {
    b: String_Builder;

    for values {
        if it_index > 0 append(*b, ",");
        print_to_builder(*b, "%", it);
    }

    return builder_to_string(*b);
}

main :: () {
    // Pointer elements are scalars: freeing `it` after `remove it` frees the removed one.
    blog: [..] *Post;
    for 1..3 array_add(*blog, make_post(it));

    for blog {
        first := it;
        remove it;
        assert(it == first);
        drop_post(it);
    }

    assert(blog.count == 0 && live_blocks == 0);
    array_free(blog);

    // Changing `it` never changes the array.
    numbers := int.[1, 2, 3];
    for numbers {
        it += 1;
        assert(it >= 2);
    }

    assert(collect(numbers) == "1,2,3");

    view: [] int = numbers;
    for view it = 9;

    assert(collect(numbers) == "1,2,3");

    // `*it` is the address of the copy.
    escaped: *int;

    for numbers {
        escaped = *it;
        assert(escaped != *numbers[it_index]);
        break;
    }

    escaped.* = 7;
    assert(numbers[0] == 1);

    // Removing keeps the removed value in `it`; the last element moves into the slot.
    items: [..] int;
    for 1..3 array_add(*items, it);

    removed := 0;

    for items {
        if it == 1 {
            removed = it;
            remove it;
            assert(it == 1);
        }
    }

    assert(removed == 1);
    assert(collect(items) == "3,2");
    array_free(items);

    // A named iterator behaves the same way.
    nums: [..] int;
    for 1..5 array_add(*nums, it * 10);

    seen: [..] int;

    for v: nums {
        if v == 10 || v == 30 {
            remove v;
            assert(v == 10 || v == 30);
            array_add(*seen, v);
        }
    }

    assert(seen.count == 2 && seen[0] == 10 && seen[1] == 30);
    assert(nums.count == 3);
    array_free(nums);
    array_free(seen);

    // continue and break after a remove.
    xs: [..] int;
    for 1..6 array_add(*xs, it);

    total := 0;

    for xs {
        if it % 2 == 0 {
            remove it;
            total += it;
            continue;
        }

        if it == 5 {
            remove it;
            total += it;
            break;
        }
    }

    assert(total == 2 + 6 + 5);
    assert(xs.count == 3);
    array_free(xs);

    // A reverse loop.
    ys: [..] int;
    for 1..6 array_add(*ys, it);

    removed_sum := 0;

    for < ys {
        if it > 3 {
            remove it;
            removed_sum += it;
        }
    }

    assert(removed_sum == 4 + 5 + 6);
    assert(ys.count == 3);
    array_free(ys);

    // A remove from an inner loop, naming the outer loop.
    grid: [..][..] int;

    for 1..3 {
        row: [..] int;
        for 1..3 array_add(*row, it);
        array_add(*grid, row);
    }

    removed_rows := 0;

    for row: grid {
        for cell: row {
            if cell == 2 {
                remove row;
                assert(row.count == 3);
                removed_rows += 1;
                break;
            }
        }
    }

    assert(removed_rows == 3 && grid.count == 0);
    array_free(grid);

    // Struct elements are aggregates: `it` names the element itself.
    posts: [..] Post;
    for 1..3 array_add(*posts, .{ it, it * 100 });
    for posts assert(*it == *posts[it_index]);

    wrappers: [..] Wrapper;
    for 1..3 array_add(*wrappers, .{ it });
    for wrappers assert(*it == *wrappers[it_index]);

    // After `remove`, `it` shows the element that moved in.
    shown := 0;

    for wrappers {
        if it.value == 1 {
            remove it;
            shown = it.value;
        }
    }

    assert(shown == 3);
    assert(wrappers.count == 2 && wrappers[0].value == 3 && wrappers[1].value == 2);

    // Through a pointer iterator the element can be changed.
    for *wrappers it.value += 10;

    assert(wrappers[0].value == 13 && wrappers[1].value == 12);

    // A pointer held by the element can be written through.
    target := 5;
    holders: [..] *int;

    array_add(*holders, *target);
    for holders it.* = 6;

    assert(target == 6);

    array_free(posts);
    array_free(wrappers);
    array_free(holders);

    // A fixed array of scalars copies too.
    fixed := int.[4, 5, 6];
    for fixed it = 0;

    assert(fixed[0] == 4 && fixed[2] == 6);

    print("ok\n");
}
