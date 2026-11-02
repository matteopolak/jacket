Main :: (out: *Out, page: *Page(Content_Index)) {
    html(out, #run trim(#string HTML
{if page.view.not_found}<h1>Page not found</h1><p>Search the docs or choose a module.</p>{else}<h1>{esc(page.site.name)}</h1>
{if page.view.description.count}<p class="home-lead">{esc(page.view.description)}</p>
{end}<p>Pick a module from the list, or press <kbd>/</kbd> to search by name or signature.</p>{end}
HTML
    ));
}

Side :: (out: *Out, page: *Page(Content_Index)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Modules);
    RenderSidebar(out, side);
}

Toc :: (out: *Out, page: *Page(Content_Index)) {
    PageToc(out, page.view);
}
