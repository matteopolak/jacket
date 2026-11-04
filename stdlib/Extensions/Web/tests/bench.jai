// These tests use native windows or worker threads, which wasm cannot provide.
#if OS != .WASM {
    #import "Basic";
    #import "Extensions/Web";

    render_large :: (out: *Out, job: *int, w: *Writer) {
        // One 10 KB constant append, with a value and recorded internal link.
        html(out, "<!doctype html><html><body><h1>Page {job.*}</h1><a href=\"/\">home</a><pre>{PAYLOAD}</pre></body></html>");
        assert(write_page(w, tprint("pages/%", job.*), out));
    }

    PAYLOAD :: #run -> string {
        text := alloc_string(10_000);
        for *text it.* = #char "x";
        return text;
    };

    render_30000_pages :: () {
        w := writer_init("unused", "/", true);

        defer deinit(*w);
        jobs := NewArray(30_000, int);

        defer free(jobs.data);
        for *jobs it.* = it_index;

        render_parallel(jobs, render_large, *w);
        assert(w.errors.count == 0 && w.files.count == 30_000);
    } @bench @samples(5)
}
