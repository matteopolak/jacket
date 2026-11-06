//! Publication URL validation and sitemap XML shared by the semantic exporter.

/// XML character data for sitemap URLs; this does not render documentation markup.
fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
}

/// `url` is an origin, not a deployment path (that belongs in --base-url).
pub fn valid_origin(origin: &str) -> bool {
    let Some(host) = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
    else {
        return false;
    };
    !host.is_empty()
        && !host
            .chars()
            .any(|c| c.is_whitespace() || "/?#@\\\"'<>".contains(c))
}

/// Limits include XML markup and escaped URLs; no shard exceeds either sitemap protocol limit.
pub fn sitemaps(
    urls: &[String],
    root: &str,
    count_limit: usize,
    byte_limit: usize,
) -> Vec<(String, Vec<u8>)> {
    const START: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">";
    const END: &str = "</urlset>";
    let mut shards = Vec::new();
    let mut text = START.to_owned();
    let mut count = 0;
    for url in urls {
        let entry = format!("<url><loc>{}</loc></url>", esc(url));
        if count > 0 && (count >= count_limit || text.len() + entry.len() + END.len() > byte_limit)
        {
            text.push_str(END);
            shards.push(std::mem::replace(&mut text, START.to_owned()));
            count = 0;
        }
        text.push_str(&entry);
        count += 1;
    }
    text.push_str(END);
    shards.push(text);
    if shards.len() == 1 {
        return vec![("sitemap.xml".into(), shards.remove(0).into_bytes())];
    }
    let mut index = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">",
    );
    let mut files = Vec::new();
    for (i, shard) in shards.into_iter().enumerate() {
        let name = format!("sitemap-{}.xml", i + 1);
        index.push_str(&format!(
            "<sitemap><loc>{}{name}</loc></sitemap>",
            esc(root)
        ));
        files.push((name, shard.into_bytes()));
    }
    index.push_str("</sitemapindex>");
    files.push(("sitemap.xml".into(), index.into_bytes()));
    files
}

#[cfg(test)]
mod tests;
