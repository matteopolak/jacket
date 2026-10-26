// Json against rluba/jaison and kujukuju/ContiguousJsonJai on three documents shaped like real
// ones: an API response (100 issues, minified), a large array of telemetry records (about 2 MB,
// minified) and a deeply nested, pretty-printed configuration. Every library decodes the same text
// into the same Jai types and encodes the same values; the untyped trees are each library's own.
//
// The community libraries are imported from corpus/upstream through `-import_dir`, in their own
// namespaces. Run through tools/bench/library_compare.py, which keeps them inside the sandbox.
//
// Each iteration allocates from a bump pool that is emptied after the call, for all three libraries
// alike (jaison's typed decoder has no free procedure, and the others free in different ways).
#import "Basic";
#import "String";
#import "Flat_Pool";
#import "Extensions/Json";
#import "Extensions/Testing";

jaison :: #import "rluba--jaison";
contig :: #import "kujukuju--ContiguousJsonJai";

User :: struct {
    login: string;
    id: int;
    avatar_url: string;
    site_admin: bool;
}

Label :: struct {
    id: int;
    name: string;
    color: string;
    default: bool;
}

Reactions :: struct {
    total: int;
    plus_one: int;
    minus_one: int;
    heart: int;
}

Issue :: struct {
    id: int;
    number: int;
    title: string;
    state: string;
    locked: bool;
    comments: int;
    created_at: string;
    updated_at: string;
    body: string;
    score: float64;
    user: User;
    assignee: *User;
    labels: [..] Label;
    reactions: Reactions;
}

Record :: struct {
    id: int;
    ts: string;
    lat: float64;
    lon: float64;
    speed: float64;
    name: string;
    ok: bool;
    samples: [..] int;
}

Node :: struct {
    name: string;
    enabled: bool;
    limit: int;
    ratio: float64;
    labels: [..] string;
    child: *Node;
}

api_text: string;
records_text: string;
config_text: string;
pool: Flat_Pool;
prepared := false;

// The values the encoders write, decoded once by the library under test.
api_value: [..] Issue;
records_value: [..] Record;
config_value: Node;

seed: u64 = 12345;

next :: (range: int) -> int {
    seed = seed * 6364136223846793005 + 1442695040888963407;
    return cast(int) ((seed >> 33) % cast(u64) range);
}

BODIES :: string.[
    "Steps to reproduce:\\n1. Open the project\\n2. Run the build\\n3. See the error \\\"unexpected token\\\"",
    "Works on my machine, but fails in CI. Could be a path issue (C:\\\\work\\\\src).",
    "The title says it all. Thanks for the quick fix, été 日本語 support would be great too.",
    "Plain text without anything special in it, just a longer sentence so that the field has some length to scan.",
];

TITLES :: string.["Crash on startup", "Add dark mode", "Memory leak in the parser", "Docs: typo in the install guide", "Support for ARM builds", "Slow incremental builds"];

LABELS :: string.["bug", "enhancement", "documentation", "good first issue", "help wanted", "performance"];

indent :: (builder: *String_Builder, count: int) {
    i := 0;

    while i < count {
        append(builder, " ");
        i += 1;
    }
}

make_api :: () -> string {
    builder: String_Builder;

    append(*builder, "[");

    for 0..99 {
        if it append(*builder, ",");

        print(*builder, "{\"id\":%,\"number\":%,\"title\":\"%\",\"state\":\"%\",\"locked\":%,\"comments\":%,", 1000000 + it * 37, it + 1, TITLES[next(TITLES.count)], ifx next(3) then "open" else "closed", ifx next(10) == 0 then "true" else "false", next(40));
        print(*builder, "\"created_at\":\"2026-0%-1%T1%:2%:0%Z\",\"updated_at\":\"2026-1%-0%T0%:3%:1%Z\",", 1 + next(9), next(10), next(10), next(10), next(10), next(2), next(3), next(10), next(10), next(10));
        print(*builder, "\"body\":\"%\",\"score\":%.%,", BODIES[next(BODIES.count)], next(100), next(1000));
        print(*builder, "\"user\":{\"login\":\"user%\",\"id\":%,\"avatar_url\":\"https://avatars.example.com/u/%?v=4\",\"site_admin\":false},", it % 17, 5000 + it % 17, 5000 + it % 17);

        if next(2) {
            append(*builder, "\"assignee\":null,");
        } else {
            print(*builder, "\"assignee\":{\"login\":\"dev%\",\"id\":%,\"avatar_url\":\"https://avatars.example.com/u/%?v=4\",\"site_admin\":true},", it % 5, 900 + it % 5, 900 + it % 5);
        }

        append(*builder, "\"labels\":[");

        for l: 0..next(4) {
            if l append(*builder, ",");
            print(*builder, "{\"id\":%,\"name\":\"%\",\"color\":\"%%%\",\"default\":%}", 200 + l, LABELS[next(LABELS.count)], "d7", "3a", "4f", ifx l == 0 then "true" else "false");
        }

        print(*builder, "],\"reactions\":{\"total\":%,\"plus_one\":%,\"minus_one\":%,\"heart\":%}}", next(30), next(20), next(3), next(8));
    }

    append(*builder, "]");

    return builder_to_string(*builder);
}

make_records :: () -> string {
    builder: String_Builder;

    append(*builder, "[");

    for 0..19999 {
        if it append(*builder, ",");
        print(*builder, "{\"id\":%,\"ts\":\"2026-10-%T%:%:%Z\",\"lat\":%.%,\"lon\":-%.%,\"speed\":%.%,\"name\":\"sensor-%\",\"ok\":%,\"samples\":[%,%,%,%]}", it, 10 + next(18), 10 + next(14), 10 + next(50), 10 + next(50), 30 + next(20), 1000 + next(90000), 100 + next(20), 1000 + next(90000), next(120), next(100), it % 64, ifx next(9) then "true" else "false", next(500), next(500), next(500), next(500));
    }

    append(*builder, "]");

    return builder_to_string(*builder);
}

DEPTH :: 100;

make_config :: () -> string {
    builder: String_Builder;

    for level: 0..DEPTH - 1 {
        indent(*builder, level * 2);
        append(*builder, "{\n");
        indent(*builder, level * 2 + 2);
        print(*builder, "\"name\": \"section-%\",\n", level);
        indent(*builder, level * 2 + 2);
        print(*builder, "\"enabled\": %,\n", ifx level % 3 then "true" else "false");
        indent(*builder, level * 2 + 2);
        print(*builder, "\"limit\": %,\n", 100 + level * 8);
        indent(*builder, level * 2 + 2);
        print(*builder, "\"ratio\": 0.%,\n", 25 + level);
        indent(*builder, level * 2 + 2);
        print(*builder, "\"labels\": [\"alpha\", \"beta\", \"gamma-%\"],\n", level);
        indent(*builder, level * 2 + 2);
        append(*builder, "\"child\": ");
    }

    append(*builder, "null");

    for level: 0..DEPTH - 1 {
        append(*builder, "\n");
        indent(*builder, (DEPTH - 1 - level) * 2);
        append(*builder, "}");
    }

    return builder_to_string(*builder);
}

prepare :: () {
    if prepared return;

    init(*pool, 512 * 1024 * 1024);
    api_text = make_api();
    records_text = make_records();
    config_text = make_config();
    prepared = true;
}

// A library that rejects the input would make its benchmark measure the failure path.
require :: (ok: bool) {
    if !ok {
        print("the library rejected the benchmark document\n");
        exit(1);
    }
}

// The loop of every benchmark: `work` runs once per iteration on the text, with the pool emptied after.
over_text :: (b: *Bench, text: string, work: (string)) {
    prepare();
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        work(black_box(text));
        reset(*pool);
    }
}

// The encoders write a value decoded before the timer starts; `work` takes nothing.
over_value :: (b: *Bench, bytes: int, work: ()) {
    set_bytes(b, bytes);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        work();
        reset(*pool);
    }
}

// ---------------------------------------------------------------------------------- Jacket

decode_api__jacket :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        value, error := from_json([..] Issue, text);
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_records__jacket :: (b: *Bench) {
    prepare();
    over_text(b, records_text, (text: string) {
        value, error := from_json([..] Record, text);
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_config__jacket :: (b: *Bench) {
    prepare();
    over_text(b, config_text, (text: string) {
        value, error := from_json(Node, text);
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_tree_api__jacket :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        value, error := from_json(Json_Value, text);
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_tree_records__jacket :: (b: *Bench) {
    prepare();
    over_text(b, records_text, (text: string) {
        value, error := from_json(Json_Value, text);
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

encode_api__jacket :: (b: *Bench) {
    prepare();
    error: Json_Error;
    api_value, error = from_json([..] Issue, api_text);
    over_value(b, to_json(api_value).count, () {
        black_box(to_json(black_box(api_value)));
    });
} @bench

encode_records__jacket :: (b: *Bench) {
    prepare();
    error: Json_Error;
    records_value, error = from_json([..] Record, records_text);
    over_value(b, to_json(records_value).count, () {
        black_box(to_json(black_box(records_value)));
    });
} @bench

encode_config__jacket :: (b: *Bench) {
    prepare();
    error: Json_Error;
    config_value, error = from_json(Node, config_text);
    over_value(b, to_json(config_value).count, () {
        black_box(to_json(black_box(config_value)));
    });
} @bench

roundtrip_api__jacket :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        value, error := from_json([..] Issue, text);
        require(error.kind == .NONE);
        black_box(to_json(value));
    });
} @bench

// ---------------------------------------------------------------------------------- jaison

decode_api__jaison :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        ok, value := jaison.json_parse_string(text, [..] Issue);
        black_box(value);
        require(ok);
    });
} @bench

decode_records__jaison :: (b: *Bench) {
    prepare();
    over_text(b, records_text, (text: string) {
        ok, value := jaison.json_parse_string(text, [..] Record);
        black_box(value);
        require(ok);
    });
} @bench

decode_config__jaison :: (b: *Bench) {
    prepare();
    over_text(b, config_text, (text: string) {
        ok, value := jaison.json_parse_string(text, Node);
        black_box(value);
        require(ok);
    });
} @bench

decode_tree_api__jaison :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        ok, value := jaison.json_parse_string(text);
        black_box(value);
        require(ok);
    });
} @bench

decode_tree_records__jaison :: (b: *Bench) {
    prepare();
    over_text(b, records_text, (text: string) {
        ok, value := jaison.json_parse_string(text);
        black_box(value);
        require(ok);
    });
} @bench

encode_api__jaison :: (b: *Bench) {
    prepare();
    ok: bool;
    ok, api_value = jaison.json_parse_string(api_text, [..] Issue);
    over_value(b, jaison.json_write_string(api_value, indent_char = "").count, () {
        black_box(jaison.json_write_string(black_box(api_value), indent_char = ""));
    });
} @bench

encode_records__jaison :: (b: *Bench) {
    prepare();
    ok: bool;
    ok, records_value = jaison.json_parse_string(records_text, [..] Record);
    over_value(b, jaison.json_write_string(records_value, indent_char = "").count, () {
        black_box(jaison.json_write_string(black_box(records_value), indent_char = ""));
    });
} @bench

encode_config__jaison :: (b: *Bench) {
    prepare();
    ok: bool;
    ok, config_value = jaison.json_parse_string(config_text, Node);
    over_value(b, jaison.json_write_string(config_value, indent_char = "").count, () {
        black_box(jaison.json_write_string(black_box(config_value), indent_char = ""));
    });
} @bench

roundtrip_api__jaison :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        ok, value := jaison.json_parse_string(text, [..] Issue);
        require(ok);
        black_box(jaison.json_write_string(value, indent_char = ""));
    });
} @bench

// ---------------------------------------------------------------------------------- ContiguousJsonJai

decode_api__contiguous :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        value, memory, ok := contig.json_parse(text, [..] Issue);
        black_box(value);
        require(ok);
    });
} @bench

decode_records__contiguous :: (b: *Bench) {
    prepare();
    over_text(b, records_text, (text: string) {
        value, memory, ok := contig.json_parse(text, [..] Record);
        black_box(value);
        require(ok);
    });
} @bench

decode_tree_api__contiguous :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        value, memory, ok := contig.json_parse(text, contig.JsonGeneric);
        black_box(value);
        require(ok);
    });
} @bench

decode_tree_records__contiguous :: (b: *Bench) {
    prepare();
    over_text(b, records_text, (text: string) {
        value, memory, ok := contig.json_parse(text, contig.JsonGeneric);
        black_box(value);
        require(ok);
    });
} @bench

encode_api__contiguous :: (b: *Bench) {
    prepare();
    memory: *u8;
    ok: bool;
    api_value, memory, ok = contig.json_parse(api_text, [..] Issue);
    over_value(b, contig.json_write(api_value).count, () {
        black_box(contig.json_write(black_box(api_value)));
    });
} @bench

encode_records__contiguous :: (b: *Bench) {
    prepare();
    memory: *u8;
    ok: bool;
    records_value, memory, ok = contig.json_parse(records_text, [..] Record);
    over_value(b, contig.json_write(records_value).count, () {
        black_box(contig.json_write(black_box(records_value)));
    });
} @bench

roundtrip_api__contiguous :: (b: *Bench) {
    prepare();
    over_text(b, api_text, (text: string) {
        value, memory, ok := contig.json_parse(text, [..] Issue);
        require(ok);
        black_box(contig.json_write(value));
    });
} @bench

// ---------------------------------------------------------------------------------- untyped configuration

decode_tree_config__jacket :: (b: *Bench) {
    prepare();
    over_text(b, config_text, (text: string) {
        value, error := from_json(Json_Value, text);
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_tree_config__jaison :: (b: *Bench) {
    prepare();
    over_text(b, config_text, (text: string) {
        ok, value := jaison.json_parse_string(text);
        black_box(value);
        require(ok);
    });
} @bench

decode_tree_config__contiguous :: (b: *Bench) {
    prepare();
    over_text(b, config_text, (text: string) {
        value, memory, ok := contig.json_parse(text, contig.JsonGeneric);
        black_box(value);
        require(ok);
    });
} @bench
