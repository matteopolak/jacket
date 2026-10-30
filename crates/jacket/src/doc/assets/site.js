// Jacket API docs. The pages read fine without this; it adds the theme button, search, the
// version menu, Copy, and Run (which compiles the example in your browser).
(function () {
  'use strict';
  var root = document.documentElement;
  var base = document.body.getAttribute('data-base') || '/';

  // Theme: system, light or dark, remembered in this browser.
  var KEY = 'jacket-docs-theme';
  var ORDER = ['system', 'light', 'dark'];
  var themeButton = document.getElementById('theme');
  function theme() { return root.getAttribute('data-theme') || 'system'; }
  function labelTheme() {
    if (!themeButton) return;
    var text = 'Theme: ' + theme() + ' (click to change)';
    themeButton.setAttribute('aria-label', text);
    themeButton.title = text;
  }
  if (themeButton) {
    labelTheme();
    themeButton.addEventListener('click', function () {
      var next = ORDER[(ORDER.indexOf(theme()) + 1) % ORDER.length];
      root.setAttribute('data-theme', next);
      try {
        if (next === 'system') localStorage.removeItem(KEY); else localStorage.setItem(KEY, next);
      } catch (e) { /* private window or blocked storage: the choice lasts for this page only */ }
      labelTheme();
    });
  }

  // The side navigation starts closed on a phone. On a wide screen an inline script right after
  // it has already put it back where it was. Its place and open groups are kept for the next page.
  var side = document.querySelector('details.side');
  if (side) {
    if (window.matchMedia('(max-width: 899px)').matches) side.open = false;
    window.addEventListener('pagehide', function () {
      try {
        var nav = side.querySelector('nav');
        var open = Array.prototype.map.call(side.querySelectorAll('details.nav-cat[open]'), function (d) { return d.getAttribute('data-key'); });
        sessionStorage.setItem('jacket-docs-side', JSON.stringify({ k: nav && nav.getAttribute('data-nav'), t: side.scrollTop, o: open }));
      } catch (e) { /* storage is blocked: the sidebar starts fresh */ }
    });
  }

  // The outline on the right marks the heading being read.
  var toc = document.querySelectorAll('.toc a[href^="#"]');
  if (toc.length) {
    var marks = Array.prototype.map.call(toc, function (a) {
      return { a: a, h: document.getElementById(a.getAttribute('href').slice(1)) };
    }).filter(function (m) { return m.h; });
    var pending = false;
    var mark = function () {
      pending = false;
      var now = marks[0];
      marks.forEach(function (m) { if (m.h.getBoundingClientRect().top <= 100) now = m; });
      marks.forEach(function (m) { m.a.classList.toggle('active', m === now); });
    };
    window.addEventListener('scroll', function () {
      if (!pending) { pending = true; requestAnimationFrame(mark); }
    }, { passive: true });
    mark();
  }

  // Search.
  var input = document.getElementById('q');
  var box = document.getElementById('results');
  var items = null;
  var loading = null;
  var hits = [];
  var picked = -1;
  var typeNames = {};
  var BUILTIN = {};
  'int float float32 float64 bool string void Type Any s8 s16 s32 s64 u8 u16 u32 u64 s128 u128 code Code'.split(' ').forEach(function (n) { BUILTIN[n] = true; });

  function kindOf(it) { return it.kind === 'proc' ? 'proc' : it.kind === 'const' ? 'const' : it.kind === 'module' ? 'mod' : 'type'; }
  function hrefOf(it) {
    if (it.kind === 'module') return base + 'api/' + it.module + '/';
    return base + 'api/' + it.module + '/' + it.name.replace(/[^A-Za-z0-9_]/g, function (c) { return '-' + ('0' + c.charCodeAt(0).toString(16)).slice(-2); }) + '/';
  }
  function load() {
    if (!loading) {
      loading = fetch(base + 'search-index.json').then(function (r) { return r.json(); }).then(function (d) {
        d.items.forEach(function (r) { if (r[2] === 'type') typeNames[r[1]] = true; });
        items = d.items.map(function (r) {
          var module = d.modules[r[0]];
          var title = module.split('/').pop();
          return {
            module: module, title: title, name: r[1], kind: r[2], summary: r[3], sig: r[4], shapes: r[5],
            lname: r[1].toLowerCase(), lpath: (title + '.' + r[1]).toLowerCase(), lsum: r[3].toLowerCase()
          };
        });
        return items;
      });
    }
    return loading;
  }
  function norm(type) {
    return type.replace(/^[a-z_][a-z0-9_]*\s*:\s*/i, '').replace(/\s+/g, '').toLowerCase();
  }
  function typesOf(text) {
    text = text.trim().replace(/^\(/, '').replace(/\)$/, '');
    if (!text) return [];
    return text.split(',').map(norm).filter(Boolean);
  }
  function contains(have, wanted) {
    var rest = have.slice();
    return wanted.every(function (w) {
      var at = rest.indexOf(w);
      if (at < 0) return false;
      rest.splice(at, 1);
      return true;
    });
  }
  function bySignature(query) {
    var cut = query.indexOf('->');
    var want = typesOf(query.slice(0, cut));
    var ret = typesOf(query.slice(cut + 2));
    var out = [];
    items.forEach(function (it) {
      if (it.kind !== 'proc') return;
      var best = -1;
      it.shapes.forEach(function (s) {
        if (!contains(s[0], want) || !contains(s[1], ret)) return;
        var exact = (want.length === 0 || s[0].length === want.length ? 1 : 0) + (ret.length === 0 || s[1].length === ret.length ? 1 : 0);
        best = Math.max(best, exact);
      });
      if (best >= 0) out.push({ it: it, score: best });
    });
    out.sort(function (a, b) { return b.score - a.score || a.it.name.length - b.it.name.length || (a.it.lpath < b.it.lpath ? -1 : 1); });
    return out.map(function (o) { return o.it; });
  }
  function byName(query) {
    var terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    var out = [];
    items.forEach(function (it) {
      var score = 0;
      for (var i = 0; i < terms.length; i++) {
        var t = terms[i];
        var s = 0;
        if (it.lname === t) s = 1000;
        else if (it.lname.indexOf(t) === 0) s = 800;
        else if (it.lname.indexOf(t) > 0) s = 600;
        else if (it.lpath.indexOf(t) >= 0) s = 400;
        else if (it.lsum.indexOf(t) >= 0) s = 100;
        if (!s) return;
        score += s;
      }
      if (it.kind === 'module') score += 50;
      out.push({ it: it, score: score });
    });
    out.sort(function (a, b) { return b.score - a.score || a.it.name.length - b.it.name.length || (a.it.lpath < b.it.lpath ? -1 : 1); });
    return out.map(function (o) { return o.it; });
  }
  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
  }
  // A procedure's signature coloured like the one on its page: parameter names, and the types
  // (builtin ones and the library's own). The result is itself a link, so the types are not.
  function sigNodes(it) {
    var out = document.createDocumentFragment();
    var text = it.sig;
    if (it.kind !== 'proc') { out.appendChild(document.createTextNode(text)); return out; }
    var re = /"(?:[^"\\]|\\.)*"|[A-Za-z_][A-Za-z0-9_]*|\s+|[\s\S]/g;
    // Polymorphic names (`$T`) are types wherever they appear in the signature.
    var poly = {};
    text.replace(/\$([A-Za-z_][A-Za-z0-9_]*)/g, function (_, n) { poly[n] = true; return _; });
    var m, prev = '', plain = '';
    function flush() { if (plain) { out.appendChild(document.createTextNode(plain)); plain = ''; } }
    while ((m = re.exec(text))) {
      var t = m[0];
      if (/^[A-Za-z_]/.test(t) && prev !== '.') {
        var named = /^\s*:/.test(text.slice(re.lastIndex)) && (prev === '(' || prev === ',' || prev === '$');
        var cls = named ? 'pname' : (BUILTIN[t] || typeNames[t] || poly[t]) ? 'ty' : '';
        if (cls) { flush(); out.appendChild(el('span', cls, t)); } else plain += t;
      } else {
        plain += t;
      }
      if (!/^\s+$/.test(t)) prev = t;
    }
    flush();
    return out;
  }
  function show() {
    box.textContent = '';
    if (!hits.length) {
      box.appendChild(el('div', 'empty', 'No matches.'));
    }
    hits.forEach(function (it, i) {
      var a = el('a', 'hit');
      a.href = hrefOf(it);
      a.id = 'hit-' + i;
      a.setAttribute('role', 'option');
      a.setAttribute('aria-selected', i === picked ? 'true' : 'false');
      a.appendChild(el('span', 'kind', kindOf(it)));
      a.appendChild(el('code', 'path', it.kind === 'module' ? it.title : it.title + '.' + it.name));
      var sig = el('span', 'sig');
      sig.appendChild(sigNodes(it));
      a.appendChild(sig);
      box.appendChild(a);
    });
    var tips = el('div', 'tips');
    ['↑↓ to move', '↵ to open', 'type -> bool to search by return type'].forEach(function (t) { tips.appendChild(el('span', '', t)); });
    box.appendChild(tips);
    box.hidden = false;
    input.setAttribute('aria-expanded', 'true');
    if (picked >= 0) input.setAttribute('aria-activedescendant', 'hit-' + picked); else input.removeAttribute('aria-activedescendant');
  }
  function hide() {
    box.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
  }
  function update() {
    var q = input.value.trim();
    if (!q) { hide(); return; }
    load().then(function () {
      if (input.value.trim() !== q) return;
      hits = (q.indexOf('->') >= 0 ? bySignature(q) : byName(q)).slice(0, 12);
      picked = hits.length ? 0 : -1;
      show();
    });
  }
  if (input && box) {
    input.addEventListener('focus', load);
    input.addEventListener('input', update);
    input.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        if (!hits.length) return;
        e.preventDefault();
        picked = (picked + (e.key === 'ArrowDown' ? 1 : hits.length - 1)) % hits.length;
        show();
      } else if (e.key === 'Enter') {
        if (picked >= 0 && hits[picked]) { e.preventDefault(); window.location.href = hrefOf(hits[picked]); }
      } else if (e.key === 'Escape') {
        if (!box.hidden) hide(); else input.blur();
      }
    });
    document.addEventListener('click', function (e) {
      if (!box.contains(e.target) && e.target !== input) hide();
    });
    document.addEventListener('keydown', function (e) {
      if (e.key !== '/' || e.ctrlKey || e.metaKey || e.altKey) return;
      var t = e.target;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)) return;
      e.preventDefault();
      input.focus();
      input.select();
    });
  }

  // Copy and Run.
  var worker = null;
  var runtime = null;
  var texts = {};
  function runtimeUrls() {
    if (!runtime) {
      runtime = fetch(base + 'runtime.json').then(function (r) { return r.json(); }).then(function (c) {
        return { engine: new URL(c.engine, location.origin + base).href, wasm: new URL(c.wasm, location.origin + base).href };
      });
    }
    return runtime;
  }
  function program(id) {
    if (!texts[id]) texts[id] = fetch(base + 'examples/' + id + '.jai').then(function (r) { return r.text(); });
    return texts[id];
  }
  function ask(request, limit) {
    return new Promise(function (resolve, reject) {
      if (!worker) worker = new Worker(base + 'assets/run-worker.js', { type: 'module' });
      var w = worker;
      var timer = setTimeout(function () {
        w.terminate();
        if (worker === w) worker = null;
        reject(new Error('Stopped after ' + limit / 1000 + ' seconds.'));
      }, limit);
      w.onmessage = function (e) {
        clearTimeout(timer);
        if (e.data.error) {
          if (e.data.crashed && worker === w) { w.terminate(); worker = null; }
          reject(new Error(e.data.error));
        } else {
          resolve(e.data);
        }
      };
      w.onerror = function (e) {
        clearTimeout(timer);
        if (worker === w) worker = null;
        reject(new Error(e.message || 'The compiler could not start in this browser.'));
      };
      w.postMessage(request);
    });
  }
  function report(out, state, status, text) {
    out.hidden = false;
    out.className = 'ex-out' + (state ? ' ' + state : '');
    out.textContent = '';
    out.appendChild(el('div', '', status));
    if (text) out.appendChild(el('pre', '', text));
  }
  function run(button) {
    var figure = button.closest('figure');
    var out = figure.querySelector('.ex-out');
    var id = button.getAttribute('data-program');
    button.disabled = true;
    report(out, 'busy', worker ? 'Compiling...' : 'Loading the compiler (the first run takes a moment)...');
    Promise.all([program(id), runtimeUrls()]).then(function (got) {
      return ask({ program: got[0], engine: got[1].engine, wasm: got[1].wasm }, 60000);
    }).then(function (done) {
      var r = done.result;
      var ms = Math.max(1, Math.round(done.ms));
      var printed = (r.stdout || '') + (r.stderr || '');
      if (r.exitCode === 0 && !(r.diagnostics || []).some(function (d) { return d.severity === 'error'; })) {
        report(out, '', '✓ Ran in ' + ms + ' ms', printed.trim() ? printed : '');
      } else {
        var why = r.rendered || printed || (r.exitCode !== null ? 'The example exited with status ' + r.exitCode + '.' : 'The example did not compile.');
        report(out, 'bad', '✗ Failed', why.trim());
      }
    }).catch(function (error) {
      report(out, 'bad', '✗ Could not run', String(error.message || error));
    }).then(function () { button.disabled = false; });
  }
  function copy(button) {
    var figure = button.closest('figure');
    var text = figure.querySelector('pre.code').textContent;
    var done = function () {
      var old = button.textContent;
      button.textContent = 'Copied';
      setTimeout(function () { button.textContent = old; }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done, function () {});
    } else {
      var area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      try { document.execCommand('copy'); done(); } catch (e) { /* nothing more to try */ }
      document.body.removeChild(area);
    }
  }
  document.addEventListener('click', function (e) {
    var button = e.target.closest && e.target.closest('button[data-act]');
    if (!button || button.disabled) return;
    if (button.getAttribute('data-act') === 'run') run(button); else if (button.getAttribute('data-act') === 'copy') copy(button);
  });

  // Clicking anywhere on an item card opens the item, except on links, buttons, examples and when selecting text.
  function cardLink(e) {
    var t = e.target;
    if (!t.closest || t.closest('a, button, summary, input, textarea, .ex')) return null;
    var row = t.closest('.row');
    var name = row && row.querySelector('.row-head a.name');
    if (!name) return null;
    var sel = window.getSelection && window.getSelection();
    if (sel && !sel.isCollapsed && row.contains(sel.anchorNode)) return null;
    return name;
  }
  document.addEventListener('click', function (e) {
    if (e.button !== 0) return;
    var name = cardLink(e);
    if (!name) return;
    if (e.metaKey || e.ctrlKey || e.shiftKey) window.open(name.href, '_blank'); else name.click();
  });
  document.addEventListener('auxclick', function (e) {
    var name = e.button === 1 && cardLink(e);
    if (name) window.open(name.href, '_blank');
  });

  // Version menu. versions.json, when the host serves one, is [{"version": "0.31.0", "url": "/docs/jacket/0.31.0/"}, ...].
  (function () {
    var btn = document.getElementById('ver'), menu = document.getElementById('ver-menu');
    if (!btn || !menu) return;
    fetch(base + 'versions.json').then(function (r) { return r.ok ? r.json() : []; }).then(function (list) {
      if (!Array.isArray(list) || list.length < 2) return;
      var current = btn.textContent.trim();
      list.forEach(function (v) {
        var a = document.createElement('a');
        a.href = v.url; a.textContent = v.version; a.setAttribute('role', 'menuitem');
        if (v.version === current) a.setAttribute('aria-current', 'true');
        menu.appendChild(a);
      });
      btn.disabled = false;
    }).catch(function () {});
    function set(open) { menu.hidden = !open; btn.setAttribute('aria-expanded', open ? 'true' : 'false'); }
    btn.addEventListener('click', function (e) { e.stopPropagation(); set(menu.hidden); });
    document.addEventListener('click', function (e) { if (!menu.contains(e.target)) set(false); });
    document.addEventListener('keydown', function (e) { if (e.key === 'Escape' && !menu.hidden) { set(false); btn.focus(); } });
  })();
})();
