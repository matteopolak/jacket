// Paths inside a site. They are always relative to the source folder, written with `/`, and have
// no leading slash: `guide/intro.md`.

// `base` followed by `url`, with `.` and `..` folded away: ("guide/", "../logo.png") is
// "logo.png". Returns "" when `..` would climb out of the site. In temporary storage.
resolve_relative :: (base: string, url: string) -> string {
    parts: [..] string;
    parts.allocator = temp;

    for split(join(base, url,, temp), "/",, temp) {
        if it.count == 0 || it == "." continue;

        if it == ".." {
            if parts.count == 0 return "";

            parts.count -= 1;
        } else {
            array_add(*parts, it);
        }
    }

    return join(..parts, separator = "/",, temp);
}

// The prefix that gets from a page back to the site's root: "" for "index.html", "../" for
// "guide/intro.html". In temporary storage.
root_prefix :: (output: string) -> string {
    prefix: String_Builder;

    for output {
        if it == #char "/" append(*prefix, "../");
    }

    return builder_to_string(*prefix,, temp);
}

// Whether `url` leaves the site or stays on the page: `https://x`, `mailto:a@b`, `#top`, `/abs`.
is_external :: (url: string) -> bool {
    return url.count == 0 || contains(url, "://") || begins_with(url, "mailto:") || begins_with(url, "#") || begins_with(url, "/");
}

// Splits "page.md#part" into "page.md" and "#part".
split_fragment :: (url: string) -> path: string, fragment: string {
    at := find_index_from_left(url, #char "#");

    if at < 0 return url, "";

    return slice(url, 0, at), slice(url, at, url.count - at);
}

// Whether a file or folder name is left out of the site: it starts with `.` or `_`.
is_private :: (relative: string) -> bool {
    for split(relative, "/",, temp) {
        if it.count > 0 && (it[0] == #char "." || it[0] == #char "_") return true;
    }

    return false;
}
