Pair :: struct { first: int; second: int; }
entries :: Pair.[
    .{1, 2},
    .{3, 4},
    .{5, 6},
];
Text_Pair :: struct { name: string; value: int; }
named_entries :: Text_Pair.[
    .{name="first", value=1},
    .{name="second", value=2},
    .{name="third", value=3},
];
after :: () {
    healthy := 42;
    result := healthy + 1;
}
