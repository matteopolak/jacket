// Compile-time HTML components and a concurrent static-site writer.
// @experimental
// Templates preserve literal bytes and escape value holes for their HTML context.
//
// ```jai
// out: Out;
// defer deinit(*out);
// title := "A & B";
// html(*out, "<h1>{title}</h1>");
// assert(to_string(*out) == "<h1>A &amp; B</h1>");
// ```

// test: skip_on(.WASM, reason = "Web uses OS threads and files")

// Markup that the caller has checked is safe for its insertion context.
// ```jai
// safe := cast(Html) ("<br>");
// assert(cast(string) safe == "<br>");
// ```
Html :: #type,distinct string;

// Reusable page storage. Do not copy it after use; each rendering thread owns one.
// Borrow link strings until the next append; the link list expires at reset.
// Disable `record_links` to skip recording.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "hello");
// reset(*out);
// assert(bytes(*out).count == 0);
// ```
Out :: struct {
    data: [..] u8;
    links: [..] string;
    record_links := true;
    link: [..] u8;
    link_start: int;
    offsets: [..] Link_Offset;
}

// Clears a page while retaining array capacity for the next page.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "old");
// reset(*out);
// append(*out, "new");
// assert(to_string(*out) == "new");
// ```
reset :: (out: *Out) {
    out.data.count = 0;
    out.links.count = 0;
    out.offsets.count = 0;
    out.link.count = 0;
    out.link_start = 0;
}

// Releases the page's storage.
// ```jai
// out: Out;
// append(*out, "page");
// deinit(*out);
// ```
deinit :: (out: *Out) {
    array_free(out.data);
    array_free(out.links);
    array_free(out.offsets);
    array_free(out.link);
}

// Appends bytes verbatim; use html or the escaping helpers for untrusted text.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "<br>");
// assert(to_string(*out) == "<br>");
// ```
append :: (out: *Out, s: string) {
    array_add(*out.data, ..(cast([] u8) s));
}

// Appends a byte range verbatim.
// ```jai
// out: Out;
// defer deinit(*out);
// s := "ok";
// append(*out, s.data, s.count);
// assert(to_string(*out) == s);
// ```
append :: (out: *Out, data: *u8, count: int) {
    append(out, string.{ count, data });
}

// Appends a byte slice verbatim.
// ```jai
// out: Out;
// defer deinit(*out);
// data := u8.[65, 66];
// append(*out, data);
// assert(to_string(*out) == "AB");
// ```
append :: (out: *Out, data: [] u8) {
    append(out, cast(string) data);
}

// Appends one byte verbatim.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, #char "x");
// assert(to_string(*out) == "x");
// ```
append :: (out: *Out, byte: u8) {
    append(out, *byte, 1);
}

// Appends checked markup.
// ```jai
// out: Out;
// defer deinit(*out);
// append_html(*out, cast(Html) ("<br>"));
// assert(to_string(*out) == "<br>");
// ```
append_html :: (out: *Out, s: Html) {
    append(out, cast(string) s);
}

// A borrowed contiguous view, valid until the next append, reset or deinit.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "abc");
// assert(bytes(*out).count == 3);
// ```
bytes :: (out: *Out) -> [] u8 {
    return out.data;
}

// A borrowed string with the same lifetime as bytes.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "abc");
// assert(to_string(*out) == "abc");
// ```
to_string :: (out: *Out) -> string {
    return cast(string) bytes(out);
}

// Escapes ampersands and angle brackets, copying safe runs together.
// ```jai
// out: Out;
// defer deinit(*out);
// escape_text(*out, "<&>");
// assert(to_string(*out) == "&lt;&amp;&gt;");
// ```
escape_text :: (out: *Out, s: string) {
    escape(out, s, false);
}

// Escapes quoted attributes, including both quote styles.
// ```jai
// out: Out;
// defer deinit(*out);
// escape_attribute(*out, "\"'");
// assert(to_string(*out) == "&quot;&#39;");
// ```
escape_attribute :: (out: *Out, s: string) {
    escape(out, s, true);
}

// Compiles a constant template in the caller's scope. The first argument must be an
// identifier or `*identifier`; its name need not be `out`. Components start with uppercase.
// Control branches must finish in the same HTML state; loops preserve their entry state.
// ```jai
// Card :: (dest: *Out, title: string) { html(dest, "<p>{title}</p>"); }
// out: Out;
// defer deinit(*out);
// html(*out, "<main>{Card(\"Hi\")}</main>");
// assert(to_string(*out) == "<main><p>Hi</p></main>");
// ```
html :: (destination: *Out, $t: string, loc := #caller_code) #expand {
    #insert,scope(loc) #run compile_template(t, loc);
}

#assert OS != .WASM "Extensions/Web writes pages from worker threads, and WebAssembly has none";

#load "template.jai";
#load "writer.jai";
#load "parallel.jai";
#load "assets.jai";

#scope_module
#import "Basic";
B :: #import "Basic";
#import "String";
#import "Compiler";
Arena :: #import "Pool";
Literal :: #import "Extensions/String_Literal";

escape :: (out: *Out, s: string, attribute: bool) {
    start := 0;

    for c, i: s {
        replacement := "";
        if c == #char "&" replacement = "&amp;";
        else if c == #char "<" replacement = "&lt;";
        else if c == #char ">" replacement = "&gt;";
        else if attribute && c == #char "\"" replacement = "&quot;";
        else if attribute && c == #char "'" replacement = "&#39;";
        if replacement.count == 0 continue;

        append(out, slice(s, start, i - start));
        append(out, replacement);
        start = i + 1;
    }

    append(out, slice(s, start, s.count - start));
}

save :: (pool: *Arena.Pool, s: string) -> string {
    result := string.{ s.count, cast(*u8) Arena.get(pool, s.count) };
    memcpy(result.data, s.data, s.count);
    return result;
}

// Offsets remain valid when the owning byte array grows; links are rebased after growth.
Link_Offset :: struct {
    start: int;
    count: int;
}
