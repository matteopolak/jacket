#import "Basic";
#import "String";
#import "Extensions/Environment";
#import "Extensions/Testing";

follows_the_variable :: (t: *Test) {
    over := environment_variable("JACKET_OUTPUT_DIR");
    directory := output_directory();

    if over.count {
        expect_equal(directory, over);
    } else {
        expect(ends_with(directory, "/target"), tprint("% is not a target folder", directory));
    }

    expect_equal(output_path("a/b.bin"), tprint("%/a/b.bin", directory));
} @test
