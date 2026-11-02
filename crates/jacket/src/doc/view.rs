//! Fully resolved input to the documentation templates. No compiler models reach the renderer.
use serde::Serialize;
mod build;
pub mod guide;
pub use build::{module, site};

#[derive(Serialize)]
pub struct SiteView {
    pub format: u32,
    pub name: String,
    pub site_name: String,
    pub base: String,
    pub version: String,
    pub logo: Option<Logo>,
    pub header_links: Vec<Link>,
    pub repo: String,
    pub categories: Vec<Category>,
    pub groups: Vec<String>,
    pub nav: Vec<NavCategory>,
    pub guides: Vec<GuideLink>,
    pub index: PageView,
    pub guides_index: Option<PageView>,
    pub not_found: Option<PageView>,
    pub module_files: Vec<String>,
    pub guide_files: Vec<String>,
    pub written_files: Vec<String>,
    pub assets: Vec<String>,
    pub resolve_links: bool,
    pub footer_version: String,
}

#[derive(Serialize)]
pub struct Logo {
    pub url: String,
    pub mime: String,
}

#[derive(Serialize)]
pub struct Category {
    pub name: String,
    pub class: String,
    pub badge: String,
    pub tip: String,
    pub color: String,
    pub color_dark: String,
    pub styled: bool,
    pub dark_override: bool,
}

#[derive(Serialize, Clone)]
pub struct Link {
    pub href: String,
    pub label: String,
}

#[derive(Serialize, Clone)]
pub struct GuideLink {
    pub href: String,
    pub label: String,
    pub summary: String,
}

#[derive(Serialize)]
pub struct ModuleView {
    pub id: String,
    pub page: PageView,
    pub items: Vec<PageView>,
    pub nav: Vec<Vec<NavLink>>,
}

/// What `<head>` says about a page beyond its title and description. Empty strings are left out.
#[derive(Serialize, Default)]
pub struct PageHead {
    pub url: String,
    pub canonical: String,
    pub og_type: String,
    pub image: String,
    pub twitter_card: String,
    pub noindex: bool,
    pub json_ld: String,
}

#[derive(Serialize)]
pub struct PageView {
    pub head: PageHead,
    pub not_found: bool,
    pub title: String,
    pub description: String,
    pub path: String,
    pub side_label: String,
    pub sidebar: Sidebar,
    pub outline: Vec<Outline>,
    pub content: Content,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Sidebar {
    Modules {
        categories: Vec<NavCategory>,
    },
    Item {
        back: Link,
        heading: String,
        nav: usize,
        start: usize,
        end: usize,
        current: usize,
        anchors: Vec<Link>,
    },
    Guides {
        heading: String,
        links: Vec<NavLink>,
    },
}

#[derive(Serialize, Clone)]
pub struct NavCategory {
    pub label: String,
    pub groups: Vec<NavGroup>,
}

#[derive(Serialize, Clone)]
pub struct NavGroup {
    pub label: String,
    pub key: String,
    pub open: bool,
    pub links: Vec<NavLink>,
}

#[derive(Serialize, Clone)]
pub struct NavLink {
    pub href: String,
    pub label: String,
    pub title: Option<String>,
    pub depth: usize,
    pub current: bool,
    pub experimental: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Content {
    Index {
        categories: Vec<CardCategory>,
    },
    Module {
        crumbs: Vec<Crumb>,
        title: Title,
        available: String,
        since: String,
        import: String,
        docs: Vec<DocSegment>,
        parameters: Vec<Definition>,
        sections: Vec<Section>,
    },
    Item {
        crumbs: Vec<Crumb>,
        title: Title,
        summary_html: String,
        docs: Vec<DocSegment>,
        overloads: Vec<OverloadView>,
        value_html: String,
        members_label: String,
        members: Vec<Definition>,
        see_also: Vec<Link>,
    },
    GuidesIndex {
        title: String,
        cards: Vec<GuideLink>,
    },
    Guide {
        segments: Vec<DocSegment>,
    },
}

#[derive(Serialize)]
pub struct CardCategory {
    pub label: String,
    pub groups: Vec<CardGroup>,
}

#[derive(Serialize)]
pub struct CardGroup {
    pub label: String,
    pub cards: Vec<Card>,
}

#[derive(Serialize)]
pub struct Card {
    pub href: String,
    pub label: String,
    pub experimental: bool,
    pub category: Option<Badge>,
    pub summary_html: String,
}

#[derive(Serialize)]
pub struct Badge {
    pub class: String,
    pub label: String,
    pub tip: String,
}

#[derive(Serialize)]
pub struct Crumb {
    pub label: String,
    pub href: Option<String>,
    pub mono: bool,
}

#[derive(Serialize)]
pub struct Title {
    pub label: String,
    pub kind: String,
    pub procedure: bool,
    pub foreign: bool,
    pub category: Option<Badge>,
    pub experimental: bool,
    pub overloads: Option<usize>,
    pub platforms: String,
    pub since: String,
    pub source_url: Option<String>,
    pub mono: bool,
}

#[derive(Serialize)]
pub struct Definition {
    pub id: Option<String>,
    pub name: String,
    pub type_html: String,
    pub default_html: String,
    pub default_separator: String,
    pub doc_html: String,
}

#[derive(Serialize)]
pub struct Section {
    pub id: String,
    pub label: String,
    pub groups: Vec<RowGroup>,
}

#[derive(Serialize)]
pub struct RowGroup {
    pub heading: Option<Link>,
    pub rows: Vec<Row>,
}

#[derive(Serialize)]
pub struct Row {
    pub id: String,
    pub href: String,
    pub kind_badge: String,
    pub foreign: bool,
    pub experimental: bool,
    pub overloads: Option<usize>,
    pub platforms: String,
    pub since: String,
    pub source_url: Option<String>,
    pub lines_html: Vec<String>,
    pub summary_html: String,
    pub members: Vec<Definition>,
    pub example: Option<Example>,
}

#[derive(Serialize)]
/// Overloads retain the legacy layout: prose, definitions, then examples. Each segment list
/// preserves display order; module/item prose and examples share a single mixed list.
pub struct OverloadView {
    pub id: String,
    pub label: Option<String>,
    pub card: bool,
    pub signature_html: String,
    pub prose: Vec<DocSegment>,
    pub parameters: Vec<Definition>,
    pub returns: Vec<Definition>,
    pub examples: Vec<DocSegment>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DocSegment {
    Html { prose_html: String },
    Example { example: Example },
    Code { code_html: String },
}

#[derive(Serialize)]
pub struct Example {
    pub plain: bool,
    pub compact: bool,
    pub code_html: String,
    pub id: Option<String>,
    pub note: Option<String>,
    pub show_note: bool,
    pub playground_url: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outline {
    Heading {
        id: String,
        label: String,
        sub: bool,
    },
    Items {
        items: Vec<OutlineItem>,
        show_all: Option<usize>,
    },
}

#[derive(Serialize)]
pub struct OutlineItem {
    pub id: String,
    pub label: String,
    pub extra: bool,
}


#[cfg(test)]
mod tests;
