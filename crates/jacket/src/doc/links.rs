//! The check on the written site: no link names a `.html` file, and every link into the site
//! leads to a file that was written.
use std::collections::BTreeSet;

/// Problems with the links of the pages in `files` (path below the site, bytes), as sentences.
/// `resolve` is off when only some modules were built, as links to the others lead nowhere.
pub fn check(files: &[(String, Vec<u8>)], base: &str, resolve: bool) -> Vec<String> {
    let written: BTreeSet<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
    let mut problems = Vec::new();
    for (path, bytes) in files {
        if !path.ends_with(".html") {
            continue;
        }
        let html = String::from_utf8_lossy(bytes);
        for link in links(&html) {
            let Some(inside) = internal(&link, base) else {
                continue;
            };
            let target = inside.split(['#', '?']).next().unwrap_or("");
            if target.ends_with(".html") {
                problems.push(format!("{path}: link `{link}` ends in .html"));
            } else if resolve {
                let file = if target.is_empty() || target.ends_with('/') {
                    format!("{target}index.html")
                } else {
                    target.to_owned()
                };
                if !written.contains(file.as_str()) {
                    problems.push(format!("{path}: link `{link}` leads to no written file"));
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    problems
}

/// The part of `link` below the site, when it points into the site.
fn internal<'a>(link: &'a str, base: &str) -> Option<&'a str> {
    if let Some(rest) = link.strip_prefix(base) {
        return Some(rest);
    }
    // A bare fragment stays on the page; anything with a scheme or another root is not ours.
    if link.is_empty() || link.starts_with('#') || link.contains("://") || link.starts_with('/') {
        return None;
    }
    Some(link)
}

/// The values of the `href` and `src` attributes in `html`, entities undone.
fn links(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for attr in [" href=\"", " src=\""] {
        let mut rest = html;
        while let Some(at) = rest.find(attr) {
            let tail = &rest[at + attr.len()..];
            let Some(end) = tail.find('"') else {
                break;
            };
            out.push(tail[..end].replace("&amp;", "&"));
            rest = &tail[end..];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(path: &str, html: &str) -> (String, Vec<u8>) {
        (path.to_owned(), html.as_bytes().to_vec())
    }

    #[test]
    fn folder_links_resolve_to_their_index() {
        let files = [
            page(
                "index.html",
                "<a href=\"/d/api/A/\">A</a><a href=\"/d/\">home</a>",
            ),
            page("api/A/index.html", ""),
        ];
        assert!(check(&files, "/d/", true).is_empty());
    }

    #[test]
    fn html_and_dangling_links_are_reported() {
        let files = [page(
            "index.html",
            "<a href=\"/d/api/A.html\">A</a><a href=\"/d/api/B/\">B</a><a href=\"https://x.test/a.html\">x</a>",
        )];
        let found = check(&files, "/d/", true);
        assert_eq!(found.len(), 2, "{found:?}");
    }
}
