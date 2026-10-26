#import "Basic";
#import "Extensions/Testing";
M :: #import "md5";
H :: #import "Hash";

Digest :: struct {
    input: string;
    want: string;
}

DIGESTS :: Digest.[
    .{ "", "d41d8cd98f00b204e9800998ecf8427e" },
    .{ "abc", "900150983cd24fb0d6963f7d28e17f72" },
    .{ "message digest", "f96b697d7cb7938d525a2f31aaf161d0" },
];

md5_matches_the_published_digests :: (input: string, want: string) {
    assert_that(M.md5(input,, temp), is(want));
} @test @cases(DIGESTS)

string_hashes :: () {
    expect(H.fnv1a_hash("hello".data, 5) == 0xa430d84680aabd0b);
    assert_that(H.djb2_hash("hello"), is(261238937));
} @test

sdbm_of_nothing_is_its_seed :: () {
    assert_that(H.sdbm_hash(null, 0), is(5381));
} @test
