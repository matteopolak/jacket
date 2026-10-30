//! What every page shares: the head, the header with search and the theme button, the footer.
use super::markdown::esc;
use super::site::Site;

/// The extension's icon, drawn at the size the header shows it.
const LOGO: &str = include_str!("../../../../editors/vscode/images/icon.svg");

/// Runs before the page paints, so a saved theme does not flash the other one first.
const EARLY: &str = "(function(){var d=document.documentElement;d.classList.add('js');try{var t=localStorage.getItem('jacket-docs-theme');if(t==='light'||t==='dark')d.setAttribute('data-theme',t)}catch(e){}})()";

/// Runs right after the sidebar, before first paint: puts the sidebar back where it was on the
/// page before, and only moves it when the current entry is out of view.
const SIDE: &str = "(function(){var s=document.querySelector('details.side');if(!s)return;var n=s.querySelector('nav'),k=n&&n.getAttribute('data-nav'),st=null;try{st=JSON.parse(sessionStorage.getItem('jacket-docs-side')||'null')}catch(e){}if(st&&st.k!==k)st=null;if(st)(st.o||[]).forEach(function(key){var l=s.querySelectorAll('details.nav-cat');for(var i=0;i<l.length;i++)if(l[i].getAttribute('data-key')===key)l[i].open=true});if(!matchMedia('(min-width:900px)').matches)return;if(st)s.scrollTop=st.t;var c=s.querySelector('[aria-current=\"page\"]');if(!c)return;var a=c.getBoundingClientRect(),b=s.getBoundingClientRect();if(a.bottom<=b.top)s.scrollTop+=a.top-b.top;else if(a.top>=b.bottom)s.scrollTop+=a.bottom-b.bottom})()";

/// A short content hash for asset URLs, so a redeployed site never pairs new pages with a cached
/// stylesheet or script.
fn stamp(text: &str) -> String {
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{:08x}", hash >> 32)
}

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
    /// The outline for the right-hand rail; empty for none.
    pub toc: String,
    /// The `/`-separated path of the page below the site, for the canonical link.
    pub path: &'a str,
}

pub fn render(site: &Site, page: &Page<'_>) -> String {
    let base = &site.base;
    let asset = |name: &str| format!("{base}assets/{name}");
    let css = format!("{base}assets/site.css?v={}", stamp(super::STYLE));
    let js = format!("{base}assets/site.js?v={}", stamp(super::SCRIPT));
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
         <div class=\"wrap\">\n<details class=\"side\" open><summary>{side_label}</summary>\n<nav aria-label=\"{side_label}\" data-nav=\"{side_label}\">\n{side}</nav></details>\n<script>{SIDE}</script>\n\
         <main id=\"main\">\n{main}</main>\n{toc}</div>\n{footer}</body>\n</html>\n",
        title = esc(page.title),
        description = esc(page.description),
        icon = asset("logo.svg"),
        css = css,
        js = js,
        path = esc(page.path),
        side_label = esc(page.side_label),
        side = page.side,
        main = page.main,
        toc = if page.toc.is_empty() {
            String::new()
        } else {
            format!(
                "<aside class=\"toc\" aria-label=\"On this page\">\n{}</aside>\n",
                page.toc
            )
        },
        header = header(site),
        footer = footer(site),
    )
}

fn header(site: &Site) -> String {
    let base = &site.base;
    format!(
        "<header class=\"top\"><div class=\"top-in\">\n\
         <a class=\"logo\" href=\"{base}\" aria-label=\"Jacket API documentation\">{logo}<span class=\"wordmark\">jacket</span></a>\n\
         <div class=\"search-wrap js-only\">\n\
         <label class=\"sr\" for=\"q\">Search the docs</label>\n\
         <div class=\"search\"><svg aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"><circle cx=\"11\" cy=\"11\" r=\"7\"/><path d=\"M20 20l-3.5-3.5\"/></svg>\
         <input id=\"q\" type=\"search\" placeholder=\"Search by name or signature\" autocomplete=\"off\" spellcheck=\"false\" role=\"combobox\" aria-expanded=\"false\" aria-controls=\"results\" aria-autocomplete=\"list\"><kbd>/</kbd></div>\n\
         <div id=\"results\" role=\"listbox\" aria-label=\"Search results\" hidden></div>\n</div>\n\
         <nav class=\"site-nav\" aria-label=\"Site\">\n\
         <a href=\"{base}\" aria-current=\"page\">API</a><a href=\"{base}tools/\">Tools</a><a href=\"{playground}\">Playground</a>\n\
         <div class=\"ver\"><button type=\"button\" id=\"ver\" aria-haspopup=\"true\" aria-expanded=\"false\" aria-controls=\"ver-menu\" disabled>{version}<svg aria-hidden=\"true\" width=\"12\" height=\"12\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M6 9l6 6 6-6\"/></svg></button><div id=\"ver-menu\" class=\"ver-menu\" role=\"menu\" hidden></div></div>\n\
         <button type=\"button\" id=\"theme\" class=\"icon-btn js-only\" aria-label=\"Theme: system (click to change)\" title=\"Theme: system (click to change)\">\
         <svg class=\"i-system\" aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><rect x=\"3\" y=\"4\" width=\"18\" height=\"12\" rx=\"2\"/><path d=\"M8 20h8M12 16v4\"/></svg>\
         <svg class=\"i-light\" aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"><circle cx=\"12\" cy=\"12\" r=\"4\"/><path d=\"M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4\"/></svg>\
         <svg class=\"i-dark\" aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"><path d=\"M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z\"/></svg>\
         </button>\n<a class=\"icon-btn\" href=\"{repo}\" aria-label=\"GitHub repository\" title=\"GitHub repository\"><svg aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\" fill=\"currentColor\"><path d=\"M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.6 7.6 0 0 1 4 0c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z\"/></svg></a>\n</nav>\n</div></header>",
        logo = logo_svg(),
        version = esc(&site.version),
        repo = esc(&site.repo),
        playground = esc(&site.playground),
    )
}

fn footer(site: &Site) -> String {
    format!(
        "<footer class=\"foot\"><div><a href=\"{}/blob/{}/docs/tools/jacket-doc.md\">jacket doc {}</a></div></footer>\n",
        esc(&site.repo),
        esc(&site.source_ref),
        esc(&site.version),
    )
}
