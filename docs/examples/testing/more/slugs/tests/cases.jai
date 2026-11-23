#import "Basic";
#import "Extensions/Testing";
#import "slugs";

Case :: struct {
    title: string;
    want: string;
}

TITLES :: Case.[
    .{ "Hello World", "hello-world" },
    .{ "  padded  ", "padded" },
    .{ "A--B", "a-b" },
];

slugifies :: (title: string, want: string) {
    got := slugify(title);
    defer free(got);
    expect_equal(got, want);
} @test @cases(TITLES)

has_no_uppercase :: (title: string) {
    got := slugify(title);
    defer free(got);
    for 0..got.count - 1 expect(got[it] < #char "A" || got[it] > #char "Z");
} @test @property
