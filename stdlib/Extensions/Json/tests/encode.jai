// Exact encoder output for a struct that uses every field kind and attribute, compact and
// pretty. The expected texts were taken before the encoder was optimised: they pin the bytes.
//
//     jaic run stdlib/Extensions/Json/tests/encode.jai

#import "Basic";
#import "String";
#import "Hash_Table";
#import "Extensions/Json";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Role :: enum {
    MEMBER;
    ADMIN;
    SUPER_USER;
}

Level :: enum u8 {
    LOW :: 1;
    HIGH :: 7;
}

Point :: struct {
    x: int;
    y: int;
}

Money :: struct {
    cents: s64;
}

Inner :: struct @json(rename_all=kebab-case) {
    first_name: string;
    last_name: string; @json(skip_serializing_if=empty)
    big: u64;
}

Flat :: struct {
    fx: int;
    fy: string; @json(rename=why)
}

Wide :: struct @json(rename_all=SCREAMING_SNAKE_CASE) {
    ident: int;
    plain_name: string;
    small: s8;
    medium: s16;
    large: s32;
    huge: s64;
    usmall: u8;
    umedium: u16;
    ularge: u32;
    uhuge: u64;
    single: float32;
    double: float64;
    flag: bool;
    text: string;
    role: Role;
    level: Level; @json(as_number)
    renamed: int; @json(rename=Custom-Key)
    skipped: int; @json(skip)
    zero_hidden: int; @json(skip_serializing_if=zero)
    zero_shown: int; @json(skip_serializing_if=zero)
    empty_hidden: string; @json(skip_serializing_if=empty)
    null_hidden: *int; @json(skip_serializing_if=null)
    null_shown: *int;
    pointer: *Point;
    flat: Flat; @json(flatten)
    money: Money; @json(with=money)
    fixed: [3] int;
    view: [] string;
    list: [..] Point;
    grid: [..][..] float64;
    empty_list: [..] int;
    roles: [..] Role;
    table: Table(string, int);
    table_of_lists: Table(string, [..] Point);
    inner: Inner;
    any: Json_Value;

    money_encode :: (w: *Json_Writer, value: *Money) {
        json_write_float64(w, cast(float64) value.cents / 100.0);
    }

    money_decode :: (r: *Json_Reader, out: *Money) -> bool {
        amount: float64;
        if !json_read_float64(r, *amount) return false;
        out.cents = cast(s64) (amount * 100.0 + 0.5);

        return true;
    }
}

make :: () -> Wide {
    w: Wide;
    w.ident = -1;
    w.plain_name = "quote \" slash \\ tab \t nl \n ctl \x01 é 😀 bad \xff end";
    w.small = -128;
    w.medium = -32768;
    w.large = -2147483648;
    w.huge = -9223372036854775807 - 1;
    w.usmall = 255;
    w.umedium = 65535;
    w.ularge = 4294967295;
    w.uhuge = 18446744073709551615;
    w.single = 0.1;
    w.double = 0.1 + 0.2;
    w.flag = true;
    w.text = "a long enough string, with plain bytes only, to take the bulk copy path in one go";
    w.role = .SUPER_USER;
    w.level = .HIGH;
    w.renamed = 9;
    w.zero_shown = 1;
    shared_five = 5;
    w.null_shown = *shared_five;
    w.pointer = New(Point);
    w.pointer.x = 3;
    w.flat = .{ 4, "w" };
    w.money = .{ 1250 };
    w.fixed = .[1, 22, 333];
    w.view = .["a", "", "c\n"];
    array_add(*w.list, .{ 1, 2 });
    array_add(*w.list, .{ -3, 4 });
    row: [..] float64;

    array_add(*row, 1.5);
    array_add(*row, 1e21);
    array_add(*row, -0.0);
    array_add(*row, 5e-324);
    array_add(*row, 123456789.125);
    empty_row: [..] float64;

    array_add(*w.grid, row);
    array_add(*w.grid, empty_row);
    array_add(*w.roles, .ADMIN);
    array_add(*w.roles, .MEMBER);
    table_set(*w.table, "one", 1);
    list: [..] Point;

    array_add(*list, .{ 7, 8 });
    table_set(*w.table_of_lists, "k\"q", list);
    w.inner = .{ "Ann", "", 18446744073709551615 };
    w.any, _ = from_json(Json_Value, "{\"k\":[1,2.5,null,true,\"s\"],\"e\":{}}");

    return w;
}

shared_five := 0;

COMPACT :: "{\"IDENT\":-1,\"PLAIN_NAME\":\"quote \\\" slash \\\\ tab \\t nl \\n ctl \\u0001 é 😀 bad � end\",\"SMALL\":-128,\"MEDIUM\":-32768,\"LARGE\":-2147483648,\"HUGE\":-9223372036854775808,\"USMALL\":255,\"UMEDIUM\":65535,\"ULARGE\":4294967295,\"UHUGE\":18446744073709551615,\"SINGLE\":0.1,\"DOUBLE\":0.30000000000000004,\"FLAG\":true,\"TEXT\":\"a long enough string, with plain bytes only, to take the bulk copy path in one go\",\"ROLE\":\"SUPER_USER\",\"LEVEL\":7,\"Custom-Key\":9,\"ZERO_SHOWN\":1,\"NULL_SHOWN\":5,\"POINTER\":{\"x\":3,\"y\":0},\"fx\":4,\"why\":\"w\",\"MONEY\":12.5,\"FIXED\":[1,22,333],\"VIEW\":[\"a\",\"\",\"c\\n\"],\"LIST\":[{\"x\":1,\"y\":2},{\"x\":-3,\"y\":4}],\"GRID\":[[1.5,1e21,-0.0,5e-324,123456789.125],[]],\"EMPTY_LIST\":[],\"ROLES\":[\"ADMIN\",\"MEMBER\"],\"TABLE\":{\"one\":1},\"TABLE_OF_LISTS\":{\"k\\\"q\":[{\"x\":7,\"y\":8}]},\"INNER\":{\"first-name\":\"Ann\",\"big\":18446744073709551615},\"ANY\":{\"k\":[1,2.5,null,true,\"s\"],\"e\":{}}}";

PRETTY :: "{\n  \"IDENT\": -1,\n  \"PLAIN_NAME\": \"quote \\\" slash \\\\ tab \\t nl \\n ctl \\u0001 é 😀 bad � end\",\n  \"SMALL\": -128,\n  \"MEDIUM\": -32768,\n  \"LARGE\": -2147483648,\n  \"HUGE\": -9223372036854775808,\n  \"USMALL\": 255,\n  \"UMEDIUM\": 65535,\n  \"ULARGE\": 4294967295,\n  \"UHUGE\": 18446744073709551615,\n  \"SINGLE\": 0.1,\n  \"DOUBLE\": 0.30000000000000004,\n  \"FLAG\": true,\n  \"TEXT\": \"a long enough string, with plain bytes only, to take the bulk copy path in one go\",\n  \"ROLE\": \"SUPER_USER\",\n  \"LEVEL\": 7,\n  \"Custom-Key\": 9,\n  \"ZERO_SHOWN\": 1,\n  \"NULL_SHOWN\": 5,\n  \"POINTER\": {\n    \"x\": 3,\n    \"y\": 0\n  },\n  \"fx\": 4,\n  \"why\": \"w\",\n  \"MONEY\": 12.5,\n  \"FIXED\": [\n    1,\n    22,\n    333\n  ],\n  \"VIEW\": [\n    \"a\",\n    \"\",\n    \"c\\n\"\n  ],\n  \"LIST\": [\n    {\n      \"x\": 1,\n      \"y\": 2\n    },\n    {\n      \"x\": -3,\n      \"y\": 4\n    }\n  ],\n  \"GRID\": [\n    [\n      1.5,\n      1e21,\n      -0.0,\n      5e-324,\n      123456789.125\n    ],\n    []\n  ],\n  \"EMPTY_LIST\": [],\n  \"ROLES\": [\n    \"ADMIN\",\n    \"MEMBER\"\n  ],\n  \"TABLE\": {\n    \"one\": 1\n  },\n  \"TABLE_OF_LISTS\": {\n    \"k\\\"q\": [\n      {\n        \"x\": 7,\n        \"y\": 8\n      }\n    ]\n  },\n  \"INNER\": {\n    \"first-name\": \"Ann\",\n    \"big\": 18446744073709551615\n  },\n  \"ANY\": {\n    \"k\": [\n      1,\n      2.5,\n      null,\n      true,\n      \"s\"\n    ],\n    \"e\": {}\n  }\n}";

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;

        w := make();
        compact := to_json(w);

        check(compact == COMPACT, tprint("compact:\n%", compact));

        pretty := to_json(w, true);

        check(pretty == PRETTY, tprint("pretty:\n%", pretty));

        // The same text through a builder that already holds something, across segments.
        builder: String_Builder;

        for 1..300 append(*builder, "0123456789");

        write_json(*builder, w);
        text := builder_to_string(*builder);

        check(text.count == 3000 + COMPACT.count, "write_json after other text, length");
        check(slice(text, 3000, COMPACT.count) == COMPACT, "write_json after other text");
    }

    assert(failures == 0);
    print("ok\n");
}
