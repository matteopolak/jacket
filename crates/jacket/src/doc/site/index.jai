Main :: (out: *Out, page: *Page(Content_Index)) {
    html(out, #run trim(#string HTML
{if page.view.not_found}<h1>Page not found</h1><p>Search the docs or choose a module.</p>{else}<h1>{esc(page.site.name)}</h1>{end}
{for c: page.content.categories}{if c.label.count}<h2 class="section">{esc(c.label)}</h2>
{end}{for g: c.groups}<section class="cat">
{if g.label.count}{if c.label.count}<h3>{esc(g.label)}</h3>{else}<h2>{esc(g.label)}</h2>{end}
{end}<ul class="cards">
{for card: g.cards}<li><a class="card" href="{esc(card.href)}"><span class="card-name mono">{esc(card.label)}{ExperimentalBadge(card.experimental, true)}{CategoryBadge(card.category, true)}</span><span class="card-sum">{cast(Html) card.summary_html}</span></a></li>
{end}</ul>
</section>
{end}{end}
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
