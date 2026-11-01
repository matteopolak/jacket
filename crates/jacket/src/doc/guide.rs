//! Guide markup, consuming only resolved page data.
use super::markdown::esc;
use super::view::Content;

pub fn content(content: &Content) -> String {
    match content {
        Content::Guide {
            segments,
        } => format!(
            "<div class=\"prose guide\">\n{}</div>\n",
            super::pages::segments(segments)
        ),
        Content::GuidesIndex {
            title,
            cards,
        } => {
            let mut out = format!(
                "<h1 class=\"page-title\">{}</h1>\n<ul class=\"cards\">\n",
                esc(title)
            );
            for c in cards {
                out.push_str(&format!("<li><a class=\"card\" href=\"{}\"><span>{}</span><span class=\"card-sum\">{}</span></a></li>\n",esc(&c.href),esc(&c.label),esc(&c.summary)));
            }
            out.push_str("</ul>\n");
            out
        }
        _ => unreachable!("guide template requires a guide view"),
    }
}
