// URLs and the text inside them: percent-decoding, query strings and the parts of an
// `http://host:port/path?query` URL. Used by the server for paths and queries and by the client.

// One decoded `name=value` of a query string or a form, or one path parameter. The same type
// `Extensions/Json` decodes typed values from.
Pair :: Text_Pair;

// Decodes `%XX` escapes (and `+` as a space when `plus_is_space`, which forms and queries use but
// paths do not). The text itself is returned, with no allocation, when there is nothing to
// decode; otherwise the result comes from context.allocator. A `%` not followed by two hex
// digits stays as it is.
percent_decode :: (text: string, plus_is_space := false) -> string {
    needs := false;

    for text {
        if it == #char "%" || (plus_is_space && it == #char "+") {
            needs = true;

            break;
        }
    }

    if !needs return text;

    out := alloc_string(text.count);
    count := 0;
    i := 0;

    while i < text.count {
        c := text[i];

        if c == #char "%" && i + 2 < text.count && hex_digit(text[i + 1]) >= 0 && hex_digit(text[i + 2]) >= 0 {
            out[count] = cast(u8) (hex_digit(text[i + 1]) * 16 + hex_digit(text[i + 2]));
            count += 1;
            i += 3;
        } else if plus_is_space && c == #char "+" {
            out[count] = #char " ";
            count += 1;
            i += 1;
        } else {
            out[count] = c;
            count += 1;
            i += 1;
        }
    }

    out.count = count;

    return out;
}

// Splits `a=1&b=two&flag` into decoded pairs (`flag` has an empty value). The pairs and the
// strings that needed decoding come from context.allocator; the server calls this with
// the temporary allocator.
parse_query :: (query: string) -> [] Pair {
    pairs: [..] Pair;

    rest := query;

    while rest.count > 0 {
        found, piece, after := split_from_left(rest, #char "&");

        if !found {
            piece = rest;
            after = "";
        }

        rest = after;

        if piece.count == 0 continue;

        equals := find_index_from_left(piece, #char "=");

        pair: Pair;

        if equals >= 0 {
            pair.key = percent_decode(slice(piece, 0, equals), true);
            pair.value = percent_decode(slice(piece, equals + 1, piece.count - equals - 1), true);
        } else {
            pair.key = percent_decode(piece, true);
        }

        array_add(*pairs, pair);
    }

    return pairs;
}

// Percent-encodes everything but unreserved characters (RFC 3986), for building queries and URLs.
percent_encode :: (text: string) -> string {
    builder: String_Builder;

    for text {
        unreserved := (it >= #char "a" && it <= #char "z") || (it >= #char "A" && it <= #char "Z") || (it >= #char "0" && it <= #char "9") || it == #char "-" || it == #char "_" || it == #char "." || it == #char "~";

        if unreserved {
            append(*builder, it);
        } else {
            append(*builder, #char "%");
            append(*builder, "0123456789ABCDEF"[it >> 4]);
            append(*builder, "0123456789ABCDEF"[it & 15]);
        }
    }

    return builder_to_string(*builder);
}

// The parts of an `http://host:port/path?query` URL. All strings point into the text given to
// `parse_url`.
Url :: struct {
    scheme: string;
    host: string; // without the brackets of an IPv6 literal
    port: u16;
    target: string; // path and query as sent: always starts with `/`
    path: string;
    query: string;
    ipv6: bool;
}

Url_Problem :: enum u8 {
    NONE;
    NOT_A_URL; // no `scheme://`
    UNSUPPORTED_SCHEME; // `https` and anything else but `http`
    NO_HOST;
    BAD_PORT;
    BAD_HOST;
}

// Reads an absolute `http://` URL. `https` is its own problem: TLS is not implemented yet.
parse_url :: (text: string) -> Url, Url_Problem {
    url: Url;

    marker := find_index_from_left(text, "://");

    if marker <= 0 return url, .NOT_A_URL;

    url.scheme = slice(text, 0, marker);

    if !equal_nocase(url.scheme, "http") return url, .UNSUPPORTED_SCHEME;

    rest := slice(text, marker + 3, text.count - marker - 3);

    end := rest.count;

    for rest {
        if it == #char "/" || it == #char "?" || it == #char "#" {
            end = it_index;

            break;
        }
    }

    authority := slice(rest, 0, end);
    tail := slice(rest, end, rest.count - end);

    hash := find_index_from_left(tail, #char "#");

    if hash >= 0 tail = slice(tail, 0, hash);

    // Userinfo is not used for anything here.
    at := find_index_from_right(authority, #char "@");

    if at >= 0 authority = slice(authority, at + 1, authority.count - at - 1);

    if authority.count == 0 return url, .NO_HOST;

    port_text: string;

    if authority[0] == #char "[" {
        close_bracket := find_index_from_left(authority, #char "]");

        if close_bracket < 0 return url, .BAD_HOST;

        url.host = slice(authority, 1, close_bracket - 1);
        url.ipv6 = true;

        after := slice(authority, close_bracket + 1, authority.count - close_bracket - 1);

        if after.count > 0 {
            if after[0] != #char ":" return url, .BAD_HOST;

            port_text = slice(after, 1, after.count - 1);
        }
    } else {
        colon := find_index_from_right(authority, #char ":");

        if colon >= 0 {
            url.host = slice(authority, 0, colon);
            port_text = slice(authority, colon + 1, authority.count - colon - 1);
        } else {
            url.host = authority;
        }
    }

    if url.host.count == 0 return url, .NO_HOST;

    url.port = 80;

    if port_text.count > 0 {
        value := 0;

        for port_text {
            if it < #char "0" || it > #char "9" return url, .BAD_PORT;

            value = value * 10 + (it - #char "0");

            if value > 65535 return url, .BAD_PORT;
        }

        url.port = cast(u16) value;
    }

    if tail.count == 0 || tail[0] == #char "?" {
        // `http://host?x=1` means the path `/`.
        url.target = ifx tail.count == 0 then "/" else tprint("/%", tail);
    } else {
        url.target = tail;
    }

    question := find_index_from_left(url.target, #char "?");

    if question >= 0 {
        url.path = slice(url.target, 0, question);
        url.query = slice(url.target, question + 1, url.target.count - question - 1);
    } else {
        url.path = url.target;
    }

    return url, .NONE;
}

#scope_module

hex_digit :: (c: u8) -> int {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;

    return -1;
}
