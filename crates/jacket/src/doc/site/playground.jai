// The playground page: a toolbar, the options as controls, an editor and the results. The page's
// script (assets/playground.js) fills in the examples and the files, and runs the code.
Main :: (out: *Out, page: *Page(Content_Playground)) {
    c := page.content;
    if c.ready PlaygroundApp(out, page.site, c);
    else PlaygroundMissing(out, c);
}

Side :: (out: *Out, page: *Page(Content_Playground)) {
    side := get_pointer(*page.view.sidebar, Sidebar_Modules);
    RenderSidebar(out, side);
}

Toc :: (out: *Out, page: *Page(Content_Playground)) {
    PageToc(out, page.view);
}

// A site built without the browser compiler cannot run code; it says how to add one.
PlaygroundMissing :: (out: *Out, c: *Content_Playground) {
    html(out, #run trim(#string HTML
<div class="prose guide"><h1>Playground</h1>
<p>This copy of the docs was built without the browser compiler, so it cannot run code.</p>
<p>Build the compiler bundle, then pass it to <code>jacket doc</code>:</p>
<pre class="code"><code>{esc(c.hint)}</code></pre>
</div>
HTML
    ));
}

PlaygroundApp :: (out: *Out, site: *Site_View, c: *Content_Playground) {
    html(out, #run trim(#string HTML
<link rel="stylesheet" href="{cast(Html) site.playground_css_url}">
<div class="pg" id="pg" data-view="code" data-runtime="{esc(c.runtime_url)}" data-base="{cast(Html) site.base}">
<h1 class="sr">Playground</h1>
<div class="pg-toast" role="status" hidden></div>
<dialog class="pg-modal" aria-labelledby="pg-modal-title"><form method="dialog"><h2 id="pg-modal-title">Replace the code in the editor?</h2><p>Loading an example replaces the files you have open. Your changes to them are lost.</p><div class="pg-modal-actions"><button type="submit" value="cancel" class="pg-tool">Cancel</button><button type="submit" value="ok" class="pg-run">Replace</button></div></form></dialog>
<div class="pg-bar" role="toolbar" aria-label="Playground">
<div class="pg-menu">
<button type="button" class="pg-tool" data-act="examples" aria-expanded="false" aria-controls="pg-examples-menu" aria-haspopup="true"><svg aria-hidden="true" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5V5a2 2 0 0 1 2-2h14v14H6a2 2 0 0 0-2 2.5zM4 19.5A2.5 2.5 0 0 0 6.5 22H20v-5"/></svg>Examples<svg aria-hidden="true" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg></button>
<div class="pg-options pg-examples-menu" id="pg-examples-menu" hidden><ul class="pg-list" id="pg-examples"></ul></div>
</div>
<button type="button" class="pg-tool pg-act" data-act="format" disabled title="Format with jaifmt (Alt+Shift+F)" aria-label="Format"><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 6H3M17 12H7M19 18H5"/></svg><span class="pg-label">Format</span></button>
<button type="button" class="pg-tool pg-act" data-act="share" title="Copy a link to this code and these options" aria-label="Copy link"><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7"/><path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7"/></svg><span class="pg-label">Copy link</span></button>
<button type="button" class="pg-tool pg-act" data-act="reset-layout" title="Put the panels back where they started"><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/></svg><span class="pg-label">Reset layout</span></button>
<div class="pg-menu">
<button type="button" class="pg-tool pg-act" data-act="options" aria-expanded="false" aria-controls="pg-options" aria-haspopup="true" title="Options" aria-label="Options"><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0"/><circle cx="16" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="18" cy="18" r="2"/></svg><span class="pg-label">Options</span></button>
<div class="pg-options" id="pg-options" hidden>
<div class="pg-group"><div class="pg-group-name">Run</div>
<div class="pg-field"><label for="pg-limit">Step limit</label><select id="pg-limit" data-opt="limit"><option value="20m">20 million</option><option value="200m" selected>200 million</option><option value="2b">2 billion</option><option value="20b">20 billion</option></select></div>
<div class="pg-field"><label for="pg-args">Arguments</label><input id="pg-args" data-opt="args" type="text" autocomplete="off" autocapitalize="off" spellcheck="false" placeholder="none"></div>
<div class="pg-field"><label for="pg-stdin">Standard input</label><textarea id="pg-stdin" data-opt="stdin" rows="2" spellcheck="false" placeholder="empty"></textarea></div>
<div class="pg-field"><label for="pg-show">Messages</label><select id="pg-show" data-opt="show"><option value="all" selected>Errors and warnings</option><option value="errors">Errors only</option></select></div>
</div>
<div class="pg-group"><div class="pg-group-name">Layout</div>
<div class="pg-field"><span class="pg-label">Drag a panel's bar to move it</span><button type="button" class="pg-tool" data-act="reset-layout">Reset</button></div>
</div>
<div class="pg-group"><div class="pg-group-name">Format</div>
<div class="pg-field"><label for="pg-indent">Indent</label><select id="pg-indent" data-opt="indent"><option value="2">2 spaces</option><option value="4" selected>4 spaces</option><option value="8">8 spaces</option></select></div>
<div class="pg-field"><label for="pg-braces">Braces</label><select id="pg-braces" data-opt="braces"><option value="same_line" selected>Same line</option><option value="preserve">As written</option></select></div>
</div>
</div>
</div>
<span class="spacer"></span>
<button type="button" class="pg-run" data-act="run" disabled title="Run (Ctrl+Enter)"><svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M7 4.5v15a1 1 0 0 0 1.5.9l12-7.5a1 1 0 0 0 0-1.8l-12-7.5A1 1 0 0 0 7 4.5z"/></svg>Run<kbd class="key-mac">⌘↵</kbd><kbd class="key-other">Ctrl+↵</kbd></button>
<button type="button" class="pg-run stop" data-act="stop" hidden><svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="5" width="14" height="14" rx="2"/></svg>Stop</button>
</div>
<div class="pg-reach pg-reach-l" data-reach="-1" title="Drag to change the workspace width" role="separator" aria-orientation="vertical"></div><div class="pg-reach pg-reach-r" data-reach="1" title="Drag to change the workspace width" role="separator" aria-orientation="vertical"></div>
<div class="pg-body">
<aside class="pg-side" aria-label="Files"><h2 class="pg-head" data-dock-bar>Files</h2><ul class="pg-list" id="pg-files"></ul></aside>
<div class="pg-main">
<section class="pg-edit" aria-label="Code">
<div class="pg-tabs" role="tablist" aria-label="Files" data-dock-bar></div>
<div class="pg-ed"><pre class="pg-gutter" aria-hidden="true"></pre><div class="ed pg-body-ed"><div class="pg-line" aria-hidden="true" hidden></div><div class="pg-link" aria-hidden="true" hidden></div><div class="pg-marks" aria-hidden="true"><div></div></div><pre class="code"><code></code></pre><textarea id="pg-code" aria-label="Code, editable" spellcheck="false" wrap="off" autocomplete="off" autocapitalize="off" disabled></textarea><div class="pg-tip" role="tooltip" hidden></div></div><button type="button" class="pg-bulb" aria-label="Show fixes" title="Show fixes" hidden><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18h6M10 22h4M12 2a7 7 0 0 0-4 12.7c.7.6 1 1.4 1 2.3h6c0-.9.3-1.7 1-2.3A7 7 0 0 0 12 2z"/></svg></button><div class="pg-options pg-actions" hidden></div></div>
</section>
<section class="pg-res" aria-label="Results">
<div class="pg-res-bar" data-dock-bar><div class="pg-tabs" role="tablist" aria-label="Results"><button type="button" role="tab" id="pg-tab-output" aria-selected="true" aria-controls="pg-output" data-tab="output">Output</button><button type="button" role="tab" id="pg-tab-problems" aria-selected="false" aria-controls="pg-problems" data-tab="problems">Problems<span class="pg-count" hidden></span></button></div>
<div class="pg-status" role="status" aria-live="polite">Loading the compiler...</div>
<button type="button" class="pg-icon sm" data-act="clear" title="Clear the output" aria-label="Clear the output"><svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14"/></svg></button></div>
<pre class="pg-output" id="pg-output" role="tabpanel" aria-labelledby="pg-tab-output" tabindex="0"></pre>
<ul class="pg-problems" id="pg-problems" role="tabpanel" aria-labelledby="pg-tab-problems" hidden></ul>
</section>
</div>
</div>
<footer class="pg-foot"><span class="pg-pos">Ln 1, Col 1</span><button type="button" class="pg-spaces" data-act="spaces" title="Indent width, for Tab and Format: click to change">Spaces: 4</button><span class="pg-lang">Jai</span><span class="pg-target" title="The browser build interprets the program for WebAssembly; optimisation levels and native targets need jacket on your machine.">WebAssembly, interpreted</span></footer>
<nav class="pg-views" aria-label="Playground views"><button type="button" data-view="files"><svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.7-.9l-.8-1.2A2 2 0 0 0 7.9 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2z"/></svg>Files</button><button type="button" data-view="code" aria-current="true"><svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m16 18 6-6-6-6M8 6l-6 6 6 6"/></svg>Code</button><button type="button" data-view="output"><svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m4 17 6-6-6-6M12 19h8"/></svg>Output</button></nav>
<noscript><p class="pg-note">The playground needs JavaScript.</p></noscript>
</div>
<script type="module" src="{cast(Html) site.playground_script_url}"></script>
HTML
    ));
}
