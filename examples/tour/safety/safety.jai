// Runtime safety: casts that lose data and indexes past the end are caught
// while the program runs, and each check has an opt-out for code that knows better.
//
// Try it: set BREAK_ON_PURPOSE below to another value and watch the program stop
// with a runtime error that points at the line.

Breakage :: enum {
    NOTHING;
    NARROWING_CAST; // cast(u8) of 300
    INDEX_PAST_END; // the fourth element of three
    NULL_POINTER; // reading through a null pointer
}

BREAK_ON_PURPOSE :: Breakage.NOTHING;

tour_safety :: () {
    big := 300;

    // A cast to a narrower integer checks that the value fits. These two opt out:
    // `trunc` says "keep the low bits", `no_check` skips the check entirely.
    print("cast,trunc(u8) % = %, cast,no_check(u8) % = %\n",
        big, cast,trunc(u8) big, big, cast,no_check(u8) big);

    // Indexing checks the bounds. #no_abc ("no array bounds check") turns that off
    // for a procedure, a loop or a block; here a view reads into its backing array.
    backing := int.[10, 20, 30, 40];
    view: [] int = backing;
    view.count = 2;
    sum := 0;
    for 0..3 #no_abc {
        sum += view[it];
    }
    print("A view of % numbers, read past its end with #no_abc: sum %\n", view.count, sum);

    // A whole program can relax the checks too: a metaprogram sets
    // Build_Options.cast_bounds_check to .NONFATAL (warn, keep going) or .OFF.

    if #complete BREAK_ON_PURPOSE == {
        case .NOTHING;
            print("BREAK_ON_PURPOSE is .NOTHING; change it to see a runtime error.\n");
        case .NARROWING_CAST;
            small := cast(u8) big;
            print("never printed: %\n", small);
        case .INDEX_PAST_END;
            three := int.[1, 2, 3];
            index := three.count;
            print("never printed: %\n", three[index]);
        case .NULL_POINTER;
            nothing: *int;
            value := nothing.*;
            print("never printed: %\n", value);
    }
}
