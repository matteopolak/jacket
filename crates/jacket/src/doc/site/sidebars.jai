NavLink :: (out: *Out, link: *Nav_Link) {
    html(out, #string HTML
<li><a href="{esc(link.href)}"{if link.title} title="{esc(link.title.*)}"{end}{if link.depth > 0} style="--depth:{link.depth}"{end}{if link.current} aria-current="page"{end}>{esc(link.label)}{ExperimentalDot(link.experimental)}</a></li>
HTML
    );
}

RenderSidebar :: (out: *Out, side: *Sidebar_Modules) {
    html(out, #run trim(#string HTML
{for c: side.categories}{if c.label.count}<div class="nav-h">{esc(c.label)}</div>
{end}{for g: c.groups}{if g.label.count}<details class="nav-cat" data-key="{esc(g.key)}"{if g.open} open{end}><summary>{esc(g.label)}</summary>
<ul class="nav-list">
{for link: g.links}{NavLink(*link)}{end}</ul>
</details>
{else}<ul class="nav-list">
{for link: g.links}{NavLink(*link)}{end}</ul>
{end}{end}{end}
HTML
    ));
}

RenderSidebar :: (out: *Out, side: *Sidebar_Item, module: *Module_View) {
    html(out, #run trim(#string HTML
<a class="back" href="{esc(side.back.href)}">&larr; {esc(side.back.label)}</a>
<div class="nav-h">{esc(side.heading)}</div>
<ul class="nav-list mono">
{ItemNav(side, module)}</ul>
{if side.anchors.count}<div class="nav-h">On this page</div>
<ul class="nav-list">
{for a: side.anchors}<li><a href="{esc(a.href)}">{esc(a.label)}</a></li>
{end}</ul>
{end}
HTML
    ));
}

RenderSidebar :: (out: *Out, side: *Sidebar_Guides) {
    html(out, "<div class=\"nav-h\">{esc(side.heading)}</div>\n<ul class=\"nav-list\">\n{for link: side.links}{NavLink(*link)}{end}</ul>\n");
}

RenderOutline :: (out: *Out, entries: [] Outline) {
    if entries.count == 0 return;
    html(out, "<div class=\"toc-h\">On this page</div>\n<ul>\n{for e: entries}{OutlineEntry(*e)}{end}</ul>\n");
}

OutlineEntry :: (out: *Out, e: *Outline) {
    h0 := get_pointer(e, Outline_Heading);
    list1 := get_pointer(e, Outline_Items);
    if h0 {
        html(out, "<li{if h0.sub} class=\"sub\"{end}><a href=\"#{esc(h0.id)}\">{esc(h0.label)}</a></li>\n");
    } else if list1 {
        html(out, #string HTML
<li class="items"><ul>
{for i: list1.items}<li{if i.extra} class="extra"{end}><a href="#{esc(i.id)}">{esc(i.label)}</a></li>
{end}{if list1.show_all}<li class="more js-only"><button type="button">Show all {list1.show_all.*}</button></li>
{end}</ul></li>
HTML
    );
    }
}

PageToc :: (out: *Out, view: *Page_View) {
    html(out, "{if view.outline.count}<nav class=\"toc\" aria-label=\"On this page\">\n{RenderOutline(view.outline)}</nav>\n{end}");
}

ItemNav :: (out: *Out, side: *Sidebar_Item, module: *Module_View) {
    assert(module != null);
    links := module.nav[side.nav];
    for n: side.start..side.end - 1 {
        link := links[n];
        link.current = n == side.current;
        NavLink(out, *link);
    }
}
