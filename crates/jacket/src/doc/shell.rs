//! What every page shares: the head, the header with search and the theme button, the footer.
use super::markdown::esc;
use super::site::Site;

/// The extension's icon, drawn at the size the header shows it.
const LOGO: &str = include_str!("../../../../editors/vscode/images/icon.svg");

/// Runs before the page paints, so a saved theme does not flash the other one first.
const EARLY: &str = "(function(){var d=document.documentElement;d.classList.add('js');try{var t=localStorage.getItem('jacket-docs-theme');if(t==='light'||t==='dark')d.setAttribute('data-theme',t)}catch(e){}})()";

pub fn logo_svg() -> String {
    LOGO.replace(
        "width=\"16\" height=\"16\"",
        "width=\"28\" height=\"28\" role=\"img\" aria-label=\"Jacket\"",
    )
}

/// One page. `side` is the navigation to the left; `label` names it for assistive technology.
pub struct Page<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub side: String,
    pub side_label: &'a str,
    pub main: String,
    /// The `/`-separated path of the page below the site, for the canonical link.
    pub path: &'a str,
}

pub fn render(site: &Site, page: &Page<'_>) -> String {
    let base = &site.base;
    let asset = |name: &str| format!("{base}assets/{name}");
    format!(
        "<!doctype html>\n<html lang=\"en\" data-theme=\"system\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n<meta name=\"description\" content=\"{description}\">\n\
         <meta name=\"color-scheme\" content=\"light dark\">\n\
         <link rel=\"icon\" href=\"{icon}\" type=\"image/svg+xml\">\n\
         <script>{EARLY}</script>\n\
         <link rel=\"stylesheet\" href=\"{css}\">\n\
         <script src=\"{js}\" defer></script>\n</head>\n\
         <body data-base=\"{base}\" data-page=\"{path}\">\n{header}\n\
         <div class=\"wrap\">\n<details class=\"side\" open><summary>{side_label}</summary>\n<nav aria-label=\"{side_label}\">\n{side}</nav></details>\n\
         <main id=\"main\">\n{main}</main>\n</div>\n{footer}</body>\n</html>\n",
        title = esc(page.title),
        description = esc(page.description),
        icon = asset("logo.svg"),
        css = asset("site.css"),
        js = asset("site.js"),
        path = esc(page.path),
        side_label = esc(page.side_label),
        side = page.side,
        main = page.main,
        header = header(site),
        footer = footer(site),
    )
}

fn header(site: &Site) -> String {
    let base = &site.base;
    format!(
        "<header class=\"top\"><div class=\"top-in\">\n\
         <a class=\"logo\" href=\"{base}\" aria-label=\"Jacket API documentation\">{logo}</a>\n\
         <label class=\"sr\" for=\"ver\">Version</label>\n\
         <select id=\"ver\" aria-label=\"Version\"><option>jacket {version}</option></select>\n\
         <div class=\"search-wrap js-only\">\n\
         <label class=\"sr\" for=\"q\">Search the docs</label>\n\
         <div class=\"search\"><svg aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"><circle cx=\"11\" cy=\"11\" r=\"7\"/><path d=\"M20 20l-3.5-3.5\"/></svg>\
         <input id=\"q\" type=\"search\" placeholder=\"Search by name or signature\" autocomplete=\"off\" spellcheck=\"false\" role=\"combobox\" aria-expanded=\"false\" aria-controls=\"results\" aria-autocomplete=\"list\"><kbd>/</kbd></div>\n\
         <div id=\"results\" role=\"listbox\" aria-label=\"Search results\" hidden></div>\n</div>\n\
         <nav class=\"site-nav\" aria-label=\"Site\">\n\
         <a href=\"{guide}\">Guide</a><a href=\"{base}\" aria-current=\"page\">API</a><a href=\"{base}tools/\">Tools</a><a href=\"{playground}\">Playground</a>\n\
         <button type=\"button\" id=\"theme\" class=\"icon-btn js-only\" aria-label=\"Theme: system (click to change)\" title=\"Theme: system (click to change)\">\
         <svg class=\"i-system\" aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><rect x=\"3\" y=\"4\" width=\"18\" height=\"12\" rx=\"2\"/><path d=\"M8 20h8M12 16v4\"/></svg>\
         <svg class=\"i-light\" aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"><circle cx=\"12\" cy=\"12\" r=\"4\"/><path d=\"M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4\"/></svg>\
         <svg class=\"i-dark\" aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"><path d=\"M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z\"/></svg>\
         </button>\n</nav>\n</div></header>",
        logo = logo_svg(),
        version = esc(&site.version),
        guide = esc(&site.guide_url),
        playground = esc(&site.playground),
    )
}

fn footer(site: &Site) -> String {
    format!(
        "<footer class=\"foot\"><div>Made by <code>jacket doc</code> from Jacket {}. Every example here is a doc test, so if it runs on this page it runs for real. <a href=\"{}\">Source on GitHub</a>.</div></footer>\n",
        esc(&site.version),
        esc(&site.repo),
    )
}
