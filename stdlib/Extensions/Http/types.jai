// The vocabulary of the codec: methods, versions, well-known header names, errors, limits and the
// head of a message.

Method :: enum u8 {
    OTHER; // any other token: the spelling is in Head.method_text
    GET;
    HEAD;
    POST;
    PUT;
    DELETE;
    CONNECT;
    OPTIONS;
    TRACE;
    PATCH;
}

method_name :: (method: Method) -> string {
    if method == {
        case .GET; return "GET";
        case .HEAD; return "HEAD";
        case .POST; return "POST";
        case .PUT; return "PUT";
        case .DELETE; return "DELETE";
        case .CONNECT; return "CONNECT";
        case .OPTIONS; return "OPTIONS";
        case .TRACE; return "TRACE";
        case .PATCH; return "PATCH";
    }

    return "";
}

// The Method for a token (case-sensitive: method names are). Unknown tokens are .OTHER.
parse_method :: (text: string) -> Method {
    if text == {
        case "GET"; return .GET;
        case "HEAD"; return .HEAD;
        case "POST"; return .POST;
        case "PUT"; return .PUT;
        case "DELETE"; return .DELETE;
        case "CONNECT"; return .CONNECT;
        case "OPTIONS"; return .OPTIONS;
        case "TRACE"; return .TRACE;
        case "PATCH"; return .PATCH;
    }

    return .OTHER;
}

Version :: enum u8 {
    HTTP_1_0;
    HTTP_1_1;
}

version_name :: (version: Version) -> string {
    if version == .HTTP_1_0 return "HTTP/1.0";

    return "HTTP/1.1";
}

// What shape the request target has (RFC 9112 3.2).
Target_Form :: enum u8 {
    NONE; // a response
    ORIGIN; // /path?query
    ABSOLUTE; // http://host/path
    AUTHORITY; // host:port, CONNECT only
    ASTERISK; // *, OPTIONS only
}

// How a message body is delimited.
Body_Kind :: enum u8 {
    NONE;
    LENGTH; // Content-Length bytes
    CHUNKED;
    UNTIL_CLOSE; // responses only: the body ends when the connection does
}

// Well-known header names. Parsed headers carry the id so code can compare an enum instead of
// strings; HEADER_NAMES holds the canonical spelling the writers use.
Header_Name :: enum u8 {
    UNKNOWN;
    ACCEPT;
    ACCEPT_CHARSET;
    ACCEPT_ENCODING;
    ACCEPT_LANGUAGE;
    ACCEPT_RANGES;
    ACCESS_CONTROL_ALLOW_CREDENTIALS;
    ACCESS_CONTROL_ALLOW_HEADERS;
    ACCESS_CONTROL_ALLOW_METHODS;
    ACCESS_CONTROL_ALLOW_ORIGIN;
    ACCESS_CONTROL_EXPOSE_HEADERS;
    ACCESS_CONTROL_MAX_AGE;
    ACCESS_CONTROL_REQUEST_HEADERS;
    ACCESS_CONTROL_REQUEST_METHOD;
    AGE;
    ALLOW;
    AUTHORIZATION;
    CACHE_CONTROL;
    CONNECTION;
    CONTENT_DISPOSITION;
    CONTENT_ENCODING;
    CONTENT_LANGUAGE;
    CONTENT_LENGTH;
    CONTENT_LOCATION;
    CONTENT_RANGE;
    CONTENT_TYPE;
    COOKIE;
    DATE;
    ETAG;
    EXPECT;
    EXPIRES;
    FORWARDED;
    FROM;
    HOST;
    IF_MATCH;
    IF_MODIFIED_SINCE;
    IF_NONE_MATCH;
    IF_RANGE;
    IF_UNMODIFIED_SINCE;
    KEEP_ALIVE;
    LAST_MODIFIED;
    LINK;
    LOCATION;
    MAX_FORWARDS;
    ORIGIN;
    PRAGMA;
    PROXY_AUTHENTICATE;
    PROXY_AUTHORIZATION;
    PROXY_CONNECTION;
    RANGE;
    REFERER;
    RETRY_AFTER;
    SEC_WEBSOCKET_ACCEPT;
    SEC_WEBSOCKET_KEY;
    SEC_WEBSOCKET_PROTOCOL;
    SEC_WEBSOCKET_VERSION;
    SERVER;
    SET_COOKIE;
    TE;
    TRAILER;
    TRANSFER_ENCODING;
    UPGRADE;
    USER_AGENT;
    VARY;
    VIA;
    WARNING;
    WWW_AUTHENTICATE;
    X_FORWARDED_FOR;
    X_FORWARDED_HOST;
    X_FORWARDED_PROTO;
    X_REQUEST_ID;
}

// Indexed by Header_Name.
HEADER_NAMES :: string.[
    "",
    "Accept",
    "Accept-Charset",
    "Accept-Encoding",
    "Accept-Language",
    "Accept-Ranges",
    "Access-Control-Allow-Credentials",
    "Access-Control-Allow-Headers",
    "Access-Control-Allow-Methods",
    "Access-Control-Allow-Origin",
    "Access-Control-Expose-Headers",
    "Access-Control-Max-Age",
    "Access-Control-Request-Headers",
    "Access-Control-Request-Method",
    "Age",
    "Allow",
    "Authorization",
    "Cache-Control",
    "Connection",
    "Content-Disposition",
    "Content-Encoding",
    "Content-Language",
    "Content-Length",
    "Content-Location",
    "Content-Range",
    "Content-Type",
    "Cookie",
    "Date",
    "ETag",
    "Expect",
    "Expires",
    "Forwarded",
    "From",
    "Host",
    "If-Match",
    "If-Modified-Since",
    "If-None-Match",
    "If-Range",
    "If-Unmodified-Since",
    "Keep-Alive",
    "Last-Modified",
    "Link",
    "Location",
    "Max-Forwards",
    "Origin",
    "Pragma",
    "Proxy-Authenticate",
    "Proxy-Authorization",
    "Proxy-Connection",
    "Range",
    "Referer",
    "Retry-After",
    "Sec-WebSocket-Accept",
    "Sec-WebSocket-Key",
    "Sec-WebSocket-Protocol",
    "Sec-WebSocket-Version",
    "Server",
    "Set-Cookie",
    "TE",
    "Trailer",
    "Transfer-Encoding",
    "Upgrade",
    "User-Agent",
    "Vary",
    "Via",
    "Warning",
    "WWW-Authenticate",
    "X-Forwarded-For",
    "X-Forwarded-Host",
    "X-Forwarded-Proto",
    "X-Request-Id",
];

HEADER_NAME_COUNT :: HEADER_NAMES.count;

#assert HEADER_NAME_COUNT == 71;

header_name_string :: (id: Header_Name) -> string {
    return HEADER_NAMES[cast(int) id];
}

HEADER_TABLE_SIZE :: 256;

header_hash :: inline (name: string) -> int {
    hash: u32 = 2166136261;

    for name {
        hash = (hash ^ lower(it)) * 16777619;
    }

    return cast(int) (hash & (HEADER_TABLE_SIZE - 1));
}

// An open-addressing table of Header_Name values, built at compile time.
make_header_table :: () -> [HEADER_TABLE_SIZE] u8 {
    table: [HEADER_TABLE_SIZE] u8;

    for 1..HEADER_NAME_COUNT - 1 {
        slot := header_hash(HEADER_NAMES[it]);

        while table[slot] != 0 slot = (slot + 1) & (HEADER_TABLE_SIZE - 1);

        table[slot] = cast(u8) it;
    }

    return table;
}

header_table :: #run make_header_table();

// The id of a well-known header name, matched without regard to case; .UNKNOWN otherwise.
lookup_header_name :: (name: string) -> Header_Name {
    if name.count < 2 || name.count > 32 return .UNKNOWN;

    slot := header_hash(name);

    while true {
        entry := header_table[slot];

        if entry == 0 return .UNKNOWN;

        candidate := HEADER_NAMES[entry];

        if candidate.count == name.count && equal_nocase(candidate, name) return cast(Header_Name) entry;

        slot = (slot + 1) & (HEADER_TABLE_SIZE - 1);
    }

    return .UNKNOWN;
}

// One header field line. `name` keeps the spelling on the wire; `id` says which well-known
// header it is (.UNKNOWN for others). `value` has no leading or trailing blanks. Strings of a
// parsed head point into the parser's buffer.
Header :: struct {
    name: string;
    value: string;
    id: Header_Name;
}

// The first header called `name` (any case); `found` says whether there was one.
find_header :: (headers: [] Header, name: string) -> value: string, found: bool {
    id := lookup_header_name(name);

    for headers {
        if id != .UNKNOWN {
            if it.id == id return it.value, true;
        } else if equal_nocase(it.name, name) {
            return it.value, true;
        }
    }

    return "", false;
}

find_header :: (headers: [] Header, id: Header_Name) -> value: string, found: bool {
    for headers {
        if it.id == id return it.value, true;
    }

    return "", false;
}

// True when a comma-separated header value (Connection, Upgrade, Accept-Encoding...) lists
// `token`, compared without regard to case. Parameters after `;` are ignored.
has_token :: (value: string, token: string) -> bool {
    i := 0;

    while i <= value.count {
        end := i;

        while end < value.count && value[end] != #char "," end += 1;

        start := i;
        stop := end;

        while start < stop && is_ows(value[start]) start += 1;
        while stop > start && is_ows(value[stop - 1]) stop -= 1;

        item := slice(value, start, stop - start);
        semi := find_byte(item.data, 0, item.count, #char ";");

        if semi >= 0 {
            item = slice(item, 0, semi);

            while item.count > 0 && is_ows(item[item.count - 1]) item.count -= 1;
        }

        if equal_nocase(item, token) return true;

        i = end + 1;
    }

    return false;
}

Error_Kind :: enum u8 {
    NONE;
    BAD_START_LINE; // 400: not `METHOD target HTTP/1.x` (or a status line)
    BAD_METHOD; // 400
    BAD_TARGET; // 400: bytes a target may not hold, or a form the method does not allow
    BAD_VERSION; // 400: not shaped like HTTP/d.d
    VERSION_NOT_SUPPORTED; // 505: HTTP/0.9, HTTP/2, HTTP/1.2...
    BAD_STATUS; // 400: a response status that is not 100-599
    BAD_LINE_ENDING; // 400: a bare LF, or a CR not followed by LF
    BAD_HEADER_NAME; // 400: not a token, or no colon
    WHITESPACE_BEFORE_COLON; // 400
    OBS_FOLD; // 400: a header line starting with SP or HTAB
    BAD_HEADER_VALUE; // 400: control bytes in a value
    MISSING_HOST; // 400: an HTTP/1.1 request without Host
    DUPLICATE_HOST; // 400
    BAD_CONTENT_LENGTH; // 400: not 1*DIGIT, or too big for 63 bits
    DUPLICATE_CONTENT_LENGTH; // 400
    CONTENT_LENGTH_WITH_TRANSFER_ENCODING; // 400: request smuggling
    BAD_TRANSFER_ENCODING; // 400: empty, repeated chunked, or chunked not last
    UNSUPPORTED_TRANSFER_ENCODING; // 501: gzip, deflate... around chunked
    BAD_EXPECT; // 417: an expectation other than 100-continue
    BAD_CHUNK; // 400: chunk size, extension or framing
    CHUNK_SIZE_OVERFLOW; // 400: more than 63 bits of chunk size
    BAD_TRAILER; // 400
    URI_TOO_LONG; // 414: the request line is over max_line_size
    HEADERS_TOO_LARGE; // 431: a header line or the head is over its limit
    TOO_MANY_HEADERS; // 431
    BODY_TOO_LARGE; // 413
    DATA_AFTER_CLOSE; // 400: bytes after a message that ends the connection
    UNEXPECTED_EOF; // 400: the connection closed inside a message
}

// Why parsing stopped. `message` is a static string.
Http_Error :: struct {
    kind: Error_Kind;
    message: string;
    offset: int; // bytes of the current message consumed before the fault
}

// The status code a server answers with for an error kind.
error_status :: (kind: Error_Kind) -> int {
    if kind == {
        case .NONE; return 200;
        case .VERSION_NOT_SUPPORTED; return 505;
        case .UNSUPPORTED_TRANSFER_ENCODING; return 501;
        case .BAD_EXPECT; return 417;
        case .URI_TOO_LONG; return 414;
        case .HEADERS_TOO_LARGE; return 431;
        case .TOO_MANY_HEADERS; return 431;
        case .BODY_TOO_LARGE; return 413;
    }

    return 400;
}

// Limits on what the parser accepts. A message over a limit is refused with the kind that maps to
// the status the limit names.
Limits :: struct {
    max_line_size := 8192; // the request line (414) or one header line (431), bytes, CRLF excluded
    max_head_size := 65536; // the request line and headers together (431)
    max_headers := 100; // header lines (431)
    max_trailer_size := 8192; // all trailer lines together (431)
    max_chunk_extension := 1024; // bytes after the chunk size on its line (400)
    max_body_size: s64 = 16 * 1024 * 1024; // decoded body bytes; -1 for no limit (413)
    require_host := true; // an HTTP/1.1 request needs exactly one Host (RFC 9112 3.2)
    skip_leading_empty_lines := true; // ignore CRLFs before the start line (RFC 9112 2.2 asks it of servers; clients benefit too)
}

Parser_Kind :: enum u8 {
    REQUEST;
    RESPONSE;
}

// The head of a message, filled in when the HEAD event is returned and valid until the next
// message's head begins.
Head :: struct {
    // Requests
    method: Method;
    method_text: string;
    target: string;
    target_form: Target_Form;

    // Responses
    status: int;
    reason: string;
    informational: bool; // 1xx other than 101: a final response follows

    version: Version;
    headers: [] Header;

    // How the body is delimited and what the connection should do afterwards.
    body: Body_Kind;
    content_length: s64; // for .LENGTH; also set for a HEAD response or 304 that names one (else -1)
    keep_alive: bool; // the connection may carry another message afterwards
    expect_continue: bool; // a request with a body that waits for `100 Continue` (RFC 9110 10.1.1)
    upgrade: bool; // Connection: upgrade with an Upgrade header
}

Event_Kind :: enum u8 {
    NEED_MORE; // all of the input was used; give more
    HEAD; // parser.head is filled in
    BODY; // event.data is a piece of the body (a slice of the caller's input)
    END; // the message is complete (parser.trailers holds chunked trailers)
    SWITCHED; // 101 or a CONNECT tunnel: the rest of the stream is not HTTP
    CLOSED; // parse_eof: the connection ended between messages
    ERROR; // parser.error says why; the parser stays failed
}

Event :: struct {
    kind: Event_Kind;
    data: [] u8;
}
