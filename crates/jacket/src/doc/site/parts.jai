// Preserve the original site's four-character escaping (including quotes in text).
// Apostrophes are literal even in attributes, which always use double quotes.
esc :: (s: string) -> Html {
    b: String_Builder;
    b.allocator = temp;
    start := 0;
    for c, i: s {
        replacement := "";
        if c == #char "&" replacement = "&amp;";
        else if c == #char "<" replacement = "&lt;";
        else if c == #char ">" replacement = "&gt;";
        else if c == #char "\"" replacement = "&quot;";
        if replacement.count == 0 continue;
        append(*b, slice(s, start, i - start));
        append(*b, replacement);
        start = i + 1;
    }
    if start == 0 return cast(Html) s;
    append(*b, advance(s, start));
    return cast(Html) builder_to_string(*b,, temp);
} @TempAllocates

ExperimentalDot :: (out: *Out, enabled: bool) {
    html(out, #run trim(#string HTML
{if enabled}<span class="dot exp" role="img" title="Experimental: may change or go away" aria-label="Experimental: may change or go away"><span class="sr">Experimental</span></span>{end}
HTML
    ));
}

ExperimentalBadge :: (out: *Out, enabled: bool, small: bool) {
    html(out, "{if enabled}<span class=\"badge exp{ifx small then \" sm\" else \"\"}\" title=\"Experimental: may change or go away\">experimental</span>{end}");
}

CategoryBadge :: (out: *Out, c: *Badge, small: bool) {
    html(out, "{if c}<span class=\"badge {esc(c.class)}{ifx small then \" sm\" else \"\"}\" title=\"{esc(c.tip)}\">{esc(c.label)}</span>{end}");
}

ForeignBadge :: (out: *Out, enabled: bool, small: bool) {
    html(out, "{if enabled}<span class=\"badge{ifx small then \" sm\" else \"\"} foreign\" title=\"Declared by a foreign library this module binds\">foreign</span>{end}");
}

Since :: (out: *Out, version: string) {
    html(out, "{if version.count}<span class=\"since\">since {esc(version)}</span>{end}");
}

Overloads :: (out: *Out, count: *int) {
    html(out, "{if count}<span class=\"since\">{count.*} overloads</span>{end}");
}

Platforms :: (out: *Out, platforms: string) {
    html(out, "{if platforms.count}<span class=\"since\" title=\"Available only on these systems\">{esc(platforms)} only</span>{end}");
}

Source :: (out: *Out, url: *string) {
    html(out, "{if url}<span class=\"spacer\"></span><a class=\"muted\" href=\"{esc(url.*)}\">Source</a>{end}");
}

Crumbs :: (out: *Out, crumbs: [] Crumb) {
    html(out, #string HTML
<nav class="crumbs" aria-label="Breadcrumb">{for c, n: crumbs}{if n > 0} / {end}{if c.href}<a href="{esc(c.href.*)}">{esc(c.label)}</a>{else}<span{if c.mono} class="mono"{end}>{esc(c.label)}</span>{end}{end}</nav>
HTML
    );
}

PageTitle :: (out: *Out, t: *Title) {
    html(out, #string HTML
<div class="titlebar"><h1{if t.mono} class="mono"{end}>{esc(t.label)}</h1>{if t.procedure}<span class="badge proc">procedure</span>{else}<span class="badge">{esc(t.kind)}</span>{end}{ForeignBadge(t.foreign, false)}{CategoryBadge(t.category, false)}{ExperimentalBadge(t.experimental, false)}{Overloads(t.overloads)}{Platforms(t.platforms)}{Since(t.since)}{Source(t.source_url)}</div>
HTML
    );
}

Heading :: (out: *Out, level: int, id: string, label: string) {
    // Tag names are constants: the template compiler can check both alternatives.
    if level == 2 {
        html(out, "<h2 id=\"{esc(id)}\">{esc(label)}<a class=\"anchor\" href=\"#{esc(id)}\" aria-label=\"Link to this heading\">#</a></h2>\n");
    } else {
        html(out, "<h3 id=\"{esc(id)}\">{esc(label)}<a class=\"anchor\" href=\"#{esc(id)}\" aria-label=\"Link to this heading\">#</a></h3>\n");
    }
}

Definitions :: (out: *Out, rows: [] Definition) {
    html(out, #string HTML
<dl class="defs">
{for r: rows}<div class="def"{if r.id} id="{esc(r.id.*)}"{end}><dt class="mono">{if r.name.count}<span class="pname">{esc(r.name)}</span>{end}{if r.type_html.count && r.name.count}<span class="colon">: </span>{end}{cast(Html) r.type_html}{if r.default_html.count}<span class="colon">{cast(Html) r.default_separator}</span>{cast(Html) r.default_html}{end}</dt><dd>{cast(Html) r.doc_html}</dd></div>
{end}</dl>
HTML
    );
}

Segments :: (out: *Out, segments: [] Doc_Segment) {
    for *s: segments {
        h0 := get_pointer(s, DocSegment_Html);
        e1 := get_pointer(s, DocSegment_Example);
        c2 := get_pointer(s, DocSegment_Code);
        d3 := get_pointer(s, DocSegment_Compare);
        if h0 append_html(out, cast(Html) h0.prose_html);
        else if e1 ExampleBlock(out, *e1.example);
        else if c2 {
            class := ifx c2.wrap then "code wrap" else "code";
            html(out, "<pre class=\"{class}\"><code>{cast(Html) c2.code_html}</code></pre>\n");
        } else if d3 Compare(out, d3);
    }
}

// Code before and after formatting, side by side; stacked on narrow screens.
Compare :: (out: *Out, c: *DocSegment_Compare) {
    html(out, #string HTML
<div class="compare">
<figure><figcaption>Before</figcaption><pre class="code"><code>{cast(Html) c.before_html}</code></pre></figure>
<figure><figcaption>After{if c.settings.count}, with <code>{esc(c.settings)}</code>{end}</figcaption><pre class="code"><code>{cast(Html) c.after_html}</code></pre></figure>
</div>
HTML
    );
}
