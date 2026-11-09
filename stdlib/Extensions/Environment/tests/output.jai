#import "Basic";
#import "String";
#import "Extensions/Environment";
#import "Extensions/Testing";

follows_the_variable :: (t: *Test) {
    over := environment_variable("JACKET_OUTPUT_DIR");
    directory := output_directory();

    if over.count {
        expect_equal(directory, resolve_output_directory(over));
    } else {
        expect(ends_with(directory, "/target"), tprint("% is not a target folder", directory));
    }

    expect_equal(output_path("a/b.bin"), tprint("%/a/b.bin", directory));
} @test

// A relative `JACKET_OUTPUT_DIR` is below the checkout, as the repository's tools read it, not below
// the working directory the test happens to run in.
relative_is_below_the_checkout :: (t: *Test) {
    target := resolve_output_directory("");

    expect(ends_with(target, "/target"), tprint("% is not a target folder", target));
    checkout := slice(target, 0, target.count - "/target".count);

    expect_equal(resolve_output_directory("out/x"), tprint("%/out/x", checkout));

    #if OS == .WINDOWS {
        expect_equal(resolve_output_directory("C:/out"), "C:/out");
    } else {
        expect_equal(resolve_output_directory("/out/x"), "/out/x");
    }
} @test
