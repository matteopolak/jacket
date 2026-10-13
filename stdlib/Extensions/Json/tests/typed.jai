// Typed encoding and decoding: every type and every @json attribute, round trips, and errors
// with their path, line and column.
//
//     jaic stdlib/Extensions/Json/tests/typed.jai -interp

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

Perms :: enum_flags u8 {
    READ;
    WRITE;
    EXEC;
}

Address :: struct {
    street: string;
    zip: int;
}

Money :: struct {
    cents: s64;
}

User :: struct @json(rename_all=camelCase) {
    id: int;
    display_name: string;
    email: string; @json(skip_serializing_if=empty,default)
    password_hash: string; @json(skip)
    created_at: string; @json(rename=created,alias=createdAt)
    role := Role.MEMBER; @json(default,rename_all=lowercase)
    address: Address; @json(flatten)
    tags: [..] string;
    spent: Money; @json(with=money)

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

Signup :: struct @json(deny_unknown_fields) {
    name: string;
    email: string;
}

Defaults :: struct @json(default) {
    a := 5;
    b := "x";
}

Wrapper :: struct @json(transparent) {
    inner: [] int;
}

Skips :: struct {
    both: int; @json(skip)
    only_out: int; @json(skip_deserializing)
    only_in: int; @json(skip_serializing)
    zero: int; @json(skip_serializing_if=zero)
    maybe: *int; @json(skip_serializing_if=null)
    flag: bool; @json(skip_serializing_if=zero)
    items: [..] int; @json(skip_serializing_if=empty)
    visible: int;
}

Everything :: struct {
    a: s8;
    b: s16;
    c: s32;
    d: s64;
    e: u8;
    f: u16;
    g: u32;
    h: u64;
    i: float32;
    j: float64;
    k: bool;
    l: string;
    m: Role;
    n: Level; @json(as_number)
    o: Perms;
    p: [3] int;
    q: [] string;
    r: [..][..] int;
    s: Table(string, int);
    t: *Address;
    u: Json_Value;
    v: [..] Role; @json(rename_all=SCREAMING-KEBAB-CASE)
    w: Table(string, [..] Address);
}

Node :: struct {
    value: int;
    next: *Node;
    children: [..] Node;
}

types :: () {
    e: Everything;
    e.a = -128;
    e.b = -32768;
    e.c = -2147483648;
    e.d = -9223372036854775807 - 1;
    e.e = 255;
    e.f = 65535;
    e.g = 4294967295;
    e.h = 18446744073709551615;
    e.i = 0.1;
    e.j = 0.1 + 0.2;
    e.k = true;
    e.l = "quote \" backslash \\ tab \t nl \n ctl \x01 é 😀";
    e.m = .SUPER_USER;
    e.n = .HIGH;
    e.o = .READ | .EXEC;
    e.p = .[1, 2, 3];
    e.q = .["a", "b"];
    empty_row: [..] int;

    array_add(*e.r, empty_row);
    inner: [..] int;

    array_add(*inner, 4);
    array_add(*inner, 5);
    array_add(*e.r, inner);
    table_set(*e.s, "one", 1);
    table_set(*e.s, "two", 2);
    e.t = New(Address);
    e.t.street = "Main";
    e.u, _ = from_json(Json_Value, "{\"k\":[1,2.5,null,true,\"s\"]}");
    array_add(*e.v, .ADMIN);
    array_add(*e.v, .MEMBER);
    list: [..] Address;

    array_add(*list, .{ "x", 1 });
    table_set(*e.w, "k", list);

    text := to_json(e);
    back, error := from_json(Everything, text);

    check(error.kind == .NONE, tprint("decode: %", error_text(error)));
    check(back.a == e.a && back.b == e.b && back.c == e.c && back.d == e.d, "signed integers are exact");
    check(back.e == e.e && back.f == e.f && back.g == e.g && back.h == e.h, "unsigned integers are exact");
    check(back.i == e.i && back.j == e.j, "floats round trip");
    check(back.k && back.l == e.l, "bool and string");
    check(back.m == .SUPER_USER && back.n == .HIGH && back.o == (Perms.READ | .EXEC), "enums");
    check(back.p[0] == 1 && back.p[2] == 3, "fixed array");
    check(back.q.count == 2 && back.q[1] == "b", "view");
    check(back.r.count == 2 && back.r[0].count == 0 && back.r[1][1] == 5, "nested dynamic arrays");
    check(back.s.count == 2, "table count");
    found, one := table_find(*back.s, "two");

    check(found && one == 2, "table entry");
    check(back.t != null && back.t.street == "Main", "pointer");
    check(json_get(*back.u, "k") != null && json_count(json_get(*back.u, "k")) == 5, "Json_Value member");
    check(back.v.count == 2 && back.v[0] == .ADMIN && back.v[1] == .MEMBER, "enum array");
    found2, entries := table_find(*back.w, "k");

    check(found2 && entries.count == 1 && entries[0].zip == 1, "table of arrays");
    check(to_json(back) == text, "encoding the decoded value gives the same text");

    check(contains(text, "\"m\":\"SUPER_USER\""), "enum by name");
    check(contains(text, "\"n\":7"), "as_number");
    check(contains(text, "\"o\":5"), "flags are numbers");
    check(contains(text, "\"v\":[\"ADMIN\",\"MEMBER\"]"), "rename_all on an enum field");
    check(contains(text, "\"t\":{\"street\":\"Main\",\"zip\":0}"), "pointer encodes its target");
    check(contains(text, "\\u0001"), "control characters are escaped");
    check(contains(text, "é 😀"), "non-ASCII passes through");
    check(contains(text, "\"h\":18446744073709551615") && contains(text, "\"d\":-9223372036854775808"), "extremes");
}

attributes :: () {
    user: User;
    user.id = 7;
    user.display_name = "Ann";
    user.created_at = "yesterday";
    user.role = .SUPER_USER;
    user.address = .{ "Elm", 12 };
    user.password_hash = "secret";
    array_add(*user.tags, "x");
    user.spent = .{ 1250 };

    text := to_json(user);

    check(text == "{\"id\":7,\"displayName\":\"Ann\",\"created\":\"yesterday\",\"role\":\"super_user\",\"street\":\"Elm\",\"zip\":12,\"tags\":[\"x\"],\"spent\":12.5}", text);

    back, error := from_json(User, text);

    check(error.kind == .NONE, error_text(error));
    check(back.id == 7 && back.display_name == "Ann" && back.created_at == "yesterday", "fields");
    check(back.role == .SUPER_USER && back.address.zip == 12 && back.address.street == "Elm", "enum and flatten");
    check(back.spent.cents == 1250, "with=");
    check(back.password_hash == "", "skip");

    alias, error2 := from_json(User, "{\"id\":1,\"displayName\":\"B\",\"createdAt\":\"t\",\"street\":\"s\",\"zip\":2,\"tags\":[],\"spent\":1}");

    check(error2.kind == .NONE && alias.created_at == "t", "alias");
    check(alias.role == .MEMBER, "default keeps the initializer");

    _, error3 := from_json(User, "{\"id\":1,\"displayName\":\"B\",\"created\":\"t\",\"street\":\"s\",\"zip\":2,\"tags\":[],\"spent\":1,\"extra\":{\"deep\":[1,2]}}");

    check(error3.kind == .NONE, "unknown fields are ignored");

    _, missing := from_json(User, "{\"id\":1}");

    check(missing.kind == .MISSING_FIELD && contains(missing.message, "displayName"), "a missing field is named");

    pretty := to_json(user, true);

    check(contains(pretty, "\n  \"id\": 7,\n"), "pretty printing");
    again, error4 := from_json(User, pretty);

    check(error4.kind == .NONE && again.id == 7, "pretty text decodes");

    _, denied := from_json(Signup, "{\"name\":\"a\",\"email\":\"b\",\"admin\":true}");

    check(denied.kind == .UNKNOWN_FIELD && contains(denied.message, "admin"), "deny_unknown_fields");
    ok, error5 := from_json(Signup, "{\"name\":\"a\",\"email\":\"b\"}");

    check(error5.kind == .NONE && ok.name == "a", "deny_unknown_fields accepts known keys");

    _, duplicate := from_json(Signup, "{\"name\":\"a\",\"name\":\"b\",\"email\":\"b\"}");

    check(duplicate.kind == .DUPLICATE_FIELD, "duplicate keys in a struct");

    d, error6 := from_json(Defaults, "{}");

    check(error6.kind == .NONE && d.a == 5 && d.b == "x", "struct-level default");
    d2, _ := from_json(Defaults, "{\"a\":9}");

    check(d2.a == 9 && d2.b == "x", "struct-level default, partly given");

    wrapper, error7 := from_json(Wrapper, "[1,2,3]");

    check(error7.kind == .NONE && wrapper.inner.count == 3, "transparent decode");
    check(to_json(wrapper) == "[1,2,3]", "transparent encode");

    skips: Skips;
    skips.both = 1;
    skips.only_out = 2;
    skips.only_in = 3;
    skips.visible = 4;
    check(to_json(skips) == "{\"only_out\":2,\"visible\":4}", tprint("skip_serializing_if and skip: %", to_json(skips)));
    skips.zero = 1;
    skips.flag = true;
    n := 5;

    skips.maybe = *n;
    array_add(*skips.items, 1);
    check(to_json(skips) == "{\"only_out\":2,\"zero\":1,\"maybe\":5,\"flag\":true,\"items\":[1],\"visible\":4}", tprint("skip_serializing_if keeps non-empty values: %", to_json(skips)));

    _, error8 := from_json(Skips, "{\"both\":9,\"only_out\":9,\"only_in\":8,\"visible\":1}");

    check(error8.kind == .MISSING_FIELD || error8.kind == .NONE, "skipped fields are not required");
}

Camel :: struct @json(rename_all=camelCase) {
    display_name: int;
    http_server: int;
}

Pascal :: struct @json(rename_all=PascalCase) {
    display_name: int;
}

Snake :: struct @json(rename_all=snake_case) {
    DisplayName: int;
    HTTPServer: int;
    parse2Json: int;
}

Kebab :: struct @json(rename_all=kebab-case) {
    display_name: int;
}

Screaming :: struct @json(rename_all=SCREAMING_SNAKE_CASE) {
    display_name: int;
}

Screaming_Kebab :: struct @json(rename_all=SCREAMING-KEBAB-CASE) {
    display_name: int;
}

Lower :: struct @json(rename_all=lowercase) {
    Display_Name: int;
}

Upper :: struct @json(rename_all=UPPERCASE) {
    display_name: int;
}

rules :: () {
    check(to_json(Camel.{}) == "{\"displayName\":0,\"httpServer\":0}", "camelCase");
    check(to_json(Pascal.{}) == "{\"DisplayName\":0}", "PascalCase");
    check(to_json(Snake.{}) == "{\"display_name\":0,\"http_server\":0,\"parse2_json\":0}", tprint("snake_case: %", to_json(Snake.{})));
    check(to_json(Kebab.{}) == "{\"display-name\":0}", "kebab-case");
    check(to_json(Screaming.{}) == "{\"DISPLAY_NAME\":0}", "SCREAMING_SNAKE_CASE");
    check(to_json(Screaming_Kebab.{}) == "{\"DISPLAY-NAME\":0}", "SCREAMING-KEBAB-CASE");
    check(to_json(Lower.{}) == "{\"display_name\":0}", "lowercase");
    check(to_json(Upper.{}) == "{\"DISPLAY_NAME\":0}", "UPPERCASE");
}

errors :: () {
    _, error := from_json(Address, "{\"street\": \"a\",\n  \"zip\": \"nope\"}");

    check(error.kind == .TYPE && error.path == "zip" && error.line == 2 && error.column == 10, tprint("type error: %", error_text(error)));

    _, error2 := from_json(Everything, "{\"p\":[1,2]}");

    check(error2.kind != .NONE && error2.path == "p", tprint("short fixed array: %", error_text(error2)));

    _, error3 := from_json(Node, "{\"value\":1,\"next\":null,\"children\":[{\"value\":1,\"next\":null,\"children\":[]},{\"value\":true}]}");

    check(error3.kind == .TYPE && error3.path == "children[1].value", tprint("deep path: %", error_text(error3)));

    _, error4 := from_json(Address, "{\"street\":\"a\",\"zip\":99999999999999999999}");

    check(error4.kind == .OUT_OF_RANGE && error4.path == "zip", "out of range");

    _, error5 := from_json([3] s8, "[1,2,300]");

    check(error5.kind == .OUT_OF_RANGE && error5.path == "[2]", tprint("s8 range: %", error_text(error5)));

    _, error6 := from_json(Role, "\"NOBODY\"");

    check(error6.kind == .INVALID_VALUE && contains(error6.message, "NOBODY"), "unknown variant");

    _, error7 := from_json(int, "1 2");

    check(error7.kind == .SYNTAX, "trailing data");

    _, error8 := from_json(int, "1.5");

    check(error8.kind == .TYPE, "a float is not an integer");

    _, error9 := from_json(Address, "{\"street\":\"a\",}");

    check(error9.kind == .SYNTAX, "trailing comma");

    deep := "";
    for 1..300 deep = join(deep, "[");

    _, error10 := from_json(Json_Value, deep);

    check(error10.kind == .LIMIT, "depth limit");

    shallow: Json_Options;
    shallow.max_depth = 2;
    _, error11 := from_json(Json_Value, "[[[1]]]", shallow);

    check(error11.kind == .LIMIT, "custom depth limit");
    _, error12 := from_json(Json_Value, "[[1]]", shallow);

    check(error12.kind == .NONE, "within the custom limit");

    small: Json_Options;
    small.max_size = 3;
    _, error13 := from_json(Json_Value, "[1,2,3]", small);

    check(error13.kind == .LIMIT, "size limit");

    _, error14 := from_json(Json_Value, "\n\n  [1, ?]");

    check(error14.kind == .SYNTAX && error14.line == 3 && error14.column == 7, tprint("position: %", error_text(error14)));
    _, error15 := from_json(Json_Value, "[\"é\", x]");

    check(error15.line == 1 && error15.column == 7, tprint("column counts characters: %", error_text(error15)));
}

values :: () {
    v := json_object();

    json_set(*v, "name", json_string("n"));
    json_set(*v, "n", json_int(-5));
    json_set(*v, "f", json_float(2.5));
    list := json_array();

    json_push(*list, json_bool(true));
    json_push(*list, json_null());
    json_set(*v, "list", list);
    json_set(*v, "name", json_string("replaced"));

    check(to_json(v) == "{\"name\":\"replaced\",\"n\":-5,\"f\":2.5,\"list\":[true,null]}", to_json(v));

    n, ok := json_as_s64(json_get(*v, "n"));

    check(ok && n == -5, "json_as_s64");
    _, ok = json_as_s64(json_get(*v, "f"));
    check(!ok, "2.5 is not an integer");
    f, ok2 := json_as_float64(json_get(*v, "n"));

    check(ok2 && f == -5.0, "an integer reads as a float");
    s, ok3 := json_as_string(json_get(*v, "name"));

    check(ok3 && s == "replaced", "json_as_string");
    check(json_get(*v, "missing") == null && json_at(json_get(*v, "list"), 1).kind == .NULL, "lookups");

    big, error := from_json(Json_Value, "[123456789012345678901234567890, 18446744073709551615]");

    check(error.kind == .NONE && to_json(big) == "[123456789012345678901234567890,18446744073709551615]", "big integers keep their digits");
    u, ok4 := json_as_u64(json_at(*big, 1));

    check(ok4 && u == 18446744073709551615, "u64");
    _, ok5 := json_as_s64(json_at(*big, 0));

    check(!ok5, "too big for s64");

    builder: String_Builder;

    write_json(*builder, v);
    check(builder_to_string(*builder) == to_json(v), "write_json");

    free_json(*v);
    free_json(*big);
    check(v.kind == .NULL, "free_json on a Json_Value");
}

recursion :: () {
    root: Node;
    root.value = 1;
    root.next = New(Node);
    root.next.value = 2;
    child: Node;
    child.value = 3;
    array_add(*root.children, child);

    text := to_json(root);

    check(text == "{\"value\":1,\"next\":{\"value\":2,\"next\":null,\"children\":[]},\"children\":[{\"value\":3,\"next\":null,\"children\":[]}]}", text);
    back, error := from_json(Node, text);

    check(error.kind == .NONE && back.next.value == 2 && back.children[0].value == 3, "recursive types");
}

strings :: () {
    s, error := from_json(string, "\"a\\u00e9\\ud83d\\ude00\\n\\/\"");

    check(error.kind == .NONE && s == "aé😀\n/", "escapes decode");

    _, bad := from_json(string, "\"\\ud83d\"");

    check(bad.kind == .SYNTAX, "lone surrogate");

    invalid := "a\xffb";
    out := to_json(invalid);

    check(out == "\"a\xef\xbf\xbdb\"", "invalid UTF-8 is replaced when encoding");
}

main :: () {
    // Every check runs on the scalar paths and again with the vector paths and the structural
    // index forced on (even for tiny texts); the results must not differ.
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        types();
        attributes();
        rules();
        errors();
        values();
        recursion();
        strings();
    }

    assert(failures == 0);
    print("ok\n");
}
