Main :: (out: *Out, page: *Page(Content_Guide)) {
    html(out, "<div class=\"prose guide\">\n{Segments(page.content.segments)}</div>\n");
}

Side :: (out: *Out, page: *Page(Content_Guide)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Modules);
    RenderSidebar(out, side);
}

Toc :: (out: *Out, page: *Page(Content_Guide)) {
    PageToc(out, page.view);
}
