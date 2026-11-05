use super::*;

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
fn publication_origins_are_safe() {
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
}
