// Layout slots dispatch on the resolved content type; no compiler facts reach these files.
Page :: struct($C: Type) {
    site: *Site_View;
    view: *Page_View;
    content: *C;
    module: *Module_View;
    nav_stamps: [] string;
}

EARLY :: "window.fold=function(){if(++fold.n>6)return;var f=document.currentScript.parentElement;if(f.scrollHeight<=(+f.getAttribute('data-fold')||440))f.classList.add('open')};fold.n=0;(function(){var d=document.documentElement;d.classList.add('js');if(/Mac|iP(hone|ad|od)/.test(navigator.platform))d.classList.add('mac');try{var t=localStorage.getItem('jacket-docs-theme');if(t==='light'||t==='dark')d.setAttribute('data-theme',t)}catch(e){}})()";


// What the sidebar keeps across pages (open groups, scroll): every page with the site-wide
// sidebar shares one key whatever its label says; an item sidebar is kept per module.
NavKey :: (view: *Page_View) -> string {
    if view.sidebar.which == .SIDEBAR_MODULES return "modules";
    return view.side_label;
}

// The header button's text: the module name alone ("Json" for "Json module"), so it fits beside
// the logo and links on a phone. The full label stays the dialog's accessible name.
SideTitle :: (label: string) -> string {
    if !ends_with(label, " module") return label;
    title := label;
    title.count -= " module".count;
    return title;
}

Layout :: (out: *Out, page: *Page($C)) {
    html(out, #string HTML
<!doctype html>
<html lang="en" data-theme="system">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{esc(page.view.title)}</title>
{HeadMeta(page.site, page.view)}
<meta name="theme-color" content="oklch(98.5% 0 0deg)" media="(prefers-color-scheme: light)">
<meta name="theme-color" content="oklch(22.4% 0.008 285.8deg)" media="(prefers-color-scheme: dark)">
<meta name="color-scheme" content="light dark">
{Icon(page.site)}<script>{cast(Html) EARLY}</script>
<script>{cast(Html) page.site.early_js}</script>
<link rel="stylesheet" href="{cast(Html) page.site.css_url}">
<link rel="stylesheet" href="{cast(Html) page.site.categories_url}"><script src="{cast(Html) page.site.script_url}" defer></script>
</head>
<body data-base="{cast(Html) page.site.base}" data-page="{esc(page.view.path)}">
<a class="skip" href="#main-content">Skip to content</a>
{Header(page.site, page.view.path, page.view.side_label)}
<div class="wrap"{if page.view.outline.count} data-toc{end}>
<dialog class="side" id="side" open aria-label="{esc(page.view.side_label)}" data-title="{esc(SideTitle(page.view.side_label))}">
<nav aria-label="{esc(page.view.side_label)}" data-nav="{esc(NavKey(page.view))}">
{Side(page)}</nav></dialog>
<script>jacketSide()</script>
<main id="main-content" tabindex="-1">
{Main(page)}</main>
{Toc(page)}<script>jacketPage(true)</script></div>
{Footer(page.site)}<script>jacketRestore()</script></body>
</html>
HTML
    );
}

// Rust resolves publication facts; the theme owns their HTML and attribute escaping.
HeadMeta :: (out: *Out, site: *Site_View, view: *Page_View) {
    html(out, "{if view.description.count}<meta name=\"description\" content=\"{esc(view.description)}\">\n{end}");
    html(out, "{if view.title.count}<meta property=\"og:title\" content=\"{esc(view.title)}\">{end}");
    html(out, "{if view.description.count}<meta property=\"og:description\" content=\"{esc(view.description)}\">{end}");
    html(out, "{if site.site_name.count}<meta property=\"og:site_name\" content=\"{esc(site.site_name)}\">{end}");
    html(out, "{if view.head.og_type.count}<meta property=\"og:type\" content=\"{esc(view.head.og_type)}\">{end}");
    html(out, "{if view.head.url.count}<meta property=\"og:url\" content=\"{esc(view.head.url)}\">{end}");
    html(out, "{if view.head.image.count}<meta property=\"og:image\" content=\"{esc(view.head.image)}\">{end}");
    html(out, "{if view.head.twitter_card.count}<meta name=\"twitter:card\" content=\"{esc(view.head.twitter_card)}\">{end}");
    html(out, "{if view.head.canonical.count}<link rel=\"canonical\" href=\"{esc(view.head.canonical)}\">{end}");
    html(out, "{if view.head.noindex}<meta name=\"robots\" content=\"noindex\">{end}");
    html(out, "{if view.head.json_ld.count}<script type=\"application/ld+json\">{cast(Html) view.head.json_ld}</script>{end}");
}

Header :: (out: *Out, site: *Site_View, path: string = "", label: string = "Modules") {
    html(out, #run trim(#string HTML
<header class="top"><div class="top-in">
<a class="logo" href="{cast(Html) site.base}" aria-label="{esc(site.name)} API documentation">{LogoImage(site)}<span class="wordmark">{esc(site.name)}</span></a>
<button type="button" id="modules-btn" class="modules-btn js-only" aria-expanded="false" aria-controls="side" aria-label="{esc(label)}"><span class="modules-label">{esc(SideTitle(label))}</span><svg aria-hidden="true" width="12" height="12"><use href="{esc(site.icons_url)}#chevron"/></svg></button>
<div class="search-wrap js-only">
<button type="button" id="search-open" class="search" aria-haspopup="dialog" aria-controls="search-dialog"><svg aria-hidden="true" width="16" height="16"><use href="{esc(site.icons_url)}#search"/></svg><span class="search-ph">Search by name or signature</span><span class="search-keys" aria-hidden="true"><kbd class="key-mac">⌘</kbd><kbd class="key-other">Ctrl</kbd><kbd>K</kbd></span></button>
<dialog id="search-dialog" class="spotlight" aria-label="Search the docs">
<div class="spot-field"><svg aria-hidden="true" width="18" height="18"><use href="{esc(site.icons_url)}#search"/></svg><label class="sr" for="search-q">Search the docs</label><div class="spot-input"><span class="spot-ghost" aria-hidden="true"></span><input id="search-q" type="search" placeholder="Search by name or signature" autocomplete="off" spellcheck="false" role="combobox" aria-expanded="false" aria-controls="search-results" aria-autocomplete="list"></div><kbd>Esc</kbd></div>
<div id="search-results" aria-live="polite" role="listbox" aria-label="Search results" hidden></div>
</dialog>
</div>
<nav class="site-nav" aria-label="Site">
<a href="{cast(Html) site.base}"{if path.count == 0} aria-current="page"{end}>API</a>{HeaderLinks(site)}
{Version(site)}<button type="button" id="theme-toggle" class="icon-btn js-only" aria-label="Theme: system (click to change)" title="Theme: system (click to change)"><svg id="theme-icon" aria-hidden="true" width="16" height="16"><use href="{esc(site.icons_url)}#system"/></svg></button>
{Repository(site)}</nav>
</div></header>
HTML
    ));
}

Icon :: (out: *Out, site: *Site_View) {
    html(out, "{if site.logo}<link rel=\"icon\" href=\"{esc(site.logo.url)}\" type=\"{cast(Html) site.logo.mime}\">\n{else}<link rel=\"icon\" href=\"{cast(Html) site.base}assets/favicon.svg\" type=\"image/svg+xml\">{end}");
}

LogoImage :: (out: *Out, site: *Site_View) {
    html(out, "{if site.logo}<img src=\"{esc(site.logo.url)}\" width=\"28\" height=\"28\" alt=\"\">{end}");
}

HeaderLinks :: (out: *Out, site: *Site_View) {
    html(out, "{for link: site.header_links}<a href=\"{esc(link.href)}\">{esc(link.label)}</a>{end}");
}

Version :: (out: *Out, site: *Site_View) {
    html(out, #run trim(#string HTML
{if site.version.count}<div class="ver"><button type="button" id="ver-button" aria-haspopup="menu" aria-expanded="false" aria-controls="ver-menu" disabled>{esc(site.version)}<svg aria-hidden="true" width="12" height="12"><use href="{esc(site.icons_url)}#chevron"/></svg></button><div id="ver-menu" class="ver-menu" role="menu" hidden></div></div>
{end}
HTML
    ));
}

Repository :: (out: *Out, site: *Site_View) {
    html(out, "{if site.repo.count}<a class=\"icon-btn\" href=\"{esc(site.repo)}\" aria-label=\"Repository\" title=\"Repository\"><svg aria-hidden=\"true\" width=\"16\" height=\"16\" fill=\"currentColor\"><use href=\"{esc(site.icons_url)}#repo\"/></svg></a>\n{end}");
}

Footer :: (out: *Out, site: *Site_View) {
    html(out, "<footer class=\"foot\"><div>made with <a href=\"https://github.com/matteopolak/jacket\">jacket doc</a> {cast(Html) site.footer_version}</div></footer>\n");
}

CategoryStyle :: (out: *Out, site: *Site_View) {
    light, dark: String_Builder;
    light.allocator = temp;
    dark.allocator = temp;
    for c: site.categories {
        if !c.styled continue;
        print_to_builder(*light, ".%{--c:%}", c.class, c.color);
        if c.dark_override print_to_builder(*dark, ".%{--c:%}", c.class, c.color_dark);
    }
    l := builder_to_string(*light,, temp);
    if l.count == 0 return;
    d := builder_to_string(*dark,, temp);
    if d.count {
        explicit := replace(d, ".cat-", ":root[data-theme=\"dark\"] .cat-");
        defer free(explicit);
        system := replace(d, ".cat-", ":root[data-theme=\"system\"] .cat-");
        defer free(system);
        d = tprint("%@media (prefers-color-scheme: dark){%}", explicit, system);
    }
    html(out, "{cast(Html) l}{cast(Html) d}");
}

Render :: (out: *Out, site: *Site_View, view: *Page_View, module: *Module_View = null, nav_stamps: [] string = .[]) {
    c := *view.content;
    content0 := get_pointer(c, Content_Index);
    content1 := get_pointer(c, Content_Module);
    content2 := get_pointer(c, Content_Item);
    content3 := get_pointer(c, Content_Guide);
    if content0 {
        page := Page(Content_Index).{ site, view, content0, module, nav_stamps };
        Layout(out, *page);
    } else if content1 {
        page := Page(Content_Module).{ site, view, content1, module, nav_stamps };
        Layout(out, *page);
    } else if content2 {
        page := Page(Content_Item).{ site, view, content2, module, nav_stamps };
        Layout(out, *page);
    } else if content3 {
        page := Page(Content_Guide).{ site, view, content3, module, nav_stamps };
        Layout(out, *page);
    }
}
