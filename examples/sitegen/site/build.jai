// The pipeline: read the source folder, render every page, write the output folder.
//
// Everything a build allocates lives in one `Flat_Pool` that is dropped when it ends, so the
// steps below never free anything. Only the `Build_Result` is copied out.

Build_Options :: struct {
    // The template file. Left empty, `_template.html` in the source folder is used when it exists,
    // and a built-in page otherwise.
    template_path: string;
    // Images up to this many bytes are written into pages as `data:` URIs.
    inline_limit: s64 = 2048;
}

Build_Result :: struct {
    pages: s64;
    assets: s64;
    // Every file written, relative to the output folder.
    files: [..] string;
    // Problems that did not stop the build: links to pages that are not there, and so on.
    warnings: [..] string;
    // What stopped the build, or "". Nothing is written then.
    error: string;
}

// Frees what `build_site` returned.
free_result :: (result: *Build_Result) {
    for result.files free(it);
    for result.warnings free(it);

    array_free(result.files);
    array_free(result.warnings);
    free(result.error);
    result.* = .{};
}

// Builds the site in `src` into `out`, creating `out` when needed. Files already in `out` that
// the site does not produce are left alone.
build_site :: (src: string, out: string, options := Build_Options.{}) -> Build_Result {
    caller := context.allocator;
    pool: Flat_Pool;

    defer fini(*pool);

    site := Site.{ src = src, out = out, options = options };
    result: Build_Result;

    {
        push_allocator(flat_pool_allocator_proc, *pool);

        error := load_site(*site);

        if error.count == 0 {
            render_pages(*site);
            error = write_site(*site);
        }

        // Hand the answer over in the caller's memory, before the pool goes away.
        result.pages = site.page_list.count;
        result.assets = site.assets.items.count;
        result.error = copy_string(error,, caller);
        result.files.allocator = caller;
        result.warnings.allocator = caller;

        for site.written array_add(*result.files, copy_string(it,, caller));

        for site.warnings array_add(*result.warnings, copy_string(it,, caller));
    }

    return result;
}

#scope_file

Site :: struct {
    src: string;
    out: string;
    options: Build_Options;
    template: Template;
    assets: Assets;
    // The pages, which stay where they are while more are added, so `by_source` and `page_list`
    // can point at them.
    pages: Xar(Page, 3, 12);
    by_source: Table(string, *Page);
    // In display order.
    page_list: [] *Page;
    warnings: [..] string;
    written: [..] string;
}

// What the Markdown renderer needs to know to write a link: which page it is in, and the site.
Link_Context :: struct {
    site: *Site;
    page: *Page;
}

load_site :: (site: *Site) -> error: string {
    template_error := load_template(site);

    if template_error.count > 0 return template_error;

    prefix := tprint("%/", site.src);
    out_prefix := tprint("%/", site.out);
    paths := file_list(site.src, recursive = true);

    quick_sort(paths, compare);

    for paths {
        if !begins_with(it, prefix) continue;
        // The output folder may be inside the source folder.
        if begins_with(it, out_prefix) continue;

        relative := slice(it, prefix.count, it.count - prefix.count);

        if is_private(relative) || is_directory(it) continue;

        data, ok := read_entire_file(it);

        if !ok return sprint("cannot read %", it);

        if ends_with(relative, ".md") load_page(site, relative, data);
        else add_asset(*site.assets, relative, data);
    }

    return "";
}

load_template :: (site: *Site) -> error: string {
    path := site.options.template_path;
    source := DEFAULT_TEMPLATE;

    if path.count == 0 {
        path = tprint("%/_template.html", site.src);

        if !file_exists(path) path = "";
    }

    if path.count > 0 {
        text, ok := read_entire_file(path);

        if !ok return sprint("cannot read the template %", path);

        source = text;
    }

    error: string;
    site.template, error = parse_template(source);

    if error.count > 0 return sprint("%: %", ifx path.count then path else "the built-in template", error);

    return "";
}

load_page :: (site: *Site, relative: string, text: string) {
    matter := split_front_matter(text);

    if matter.draft return;

    page := xar_add(*site.pages, .{ source = relative, output = copy_string(page_output(relative)), title = copy_string(matter.title), body = copy_string(matter.body) });

    table_set(*site.by_source, page.source, page);
}

// Turns every page's Markdown into HTML, then fixes the order and the titles the navigation uses.
render_pages :: (site: *Site) {
    list: [..] *Page;

    for *site.pages {
        context_for_links := Link_Context.{ site, it };
        options := Options.{ rewrite_url = rewrite_url, data = *context_for_links };
        rendered := render_markdown(it.body, *options);

        it.body = rendered.html;

        if it.title.count == 0 it.title = ifx rendered.title.count then rendered.title else copy_string(title_from_name(it.source));

        array_add(*list, it);
    }

    quick_sort(list, page_before);

    site.page_list = list;
}

// Writes the pages and then the assets. Returns what went wrong, or "".
write_site :: (site: *Site) -> error: string {
    for site.page_list {
        values := Page_Values.{
            title = it.title,
            content = it.body,
            nav = render_nav(site.page_list, it),
            path = it.output,
        };

        html := expand_template(*site.template, values, *site.assets, site.options.inline_limit, *site.warnings);

        if !write_output(site, it.output, html) return sprint("cannot write %", it.output);
    }

    for site.assets.items {
        if !write_output(site, it.output, it.data) return sprint("cannot write %", it.output);
    }

    return "";
}

write_output :: (site: *Site, relative: string, data: string) -> bool {
    path := tprint("%/%", site.out, relative);

    if !make_directory_if_it_does_not_exist(path_strip_filename(path), recursive = true) return false;

    if !write_entire_file(path, data) return false;

    array_add(*site.written, relative);

    return true;
}

warn :: (site: *Site, page: *Page, message: string, args: ..Any) {
    array_add(*site.warnings, sprint("%: %", page.source, tprint(message, ..args)));
}

// The hook that decides what every link and image in a page points at.
//
// - A link to another page keeps its relative form but ends in `.html`.
// - A link or image that names an asset points at the asset's hashed file; a small image is
//   written in as a `data:` URI.
// - Anything else (other sites, `#fragments`, files that do not exist) is left as typed, and a
//   missing target is a warning.
rewrite_url :: (url: string, kind: Link_Kind, data: *void) -> string {
    link := cast(*Link_Context) data;
    site := link.site;
    path, fragment := split_fragment(url);

    if is_external(url) return url;

    target := resolve_relative(page_folder(link.page.source), path);

    if target.count == 0 {
        warn(site, link.page, "`%` points outside the site", url);

        return url;
    }

    if ends_with(target, ".md") {
        if !table_contains(*site.by_source, target) warn(site, link.page, "`%` links to a page that does not exist", url);

        return tprint("%.html%", path_strip_extension(path), fragment);
    }

    asset := find_asset(*site.assets, target);

    if !asset {
        warn(site, link.page, "`%` is not a page or an asset", url);

        return url;
    }

    if kind == .IMAGE && is_inlinable(asset, site.options.inline_limit) return data_uri(asset);

    return join(root_prefix(link.page.output), asset.output,, temp);
}
