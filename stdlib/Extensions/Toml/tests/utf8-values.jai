#import "Basic";
#import "Extensions/Toml";

main :: () {
    Case :: struct {
        bytes: [] u8;
        offset: int;
        message: string;
    }

    for Case.[
        .{ u8.[0x80], 4, "invalid UTF-8" },
        .{ u8.[0xe2, 0x82], 4, "truncated UTF-8" },
        .{ u8.[0xe2, 0x28, 0xa1], 5, "invalid UTF-8 continuation" },
        .{ u8.[0xe0, 0x80, 0x80], 4, "invalid Unicode scalar" },
        .{ u8.[0xed, 0xa0, 0x80], 4, "invalid Unicode scalar" },
        .{ u8.[0xf4, 0x90, 0x80, 0x80], 4, "invalid Unicode scalar" },
        .{ u8.[0xc0, 0xaf], 4, "invalid UTF-8" },
    ] {
        bad := tprint("a£\n%", string.{ it.bytes.count, it.bytes.data });
        parsed, error := parse_toml(bad);

        assert(error.kind == .ENCODING && error.span.start == it.offset && error.message == it.message);
        assert(error.line == 2 && error.column == it.offset - 3);
        free_toml(*parsed);

        doc, _ := parse_toml("x='kept'");
        value := toml_get(doc.root, "x");
        retained := doc.retained_bytes;
        set_error := toml_set_text(*doc, value, bad);

        assert(set_error.kind == .ENCODING && set_error.span.start == it.offset && set_error.message == it.message);
        assert(set_error.line == 2 && set_error.column == it.offset - 3 && toml_error_path(*set_error) == "$");
        assert(value.text == "kept" && doc.retained_bytes == retained);
        detached, _ := toml_new_value(*doc, .INTEGER);
        insert_error := toml_insert(*doc, doc.root, bad, detached);

        assert(insert_error.kind == .ENCODING && insert_error.span.start == it.offset && insert_error.message == it.message);
        free_toml(*doc);

        root: Toml_Value;
        raw: Toml_Value;
        raw.kind = .STRING;
        raw.text = bad;
        array_add(*root.members, Toml_Member.{ key = "x", value = *raw });
        _, write_error := write_toml(*root);

        assert(write_error.kind == .OUTPUT && write_error.message == "string is not valid UTF-8");
        assert(toml_error_path(*write_error) == "$[\"x\"]");
        array_free(root.members);
    }

    print("ok\n");
}
