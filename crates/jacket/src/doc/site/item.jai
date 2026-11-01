Main :: (out: *Out, page: *Page(Content_Item)) {
    c := page.content;
    html(out, #run trim(#string HTML
{Crumbs(c.crumbs)}{PageTitle(*c.title)}{if c.summary_html.count}<p class="lead">{cast(Html) c.summary_html}</p>
{end}{Segments(c.docs)}{for o: c.overloads}{Overload(*o)}{end}{if c.value_html.count}<pre class="sig"><code><span class="fn">{esc(c.title.label)}</span> :: {cast(Html) c.value_html}</code></pre>
{end}{if c.members.count}<section id="members">
<h2>{cast(Html) c.members_label}</h2>
{Definitions(c.members)}</section>
{end}{if c.see_also.count}<section id="see" class="see">
<h2>See also</h2>
<div class="pills">{for p: c.see_also}<a class="pill link mono" href="{esc(p.href)}">{esc(p.label)}</a>{end}</div>
</section>
{end}
HTML
    ));
}

Overload :: (out: *Out, o: *Overload_View) {
    html(out, #string HTML
<section id="{esc(o.id)}"{if o.card} class="overload"{end}>
{if o.label}<div class="overload-head"><a href="#{esc(o.id)}">{esc(o.label.*)}</a></div>
{end}<pre class="sig"><code>{cast(Html) o.signature_html}</code></pre>
{Segments(o.prose)}{if o.parameters.count}<h3>Parameters</h3>
{Definitions(o.parameters)}{end}{if o.returns.count}<h3>Returns</h3>
{Definitions(o.returns)}{end}{Segments(o.examples)}</section>
HTML
    );
}

Side :: (out: *Out, page: *Page(Content_Item)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Item);
    RenderSidebar(out, side, page.module);
}

Toc :: (out: *Out, page: *Page(Content_Item)) {
    PageToc(out, page.view);
}
