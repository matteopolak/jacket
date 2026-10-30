// Lowering a string in place.
#import "Extensions/Testing";

lowering_changes_the_bytes_in_place :: () {
    letters: [5] u8 = .[77, 105, 88, 101, 68];
    mutable: string;
    mutable.data = letters.data;
    mutable.count = letters.count;
    String.to_lower_in_place(mutable);
    expect_equal(mutable, "mixed");
} @test

#scope_file

String :: #import "String";
