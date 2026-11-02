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

RenderSidebar :: (out: *Out, side: *Sidebar_Item, module: *Module_View, nav_stamps: [] string) {
    html(out, #run trim(#string HTML
<a class="back" href="{esc(side.back.href)}">&larr; {esc(side.back.label)}</a>
<div class="nav-h">{esc(side.heading)}</div>
{ItemNavList(side, module, nav_stamps)}
{if side.anchors.count}<div class="nav-h">On this page</div>
<ul class="nav-list">
{for a: side.anchors}<li><a href="{esc(a.href)}">{esc(a.label)}</a></li>
{end}</ul>
{end}
HTML
    ));
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
{for i: list1.items}<li><a href="#{esc(i.id)}">{esc(i.label)}</a></li>
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
    first := side.start;
    if side.end - side.start > 5 first = max(side.start, side.current - 2);
    last := side.end;
    if side.end - side.start > 5 last = min(side.end, side.current + 3);
    for n: first..last - 1 {
        link := links[n];
        link.current = n == side.current;
        NavLink(out, *link);
    }
}

// The view still describes the full window. Only the no-JS fallback is shortened.
ItemNavPath :: (module: *Module_View, group: int) -> string {
    path := module.page.path;
    return tprint("assets/nav/%%.html", slice(path, 0, path.count - 10), group);
}

ItemNavList :: (out: *Out, side: *Sidebar_Item, module: *Module_View, nav_stamps: [] string) {
    assert(side.end - side.start <= 5 || nav_stamps.count == module.nav.count);
    first := max(side.start, side.current - 2);
    last := min(side.end, side.current + 3);
    // `--before`/`--after` reserve the rows the script fills in, so the list keeps its height.
    html(out, "<ul class=\"nav-list mono\"{if side.end - side.start > 5} data-source=\"{esc(ItemNavPath(module, side.nav))}?v={esc(nav_stamps[side.nav])}\" data-start=\"{side.start}\" data-end=\"{side.end}\" data-first=\"{first}\" style=\"--before:{first - side.start};--after:{side.end - last}\"{end}>\n{ItemNav(side, module)}</ul>");
}

// One file per module/group, shared by every item page in that group.
// Returned stamps own their storage and outlive the temporary render buffer.
WriteItemNav :: (out: *Out, module: *Module_View, writer: *Writer) -> [] string {
    stamps := NewArray(module.nav.count, string);
    for links, group: module.nav {
        for link: links NavLink(out, *link);
        data := to_string(out);
        stamps[group] = copy_string(content_stamp(data));
        write_file(writer, ItemNavPath(module, group), data);
        reset(out);
    }
    return stamps;
}
