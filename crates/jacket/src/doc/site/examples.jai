ExampleButtons :: (out: *Out, e: *Example) {
    html(out, #run trim(#string HTML
{if e.id}<button type="button" class="btn sm js-only" data-act="reset" hidden title="Put the original example back">Reset</button>{end}<button type="button" class="btn sm js-only" data-act="copy" aria-label="Copy this example" title="Copy">Copy</button>{if e.playground_url}<a class="btn sm" href="{esc(e.playground_url.*)}" aria-label="Open this example in the playground" title="Open in the playground">↗</a>{end}{if e.id}<button type="button" class="btn sm js-only" data-act="run" data-program="{cast(Html) e.id.*}">Run ▸</button>{else if e.note}<button type="button" class="btn sm js-only" disabled title="{esc(e.note.*)}">Run ▸</button>{end}
HTML
    ));
}

ExampleNote :: (out: *Out, e: *Example) {
    html(out, "{if e.note && e.show_note}<p class=\"ex-note\">{esc(e.note.*)}</p>{end}");
}

ExampleBlock :: (out: *Out, e: *Example) {
    html(out, #run trim(#string HTML
{if e.plain}<figure class="ex plain"><pre class="code"><code>{cast(Html) e.code_html}</code></pre></figure>{else if e.compact}<figure class="ex compact"><div class="ex-row"><pre class="code"><code>{cast(Html) e.code_html}</code></pre>{ExampleButtons(e)}</div>{ExampleNote(e)}<div class="ex-out" hidden></div></figure>{else}<figure class="ex"><div class="ex-bar"><span class="ex-title">Example</span><span class="spacer"></span>{ExampleButtons(e)}</div><pre class="code"><code>{cast(Html) e.code_html}</code></pre>{ExampleNote(e)}<div class="ex-out" hidden></div></figure>{end}
HTML
    ));
}
