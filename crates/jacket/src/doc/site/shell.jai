// Layout slots dispatch on the resolved content type; no compiler facts reach these files.
Page :: struct($C: Type) {
    site: *Site_View;
    view: *Page_View;
    content: *C;
    module: *Module_View;
}

EARLY :: "window.fold=function(){var f=document.currentScript.parentElement;if(f.scrollHeight<=(+f.getAttribute('data-fold')||440))f.classList.add('open')};(function(){var d=document.documentElement;d.classList.add('js');try{var t=localStorage.getItem('jacket-docs-theme');if(t==='light'||t==='dark')d.setAttribute('data-theme',t)}catch(e){}})()";

SIDE :: "(function(){var s=document.querySelector('details.side');if(!s)return;var n=s.querySelector('nav'),k=n&&n.getAttribute('data-nav'),st=null;try{st=JSON.parse(sessionStorage.getItem('jacket-docs-side')||'null')}catch(e){}if(st&&st.k!==k)st=null;if(st)(st.o||[]).forEach(function(key){var l=s.querySelectorAll('details.nav-cat');for(var i=0;i<l.length;i++)if(l[i].getAttribute('data-key')===key)l[i].open=true});if(!matchMedia('(min-width:900px)').matches)return;if(st)s.scrollTop=st.t;var c=s.querySelector('[aria-current=\"page\"]');if(!c)return;var a=c.getBoundingClientRect(),b=s.getBoundingClientRect();if(a.bottom<=b.top)s.scrollTop+=a.top-b.top;else if(a.top>=b.bottom)s.scrollTop+=a.bottom-b.bottom})()";

REPO_ICON :: "<svg aria-hidden=\"true\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\" fill=\"currentColor\"><path d=\"M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.6 7.6 0 0 1 4 0c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z\"/></svg>";

Layout :: (out: *Out, page: *Page($C)) {
    html(out, #string HTML
<!doctype html>
<html lang="en" data-theme="system">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{esc(page.view.title)}</title>
<meta name="description" content="{esc(page.view.description)}">
<meta name="color-scheme" content="light dark">
{Icon(page.site)}<script>{cast(Html) EARLY}</script>
<link rel="stylesheet" href="{cast(Html) page.site.css_url}">
{CategoryStyle(page.site)}<script src="{cast(Html) page.site.script_url}" defer></script>
</head>
<body data-base="{cast(Html) page.site.base}" data-page="{esc(page.view.path)}">
{Header(page.site)}
<div class="wrap">
<details class="side" open><summary>{esc(page.view.side_label)}</summary>
<nav aria-label="{esc(page.view.side_label)}" data-nav="{esc(page.view.side_label)}">
{Side(page)}</nav></details>
<script>{cast(Html) SIDE}</script>
<main id="main">
{Main(page)}</main>
{Toc(page)}</div>
{Footer(page.site)}</body>
</html>
HTML
    );
}

Header :: (out: *Out, site: *Site_View) {
    html(out, #run trim(#string HTML
<header class="top"><div class="top-in">
<a class="logo" href="{cast(Html) site.base}" aria-label="{esc(site.name)} API documentation">{LogoImage(site)}<span class="wordmark">{esc(site.name)}</span></a>
<div class="search-wrap js-only">
<label class="sr" for="q">Search the docs</label>
<div class="search"><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/></svg><input id="q" type="search" placeholder="Search by name or signature" autocomplete="off" spellcheck="false" role="combobox" aria-expanded="false" aria-controls="results" aria-autocomplete="list"><kbd>/</kbd></div>
<div id="results" role="listbox" aria-label="Search results" hidden></div>
</div>
<nav class="site-nav" aria-label="Site">
<a href="{cast(Html) site.base}" aria-current="page">API</a>{HeaderLinks(site)}
{Version(site)}<button type="button" id="theme" class="icon-btn js-only" aria-label="Theme: system (click to change)" title="Theme: system (click to change)"><svg class="i-system" aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/></svg><svg class="i-light" aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg><svg class="i-dark" aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/></svg></button>
{Repository(site)}</nav>
</div></header>
HTML
    ));
}

Icon :: (out: *Out, site: *Site_View) {
    html(out, "{if site.logo}<link rel=\"icon\" href=\"{esc(site.logo.url)}\" type=\"{cast(Html) site.logo.mime}\">\n{end}");
}

LogoImage :: (out: *Out, site: *Site_View) {
    html(out, "{if site.logo}<img src=\"{esc(site.logo.url)}\" width=\"28\" height=\"28\" alt=\"\">{end}");
}

HeaderLinks :: (out: *Out, site: *Site_View) {
    html(out, "{for link: site.header_links}<a href=\"{esc(link.href)}\">{esc(link.label)}</a>{end}");
}

Version :: (out: *Out, site: *Site_View) {
    html(out, #run trim(#string HTML
{if site.version.count}<div class="ver"><button type="button" id="ver" aria-haspopup="true" aria-expanded="false" aria-controls="ver-menu" disabled>{esc(site.version)}<svg aria-hidden="true" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg></button><div id="ver-menu" class="ver-menu" role="menu" hidden></div></div>
{end}
HTML
    ));
}

Repository :: (out: *Out, site: *Site_View) {
    html(out, "{if site.repo.count}<a class=\"icon-btn\" href=\"{esc(site.repo)}\" aria-label=\"Repository\" title=\"Repository\">{cast(Html) REPO_ICON}</a>\n{end}");
}

Footer :: (out: *Out, site: *Site_View) {
    html(out, "<footer class=\"foot\"><div>Made with jacket doc {cast(Html) site.footer_version}</div></footer>\n");
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
    html(out, "<style>{cast(Html) l}{cast(Html) d}</style>\n");
}

Render :: (out: *Out, site: *Site_View, view: *Page_View, module: *Module_View = null) {
    c := *view.content;
    content0 := get_pointer(c, Content_Index);
    content1 := get_pointer(c, Content_Module);
    content2 := get_pointer(c, Content_Item);
    content3 := get_pointer(c, Content_GuidesIndex);
    content4 := get_pointer(c, Content_Guide);
    if content0 {
        page := Page(Content_Index).{ site, view, content0, module };
        Layout(out, *page);
    } else if content1 {
        page := Page(Content_Module).{ site, view, content1, module };
        Layout(out, *page);
    } else if content2 {
        page := Page(Content_Item).{ site, view, content2, module };
        Layout(out, *page);
    } else if content3 {
        page := Page(Content_GuidesIndex).{ site, view, content3, module };
        Layout(out, *page);
    } else if content4 {
        page := Page(Content_Guide).{ site, view, content4, module };
        Layout(out, *page);
    }
}
