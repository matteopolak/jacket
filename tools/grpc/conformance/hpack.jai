// Exercise the owning HPACK codec on whole stateful stories from a JSON path manifest.
// Emit one JSON record per case for an independent decoder to check encoder output.
#load "../../../stdlib/Extensions/Http/module.jai";
#import "File";
J :: #import "Extensions/Json";

Record :: struct {
    story: string;
    index: int;
    decoded: bool;
    encoded: string;
    table_limit: int;
    detail: string;
}

unhex :: (text: string) -> [..] u8 {
    result: [..] u8;

    assert(text.count % 2 == 0);
    high: u8;

    for text {
        digit: u8;
        if it >= #char "0" && it <= #char "9" digit = it - #char "0";
        else if it >= #char "a" && it <= #char "f" digit = it - #char "a" + 10;
        else if it >= #char "A" && it <= #char "F" digit = it - #char "A" + 10;
        else assert(false, "invalid corpus hex");
        if it_index % 2 == 0 high = digit;
        else array_add(*result, cast(u8) (high * 16 + digit));
    }

    return result;
}

hex :: (data: [] u8) -> string {
    digits := "0123456789abcdef";
    builder: String_Builder;

    for data {
        append(*builder, digits[it >> 4]);
        append(*builder, digits[it & 15]);
    }

    return builder_to_string(*builder);
}

story :: (path: string) {
    text, read := read_entire_file(path);

    assert(read);
    defer free(text);
    doc, error := J.from_json(J.Json_Value, text);

    assert(error.kind == .NONE);
    defer J.free_json(*doc);
    cases := J.json_get(*doc, "cases");

    assert(cases && cases.kind == .ARRAY);
    decoder: Hpack_Decoder;
    encoder: Hpack_Encoder;

    hpack_decoder_init(*decoder);
    hpack_encoder_init(*encoder);
    defer hpack_decoder_free(*decoder);
    defer hpack_encoder_free(*encoder);

    for *case: cases.items {
        limit := J.json_get(case, "header_table_size");
        if limit && limit.kind != .NULL {
            size, valid := J.json_as_s64(limit);
            assert(valid && size >= 0);
            decoder.limit = size;
            hpack_encoder_limit(*encoder, size);
        }

        headers := J.json_get(case, "headers");

        assert(headers && headers.kind == .ARRAY);
        block: Hpack_Block;

        defer hpack_block_free(*block);
        wire := J.json_get(case, "wire");
        decoded := true;
        detail := "";

        if wire {
            bytes := unhex(wire.text);

            defer array_reset(*bytes);
            decode_error := hpack_decode(*decoder, bytes, 1 << 26, *block);

            decoded = decode_error == .NO_ERROR;
            if !decoded detail = tprint("decode error %", decode_error);

            decoded = decoded && !block.too_large && block.fields.count == headers.items.count;
            if decoded {
                for header: headers.items {
                    assert(header.kind == .OBJECT && header.members.count == 1);
                    field := header.members[0];
                    same := block.fields[it_index].name == field.key && block.fields[it_index].value == field.value.text;

                    if !same detail = tprint("field % expected %:% got %:%", it_index, field.key, field.value.text, block.fields[it_index].name, block.fields[it_index].value);

                    decoded = decoded && same;
                }
            }
        }

        encoded: [..] u8;

        defer array_reset(*encoded);
        hpack_encode_start(*encoder, *encoded);

        for header: headers.items {
            assert(header.kind == .OBJECT && header.members.count == 1);
            field := header.members[0];
            hpack_encode_field(*encoder, *encoded, field.key, field.value.text);
        }

        encoded_hex := hex(encoded);

        defer free(encoded_hex);
        record := Record.{ path, it_index, decoded, encoded_hex, decoder.limit, detail };
        output := J.to_json(record);

        defer free(output);
        print("%\n", output);
    }
}

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 2, "supply a JSON array of story paths");
    text, read := read_entire_file(args[1]);

    assert(read);
    defer free(text);
    paths, error := J.from_json([] string, text);

    assert(error.kind == .NONE);
    defer J.free_json(*paths);
    for paths story(it);
}
