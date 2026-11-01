Main :: (out: *Out, page: *Page(Content_Guide)) {
    html(out, "<div class=\"prose guide\">\n{Segments(page.content.segments)}</div>\n");
}

Main :: (out: *Out, page: *Page(Content_GuidesIndex)) {
    html(out, #string HTML
<h1 class="page-title">{esc(page.content.title)}</h1>
<ul class="cards">
{for c: page.content.cards}<li><a class="card" href="{esc(c.href)}"><span>{esc(c.label)}</span><span class="card-sum">{esc(c.summary)}</span></a></li>
{end}</ul>
HTML
    );
}

Side :: (out: *Out, page: *Page(Content_Guide)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Guides);
    RenderSidebar(out, side);
}

Toc :: (out: *Out, page: *Page(Content_Guide)) {
    PageToc(out, page.view);
}

Side :: (out: *Out, page: *Page(Content_GuidesIndex)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Guides);
    RenderSidebar(out, side);
}

Toc :: (out: *Out, page: *Page(Content_GuidesIndex)) {
    PageToc(out, page.view);
}
