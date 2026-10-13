// Decoding of the chunked transfer coding (RFC 9112 7.1): chunk sizes, extensions that are
// checked and dropped, chunk data handed out in place, and trailers.

#scope_module

chunk_fail :: (p: *Parser, pos: int, kind: Error_Kind, message: string) -> int, Event {
    return pos, fail(p, kind, message);
}

parse_chunked :: (p: *Parser, data: [] u8) -> consumed: int, event: Event {
    event: Event;
    pos := 0;

    while pos < data.count || p.state == .TRAILERS {
        if p.state == {
            case .CHUNK_SIZE;
                c := data[pos];

                pos += 1;
                digit := hex_value(c);

                if digit >= 0 {
                    p.chunk_digits += 1;

                    if p.chunk_digits > 64 return chunk_fail(p, pos, .BAD_CHUNK, "chunk size has too many digits");

                    if p.chunk_size > (0x7fff_ffff_ffff_ffff - digit) / 16 {
                        return chunk_fail(p, pos, .CHUNK_SIZE_OVERFLOW, "chunk size does not fit 63 bits");
                    }

                    p.chunk_size = p.chunk_size * 16 + digit;
                } else if p.chunk_digits == 0 {
                    return chunk_fail(p, pos, .BAD_CHUNK, "a chunk starts with its size in hex");
                } else if c == #char "\r" {
                    p.state = .CHUNK_SIZE_LF;
                } else if c == #char ";" {
                    p.state = .CHUNK_EXT;
                    p.ext = .NAME_START;
                    p.chunk_ext = 1;
                    p.ext_token = 0;
                } else if is_ows(c) {
                    p.state = .CHUNK_EXT;
                    p.ext = .BEFORE_SEMI;
                    p.chunk_ext = 1;
                } else {
                    return chunk_fail(p, pos, .BAD_CHUNK, "a chunk size is followed by CRLF or an extension");
                }

            case .CHUNK_EXT;
                c := data[pos];

                pos += 1;
                p.chunk_ext += 1;

                if p.chunk_ext > p.limits.max_chunk_extension {
                    return chunk_fail(p, pos, .BAD_CHUNK, "a chunk extension is over the length limit");
                }

                if !extension_step(p, c) {
                    return chunk_fail(p, pos, .BAD_CHUNK, "a chunk extension is not name[=value];...");
                }

            case .CHUNK_SIZE_LF;
                c := data[pos];

                pos += 1;

                if c != #char "\n" return chunk_fail(p, pos, .BAD_CHUNK, "a chunk line must end in CRLF");

                if p.chunk_size == 0 {
                    p.state = .TRAILERS;
                    p.tline_start = 0;
                    p.tbytes.count = 0;
                    p.tspans.count = 0;
                    p.tlist.count = 0;
                } else {
                    if p.limits.max_body_size >= 0 && p.chunk_size > p.limits.max_body_size - p.total {
                        return chunk_fail(p, pos, .BODY_TOO_LARGE, "body is over the size limit");
                    }

                    p.remaining = p.chunk_size;
                    p.state = .CHUNK_DATA;
                }

            case .CHUNK_DATA;
                n := data.count - pos;
                if n > p.remaining n = cast(int) p.remaining;

                event.kind = .BODY;
                event.data.count = n;
                event.data.data = data.data + pos;

                p.remaining -= n;
                p.total += n;
                pos += n;

                if p.remaining == 0 p.state = .CHUNK_DATA_CR;

                return pos, event;

            case .CHUNK_DATA_CR;
                c := data[pos];

                pos += 1;

                if c != #char "\r" return chunk_fail(p, pos, .BAD_CHUNK, "chunk data must be followed by CRLF");

                p.state = .CHUNK_DATA_LF;

            case .CHUNK_DATA_LF;
                c := data[pos];

                pos += 1;

                if c != #char "\n" return chunk_fail(p, pos, .BAD_CHUNK, "chunk data must be followed by CRLF");

                p.state = .CHUNK_SIZE;
                p.chunk_size = 0;
                p.chunk_digits = 0;

            case .TRAILERS;
                if pos >= data.count break;

                status := collect_line(p, data, *pos, *p.tbytes, p.tline_start, .HEADERS_TOO_LARGE, p.limits.max_trailer_size);

                if status < 0 {
                    event.kind = .ERROR;

                    return pos, event;
                }

                if status == 0 continue;

                count := p.tbytes.count;

                if count - p.tline_start < 2 || p.tbytes[count - 2] != #char "\r" {
                    return chunk_fail(p, pos, .BAD_LINE_ENDING, "a line must end in CRLF");
                }

                start := p.tline_start;
                end := count - 2;

                if start == end {
                    base := p.tbytes.data;

                    for span: p.tspans {
                        header: Header_Field;
                        header.name = view(base, span.name_start, span.name_end);
                        header.value = view(base, span.value_start, span.value_end);
                        header.id = lookup_header_name(header.name);

                        array_add(*p.tlist, header);
                    }

                    p.trailers = p.tlist;

                    return pos, finish_message(p);
                }

                if p.tspans.count >= p.limits.max_headers {
                    return chunk_fail(p, pos, .TOO_MANY_HEADERS, "too many trailer lines");
                }

                span: Span;

                if !parse_field_line(p, p.tbytes.data, start, end, *span, true) {
                    event.kind = .ERROR;

                    return pos, event;
                }

                name := view(p.tbytes.data, span.name_start, span.name_end);

                if equal_nocase(name, "content-length") || equal_nocase(name, "transfer-encoding") || equal_nocase(name, "host") {
                    return chunk_fail(p, pos, .BAD_TRAILER, "a trailer may not carry framing or routing fields");
                }

                array_add(*p.tspans, span);
                p.tline_start = count;

            case;
                // Not a chunk state.
                break;
        }
    }

    event.kind = .NEED_MORE;

    return data.count, event;
}

// One byte of a chunk extension. Moves to CHUNK_SIZE_LF at the CR. False when the byte does not
// belong where it is.
extension_step :: (p: *Parser, c: u8) -> bool {
    if p.ext == {
        case .BEFORE_SEMI;
            if is_ows(c) return true;

            if c == #char ";" {
                p.ext = .NAME_START;
                p.ext_token = 0;

                return true;
            }

        case .NAME_START;
            if is_ows(c) && p.ext_token == 0 return true;

            if is_token_char(c) {
                p.ext = .NAME;
                p.ext_token = 1;

                return true;
            }

        case .NAME;
            if is_token_char(c) return true;

            if is_ows(c) {
                p.ext = .AFTER_NAME;

                return true;
            }

            if c == #char "=" {
                p.ext = .VALUE_START;

                return true;
            }

            if c == #char ";" {
                p.ext = .NAME_START;
                p.ext_token = 0;

                return true;
            }

        case .AFTER_NAME;
            if is_ows(c) return true;

            if c == #char "=" {
                p.ext = .VALUE_START;

                return true;
            }

            if c == #char ";" {
                p.ext = .NAME_START;
                p.ext_token = 0;

                return true;
            }

        case .VALUE_START;
            if is_ows(c) return true;

            if c == #char "\"" {
                p.ext = .QUOTED;

                return true;
            }

            if is_token_char(c) {
                p.ext = .TOKEN_VALUE;

                return true;
            }

        case .TOKEN_VALUE;
            if is_token_char(c) return true;

            if is_ows(c) {
                p.ext = .AFTER_VALUE;

                return true;
            }

            if c == #char ";" {
                p.ext = .NAME_START;
                p.ext_token = 0;

                return true;
            }

        case .QUOTED;
            if c == #char "\"" {
                p.ext = .AFTER_QUOTED;

                return true;
            }

            if c == #char "\\" {
                p.ext = .QUOTED_PAIR;

                return true;
            }

            if c == 9 || (c >= 0x20 && c != 0x7f) return true;

        case .QUOTED_PAIR;
            if c == 9 || (c >= 0x20 && c != 0x7f) {
                p.ext = .QUOTED;

                return true;
            }

        case .AFTER_QUOTED;
            if is_ows(c) {
                p.ext = .AFTER_VALUE;

                return true;
            }

            if c == #char ";" {
                p.ext = .NAME_START;
                p.ext_token = 0;

                return true;
            }

        case .AFTER_VALUE;
            if is_ows(c) return true;

            if c == #char ";" {
                p.ext = .NAME_START;
                p.ext_token = 0;

                return true;
            }
    }

    // The CR that ends the line, where a name or value is complete. Blanks are only allowed before
    // a `;` or `=`, so none may stand between the last extension and the CR.
    if c == #char "\r" {
        if p.ext == .NAME || p.ext == .TOKEN_VALUE || p.ext == .AFTER_QUOTED {
            p.state = .CHUNK_SIZE_LF;

            return true;
        }
    }

    return false;
}

#scope_export
