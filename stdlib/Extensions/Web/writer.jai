// Concurrent output-tree metadata and collected errors. Use writer_init, then deinit after
// all render threads join. Read errors and check_links only after the writes finish.
// ```jai
// w := writer_init("unused", "/docs/", dry_run = true);
// defer deinit(*w);
// assert(write_file(*w, "site.css", "body {}"));
// assert(w.errors.count == 0);
// ```
Writer :: struct {
    out_dir: string;
    base_url: string;
    dry_run: bool;
    errors: [..] string;
    files: Maps.Map(string, [] string);
    folders: Maps.Map(string, bool);
    mutex: Threads.Mutex;
    pool: Arena.Pool;
}

// Initializes a writer. Dry runs retain paths and links, without retaining page bodies.
// ```jai
// w := writer_init("unused", "/", true);
// defer deinit(*w);
// assert(w.dry_run);
// ```
writer_init :: (out_dir: string, base_url: string, dry_run := false) -> Writer {
    w: Writer;
    w.out_dir = copy_string(out_dir);
    w.base_url = copy_string(base_url);
    w.dry_run = dry_run;
    Threads.init(*w.mutex);

    return w;
}

// Releases metadata once all workers are finished.
// ```jai
// w := writer_init("unused", "/", true);
// deinit(*w);
// ```
deinit :: (w: *Writer) {
    Maps.deinit(*w.files);
    Maps.deinit(*w.folders);
    array_free(w.errors);
    Arena.release(*w.pool);
    free(w.out_dir);
    free(w.base_url);
    Threads.destroy(*w.mutex);
}

// Writes a folder's index.html and copies its unescaped links into writer-owned storage.
// Paths are relative, empty for the root, or end in `/`; each page path has one producer.
// ```jai
// w := writer_init("unused", "/", true);
// defer deinit(*w);
// out: Out;
// defer deinit(*out);
// html(*out, "<p>Hi</p>");
// assert(write_page(*w, "api/", *out));
// ```
write_page :: (w: *Writer, path: string, out: *Out) -> bool {
    if path.count && !ends_with(path, "/") return writer_error(w, "page path must end in /", path);

    file := tprint("%index.html", path);
    // Trusted Html fragments may contain links too. Scan the complete page so a dry run
    // checks exactly the same attributes a written page contains.
    text := to_string(out);
    links: [..] string;
    links.allocator = temp;
    if out.record_links collect_page_links(text, *links);

    data := ifx w.dry_run then "" else text;

    return write_output(w, file, data, links);
}

// Writes an asset, which also counts as a link target. Returns false and collects any error.
// ```jai
// w := writer_init("unused", "/", true);
// defer deinit(*w);
// assert(write_file(*w, "assets/a.js", "let a = 1;"));
// ```
write_file :: (w: *Writer, path: string, data: string) -> bool {
    return write_output(w, path, data, .[]);
}

// Writes a byte slice as an asset.
// ```jai
// w := writer_init("unused", "/", true);
// defer deinit(*w);
// data := u8.[1, 2, 3];
// assert(write_file(*w, "data.bin", data));
// ```
write_file :: (w: *Writer, path: string, data: [] u8) -> bool {
    return write_file(w, path, cast(string) data);
}

// Reports sorted, unique internal-link problems with the API site's resolution rules.
// The returned array and its strings belong to the caller; free both when done.
// ```jai
// w := writer_init("unused", "/d/", true);
// defer deinit(*w);
// out: Out;
// defer deinit(*out);
// html(*out, "<a href=\"/d/\">home</a>");
// write_page(*w, "", *out);
// problems := check_links(*w);
// assert(problems.count == 0);
// array_free(problems);
// ```
check_links :: (w: *Writer, resolve := true) -> [..] string {
    problems: [..] string;
    unique: Maps.Map(string, bool);

    defer Maps.deinit(*unique);

    for links, path: w.files {
        if !ends_with(path, ".html") continue;

        for link: links {
            inside := link;
            if begins_with(link, w.base_url) inside = advance(link, w.base_url.count);
            else if link.count == 0 || begins_with(link, "#") || contains(link, "://") || begins_with(link, "/") continue;

            end := inside.count;

            for c, i: inside if c == #char "#" || c == #char "?" {
                end = i;
                break;
            }

            target := slice(inside, 0, end);
            message := "";

            if ends_with(target, ".html") message = tprint("%: link `%` ends in .html", path, link);
            else if resolve {
                file := target;
                if target.count == 0 || ends_with(target, "/") file = tprint("%index.html", target);
                if !Maps.find(*w.files, file) message = tprint("%: link `%` leads to no written file", path, link);
            }

            if message.count && !Maps.find(*unique, message) {
                owned := copy_string(message);
                Maps.insert(*unique, owned, true);
                array_add(*problems, owned);
            }
        }
    }

    Sorting.quick_sort(problems, Sorting.compare_strings);

    return problems;
}

#scope_module
Maps :: #import "Extensions/Hash_Map";
Threads :: #import "Thread";
Files :: #import "File";
Sorting :: #import "Sort";

writer_error :: (w: *Writer, what: string, path: string) -> bool {
    Threads.lock(*w.mutex);
    defer Threads.unlock(*w.mutex);
    array_add(*w.errors, save(*w.pool, tprint("%: %", path, what)));
    return false;
}

safe_path :: (path: string) -> bool {
    if path.count == 0 || begins_with(path, "/") || contains(path, "\\") || contains(path, ":") return false;

    parts := split(path, "/");

    defer array_free(parts);
    for parts if it == ".." || it == "." return false;

    return true;
}

write_output :: (w: *Writer, path: string, data: string, links: [] string) -> bool {
    if !safe_path(path) return writer_error(w, "invalid relative path", path);

    full := tprint("%/%", w.out_dir, path);
    directory := w.out_dir;
    slash := find_index_from_right(path, #char "/");

    if slash >= 0 directory = tprint("%/%", w.out_dir, slice(path, 0, slash));

    Threads.lock(*w.mutex);

    if !w.dry_run && !Maps.find(*w.folders, directory) {
        if !ensure_folder(w, directory) {
            array_add(*w.errors, save(*w.pool, tprint("%: cannot create folder", path)));
            Threads.unlock(*w.mutex);
            return false;
        }

    }

    Threads.unlock(*w.mutex);
    if !w.dry_run && !Files.write_entire_file(full, data) return writer_error(w, "cannot write file", path);

    Threads.lock(*w.mutex);
    defer Threads.unlock(*w.mutex);
    owned := cast(*string) Arena.get(*w.pool, links.count * size_of(string));
    for link, i: links owned[i] = save(*w.pool, link);

    Maps.insert(*w.files, save(*w.pool, path), .{ links.count, owned });

    return true;
}

// Called with the writer lock: sibling paths share cached ancestors.
ensure_folder :: (w: *Writer, directory: string) -> bool {
    if Maps.find(*w.folders, directory) return true;

    if directory != w.out_dir {
        slash := find_index_from_right(directory, #char "/");
        if slash >= w.out_dir.count && !ensure_folder(w, slice(directory, 0, slash)) return false;
    }

    if !Files.make_directory_if_it_does_not_exist(directory, recursive = directory == w.out_dir) return false;

    Maps.insert(*w.folders, save(*w.pool, directory), true);

    return true;
}

// Attributes in trusted fragments count too, with case-insensitive HTML names.
collect_page_links :: (html: string, links: *[..] string) {
    i := 0;
    while i < html.count {
        if !html_space(html[i]) {
            i += 1;
            continue;
        }

        start := i + 1;
        count := 0;

        if attribute_at(html, start, "href") count = 4;
        else if attribute_at(html, start, "src") count = 3;

        if count == 0 {
            i += 1;
            continue;
        }

        at := start + count;
        while at < html.count && html_space(html[at]) at += 1;

        if at >= html.count || html[at] != #char "=" {
            i += 1;
            continue;
        }

        at += 1;
        while at < html.count && html_space(html[at]) at += 1;

        if at >= html.count || (html[at] != #char "\"" && html[at] != #char "'") {
            i += 1;
            continue;
        }

        quote := html[at];

        at += 1;
        end := find_index_from_left(html, quote, at);
        if end < 0 break;

        link := slice(html, at, end - at);
        if contains(link, "&amp;") {
            replaced, _ := replace(link, "&amp;", "&");
            link = copy_temporary_string(replaced);
            free(replaced);
        }

        array_add(links, link);
        i = end + 1;
    }
}

attribute_at :: (html: string, at: int, name: string) -> bool {
    if at + name.count >= html.count return false;
    for c, n: name if (html[at + n] | 32) != c return false;
    next := html[at + name.count];
    return next == #char "=" || html_space(next);
}

html_space :: (c: u8) -> bool {
    return c == 32 || c == 9 || c == 10 || c == 13;
}
