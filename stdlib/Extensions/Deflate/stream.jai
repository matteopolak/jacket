#scope_module

// Decodes one dynamic block header into the tables.
read_dynamic_tables :: (s: *Inflater) -> string {
    if !need_bits(s, 14) return "unexpected end of data";

    literal_count := cast(s64) take_bits(s, 5) + 257;
    distance_count := cast(s64) take_bits(s, 5) + 1;
    code_length_count := cast(s64) take_bits(s, 4) + 4;

    if literal_count > 286 || distance_count > 30 return "too many length or distance codes";

    code_length_lengths: [19] u8;

    for i: 0..code_length_count - 1 {
        if !need_bits(s, 3) return "unexpected end of data";
        code_length_lengths[CODE_LENGTH_ORDER[i]] = cast(u8) take_bits(s, 3);
    }

    // The code length code has at most 7 bits: a 128-entry table of (symbol, length) is enough.
    code_table: [128] u32;
    counts: [8] s32;

    for i: 0..18 counts[code_length_lengths[i]] += 1;

    counts[0] = 0;

    left: s32 = 1;

    for length: 1..7 {
        left = (left << 1) - counts[length];
        if left < 0 return "over-subscribed code length code";
    }

    if left != 0 return "incomplete code length code";

    first: [8] u32;
    code: u32;

    for length: 1..7 {
        code = (code + cast(u32) counts[length - 1]) << 1;
        first[length] = code;
    }

    for symbol: 0..18 {
        length := cast(u32) code_length_lengths[symbol];
        if length == 0 continue;

        reversed := reverse_bits(first[length], length);
        symbol32 := cast(u32) symbol;

        first[length] += 1;

        index := reversed;

        while index < 128 {
            code_table[index] = symbol32 | (length << 8);
            index += 1 << length;
        }
    }

    lengths: [320] u8;
    total := literal_count + distance_count;
    filled: s64;

    while filled < total {
        if !need_bits(s, 7) {
            // Near the end of the input fewer bits may remain; the symbol itself decides.
            if s.nbits == 0 return "unexpected end of data";
        }

        entry := code_table[s.bits & 127];
        length := cast(s64) (entry >> 8);

        if length == 0 || length > s.nbits return "unexpected end of data";

        symbol := entry & 255;

        s.bits >>= length;
        s.nbits -= length;

        if symbol < 16 {
            lengths[filled] = cast(u8) symbol;
            filled += 1;
            continue;
        }

        repeat: s64;
        value: u8;

        if symbol == 16 {
            if filled == 0 return "repeat with no previous length";
            if !need_bits(s, 2) return "unexpected end of data";

            value = lengths[filled - 1];
            repeat = 3 + cast(s64) take_bits(s, 2);
        } else if symbol == 17 {
            if !need_bits(s, 3) return "unexpected end of data";
            repeat = 3 + cast(s64) take_bits(s, 3);
        } else {
            if !need_bits(s, 7) return "unexpected end of data";
            repeat = 11 + cast(s64) take_bits(s, 7);
        }

        if filled + repeat > total return "code lengths overflow";

        for i: 0..repeat - 1 lengths[filled + i] = value;

        filled += repeat;
    }

    if lengths[256] == 0 return "missing end-of-block code";

    if !build_table(s.literal_table.data, LITERAL_TABLE_SIZE, lengths.data, literal_count, LITERAL_ROOT, false) return "invalid literal/length code lengths";
    if !build_table(s.distance_table.data, DISTANCE_TABLE_SIZE, lengths.data + literal_count, distance_count, DISTANCE_ROOT, true) return "invalid distance code lengths";

    s.fixed_loaded = false;

    return "";
}

inflate_blocks :: (s: *Inflater) -> string {
    while true {
        if !need_bits(s, 3) return "unexpected end of data";

        final := take_bits(s, 1);
        kind := take_bits(s, 2);

        if kind == 0 {
            // Drop the rest of the byte, then LEN and NLEN come from the byte stream.
            drop := s.nbits & 7;

            s.bits >>= drop;
            s.nbits -= drop;
            s.ip -= s.nbits >> 3;
            s.bits = 0;
            s.nbits = 0;

            if s.ip + 4 > s.size return "unexpected end of data";

            length := cast(s64) le16(s.data + s.ip);
            complement := cast(s64) le16(s.data + s.ip + 2);

            s.ip += 4;

            if length != (~complement & 0xFFFF) return "stored block length check failed";
            if s.ip + length > s.size return "unexpected end of data";

            error := grow_output(s, s.o + length);
            if error.count return error;

            if length memcpy(s.out + s.o, s.data + s.ip, length);

            s.ip += length;
            s.o += length;
        } else if kind == 1 {
            load_fixed_tables(s);

            error := inflate_codes(s);
            if error.count return error;
        } else if kind == 2 {
            error := read_dynamic_tables(s);
            if error.count return error;

            error = inflate_codes(s);
            if error.count return error;
        } else {
            return "invalid block type";
        }

        if final break;
    }

    return "";
}

// The byte position after the last whole byte the decoder has used.
stream_position :: (s: *Inflater) -> s64 {
    return s.ip - (s.nbits >> 3);
}

// Parses the gzip header; returns the offset of the DEFLATE data, or -1 with `error` set.
parse_gzip_header :: (data: [] u8) -> s64, string {
    if data.count < 18 return -1, "unexpected end of data";
    if data[0] != 0x1F || data[1] != 0x8B return -1, "bad gzip magic";
    if data[2] != 8 return -1, "unsupported gzip compression method";

    flags := data[3];
    if flags & 0xE0 return -1, "bad gzip flags";

    at: s64 = 10;

    if flags & 4 {
        if at + 2 > data.count return -1, "unexpected end of data";

        at += 2 + cast(s64) le16(data.data + at);
        if at > data.count return -1, "unexpected end of data";
    }

    for 0..1 {
        mask: u8 = 8;
        if it == 1 mask = 16;
        if !(flags & mask) continue;

        while at < data.count && data[at] != 0 at += 1;
        if at >= data.count return -1, "unexpected end of data";

        at += 1;
    }

    if flags & 2 {
        if at + 2 > data.count return -1, "unexpected end of data";

        header: [] u8;
        header.data = data.data;
        header.count = at;

        if le16(data.data + at) != (crc32_update(header, 0) & 0xFFFF) return -1, "bad gzip header checksum";

        at += 2;
    }

    return at, "";
}

// Runs a whole stream, container included, into `s`.
inflate_container :: (s: *Inflater, input: [] u8, container: Deflate_Container) -> string {
    s.data = input.data;
    s.size = input.count;

    if container == .ZLIB {
        if input.count < 2 return "unexpected end of data";

        cmf := cast(u32) input[0];
        flg := cast(u32) input[1];

        if (cmf * 256 + flg) % 31 != 0 return "bad zlib header";
        if (cmf & 15) != 8 return "unsupported zlib compression method";
        if (cmf >> 4) > 7 return "bad zlib window size";
        if flg & 0x20 return "zlib preset dictionaries are not supported";

        s.ip = 2;
    } else if container == .GZIP {
        at, error := parse_gzip_header(input);
        if at < 0 return error;

        s.ip = at;
    }

    error := inflate_blocks(s);
    if error.count return error;

    end := stream_position(s);
    result: [] u8;
    result.data = s.out;
    result.count = s.o;

    if container == .ZLIB {
        if end + 4 > input.count return "unexpected end of data";

        if be32(input.data + end) != adler32_update(result, 1) return "adler32 checksum mismatch";
    } else if container == .GZIP {
        if end + 8 > input.count return "unexpected end of data";

        if le32(input.data + end) != crc32_update(result, 0) return "crc32 checksum mismatch";
        if le32(input.data + end + 4) != cast(u32) (s.o & 0xFFFFFFFF) return "gzip size mismatch";
    }

    return "";
}
