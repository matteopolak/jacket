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
// `links` expire at reset. Disable `record_links` to skip recording.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "hello");
// reset(*out);
// assert(bytes(*out).count == 0);
// ```
Out :: struct {
    builder: String_Builder;
    links: [..] string;
    record_links := true;
    pool: Arena.Pool;
    flat: [..] u8;
    link: [..] u8;
    cached: [..] Cached_Block;
    backing: Allocator;
}

// Clears a page while retaining pooled blocks and array capacity for the next page.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "old");
// reset(*out);
// append(*out, "new");
// assert(to_string(*out) == "new");
// ```
reset :: (out: *Out) {
    B.free_buffers(*out.builder);
    Arena.reset(*out.pool);
    out.links.count = 0;
    out.flat.count = 0;
    out.link.count = 0;
}

// Releases the page's storage.
// ```jai
// out: Out;
// append(*out, "page");
// deinit(*out);
// ```
deinit :: (out: *Out) {
    B.free_buffers(*out.builder);
    Arena.release(*out.pool);
    for block: out.cached out.backing.proc(.FREE, 0, block.size, block.data, out.backing.data);

    array_free(out.cached);
    array_free(out.links);
    array_free(out.flat);
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
    prepare(out);
    out.builder.allocator = .{ Arena.pool_allocator_proc, *out.pool };
    B.append(*out.builder, s);
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

// A borrowed contiguous view, valid until the next bytes, to_string, reset or deinit.
// ```jai
// out: Out;
// defer deinit(*out);
// append(*out, "abc");
// assert(bytes(*out).count == 3);
// ```
bytes :: (out: *Out) -> [] u8 {
    out.flat.count = 0;
    if out.builder.initialized {
        buffer := B.get_base_buffer(*out.builder);
        while buffer {
            array_add(*out.flat, ..(cast([] u8) string.{ buffer.count, B.get_buffer_data(buffer) }));
            buffer = buffer.next;
        }
    }

    return out.flat;
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

// The pool normally frees oversized blocks at reset. Cache its backing allocations as well,
// so large pages and long links have the same warm reuse as small ones.
Cached_Block :: struct {
    data: *void;
    size: int;
    available: bool;
}

prepare :: (out: *Out) {
    if out.backing.proc return;
    out.backing = context.allocator;
    out.pool.block_allocator = .{ retain_block, out };
}

retain_block :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    out := cast(*Out) data;

    if mode == .ALLOCATE {
        for *block: out.cached {
            if block.available && block.size >= size {
                block.available = false;
                return block.data;
            }
        }

        result := out.backing.proc(.ALLOCATE, size, 0, null, out.backing.data);
        if result array_add(*out.cached, .{ result, size, false });

        return result;
    }

    if mode == .FREE {
        for *block: out.cached if block.data == memory {
            block.available = true;
            return null;
        }

        assert(false, "Web: pool returned an unknown backing block");
    }

    return null;
}
