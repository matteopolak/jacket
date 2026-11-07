#import "Basic";
#import "String";
#import "Extensions/Toml";

// The private index must stay balanced and searchable across sorted and reverse insertion.
// Public iteration order remains source/insertion order rather than index order.
main :: () {
    for reverse: 0..1 {
        doc, error := new_toml_document();

        assert(error.kind == .NONE);

        for 0..1023 {
            number := ifx reverse then 1023 - it else it;
            value, value_error := toml_new_value(*doc, .INTEGER);

            assert(value_error.kind == .NONE);
            value.integer = number;
            assert(toml_insert(*doc, doc.root, index_key(number), value).kind == .NONE);
        }

        index := doc.root.internal_lookup;

        assert(index && index.nodes.count == doc.root.members.count);

        for index.nodes {
            left := ifx it.left >= 0 then index.nodes[it.left].height else 0;
            right := ifx it.right >= 0 then index.nodes[it.right].height else 0;
            assert(left - right <= 1 && right - left <= 1);
            assert(it.height == max(left, right) + 1);
        }

        assert(index.nodes[index.root].height < 20);

        for 0..1023 {
            node := toml_get(doc.root, index_key(it));
            assert(node && node.integer == it);
        }

        assert(doc.root.members[0].value.integer == ifx reverse then 1023 else 0);
        assert(toml_get(doc.root, "missing") == null);
        free_toml(*doc);
    }

    check_parse_insertions();
    print("ok\n");
}

index_key :: (number: int) -> string {
    text := tprint("%", number);
    return tprint("shared-prefix-%%", slice("0000", 0, 4 - text.count), text);
}

// A value may grow the global node/provenance arrays and its own indices between the
// outer key's duplicate check and attachment. The outer lookup must remain correct.
check_parse_insertions :: () {
    for count: int.[15, 16, 17, 31, 32, 33, 63, 64, 65] {
        for order: 0..2 {
            builder: String_Builder;

            for i: 0..count - 1 {
                number := i;
                if order == 1 number = count - 1 - i;
                if order == 2 number = ifx i % 2 == 0 then i / 2 else count - 1 - i / 2;
                print_to_builder(*builder, "\"%\" = %\n", index_key(number), formatInt(number));
            }

            insertion_at := builder_string_length(*builder);

            append(*builder, "inserted = {nested = {");

            for 0..79 {
                if it append(*builder, ", ");
                print_to_builder(*builder, "field% = [{v = %}, %]", formatInt(it), formatInt(it), formatInt(it + 1));
            }

            append(*builder, "}}\n");
            text := builder_to_string(*builder);

            free_buffers(*builder);
            doc, error := parse_toml(text, .{ record_statements = true });

            assert(error.kind == .NONE, error.message);
            assert(doc.root.members.count == count + 1);
            index := doc.root.internal_lookup;

            assert(index && index.nodes.count == count + 1);

            for index.nodes {
                left := ifx it.left >= 0 then index.nodes[it.left].height else 0;
                right := ifx it.right >= 0 then index.nodes[it.right].height else 0;
                assert(left - right <= 1 && right - left <= 1);
                assert(it.height == max(left, right) + 1);
            }

            for 0..count - 1 assert(toml_get(doc.root, index_key(it)).integer == it);

            nested := toml_get(toml_get(doc.root, "inserted"), "nested");

            assert(toml_get(nested, "field79").items[1].integer == 80);
            free_toml(*doc);
            duplicate, duplicate_error := parse_toml(tprint("%inserted = [broken", text));

            assert(duplicate_error.kind == .DUPLICATE && duplicate.root == null);
            assert(duplicate_error.span.start == text.count && duplicate_error.related.start == insertion_at);
            assert(toml_error_path(*duplicate_error) == "$[\"inserted\"]");
            free(text);
        }
    }
}
