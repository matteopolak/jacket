// HPACK (RFC 7541), the header compression of HTTP/2: an encoder and a decoder that each keep the
// dynamic table of one direction of a connection, and the Huffman code the strings use.
//
// ```jai,ignore
// encoder: Hpack_Encoder;
// decoder: Hpack_Decoder;
// hpack_encoder_init(*encoder);
// hpack_decoder_init(*decoder);
// defer hpack_encoder_free(*encoder);
// defer hpack_decoder_free(*decoder);
//
// out: [..] u8;
// defer array_reset(*out);
//
// hpack_encode_start(*encoder, *out);
// hpack_encode_field(*encoder, *out, ":method", "GET");
// hpack_encode_field(*encoder, *out, "x-token", "abc");
//
// block: Hpack_Block;
// defer hpack_block_free(*block);
//
// assert(hpack_decode(*decoder, out, 4096, *block) == .NO_ERROR);
// assert(block.fields.count == 2 && block.fields[1].value == "abc");
// ```

#scope_module

#load "hpack_huffman.jai";

// The most the encoder lets its table grow, whatever the peer allows.
HPACK_ENCODER_TABLE_LIMIT :: 4096;

// What each table entry costs besides its name and value (RFC 7541 section 4.1).
HPACK_ENTRY_OVERHEAD :: 32;

// The decoded fields of one header block. `fields` point into `storage`, so the block owns them until `hpack_block_free`. When the
// fields add up to more than the limit given to `hpack_decode`, `too_large` is set and `fields` stops growing (the table is still brought up to date).
Hpack_Block :: struct {
    fields: [..] Header_Field;
    storage: [..] u8;
    too_large: bool;
}

// Frees what a decoded block holds.
hpack_block_free :: (block: *Hpack_Block) {
    array_reset(*block.fields);
    array_reset(*block.storage);

    block.too_large = false;
}

// The state of the encoding side of one connection direction.
Hpack_Encoder :: struct {
    table: Hpack_Table;
    update_pending: bool;
    smallest: s64; // the smallest limit since the last block, which must be announced first
}

// The state of the decoding side of one connection direction.
Hpack_Decoder :: struct {
    table: Hpack_Table;
    limit: s64; // the table size we told the peer it may use
}

// Starts an encoder whose table holds `table_size` bytes at most (and never more than `HPACK_ENCODER_TABLE_LIMIT`).
hpack_encoder_init :: (encoder: *Hpack_Encoder, table_size := 4096) {
    encoder.* = .{};
    encoder.table.max_size = min(table_size, HPACK_ENCODER_TABLE_LIMIT);
}

// Releases the table.
hpack_encoder_free :: (encoder: *Hpack_Encoder) {
    hpack_table_free(*encoder.table);
}

// The peer allows a table of `size` bytes (its SETTINGS_HEADER_TABLE_SIZE): the next block starts by announcing the size in use.
hpack_encoder_limit :: (encoder: *Hpack_Encoder, size: s64) {
    wanted := min(size, HPACK_ENCODER_TABLE_LIMIT);

    if wanted == encoder.table.max_size return;

    if !encoder.update_pending || wanted < encoder.smallest encoder.smallest = wanted;

    encoder.update_pending = true;

    hpack_table_resize(*encoder.table, wanted);
}

// Starts a decoder; `limit` is the table size announced to the peer in SETTINGS_HEADER_TABLE_SIZE.
hpack_decoder_init :: (decoder: *Hpack_Decoder, limit := 4096) {
    decoder.* = .{};
    decoder.limit = limit;
    decoder.table.max_size = limit;
}

// Releases the table.
hpack_decoder_free :: (decoder: *Hpack_Decoder) {
    hpack_table_free(*decoder.table);
}

// Appends what has to open a header block: the table size changes the peer has not seen yet. Call it before the first field.
hpack_encode_start :: (encoder: *Hpack_Encoder, out: *[..] u8) {
    if !encoder.update_pending return;

    encoder.update_pending = false;

    if encoder.smallest < encoder.table.max_size hpack_put_integer(out, 5, 0x20, encoder.smallest);

    hpack_put_integer(out, 5, 0x20, encoder.table.max_size);
}

// Appends one header field. The name must be lowercase. Fields that carry credentials are never put in the table.
hpack_encode_field :: (encoder: *Hpack_Encoder, out: *[..] u8, name: string, value: string) {
    full, named := hpack_find_field(*encoder.table, name, value);

    if full > 0 {
        hpack_put_integer(out, 7, 0x80, full);

        return;
    }

    if hpack_is_sensitive(name) {
        hpack_put_integer(out, 4, 0x10, named);
        if named == 0 hpack_put_string(out, name);
        hpack_put_string(out, value);

        return;
    }

    entry_size := name.count + value.count + HPACK_ENTRY_OVERHEAD;

    // An entry the table cannot hold would only empty it.
    if entry_size > encoder.table.max_size {
        hpack_put_integer(out, 4, 0, named);
        if named == 0 hpack_put_string(out, name);
        hpack_put_string(out, value);

        return;
    }

    hpack_put_integer(out, 6, 0x40, named);
    if named == 0 hpack_put_string(out, name);

    hpack_put_string(out, value);

    hpack_table_add(*encoder.table, name, value);
}

// Decodes a whole header block into `result` (which must be empty). `max_list_size` bounds the size of the fields, counted as in
// SETTINGS_MAX_HEADER_LIST_SIZE. Anything but `.NO_ERROR` is a connection error (`.COMPRESSION_ERROR`) and ends the connection, since the table
// is lost.
hpack_decode :: (decoder: *Hpack_Decoder, block: [] u8, max_list_size: s64, result: *Hpack_Block) -> H2_Error {
    Span :: struct {
        name_at: int;
        name_count: int;
        value_at: int;
        value_count: int;
    }

    spans: [..] Span;

    defer array_reset(*spans);

    list_size := 0;
    at := 0;
    fields_started := false;

    while at < block.count {
        first := block[at];
        name: string;
        value: string;
        name_scratch: [..] u8; // Huffman strings are decoded here
        value_scratch: [..] u8;

        defer array_reset(*name_scratch);
        defer array_reset(*value_scratch);

        add_to_table := false;

        if (first & 0x80) != 0 {
            index, ok := hpack_read_integer(block, *at, 7);

            if !ok || index == 0 return .COMPRESSION_ERROR;

            entry, found := hpack_table_lookup(*decoder.table, index);

            if !found return .COMPRESSION_ERROR;

            name = entry.name;
            value = entry.value;
            fields_started = true;
        } else if (first & 0xe0) == 0x20 {
            // The size of the table: only before the first field of a block.
            size, ok := hpack_read_integer(block, *at, 5);

            if !ok || fields_started || size > decoder.limit return .COMPRESSION_ERROR;

            hpack_table_resize(*decoder.table, size);

            continue;
        } else {
            prefix := 4;

            if (first & 0xc0) == 0x40 {
                prefix = 6;
                add_to_table = true;
            }

            index, ok := hpack_read_integer(block, *at, prefix);

            if !ok return .COMPRESSION_ERROR;

            if index > 0 {
                entry, found := hpack_table_lookup(*decoder.table, index);

                if !found return .COMPRESSION_ERROR;

                name = entry.name;
            } else {
                text, text_ok := hpack_read_string(block, *at, *name_scratch, max_list_size);

                if !text_ok return .COMPRESSION_ERROR;

                name = text;
            }

            text, text_ok := hpack_read_string(block, *at, *value_scratch, max_list_size);

            if !text_ok return .COMPRESSION_ERROR;

            value = text;
            fields_started = true;
        }

        if add_to_table hpack_table_add(*decoder.table, name, value);

        list_size += name.count + value.count + HPACK_ENTRY_OVERHEAD;

        if max_list_size >= 0 && list_size > max_list_size result.too_large = true;

        if result.too_large continue;

        span: Span;

        span.name_at = result.storage.count;
        span.name_count = name.count;

        append_bytes(*result.storage, name.data, name.count);

        span.value_at = result.storage.count;
        span.value_count = value.count;

        append_bytes(*result.storage, value.data, value.count);
        array_add(*spans, span);
    }

    // Now that the storage has stopped growing, the fields can point into it.
    for spans {
        field: Header_Field;

        field.name = .{ it.name_count, result.storage.data + it.name_at };
        field.value = .{ it.value_count, result.storage.data + it.value_at };
        field.id = lookup_header_name(field.name);

        array_add(*result.fields, field);
    }

    return .NO_ERROR;
}

#scope_module

// ---- The tables

HPACK_STATIC_NAMES :: string.[
    ":authority", ":method", ":method", ":path", ":path", ":scheme", ":scheme", ":status", ":status", ":status",
    ":status", ":status", ":status", ":status", "accept-charset", "accept-encoding", "accept-language", "accept-ranges", "accept", "access-control-allow-origin",
    "age", "allow", "authorization", "cache-control", "content-disposition", "content-encoding", "content-language", "content-length", "content-location", "content-range",
    "content-type", "cookie", "date", "etag", "expect", "expires", "from", "host", "if-match", "if-modified-since",
    "if-none-match", "if-range", "if-unmodified-since", "last-modified", "link", "location", "max-forwards", "proxy-authenticate", "proxy-authorization", "range",
    "referer", "refresh", "retry-after", "server", "set-cookie", "strict-transport-security", "transfer-encoding", "user-agent", "vary", "via",
    "www-authenticate",
];

HPACK_STATIC_VALUES :: string.[
    "", "GET", "POST", "/", "/index.html", "http", "https", "200", "204", "206",
    "304", "400", "404", "500", "", "gzip, deflate", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "",
    "",
];

HPACK_STATIC_COUNT :: 61;

// One field in the dynamic table; the name and value share an allocation that starts at `name.data`.
Hpack_Entry :: struct {
    name: string;
    value: string;
}

// The dynamic table: a ring of entries, newest last, that is kept under `max_size` bytes.
Hpack_Table :: struct {
    entries: [] Hpack_Entry;
    next: int; // where the next entry goes
    count: int;
    size: int;
    max_size: s64;
}

HPACK_TABLE_FIRST_CAPACITY :: 16;

hpack_table_free :: (table: *Hpack_Table) {
    age := 0;

    while age < table.count {
        hpack_table_free_entry(hpack_table_at(table, age));

        age += 1;
    }

    array_free(table.entries);

    table.* = .{};
}

hpack_table_free_entry :: (entry: Hpack_Entry) {
    free(entry.name.data);
}

// The entry `age` places from the newest (0 is the newest).
hpack_table_at :: (table: *Hpack_Table, age: int) -> Hpack_Entry {
    capacity := table.entries.count;

    return table.entries[(table.next - 1 - age + 2 * capacity) % capacity];
}

hpack_table_evict_oldest :: (table: *Hpack_Table) {
    capacity := table.entries.count;
    slot := (table.next - table.count + capacity) % capacity;
    entry := table.entries[slot];

    table.size -= entry.name.count + entry.value.count + HPACK_ENTRY_OVERHEAD;
    table.count -= 1;

    hpack_table_free_entry(entry);
}

// Changes the size limit, dropping the oldest entries that no longer fit.
hpack_table_resize :: (table: *Hpack_Table, max_size: s64) {
    table.max_size = max_size;

    while table.count > 0 && table.size > table.max_size hpack_table_evict_oldest(table);
}

hpack_table_add :: (table: *Hpack_Table, name: string, value: string) {
    size := name.count + value.count + HPACK_ENTRY_OVERHEAD;

    // `name` and `value` may belong to an entry the eviction below frees: copy them first.
    memory := cast(*u8) alloc(name.count + value.count + 1);

    memcpy(memory, name.data, name.count);
    memcpy(memory + name.count, value.data, value.count);

    while table.count > 0 && table.size + size > table.max_size hpack_table_evict_oldest(table);

    // An entry larger than the whole table empties it and is not added.
    if size > table.max_size {
        free(memory);

        return;
    }

    capacity := table.entries.count;

    if table.count == capacity {
        grown := NewArray(max(capacity * 2, HPACK_TABLE_FIRST_CAPACITY), Hpack_Entry);
        age := 0;

        while age < table.count {
            grown[table.count - 1 - age] = hpack_table_at(table, age);

            age += 1;
        }

        array_free(table.entries);

        table.entries = grown;
        table.next = table.count;
    }

    entry: Hpack_Entry;

    entry.name = .{ name.count, memory };
    entry.value = .{ value.count, memory + name.count };

    table.entries[table.next] = entry;
    table.next = (table.next + 1) % table.entries.count;
    table.count += 1;
    table.size += size;
}

// The field at a 1-based index of the combined static and dynamic table.
hpack_table_lookup :: (table: *Hpack_Table, index: int) -> Hpack_Entry, bool {
    if index < 1 return .{}, false;

    if index <= HPACK_STATIC_COUNT return .{ HPACK_STATIC_NAMES[index - 1], HPACK_STATIC_VALUES[index - 1] }, true;

    age := index - HPACK_STATIC_COUNT - 1;

    if age >= table.count return .{}, false;

    return hpack_table_at(table, age), true;
}

// The index of a field that matches in name and value, and of one that matches in name (0: none).
hpack_find_field :: (table: *Hpack_Table, name: string, value: string) -> full: int, named: int {
    named := 0;

    for HPACK_STATIC_NAMES {
        if it != name continue;

        if named == 0 named = it_index + 1;

        if HPACK_STATIC_VALUES[it_index] == value return it_index + 1, named;
    }

    age := 0;

    while age < table.count {
        entry := hpack_table_at(table, age);

        if entry.name == name {
            if named == 0 named = HPACK_STATIC_COUNT + 1 + age;

            if entry.value == value return HPACK_STATIC_COUNT + 1 + age, named;
        }

        age += 1;
    }

    return 0, named;
}

// Fields whose values must not end up in a shared table (RFC 7541 section 7.1.3).
hpack_is_sensitive :: (name: string) -> bool {
    return name == "authorization" || name == "proxy-authorization" || name == "cookie" || name == "set-cookie";
}

// ---- Integers and strings

// Appends `value` as an integer with an N-bit prefix; `flags` are the bits of the first byte above the prefix.
hpack_put_integer :: (out: *[..] u8, prefix: int, flags: u8, value: s64) {
    limit := (1 << prefix) - 1;

    if value < limit {
        byte := flags | cast(u8) value;

        append_bytes(out, *byte, 1);

        return;
    }

    byte := flags | cast(u8) limit;

    append_bytes(out, *byte, 1);

    rest := value - limit;

    while rest >= 128 {
        byte = cast(u8) ((rest & 0x7f) | 0x80);

        append_bytes(out, *byte, 1);

        rest >>= 7;
    }

    byte = cast(u8) rest;

    append_bytes(out, *byte, 1);
}

// Reads an integer with an N-bit prefix at `at`. Fails when the block ends first or the number does not fit 31 bits.
hpack_read_integer :: (block: [] u8, at: *int, prefix: int) -> s64, bool {
    if at.* >= block.count return 0, false;

    limit := (1 << prefix) - 1;
    value := cast(s64) (block[at.*] & cast(u8) limit);

    at.* += 1;

    if value < limit return value, true;

    shift := 0;

    while true {
        if at.* >= block.count return 0, false;

        byte := block[at.*];

        at.* += 1;

        // Five bytes of seven bits cover 31 bits.
        if shift > 28 return 0, false;

        value += (cast(s64) (byte & 0x7f)) << shift;
        shift += 7;

        if (byte & 0x80) == 0 break;
    }

    if value > H2_LARGEST_WINDOW return 0, false;

    return value, true;
}

// Appends a string, Huffman coded when that is shorter.
hpack_put_string :: (out: *[..] u8, text: string) {
    coded := hpack_huffman_size(text);

    if coded < text.count {
        hpack_put_integer(out, 7, 0x80, coded);
        hpack_huffman_encode(out, text);

        return;
    }

    hpack_put_integer(out, 7, 0, text.count);
    append_bytes(out, text.data, text.count);
}

// Reads a string at `at` into `scratch` (when Huffman coded) or in place; fails past the block, on bad Huffman data, or over `limit` bytes.
hpack_read_string :: (block: [] u8, at: *int, scratch: *[..] u8, limit: s64) -> string, bool {
    if at.* >= block.count return "", false;

    coded := (block[at.*] & 0x80) != 0;
    count, ok := hpack_read_integer(block, at, 7);

    if !ok || count > block.count - at.* return "", false;

    // The limit bounds what a hostile block can make us hold.
    if limit >= 0 && count > limit return "", false;

    bytes: [] u8 = .{ count, block.data + at.* };

    at.* += count;

    if !coded return .{ count, bytes.data }, true;

    scratch.count = 0;

    if !hpack_huffman_decode(bytes, scratch) return "", false;

    return .{ scratch.count, scratch.data }, true;
}

// ---- Huffman

// The bytes the Huffman coding of `text` takes.
hpack_huffman_size :: (text: string) -> int {
    bits := 0;

    for text bits += HPACK_HUFFMAN_LENGTHS[it];

    return (bits + 7) / 8;
}

// Appends the Huffman coding of `text`, the last byte padded with ones (a prefix of the end-of-string code).
hpack_huffman_encode :: (out: *[..] u8, text: string) {
    buffer: u64;
    held := 0;

    for text {
        length := HPACK_HUFFMAN_LENGTHS[it];

        buffer = (buffer << length) | HPACK_HUFFMAN_CODES[it];
        held += length;

        while held >= 8 {
            byte := cast(u8) ((buffer >> (held - 8)) & 0xff);

            append_bytes(out, *byte, 1);

            held -= 8;
        }
    }

    if held > 0 {
        byte := cast(u8) (((buffer << (8 - held)) | ((1 << (8 - held)) - 1)) & 0xff);

        append_bytes(out, *byte, 1);
    }
}

#scope_module

// The codes grouped by length, to decode bit by bit: how many codes each length has, the first
// code of each length, and the symbols in code order.
Hpack_Huffman_Decoder :: struct {
    count: [31] s32;
    first: [31] s32;
    start: [31] s32;
    symbols: [257] u16;
}

hpack_make_huffman_decoder :: () -> Hpack_Huffman_Decoder {
    decoder: Hpack_Huffman_Decoder;

    for HPACK_HUFFMAN_LENGTHS decoder.count[it] += 1;

    code := 0;
    index := 0;

    for length: 1..30 {
        decoder.first[length] = cast(s32) code;
        decoder.start[length] = cast(s32) index;

        code = (code + decoder.count[length]) << 1;
        index += decoder.count[length];
    }

    // Within one length the codes follow the order of the symbols.
    next: [31] s32;

    for symbol: 0..256 {
        length := HPACK_HUFFMAN_LENGTHS[symbol];

        decoder.symbols[decoder.start[length] + next[length]] = cast(u16) symbol;
        next[length] += 1;
    }

    return decoder;
}

hpack_huffman_decoder :: #run hpack_make_huffman_decoder();

// Appends the bytes `data` codes. False for the end-of-string symbol inside the data, for padding longer than seven bits and for
// padding that is not ones.
hpack_huffman_decode :: (data: [] u8, out: *[..] u8) -> bool {
    code := 0;
    length := 0;

    for byte: data {
        for bit: 0..7 {
            code = (code << 1) | (((cast(int) byte) >> (7 - bit)) & 1);
            length += 1;

            if length > 30 return false;

            count := hpack_huffman_decoder.count[length];
            offset := code - hpack_huffman_decoder.first[length];

            if count > 0 && offset >= 0 && offset < count {
                symbol := hpack_huffman_decoder.symbols[hpack_huffman_decoder.start[length] + offset];

                if symbol == 256 return false;

                value := cast(u8) symbol;

                append_bytes(out, *value, 1);

                code = 0;
                length = 0;
            }
        }
    }

    if length > 7 return false;

    return code == (1 << length) - 1;
}
