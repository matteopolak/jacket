Main :: (out: *Out, page: *Page(Content_Module)) {
    c := page.content;
    html(out, #run trim(#string HTML
{Crumbs(c.crumbs)}{PageTitle(*c.title)}<div class="meta">{PlatformList(c.platforms)}{if c.since.count}<span class="meta-sep">·</span>{Since(c.since)}{end}</div>
<p class="import"><code><span class="d">#import</span> <span class="s">"{esc(c.import)}"</span>;</code></p>
{if c.docs.count}<div class="fold">
{Segments(c.docs)}<script>fold()</script></div>
{end}{if c.parameters.count}{Heading(2, "parameters", "Module parameters")}{Definitions(c.parameters)}{end}{for s: c.sections}{Heading(2, s.id, s.label)}{for g: s.groups}<section class="group">
{if g.heading}{Heading(3, g.heading.href, g.heading.label)}{end}{for r: g.rows}{ItemRow(*r)}{end}</section>
{end}{end}
HTML
    ));
}

ItemRow :: (out: *Out, r: *Row) {
    html(out, #string HTML
<article class="row" id="{esc(r.id)}">
<div class="row-head"><a class="name" href="{esc(r.href)}">{esc(r.id)}</a>{if r.kind_badge.count}<span class="badge sm">{esc(r.kind_badge)}</span>{end}{ForeignBadge(r.foreign, true)}{ExperimentalBadge(r.experimental, true)}{Overloads(r.overloads)}{Missing(r.platforms)}{Since(r.since)}{Source(r.source_url)}</div>
{for line: r.lines_html}<div class="sig-line">{cast(Html) line}</div>
{end}{if r.summary_html.count}<p class="summary">{cast(Html) r.summary_html}</p>
{end}{if r.members.count}<div class="fold members" data-fold="240">
{Definitions(r.members)}<script>fold()</script></div>
{end}{if r.example}{ExampleBlock(r.example)}{end}</article>
HTML
    );
}

Side :: (out: *Out, page: *Page(Content_Module)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Modules);
    RenderSidebar(out, side);
}

Toc :: (out: *Out, page: *Page(Content_Module)) {
    PageToc(out, page.view);
}
