#load "theme.jai";
Arena :: #import "Pool";

read_view :: ($T: Type, path: string) -> T {
    text, ok := F.read_entire_file(path, log_errors = false);
    if !ok {
        print("cannot read %\n", path);
        exit(2);
    }
    defer free(text);
    value, error := from_json(T, text);
    if error.kind != .NONE {
        print("%: %\n", path, error_text(error));
        exit(2);
    }
    return value;
}

// Each module owns a pool, populated by exactly one decoder. Rendering only borrows it.
Module_Data :: struct {
    view: Module_View;
    nav_stamps: [] string;
    pool: Arena.Pool;
}

Guide_Data :: struct {
    view: Page_View;
    pool: Arena.Pool;
}

Load_Job :: struct {
    path: string;
    module: *Module_Data;
    guide: *Guide_Data;
}

Job :: struct {
    module: *Module_View;
    page: *Page_View;
    start: int;
    end: int;
    nav_stamps: [] string;
}

Run :: struct {
    site: *Site_View;
    writer: *Writer;
}

load_job :: (out: *Out, job: *Load_Job, run: *Run) {
    if job.module {
        Arena.set_allocators(*job.module.pool);
        push_allocator(Arena.pool_allocator_proc, *job.module.pool);
        job.module.view = read_view(Module_View, job.path);
    } else {
        Arena.set_allocators(*job.guide.pool);
        push_allocator(Arena.pool_allocator_proc, *job.guide.pool);
        job.guide.view = read_view(Page_View, job.path);
    }
}

write_view :: (out: *Out, view: *Page_View, module: *Module_View, run: *Run, nav_stamps: [] string) {
    validate_page(view, module);
    Render(out, run.site, view, module, nav_stamps);
    if view.not_found {
        // The root 404 is also served for missing paths at any depth; its links are
        // absolute from the base URL, so they work there too.
        write_file(run.writer, "404.html", to_string(out));
        reset(out);
        return;
    }
    path := view.path;
    if path.count && !ends_with(path, ".html") {
        print("invalid page path %\n", path);
        exit(2);
    }
    if path.count path = slice(path, 0, path.count - 5);
    write_page(run.writer, path, out);
    reset(out);
}

render_job :: (out: *Out, job: *Job, run: *Run) {
    if job.page write_view(out, job.page, job.module, run, job.nav_stamps);
    if job.module {
        for n: job.start..job.end - 1 write_view(out, *job.module.items[n], job.module, run, job.nav_stamps);
    }
}

main :: () {
    args := get_command_line_arguments();
    defer free(args.data);
    if args.count < 3 || args.count > 4 || (args.count == 4 && args[3] != "--check") {
        print("usage: doc-site INPUT OUTPUT [--check]\n");
        exit(2);
    }
    site := read_view(Site_View, tprint("%/site.json", args[1]));
    defer free_json(*site);
    if site.format != 6 {
        print("unsupported site format %\n", site.format);
        exit(2);
    }
    for site.module_files if !relative_file(it) bad_input("invalid module input path");
    for site.guide_files if !relative_file(it) bad_input("invalid guide input path");
    for site.written_files if !relative_file(it) bad_input("invalid written file path");
    for site.assets if !begins_with(it, "assets/") || !relative_file(it) bad_input("invalid asset path");
    w := writer_init(args[2], site.base, true);
    defer deinit(*w);
    // Rust writes these files. Register their paths without overwriting their contents.
    for site.written_files write_file(*w, it, "");
    w.dry_run = args.count == 4;
    for path: site.assets {
        data, ok := F.read_entire_file(tprint("%%", #filepath, path), log_errors = false);
        if !ok {
            print("cannot read theme asset %\n", path);
            exit(2);
        }
        original := data;
        defer free(original);
        if path == "assets/site.css" data = compact_css(data);
        if path == "assets/site.js" || path == "assets/early.js" data = compact_js(data);
        if path == "assets/early.js" {
            // Inlined into every page, so it runs before the page is first painted.
            site.early_js = copy_string(data);
            continue;
        }
        write_file(*w, path, data);
        if path == "assets/site.css" site.css_url = copy_string(tprint("%assets/site.css?v=%", site.base, content_stamp(data)));
        if path == "assets/site.js" site.script_url = copy_string(tprint("%assets/site.js?v=%", site.base, content_stamp(data)));
        if path == "assets/icons.svg" site.icons_url = copy_string(tprint("%assets/icons.svg?v=%", site.base, content_stamp(data)));
    }
    category_out: Out;
    CategoryStyle(*category_out, *site);
    categories := to_string(*category_out);
    site.categories_url = copy_string(tprint("%assets/categories.css?v=%", site.base, content_stamp(categories)));
    write_file(*w, "assets/categories.css", categories);
    deinit(*category_out);
    run := Run.{ *site, *w };
    modules := NewArray(site.module_files.count, Module_Data);
    guides := NewArray(site.guide_files.count, Guide_Data);
    defer {
        for *modules {
            for stamp: it.nav_stamps free(stamp);
            free(it.nav_stamps.data);
            Arena.release(*it.pool);
        }
        for *guides Arena.release(*it.pool);
        free(modules.data);
        free(guides.data);
    }
    loads: [..] Load_Job;
    defer array_free(loads);
    for path, n: site.module_files array_add(*loads, .{ tprint("%/%", args[1], path), *modules[n], null });
    for path, n: site.guide_files array_add(*loads, .{ tprint("%/%", args[1], path), null, *guides[n] });
    // Bounded decoding avoids a large JSON file monopolizing a render worker.
    render_parallel(loads, load_job, *run, threads = 3);
    nav_out: Out;
    for *bundle: modules bundle.nav_stamps = WriteItemNav(*nav_out, *bundle.view, *w);
    deinit(*nav_out);
    jobs: [..] Job;
    defer array_free(jobs);
    array_add(*jobs, .{ page = *site.index });
    if site.not_found array_add(*jobs, .{ page = site.not_found });
    for *guide: guides array_add(*jobs, .{ page = *guide.view });
    for *bundle: modules {
        count := bundle.view.items.count;
        start := 0;
        while start < count || start == 0 {
            end := min(start + 128, count);
            page: *Page_View;
            if start == 0 page = *bundle.view.page;
            array_add(*jobs, .{ *bundle.view, page, start, end, bundle.nav_stamps });
            if end == count break;
            start = end;
        }
    }
    render_parallel(jobs, render_job, *run, threads = 3);
    problems := check_links(*w, site.resolve_links);
    defer {
        for problems free(it);
        array_free(problems);
    }
    for problem: problems print("jacket: error: %\n", problem);
    for error: w.errors print("jacket: error: %\n", error);
    if problems.count || w.errors.count exit(1);
}

bad_input :: (message: string) {
    print("jacket: bad theme input: %\n", message);
    exit(2);
}

relative_file :: (path: string) -> bool {
    if path.count == 0 || begins_with(path, "/") || contains(path, "\\") || contains(path, ":") return false;
    parts := split(path, "/");
    defer array_free(parts);
    for parts if it == ".." || it == "." || it.count == 0 return false;
    return true;
}

validate_page :: (page: *Page_View, module: *Module_View) {
    if page.content.which == .CONTENT_ITEM {
        if page.sidebar.which != .SIDEBAR_ITEM bad_input("item page needs item sidebar");
    } else if page.sidebar.which != .SIDEBAR_MODULES bad_input("page needs module sidebar");
    if !page.not_found && page.path.count && (!relative_file(page.path) || !ends_with(page.path, ".html")) bad_input("invalid page path");
    side := get_pointer(*page.sidebar, Sidebar_Item);
    if side {
        if !module || side.nav < 0 || side.nav >= module.nav.count bad_input("invalid nav group");
        links := module.nav[side.nav];
        // A page of a type the module does not export has no neighbours to list.
        empty := side.start == side.end;
        if side.start < 0 || side.end < side.start || side.end > links.count || (!empty && (side.current < side.start || side.current >= side.end)) bad_input("invalid nav window");
    }
}
