// Run with jacket bench --path stdlib/Extensions/Variant.
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Variant";

Point :: struct {
    x: float64;
    y: float64;
}

Record :: struct {
    id: s64;
    name: string;
    origin: Point;
    flags: [4] u8;
}

Shape :: Variant(struct {
    none: void;
    number: s64;
    record: Record;
    list: [..] string;
});

variant_equal_records :: (b: *Bench) {
    first := variant(Shape, Record.{ id = 7, name = "a record name", origin = .{ 1, 2 } });
    second := variant(Shape, Record.{ id = 7, name = "a record name", origin = .{ 1, 2 } });

    reset_timer(b);

    for 0..b.iterations - 1 {
        black_box(black_box(first) == black_box(second));
    }
} @bench

variant_equal_lists :: (b: *Bench) {
    one: [..] string;
    two: [..] string;

    for 0..7 {
        array_add(*one, "element");
        array_add(*two, "element");
    }

    first := variant(Shape, one);
    second := variant(Shape, two);

    defer array_free(one);
    defer array_free(two);
    reset_timer(b);

    for 0..b.iterations - 1 {
        black_box(black_box(first) == black_box(second));
    }
} @bench

// Builds a variant that owns a list of strings and frees it again: the allocation is in the time
// too, and the same before and after a change to how it is released.
variant_free_list :: (b: *Bench) {
    reset_timer(b);

    for 0..b.iterations - 1 {
        list: [..] string;

        for 0..3 array_add(*list, copy_string("owned text"));

        owned := variant(Shape, list);

        free_variant(black_box(*owned));
    }
} @bench

variant_free_number :: (b: *Bench) {
    owned := variant(Shape, cast(s64) 5);

    reset_timer(b);

    for 0..b.iterations - 1 {
        free_variant(black_box(*owned));
        owned = variant(Shape, cast(s64) 5);
    }
} @bench
