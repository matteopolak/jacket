use super::*;
use crate::doc::model::{Item, Module};
use crate::doc::{site::Site, view};

fn metadata() -> Metadata {
    Metadata {
        origin: "https://example.com".into(),
        base: "/docs/".into(),
        name: "Tiny".into(),
        image: "https://example.com/image.png".into(),
        version: "1".into(),
        canonical_version: None,
    }
}

fn site() -> Site {
    let mut site = Site::new(
        vec![Module {
            id: "String".into(),
            items: vec![Item {
                name: "trim".into(),
                ..Item::default()
            }],
            ..Module::default()
        }],
        vec![],
        "/docs/".into(),
        "1".into(),
        String::new(),
        String::new(),
        false,
    );
    site.name = "Tiny".into();
    site
}

#[test]
fn descriptions_are_plain_collapsed_and_bounded() {
    assert_eq!(
        description("<p>Hello &amp; &#x41; &nbsp; world</p>"),
        "Hello & A world"
    );
    let text = description(&"word ".repeat(50));
    assert!(text.chars().count() <= 155);
    assert!(text.ends_with('…'));
    assert!(!text.ends_with(" …"));
}

#[test]
fn metadata_covers_every_page_kind_and_version() {
    let site = site();
    let module = view::module(&site, 0);
    let shared = view::site(&site);
    let mut pages = vec![shared.index, module.page];
    pages.extend(module.items);
    let mut guide = view::site(&site).index;
    guide.title = site.title("Guide");
    guide.path = "guides/guide/index.html".into();
    guide.content = Content::Guide {
        segments: vec![],
    };
    pages.push(guide);
    let mut not_found = view::site(&site).index;
    not_found.path = "404.html".into();
    not_found.not_found = true;
    not_found.title = site.title("Page not found");
    pages.push(not_found);
    let mut metadata = metadata();
    let mut titles = std::collections::BTreeSet::new();
    for page in &mut pages {
        page.description = "<p>Some &amp; text</p>".into();
        metadata.apply(page);
        assert!(titles.insert(page.title.clone()));
        assert_eq!(page.description, "Some & text");
        // A 404 is served at many URLs, so it has no canonical one.
        assert_eq!(
            page.head_html.contains("rel=\"canonical\""),
            !page.not_found
        );
        assert!(
            page.head_html
                .contains(&format!("content=\"{}\"", esc(&page.title)))
        );
        assert!(page.head_html.contains("og:description"));
        assert!(page.head_html.contains("og:url"));
        assert!(page.head_html.contains("og:image"));
        assert!(page.head_html.contains("summary_large_image"));
        if let Some((_, tail)) = page.head_html.split_once("application/ld+json\">") {
            let json = tail.split_once("</script>").unwrap().0;
            let data: serde_json::Value = serde_json::from_str(json).unwrap();
            let (Content::Module {
                crumbs, ..
            }
            | Content::Item {
                crumbs, ..
            }) = &page.content
            else {
                panic!("unexpected JSON-LD")
            };
            let breadcrumb = data
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["@type"] == "BreadcrumbList")
                .unwrap();
            for (i, c) in crumbs.iter().enumerate() {
                assert_eq!(breadcrumb["itemListElement"][i]["name"], c.label);
            }
            if matches!(page.content, Content::Item { .. }) {
                assert_eq!(data[0]["@type"], "TechArticle");
            }
        }
    }
    metadata.canonical_version = Some("2".into());
    metadata.apply(&mut pages[0]);
    assert!(
        pages[0]
            .head_html
            .contains("https://example.com/docs/versions/2/")
    );
    assert!(pages[0].head_html.contains("content=\"noindex\""));
    metadata.base = "/docs/versions/1/".into();
    assert_eq!(
        metadata.url("api/String/index.html", true),
        "https://example.com/docs/versions/2/api/String/"
    );
    metadata.origin.clear();
    metadata.image.clear();
    metadata.apply(&mut pages[0]);
    assert!(!pages[0].head_html.contains("rel=\"canonical\""));
    assert!(!pages[0].head_html.contains("og:url"));
    assert!(metadata.files(&[], None).is_empty());
    assert!(pages[0].head_html.contains("content=\"summary\""));
}

#[test]
fn sitemap_shards_by_count_and_bytes() {
    let urls: Vec<_> = (0..5)
        .map(|n| format!("https://example.com/{n}/?a=1&b=2"))
        .collect();
    let single = sitemaps(&urls, "https://example.com/", 10, 10_000);
    assert_eq!(single.len(), 1);
    assert!(String::from_utf8_lossy(&single[0].1).contains("&amp;"));
    for (count, bytes) in [(2, 10_000), (10, 250)] {
        let files = sitemaps(&urls, "https://example.com/", count, bytes);
        assert!(files.len() > 2);
        let index = String::from_utf8_lossy(&files.last().unwrap().1);
        assert!(index.contains("<sitemapindex"));
        assert!(index.contains("https://example.com/sitemap-1.xml"));
        assert_eq!(
            files[..files.len() - 1]
                .iter()
                .map(|(_, b)| String::from_utf8_lossy(b).matches("<url>").count())
                .sum::<usize>(),
            5
        );
        assert!(
            files[..files.len() - 1]
                .iter()
                .all(|(_, b)| b.len() <= bytes)
        );
    }
}

#[test]
fn publication_origins_and_json_are_safe() {
    assert!(valid_origin("https://example.com"));
    assert!(valid_origin("http://localhost:8000"));
    for origin in [
        "https://",
        "https://example.com/path",
        "https://example.com?x",
        "https://a@b",
        "javascript:alert(1)",
    ] {
        assert!(!valid_origin(origin), "{origin}");
    }
    let mut page = view::module(&site(), 0).items.remove(0);
    page.title = "</script>\"&".into();
    metadata().apply(&mut page);
    let json = page
        .head_html
        .split_once("application/ld+json\">")
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    assert!(!json.contains("</"));
    let data: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(data[0]["headline"], page.title);
}
