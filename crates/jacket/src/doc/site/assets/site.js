// @ts-check
// Jacket API docs. The pages read fine without this; it adds the theme button, search, the
// version menu, Copy, and Run (which compiles the example in your browser).
(function () {
  'use strict';
  /**
   * @typedef {{ l: Element, b: Element, user: boolean | null, seen: boolean, open: boolean }} Group
   * An outline list that folds, with its toggle button.
   */
  /**
   * @typedef {{ a: HTMLAnchorElement, h: HTMLElement, g?: Group | null, item?: boolean }} Mark
   * An outline entry and the heading it points at.
   */
  /** @typedef {[string[], string[]]} Shape A signature as parameter types and return types. */
  /** @typedef {[number, string, string, string, string, Shape[], number, string?]} Row One item of search-index.json. */
  /**
   * @typedef {{
   *   module: string, title: string, name: string, kind: string, summary: string, sig: string,
   *   shapes: Shape[], experimental: boolean, native: boolean, cat: number, folder?: string,
   *   lname: string, lpath: string, lsum: string,
   * }} Item  A search-index entry with the lowercase forms the search matches against.
   */
  /** @typedef {{ head: string, middle: string, tail: string, wrapped?: boolean, pad?: boolean }} Template */
  /** @typedef {{ run: Template, open?: Template, playground: string }} Parts An example's program. */
  /**
   * @typedef {{ result: { stdout?: string, stderr?: string, exitCode: number | null,
   *   diagnostics?: { severity: string }[], rendered?: string }, ms: number }} Ran
   * What the compile worker answers.
   */

  var root = document.documentElement;
  var base = document.body.getAttribute('data-base') || '/';

  // The sidebar and the outline stick below the header, which is taller when it wraps.
  var header = /** @type {Element} */ (document.querySelector('.top'));
  if (header && window.ResizeObserver) {
    new ResizeObserver(function () {
      root.style.setProperty('--header-offset', header.getBoundingClientRect().height + 'px');
    }).observe(header);
  }

  // Pages have no trailing slash. An old link that has one lands on the 404 page, which sends it on.
  if (document.body.getAttribute('data-page') === '404.html') {
    var old = location.pathname;
    if (old.length > base.length && old.slice(-1) === '/' && old.indexOf(base) === 0) {
      location.replace(old.slice(0, -1) + location.search + location.hash);
      return;
    }
  }

  // Theme: system, light or dark, remembered in this browser.
  var KEY = 'jacket-docs-theme';
  var ORDER = ['system', 'light', 'dark'];
  var themeButton = document.getElementById('theme-toggle');
  function theme() {
    return root.getAttribute('data-theme') || 'system';
  }
  function labelTheme() {
    if (!themeButton) return;
    var text = 'Theme: ' + theme() + ' (click to change)';
    themeButton.setAttribute('aria-label', text);
    themeButton.title = text;
    var icon = document.querySelector('#theme-icon use');
    if (icon)
      icon.setAttribute(
        'href',
        /** @type {string} */ (icon.getAttribute('href')).split('#')[0] + '#' + theme(),
      );
  }
  if (themeButton) {
    labelTheme();
    themeButton.addEventListener('click', function () {
      var next = ORDER[(ORDER.indexOf(theme()) + 1) % ORDER.length];
      root.setAttribute('data-theme', next);
      try {
        if (next === 'system') localStorage.removeItem(KEY);
        else localStorage.setItem(KEY, next);
      } catch {
        /* private window or blocked storage: the choice lasts for this page only */
      }
      labelTheme();
    });
  }

  // Markup sets the viewport state before parsing the navigation; the blocking script after
  // it restores groups and scroll. Keep later reader changes when this deferred script arrives.
  var side = /** @type {HTMLDetailsElement} */ (document.querySelector('details.side'));
  if (side) {
    window.addEventListener('pagehide', function () {
      try {
        var nav = side.querySelector('nav');
        var open = /** @type {string[]} */ (
          Array.prototype.map.call(side.querySelectorAll('details.nav-cat[open]'), function (d) {
            return d.getAttribute('data-key');
          })
        );
        sessionStorage.setItem(
          'jacket-docs-side',
          JSON.stringify({ k: nav && nav.getAttribute('data-nav'), t: side.scrollTop, o: open }),
        );
      } catch {
        /* storage is blocked: the sidebar starts fresh */
      }
    });
    // The menu is closed on a phone and always open beside the page, so a closed one must not
    // survive a resize to the wide layout, where nothing could open it again. A phone remembers
    // what the reader chose there.
    var wideQuery = matchMedia('(min-width: 900px)');
    var phoneOpen = side.open && !wideQuery.matches;
    side.addEventListener('toggle', function () {
      if (!wideQuery.matches) phoneOpen = side.open;
    });
    wideQuery.addEventListener('change', function (e) {
      side.open = e.matches || phoneOpen;
    });
  }

  function sideGroups() {
    Array.prototype.forEach.call(document.querySelectorAll('.nav-cat'), function (d, i) {
      var summary = d.querySelector('summary'),
        list = d.querySelector('.nav-list');
      list.id = 'nav-group-' + i;
      summary.setAttribute('aria-controls', list.id);
      function state() {
        summary.setAttribute('aria-expanded', String(d.open));
      }
      d.addEventListener('toggle', state);
      state();
    });
  }
  sideGroups();

  // The outline on the right marks every item on screen. Its item lists fold like the
  // sidebar's groups: lists with a section on screen are open, the rest closed, and a list
  // the reader opened stays open. Without JS every list is open. `signal` ends its listeners
  // when another page is swapped in.
  /** @param {AbortSignal} signal */
  function outline(signal) {
    var toc = document.querySelectorAll('.toc a[href^="#"]');
    if (toc.length) {
      var marks = /** @type {Mark[]} */ (
        Array.prototype.map.call(toc, function (/** @type {HTMLAnchorElement} */ a) {
          return {
            a: a,
            h: /** @type {HTMLElement} */ (
              document.getElementById(/** @type {string} */ (a.getAttribute('href')).slice(1))
            ),
          };
        })
      ).filter(function (m) {
        return m.h;
      });
      // Each mark knows its group, so a scroll touches only what is on screen, never the whole
      // outline: big modules have thousands of entries.
      /** @type {Group[]} */
      var groups = [];
      /** @param {Group} g */
      var setOpen = function (g) {
        var open = g.user === null ? g.seen : g.user;
        if (open === g.open) return;
        g.open = open;
        g.l.classList.toggle('open', open);
        g.b.setAttribute('aria-expanded', String(open));
        moved = true;
      };
      /** @type {Map<Element, Group>} */
      var groupOf = new Map();
      Array.prototype.forEach.call(document.querySelectorAll('.toc li.items'), function (l) {
        var head = l.previousElementSibling;
        if (!head || head.classList.contains('items')) return;
        var list = l.querySelector('ul');
        // early.js added the button and opened the lists on screen.
        var b = head.querySelector('.tog');
        if (!b) return;
        var open = l.classList.contains('open');
        /** @type {Group} */
        var g = { l: l, b: b, user: null, seen: open, open: open };
        b.addEventListener('click', function () {
          g.user = !g.open;
          setOpen(g);
        });
        groupOf.set(head, g);
        Array.prototype.forEach.call(list.children, function (li) {
          groupOf.set(li, g);
        });
        groups.push(g);
      });
      marks.forEach(function (m) {
        var li = /** @type {HTMLLIElement} */ (m.a.closest('li'));
        m.g = groupOf.get(li) || null;
        // A heading with no list under it (a guide's outline) is an item too; a heading with a
        // list only opens it.
        m.item = !m.g || /** @type {Node} */ (li.parentNode).parentNode === m.g.l;
        if (m.item) m.a.setAttribute('data-t', m.a.textContent);
      });
      var rail = /** @type {HTMLElement} */ (document.querySelector('.toc'));
      var from = -1,
        to = -1,
        first = true,
        moved = false;
      var still = window.matchMedia('(prefers-reduced-motion: reduce)');
      var pending = false;
      // The first mark whose heading is below y (marks follow the page order).
      /** @param {number} y */
      var below = function (y) {
        var lo = 0,
          hi = marks.length;
        while (lo < hi) {
          var mid = (lo + hi) >> 1;
          if (marks[mid].h.getBoundingClientRect().top > y) hi = mid;
          else lo = mid + 1;
        }
        return lo;
      };
      // The rail scroll that puts entries a..b in its middle.
      /** @param {number} a @param {number} b */
      var centre = function (a, b) {
        var top = marks[a].a.getBoundingClientRect().top,
          end = marks[b].a.getBoundingClientRect().bottom;
        var r = rail.getBoundingClientRect();
        return Math.round(rail.scrollTop + (top + end) / 2 - (r.top + r.bottom) / 2);
      };
      // Keep the marked entries in the middle of the rail, gliding there as the page scrolls.
      /** @param {number} want */
      var glide = function (want) {
        if (Math.abs(want - rail.scrollTop) > 1)
          rail.scrollTo({ top: want, behavior: first || still.matches ? 'instant' : 'smooth' });
        first = false;
      };
      var mark = function () {
        pending = false;
        if (!marks.length) return;
        // Every section with some part on screen is marked. A section runs from its heading to
        // the next one in the outline; the sticky header covers the top 70px.
        var a = Math.max(0, below(70) - 1);
        var b = Math.max(a, below(window.innerHeight - 1) - 1);
        if (a === from && b === to) return;
        // Read where the rail should scroll before changing any class. Marking an entry never
        // moves the others, so these positions stay right, and the frame lays out only once.
        var want = centre(a, b);
        moved = false;
        /** @type {Group[]} */
        var seen = [];
        var i;
        for (i = from; i >= 0 && i <= to; i++) {
          if (i < a || i > b) {
            marks[i].a.classList.remove('active');
            var gone = marks[i].g;
            if (gone) gone.seen = false;
          }
        }
        if (from >= 0) marks[from].a.removeAttribute('aria-current');
        for (i = a; i <= b; i++) {
          // Only item names are coloured; headings just open their lists.
          if (marks[i].item) marks[i].a.classList.add('active');
          var shown = marks[i].g;
          if (shown) {
            shown.seen = true;
            seen.push(shown);
          }
        }
        marks[a].a.setAttribute('aria-current', 'location');
        for (i = from; i >= 0 && i <= to; i++) {
          var g = marks[i].g;
          // A list the reader closed opens again once they have moved on and come back.
          if (g && !g.seen && g.user === false) g.user = null;
          if (g) setOpen(g);
        }
        seen.forEach(setOpen);
        from = a;
        to = b;
        // Animate folding only after the first layout, so a page load doesn't show lists closing.
        if (!rail.classList.contains('ready'))
          requestAnimationFrame(function () {
            rail.classList.add('ready');
          });
        // A list opening or closing does move entries, so then the rail is placed a frame later.
        if (moved)
          requestAnimationFrame(function () {
            glide(centre(from, to));
          });
        else glide(want);
      };
      window.addEventListener(
        'scroll',
        function () {
          if (!pending) {
            pending = true;
            requestAnimationFrame(mark);
          }
        },
        { passive: true, signal: signal },
      );
      mark();
    }
  }

  // Search.
  var input = /** @type {HTMLInputElement} */ (document.getElementById('search-q'));
  var box = /** @type {HTMLElement} */ (document.getElementById('search-results'));
  /** @type {Item[] | null} */
  var items = null;
  /** @type {string[][]} */
  var cats = [];
  /** @type {Promise<Item[]> | null} */
  var loading = null;
  /** @type {Item[]} */
  var hits = [];
  var picked = -1;
  // Which search is current; a lookup that finishes after the box closed or the text changed is stale.
  var current = 0;
  /** @type {Record<string, boolean>} */
  var BUILTIN = {};
  'int float float32 float64 bool string void Type Any s8 s16 s32 s64 u8 u16 u32 u64 s128 u128 code Code'
    .split(' ')
    .forEach(function (n) {
      BUILTIN[n] = true;
    });

  /** @param {string} cls @param {string} label */
  function dot(cls, label) {
    var d = el('span', 'dot ' + cls, '');
    d.title = label;
    d.setAttribute('role', 'img');
    d.setAttribute('aria-label', label);
    d.appendChild(el('span', 'sr', label));
    return d;
  }
  /** @param {Item} it */
  function hrefOf(it) {
    if (it.kind === 'module') return base + 'api/' + it.module;
    if (it.folder) return base + 'api/' + it.module + '/' + it.folder;
    return (
      base +
      'api/' +
      it.module +
      '/' +
      it.name.replace(/[^A-Za-z0-9_]/g, function (c) {
        return '-' + ('0' + c.charCodeAt(0).toString(16)).slice(-2);
      })
    );
  }
  function load() {
    if (!loading) {
      loading = fetch(base + 'search-index.json')
        .then(function (r) {
          return r.json();
        })
        .then(function (/** @type {{ cats?: string[][], modules: string[], items: Row[] }} */ d) {
          cats = d.cats || [];
          items = d.items.map(function (/** @type {Row} */ r) {
            var module = d.modules[r[0]];
            var title = /** @type {string} */ (module.split('/').pop());
            return {
              module: module,
              title: title,
              name: r[1],
              kind: r[2],
              summary: r[3],
              sig: r[4],
              shapes: r[5],
              experimental: (r[6] & 1) === 1,
              native: (r[6] & 2) === 2,
              cat: (r[6] >> 2) - 1,
              folder: r[7],
              lname: r[1].toLowerCase(),
              lpath: (title + '.' + r[1]).toLowerCase(),
              lsum: r[3].toLowerCase(),
            };
          });
          return /** @type {Item[]} */ (items);
        });
    }
    return loading;
  }
  /** @param {string} type */
  function norm(type) {
    return type
      .replace(/^[a-z_][a-z0-9_]*\s*:\s*/i, '')
      .replace(/\s+/g, '')
      .toLowerCase();
  }
  /** @param {string} text */
  function typesOf(text) {
    text = text.trim().replace(/^\(/, '').replace(/\)$/, '');
    if (!text) return [];
    return text.split(',').map(norm).filter(Boolean);
  }
  /** @param {string[]} have @param {string[]} wanted */
  function contains(have, wanted) {
    var rest = have.slice();
    return wanted.every(function (w) {
      var at = rest.indexOf(w);
      if (at < 0) return false;
      rest.splice(at, 1);
      return true;
    });
  }
  /** @param {string} query */
  function bySignature(query) {
    var cut = query.indexOf('->');
    var want = typesOf(query.slice(0, cut));
    var ret = typesOf(query.slice(cut + 2));
    /** @type {{ it: Item, score: number }[]} */
    var out = [];
    /** @type {Item[]} */ (items).forEach(function (it) {
      if (it.kind !== 'proc') return;
      var best = -1;
      it.shapes.forEach(function (s) {
        if (!contains(s[0], want) || !contains(s[1], ret)) return;
        var exact =
          (want.length === 0 || s[0].length === want.length ? 1 : 0) +
          (ret.length === 0 || s[1].length === ret.length ? 1 : 0);
        best = Math.max(best, exact);
      });
      if (best >= 0) out.push({ it: it, score: best });
    });
    out.sort(function (a, b) {
      return (
        (a.it.native ? 1 : 0) - (b.it.native ? 1 : 0) ||
        b.score - a.score ||
        a.it.name.length - b.it.name.length ||
        (a.it.lpath < b.it.lpath ? -1 : 1)
      );
    });
    return out.map(function (o) {
      return o.it;
    });
  }
  /** @param {string} query */
  function byName(query) {
    var terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    /** @type {{ it: Item, score: number }[]} */
    var out = [];
    /** @type {Item[]} */ (items).forEach(function (it) {
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
      // Native bindings come after the library's own API that matches as well.
      if (it.native) score = Math.floor(score / 4);
      out.push({ it: it, score: score });
    });
    out.sort(function (a, b) {
      return (
        b.score - a.score ||
        a.it.name.length - b.it.name.length ||
        (a.it.lpath < b.it.lpath ? -1 : 1)
      );
    });
    return out.map(function (o) {
      return o.it;
    });
  }
  /**
   * The nearest element at or above an event's target that matches `selector`.
   * @param {Event} e
   * @param {string} selector
   */
  function ancestor(e, selector) {
    return e.target instanceof Element ? e.target.closest(selector) : null;
  }
  /**
   * @template {keyof HTMLElementTagNameMap} K
   * @param {K} tag
   * @param {string} [cls]
   * @param {string} [text]
   * @returns {HTMLElementTagNameMap[K]}
   */
  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
  }
  // A procedure's signature coloured like the one on its page: parameter names, builtin types,
  // and the library's types as links to the declarations the compiler resolved them to. The index
  // writes a link as \u0001Module/item\u0002Name\u0003.
  /** @param {Item} it */
  function sigNodes(it) {
    var out = document.createDocumentFragment();
    var text = it.sig;
    if (it.kind !== 'proc') {
      out.appendChild(document.createTextNode(text));
      return out;
    }
    // oxlint-disable no-control-regex -- the index marks a link with U+0001, U+0002 and U+0003
    var re =
      /\u0001([^\u0002]*)\u0002([^\u0003]*)\u0003|"(?:[^"\\]|\\.)*"|[A-Za-z_][A-Za-z0-9_]*|\s+|[\s\S]/g;
    // oxlint-enable no-control-regex
    // Polymorphic names (`$T`) are types wherever they appear in the signature.
    /** @type {Record<string, boolean>} */
    var poly = {};
    text.replace(/\$([A-Za-z_][A-Za-z0-9_]*)/g, function (_, /** @type {string} */ n) {
      poly[n] = true;
      return _;
    });
    var m,
      prev = '',
      plain = '';
    function flush() {
      if (plain) {
        out.appendChild(document.createTextNode(plain));
        plain = '';
      }
    }
    while ((m = re.exec(text))) {
      var t = m[0];
      if (m[1] !== undefined) {
        flush();
        var a = el('span', 'ty', m[2]);
        out.appendChild(a);
        prev = m[2];
        continue;
      }
      if (/^[A-Za-z_]/.test(t) && prev !== '.') {
        var named =
          /^\s*:/.test(text.slice(re.lastIndex)) && (prev === '(' || prev === ',' || prev === '$');
        var cls = named ? 'pname' : BUILTIN[t] || poly[t] ? 'ty' : '';
        if (cls) {
          flush();
          out.appendChild(el('span', cls, t));
        } else plain += t;
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
      box.appendChild(el('div', 'empty', 'No matches.')).setAttribute('role', 'presentation');
    }
    hits.forEach(function (it, i) {
      var a = el('div', 'hit');
      a.id = 'hit-' + i;
      a.setAttribute('role', 'option');
      a.setAttribute('aria-selected', i === picked ? 'true' : 'false');
      var name = el('span', 'path', it.kind === 'module' ? it.title : it.title + '.' + it.name);
      a.setAttribute(
        'aria-label',
        (it.kind === 'module' ? it.title : it.title + '.' + it.name) + ': ' + it.summary,
      );
      a.appendChild(el('code', '', '')).appendChild(name);
      // Flags are dots here, coloured like the badges on the pages; the word is the tooltip.
      if (it.native) a.appendChild(dot('foreign', 'Foreign: a binding to a native library'));
      if (it.cat >= 0 && cats[it.cat]) a.appendChild(dot('cat-' + it.cat, cats[it.cat][1]));
      if (it.experimental) a.appendChild(dot('exp', 'Experimental: may change or go away'));
      // The signature column already says what procedures, types and constants are.
      if (it.kind === 'module') a.appendChild(el('span', 'what', 'module'));
      var sig = el('span', 'sig');
      sig.appendChild(sigNodes(it));
      a.appendChild(sig);
      box.appendChild(a);
    });
    var tips = el('div', 'tips');
    ['↑↓ to move', '↵ to open', 'type -> bool to search by return type'].forEach(function (t) {
      tips.appendChild(el('span', '', t));
    });
    tips.setAttribute('role', 'presentation');
    box.appendChild(tips);
    box.hidden = false;
    input.setAttribute('aria-expanded', 'true');
    if (picked >= 0) input.setAttribute('aria-activedescendant', 'hit-' + picked);
    else input.removeAttribute('aria-activedescendant');
  }
  function hide() {
    current++;
    box.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
  }
  function update() {
    var q = input.value.trim();
    var mine = ++current;
    if (!q) {
      hits = [];
      picked = -1;
      hide();
      return;
    }
    load().then(function () {
      if (mine !== current) return;
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
        if (picked >= 0 && hits[picked]) {
          e.preventDefault();
          window.location.href = hrefOf(hits[picked]);
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        hide();
      }
    });
    box.addEventListener('click', function (e) {
      var row = ancestor(e, '.hit');
      if (row && !ancestor(e, 'a') && hits[+row.id.slice(4)])
        window.location.href = hrefOf(hits[+row.id.slice(4)]);
    });
    document.addEventListener('click', function (e) {
      if (!box.contains(/** @type {Node | null} */ (e.target)) && e.target !== input) hide();
    });
    document.addEventListener('keydown', function (e) {
      if (e.key !== '/' || e.ctrlKey || e.metaKey || e.altKey) return;
      var t = /** @type {HTMLElement | null} */ (e.target);
      if (
        t &&
        (t.tagName === 'INPUT' ||
          t.tagName === 'TEXTAREA' ||
          t.tagName === 'SELECT' ||
          t.isContentEditable)
      )
        return;
      e.preventDefault();
      input.focus();
      input.select();
    });
  }

  // Copy and Run.
  /** @type {Worker | null} */
  var worker = null;
  /** @type {Promise<{ engine: string, wasm: string }> | null} */
  var runtime = null;
  /** @type {Record<string, Promise<Parts>>} */
  var texts = {};
  // What an editable example holds, by its figure.
  /** @type {WeakMap<Element, () => string>} */
  var editors = new WeakMap();
  function runtimeUrls() {
    if (!runtime) {
      runtime = fetch(base + 'runtime.json')
        .then(function (r) {
          return r.json();
        })
        .then(function (c) {
          return {
            engine: new URL(c.engine, new URL(base, location.href).href).href,
            wasm: new URL(c.wasm, new URL(base, location.href).href).href,
          };
        });
    }
    return runtime;
  }
  // An example's program is stored as the parts around its text (examples/<id>.json), so the
  // text a reader edited can be put back in. `fill` is `Template::fill` of doctest.rs.
  /** @param {string} id */
  function parts(id) {
    if (!texts[id])
      texts[id] = fetch(base + 'examples/' + id + '.json').then(function (r) {
        return r.json();
      });
    return texts[id];
  }
  /** @param {Template} t @param {string} code */
  function fill(t, code) {
    var lines = code.replace(/\r/g, '').split('\n');
    if (lines[lines.length - 1] === '') lines.pop();
    var out = t.head;
    if (t.wrapped) {
      /** @type {string[]} */
      var body = [];
      lines.forEach(function (l) {
        if (/^\s*#(import|load)/.test(l)) out += l.replace(/^\s+/, '') + '\n';
        else body.push(l);
      });
      out += t.middle;
      body.forEach(function (l) {
        out += (l !== '' || t.pad ? '    ' : '') + l + '\n';
      });
    } else {
      lines.forEach(function (l) {
        out += l + '\n';
      });
      out += t.middle;
    }
    return out + t.tail;
  }
  /** @param {string} text */
  function base64url(text) {
    var bytes = new TextEncoder().encode(text),
      bin = '';
    for (var i = 0; i < bytes.length; i += 0x8000)
      bin += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + 0x8000)));
    return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }

  // Editable examples: a textarea sits over the highlighted code and shows through it. The code
  // is coloured again as you type, by the same rules as highlight.rs (minus the links).
  /** @type {Record<string, string>} */
  var KW = {};
  /** @type {Record<string, string>} */
  var TY = {};
  'if else then for while return defer using struct enum enum_flags union cast xx inline break continue case remove true false null ifx push_context context operator interface'
    .split(' ')
    .forEach(function (w) {
      KW[w] = 'k';
    });
  'int float float32 float64 bool string void Type Any s8 s16 s32 s64 u8 u16 u32 u64 u128 s128 assert'
    .split(' ')
    .forEach(function (w) {
      TY[w] = 't';
    });
  /** @param {string} s */
  function escapeHtml(s) {
    return s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }
  /** @param {string} code */
  function highlight(code) {
    var re =
      /\/\/[^\n]*|"(?:[^"\\]|\\[\s\S])*"?|#[A-Za-z_][A-Za-z0-9_]*|\d(?:[A-Za-z0-9_]|\.(?!\.))*|[A-Za-z_][A-Za-z0-9_]*|[\s\S]/g;
    return code.replace(re, function (m) {
      var cls =
        m.charAt(0) === '/' && m.charAt(1) === '/'
          ? 'c'
          : m.charAt(0) === '"'
            ? 's'
            : m.charAt(0) === '#' && m.length > 1
              ? 'd'
              : /^\d/.test(m)
                ? 'n'
                : KW[m] || TY[m];
      return cls ? '<span class="' + cls + '">' + escapeHtml(m) + '</span>' : escapeHtml(m);
    });
  }
  /** @param {Element} figure */
  function editable(figure) {
    var runButton = /** @type {HTMLButtonElement} */ (
      figure.querySelector('button[data-act="run"][data-program]')
    );
    var pre = /** @type {HTMLElement} */ (figure.querySelector('pre.code'));
    var wrap = /** @type {HTMLElement} */ (pre && pre.parentElement);
    if (
      !runButton ||
      !pre ||
      !wrap ||
      !wrap.classList.contains('ed') ||
      wrap.querySelector('textarea')
    )
      return;
    var code = /** @type {HTMLElement} */ (pre.querySelector('code'));
    var original = pre.textContent;
    var originalHtml = code.innerHTML;
    var reset = /** @type {HTMLButtonElement | null} */ (
      figure.querySelector('button[data-act="reset"]')
    );
    var open = /** @type {HTMLAnchorElement | null} */ (figure.querySelector('a.btn[href]'));
    var openHref = open && open.getAttribute('href');
    var area = el('textarea');
    area.value = original;
    area.spellcheck = false;
    area.wrap = 'off';
    area.setAttribute('autocapitalize', 'off');
    area.setAttribute('autocomplete', 'off');
    area.setAttribute('aria-label', 'Example code, editable');
    wrap.appendChild(area);
    var escaped = false;
    function changed() {
      var text = area.value;
      var edited = text !== original;
      code.innerHTML = edited ? highlight(text) + (text.endsWith('\n') ? ' ' : '') : originalHtml;
      if (reset) reset.hidden = !edited;
      if (open) {
        var link = open;
        if (!edited) link.setAttribute('href', /** @type {string} */ (openHref));
        else
          parts(/** @type {string} */ (runButton.getAttribute('data-program'))).then(function (p) {
            if (p.open && area.value === text)
              link.setAttribute('href', p.playground + '#code=' + base64url(fill(p.open, text)));
          });
      }
    }
    /** @param {string} text */
    function insert(text) {
      area.focus();
      if (!document.execCommand || !document.execCommand('insertText', false, text)) {
        area.setRangeText(text, area.selectionStart, area.selectionEnd, 'end');
        changed();
      }
    }
    area.addEventListener('input', changed);
    // On phones the textarea is drawn scaled (site.css, --k), so its scroll offsets are too.
    area.addEventListener('scroll', function () {
      var k = parseFloat(getComputedStyle(wrap).getPropertyValue('--k')) || 1;
      pre.scrollLeft = area.scrollLeft * k;
      pre.scrollTop = area.scrollTop * k;
    });
    area.addEventListener('keydown', function (e) {
      var plain = !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey;
      if (e.key === 'Tab' && plain && !escaped) {
        e.preventDefault();
        insert('    ');
      } else if (e.key === 'Enter' && plain) {
        // Keep the indentation of the line you were on.
        var before = area.value.slice(0, area.selectionStart);
        var indent = /** @type {RegExpExecArray} */ (
          /^[ \t]*/.exec(before.slice(before.lastIndexOf('\n') + 1))
        )[0];
        e.preventDefault();
        insert('\n' + indent);
      } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        if (!runButton.disabled) run(runButton);
      }
      // Escape lets the next Tab leave the box, so the keyboard is never stuck in it.
      escaped = e.key === 'Escape';
    });
    area.addEventListener('blur', function () {
      escaped = false;
    });
    if (reset)
      reset.addEventListener('click', function () {
        area.value = original;
        changed();
        area.focus();
      });
    editors.set(figure, function () {
      return area.value;
    });
  }
  /**
   * @param {{ program: string, engine: string, wasm: string }} request
   * @param {number} limit Milliseconds before the worker is stopped.
   * @returns {Promise<Ran>}
   */
  function ask(request, limit) {
    return new Promise(function (resolve, reject) {
      if (!worker) worker = new Worker(base + 'assets/run-worker.js', { type: 'module' });
      var w = worker;
      var timer = setTimeout(function () {
        w.terminate();
        if (worker === w) worker = null;
        reject(new Error('Stopped after ' + limit / 1000 + ' seconds.'));
      }, limit);
      w.addEventListener('message', function (e) {
        clearTimeout(timer);
        if (e.data.error) {
          if (e.data.crashed && worker === w) {
            w.terminate();
            worker = null;
          }
          reject(new Error(e.data.error));
        } else {
          resolve(e.data);
        }
      });
      w.addEventListener('error', function (e) {
        clearTimeout(timer);
        if (worker === w) worker = null;
        reject(new Error(e.message || 'The compiler could not start in this browser.'));
      });
      w.postMessage(request);
    });
  }
  /** @param {HTMLElement} out @param {string} state @param {string} status @param {string} [text] */
  function report(out, state, status, text) {
    out.hidden = false;
    out.className = 'ex-out' + (state ? ' ' + state : '');
    out.textContent = '';
    out.appendChild(el('div', '', status));
    if (text) out.appendChild(el('pre', '', text));
  }
  /**
   * The code in an example as it is now, with any edits.
   * @param {Element} figure
   */
  function exampleText(figure) {
    var edited = editors.get(figure);
    return edited
      ? edited()
      : /** @type {HTMLElement} */ (figure.querySelector('pre.code')).textContent;
  }
  /** @param {HTMLButtonElement} button */
  function run(button) {
    var figure = /** @type {HTMLElement} */ (button.closest('figure'));
    var out = /** @type {HTMLElement} */ (figure.querySelector('.ex-out'));
    var id = /** @type {string} */ (button.getAttribute('data-program'));
    button.disabled = true;
    report(
      out,
      'busy',
      worker ? 'Compiling...' : 'Loading the compiler (the first run takes a moment)...',
    );
    var text = exampleText(figure);
    Promise.all([parts(id), runtimeUrls()])
      .then(function (got) {
        return ask(
          { program: fill(got[0].run, text), engine: got[1].engine, wasm: got[1].wasm },
          60000,
        );
      })
      .then(function (done) {
        var r = done.result;
        var ms = Math.max(1, Math.round(done.ms));
        var printed = (r.stdout || '') + (r.stderr || '');
        if (
          r.exitCode === 0 &&
          !(r.diagnostics || []).some(function (d) {
            return d.severity === 'error';
          })
        ) {
          report(out, '', '✓ Ran in ' + ms + ' ms', printed.trim() ? printed : '');
        } else {
          var why =
            r.rendered ||
            printed ||
            (r.exitCode !== null
              ? 'The example exited with status ' + r.exitCode + '.'
              : 'The example did not compile.');
          report(out, 'bad', '✗ Failed', why.trim());
        }
      })
      .catch(function (error) {
        report(out, 'bad', '✗ Could not run', String(error.message || error));
      })
      .then(function () {
        button.disabled = false;
      });
  }
  /** @param {HTMLButtonElement} button */
  function copy(button) {
    var figure = /** @type {HTMLElement} */ (button.closest('figure'));
    var text = exampleText(figure);
    var done = function () {
      var label = button.textContent;
      button.textContent = 'Copied';
      setTimeout(function () {
        button.textContent = label;
      }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done, function () {});
    } else {
      var area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      try {
        document.execCommand('copy');
        done();
      } catch {
        /* nothing more to try */
      }
      document.body.removeChild(area);
    }
  }
  document.addEventListener('click', function (e) {
    var button = /** @type {HTMLButtonElement | null} */ (ancestor(e, 'button[data-act]'));
    if (!button || button.disabled) return;
    if (button.getAttribute('data-act') === 'run') run(button);
    else if (button.getAttribute('data-act') === 'copy') copy(button);
  });

  // Clicking anywhere on an item card opens the item, except on links, buttons, examples and when selecting text.
  /** @param {MouseEvent} e */
  function cardLink(e) {
    var t = e.target;
    if (!(t instanceof Element) || t.closest('a, button, summary, input, textarea, .ex'))
      return null;
    var row = t.closest('.row');
    var name = /** @type {HTMLAnchorElement | null} */ (
      row && row.querySelector('.row-head a.name')
    );
    if (!row || !name) return null;
    var sel = window.getSelection && window.getSelection();
    if (sel && !sel.isCollapsed && row.contains(sel.anchorNode)) return null;
    return name;
  }
  document.addEventListener('click', function (e) {
    if (e.button !== 0) return;
    var name = cardLink(e);
    if (!name) return;
    if (e.metaKey || e.ctrlKey || e.shiftKey) window.open(name.href, '_blank');
    else name.click();
  });
  document.addEventListener('auxclick', function (e) {
    var name = e.button === 1 && cardLink(e);
    if (name) window.open(name.href, '_blank');
  });

  // Clicking a field's line in a declaration links to it.
  document.addEventListener('click', function (e) {
    var line = ancestor(e, '.decl .fl');
    if (!line || ancestor(e, 'a') || String(getSelection()) || location.hash === '#' + line.id)
      return;
    location.hash = line.id;
  });

  // A link into folded members opens them and scrolls there.
  function openTarget() {
    var id = decodeURIComponent(location.hash.slice(1));
    var target = id ? document.getElementById(id) : null;
    if (id && !target) {
      // A section split into pages keeps its old `#heading` links: the overview lists each page.
      var page = /** @type {HTMLAnchorElement | null} */ (
        document.querySelector('main a[href$="/' + id.replace(/["\\]/g, '') + '"]')
      );
      if (page && page.closest('li')) {
        location.replace(page.href);
        return;
      }
    }
    if (!target) return;
    var opened = false;
    /** @type {HTMLElement | null} */
    var node;
    for (node = target; node; node = node.parentElement) {
      if (node instanceof HTMLDetailsElement && !node.open) {
        node.open = true;
        opened = true;
      }
      if (node.classList.contains('fold') && !node.classList.contains('open')) {
        node.classList.add('open');
        if (node.nextElementSibling && node.nextElementSibling.classList.contains('fold-btn'))
          node.nextElementSibling.remove();
        opened = true;
      }
    }
    if (opened)
      target.scrollIntoView({
        behavior: matchMedia('(prefers-reduced-motion: no-preference)').matches
          ? 'smooth'
          : 'instant',
      });
  }
  window.addEventListener('hashchange', openTarget);

  // Version menu. versions.json is [{"version": "0.31.0", "url": "../0.31.0/"}, ...]; a relative url is read from versions.json's folder.
  (function () {
    var btn = /** @type {HTMLButtonElement} */ (document.getElementById('ver-button')),
      menu = /** @type {HTMLElement} */ (document.getElementById('ver-menu'));
    if (!btn || !menu) return;
    var listUrl = new URL(base + 'versions.json', location.href);
    fetch(listUrl)
      .then(function (r) {
        return r.ok ? r.json() : [];
      })
      .then(function (list) {
        if (!Array.isArray(list) || list.length < 2) return;
        var shown = btn.textContent.trim();
        list.forEach(function (v) {
          var a = document.createElement('a');
          a.href = new URL(v.url, listUrl).href;
          a.textContent = v.version;
          a.setAttribute('role', 'menuitem');
          a.tabIndex = -1;
          if (v.version === shown) a.setAttribute('aria-current', 'true');
          menu.appendChild(a);
        });
        btn.disabled = false;
      })
      .catch(function () {});
    /** @param {boolean} open */
    function set(open) {
      menu.hidden = !open;
      btn.setAttribute('aria-expanded', open ? 'true' : 'false');
    }
    /** @param {boolean} last */
    function openAt(last) {
      set(true);
      var links = menu.querySelectorAll('a');
      if (links.length) links[last ? links.length - 1 : 0].focus();
    }
    btn.addEventListener('click', function (e) {
      e.stopPropagation();
      if (menu.hidden) openAt(false);
      else set(false);
    });
    btn.addEventListener('keydown', function (e) {
      if (['ArrowDown', 'ArrowUp', 'Enter', ' '].indexOf(e.key) < 0) return;
      e.preventDefault();
      openAt(e.key === 'ArrowUp');
    });
    menu.addEventListener('keydown', function (e) {
      if (e.key === ' ') {
        e.preventDefault();
        /** @type {HTMLElement} */ (document.activeElement).click();
        return;
      }
      var links = Array.prototype.slice.call(menu.querySelectorAll('a'));
      var i = links.indexOf(document.activeElement);
      if (e.key === 'Tab') {
        set(false);
        btn.focus();
        return;
      }
      if (['ArrowDown', 'ArrowUp', 'Home', 'End'].indexOf(e.key) < 0) return;
      e.preventDefault();
      var n =
        e.key === 'Home'
          ? 0
          : e.key === 'End'
            ? links.length - 1
            : (i + (e.key === 'ArrowUp' ? -1 : 1) + links.length) % links.length;
      links[n].focus();
    });
    document.addEventListener('click', function (e) {
      if (!menu.contains(/** @type {Node | null} */ (e.target))) set(false);
    });
    document.addEventListener('keydown', function (e) {
      if (e.key === 'Escape' && !menu.hidden) {
        set(false);
        btn.focus();
      }
    });
  })();

  // Content wider than its column scrolls on its own; give each overflowing region a tab
  // stop so the keyboard can scroll code, tables and field lists too.
  // All widths are read before any tab stop changes, so this lays the page out once.
  /** @type {ArrayLike<HTMLElement>} */
  var scrollRegions = [];
  var sizing = false;
  function scrollable() {
    sizing = false;
    var wide = Array.prototype.map.call(scrollRegions, function (region) {
      return region.scrollWidth > region.clientWidth;
    });
    Array.prototype.forEach.call(scrollRegions, function (region, i) {
      if (wide[i]) region.tabIndex = 0;
      else region.removeAttribute('tabindex');
    });
  }
  window.addEventListener('resize', function () {
    if (!sizing) {
      sizing = true;
      requestAnimationFrame(scrollable);
    }
  });

  // A long block shows its top and a Show more button. early.js has already opened the blocks
  // that fit; the button is in the HTML, so nothing moves when this runs. Focus inside a block,
  // from Tab or a search result, opens it too, so a focused link is never hidden.
  function folds() {
    Array.prototype.forEach.call(document.querySelectorAll('main .fold'), function (fold, i) {
      var btn = fold.nextElementSibling;
      if (!btn || !btn.classList.contains('fold-btn')) return;
      if (fold.classList.contains('open')) {
        btn.remove();
        return;
      }
      if (!fold.id) fold.id = 'fold-' + i;
      btn.setAttribute('aria-controls', fold.id);
      function open() {
        fold.classList.add('open');
        btn.remove();
      }
      btn.addEventListener('click', function () {
        // Keep keyboard focus in place of the removed button.
        fold.tabIndex = -1;
        open();
        fold.focus({ preventScroll: true });
      });
      fold.addEventListener('focusin', function () {
        if (!fold.classList.contains('open')) open();
      });
    });
  }

  // What each page sets up for its own content. It runs again when another page is swapped in.
  /** @type {AbortController | null} */
  var page = null;
  function setupPage() {
    if (page) page.abort();
    page = new AbortController();
    outline(page.signal);
    Array.prototype.forEach.call(document.querySelectorAll('figure.ex'), editable);
    scrollRegions = /** @type {NodeListOf<HTMLElement>} */ (
      document.querySelectorAll('main pre, main table, main .defs')
    );
    scrollable();
    folds();
    openTarget();
  }

  setupPage();

  // Moving between pages of the site fetches the next page and swaps in its content, so the
  // header and sidebar stay where they are. A link is fetched once the pointer rests on it, so
  // most clicks find the page ready. Anything unexpected falls back to a normal page load.
  /** @type {Map<string, Promise<Document>>} */
  var pages = new Map();
  /** @param {number} y */
  function jump(y) {
    window.scrollTo({ top: y, behavior: 'instant' });
  }
  var siteRoot = new URL(base, location.href);
  function remember() {
    try {
      history.replaceState({ y: window.scrollY }, '');
    } catch {
      /* too many calls: skip one */
    }
  }
  /** @param {string} href */
  function bare(href) {
    var u = new URL(href, location.href);
    u.hash = '';
    return u.href;
  }
  /** @param {HTMLAnchorElement | null} a */
  function inSite(a) {
    if (!a || !a.href || (a.target && a.target !== '_self') || a.hasAttribute('download'))
      return null;
    var u = new URL(a.href, location.href);
    if (u.origin !== siteRoot.origin || u.pathname.indexOf(siteRoot.pathname) !== 0) return null;
    if (!/(\/|\.html|\/[^/.]*)$/.test(u.pathname) || /\/(assets|runtime)\//.test(u.pathname))
      return null;
    return u;
  }
  /** @param {string} href */
  function fetchPage(href) {
    var key = bare(href);
    var got = pages.get(key);
    if (!got) {
      got = fetch(key)
        .then(function (r) {
          if (!r.ok || !/html/.test(r.headers.get('content-type') || ''))
            throw new Error('not a page');
          return r.text();
        })
        .then(function (text) {
          return new DOMParser().parseFromString(text, 'text/html');
        });
      got.catch(function () {
        pages.delete(key);
      });
      pages.set(key, got);
      // Keep the few most recent pages.
      if (pages.size > 40) pages.delete(/** @type {string} */ (pages.keys().next().value));
    }
    return got;
  }
  if (typeof window.fetch === 'function' && window.DOMParser && history.pushState) {
    history.scrollRestoration = 'manual';
    /** @type {number | undefined} */
    var rest;
    /** @param {Event} e */
    var hover = function (e) {
      var u = inSite(/** @type {HTMLAnchorElement | null} */ (ancestor(e, 'a[href]')));
      if (!u || bare(u.href) === bare(location.href)) return;
      var href = u.href;
      clearTimeout(rest);
      rest = setTimeout(
        function () {
          fetchPage(href);
        },
        e.type === 'pointerover' ? 65 : 0,
      );
    };
    document.addEventListener('pointerover', hover, { passive: true });
    document.addEventListener(
      'pointerout',
      function () {
        clearTimeout(rest);
      },
      { passive: true },
    );
    document.addEventListener('touchstart', hover, { passive: true });
    document.addEventListener('focusin', hover);

    /** @type {number | undefined} */
    var saving;
    window.addEventListener(
      'scroll',
      function () {
        clearTimeout(saving);
        saving = setTimeout(remember, 150);
      },
      { passive: true },
    );
    window.addEventListener('pagehide', remember);

    var going = 0;
    /** @param {string} href @param {boolean} push @param {boolean} keyboard */
    var go = function (href, push, keyboard) {
      var n = ++going;
      var slow = setTimeout(function () {
        root.classList.add('loading');
      }, 150);
      fetchPage(href)
        .then(function (doc) {
          if (n !== going) return;
          if (doc.body.getAttribute('data-base') !== document.body.getAttribute('data-base'))
            throw new Error('another site');
          if (push) {
            remember();
            history.pushState({ y: 0 }, '', href);
          }
          swap(doc);
          var id = decodeURIComponent(new URL(href).hash.slice(1));
          var target = id && document.getElementById(id);
          // A new page starts where it should be, without the smooth scroll the page has.
          if (!push && history.state) jump(history.state.y || 0);
          else if (target) target.scrollIntoView({ behavior: 'instant' });
          else jump(0);
          if (keyboard)
            /** @type {HTMLElement} */ (document.getElementById('main-content')).focus({
              preventScroll: true,
            });
        })
        .catch(function () {
          if (n === going) location.assign(href);
        })
        .then(function () {
          clearTimeout(slow);
          if (n === going) root.classList.remove('loading');
        });
    };
    document.addEventListener('click', function (e) {
      if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey)
        return;
      var u = inSite(/** @type {HTMLAnchorElement | null} */ (ancestor(e, 'a[href]')));
      if (!u) return;
      if (bare(u.href) === bare(location.href)) {
        // The page that is already open: a heading link scrolls as usual, the page itself goes
        // back to the top without loading again.
        if (u.hash) return;
        e.preventDefault();
        if (u.href !== location.href) history.pushState({ y: 0 }, '', u.href);
        window.scrollTo(0, 0);
        return;
      }
      e.preventDefault();
      go(u.href, true, e.detail === 0);
    });
    window.addEventListener('popstate', function () {
      // Back or forward within the page that is shown: only the scroll changes.
      if (document.body.getAttribute('data-shown') === bare(location.href)) {
        if (history.state) jump(history.state.y || 0);
        return;
      }
      go(location.href, false, false);
    });
    document.body.setAttribute('data-shown', bare(location.href));
  }

  // Puts `doc`'s page in place of this one, keeping the header and, where it is the same list,
  // the sidebar.
  /** @param {Document} doc */
  function swap(doc) {
    document.title = doc.title;
    ['meta[name="description"]', 'link[rel="canonical"]'].forEach(function (sel) {
      var mine = document.head.querySelector(sel),
        next = doc.head.querySelector(sel);
      if (mine && next) mine.replaceWith(document.importNode(next, true));
    });
    document.body.setAttribute('data-page', doc.body.getAttribute('data-page') || '');
    document.body.setAttribute('data-shown', bare(location.href));
    // The header's links stay; only which one is current changes.
    var here = doc.querySelectorAll('.site-nav > a');
    Array.prototype.forEach.call(document.querySelectorAll('.site-nav > a'), function (a, i) {
      var mark = here[i] && here[i].getAttribute('aria-current');
      if (mark) a.setAttribute('aria-current', mark);
      else a.removeAttribute('aria-current');
    });
    swapSide(doc);
    var main = /** @type {HTMLElement} */ (document.getElementById('main-content')),
      next = /** @type {HTMLElement} */ (doc.getElementById('main-content'));
    main.replaceWith(document.importNode(next, true));
    var wrap = document.querySelector('body > .wrap'),
      nextWrap = doc.querySelector('body > .wrap');
    if (wrap && nextWrap) wrap.toggleAttribute('data-toc', nextWrap.hasAttribute('data-toc'));
    var toc = document.querySelector('body > .wrap > .toc'),
      nextToc = doc.querySelector('body > .wrap > .toc');
    if (toc) toc.remove();
    if (nextToc)
      /** @type {HTMLElement} */ (document.getElementById('main-content')).after(
        document.importNode(nextToc, true),
      );
    // Inline scripts in the new content don't run; this does their work.
    window.jacketPage(false);
    setupPage();
  }

  // The next page's sidebar replaces this one in the same frame, keeping the groups the reader
  // opened and its scroll. A long group this one already has in full is moved across, so it never
  // shrinks back to the rows around the current link.
  /** @param {Document} doc */
  function swapSide(doc) {
    var rail = /** @type {HTMLDetailsElement | null} */ (document.querySelector('details.side')),
      nav = rail && rail.querySelector('nav');
    var next = doc.querySelector('details.side nav');
    if (!rail || !nav || !next) return;
    var loaded = nav;
    var open = /** @type {string[]} */ (
      Array.prototype.map.call(nav.querySelectorAll('details.nav-cat[open]'), function (d) {
        return d.getAttribute('data-key');
      })
    );
    var state = { k: nav.getAttribute('data-nav'), t: rail.scrollTop, o: open };
    next = document.importNode(next, true);
    Array.prototype.forEach.call(next.querySelectorAll('[data-source]'), function (list) {
      var source = list.getAttribute('data-source');
      var full = loaded.querySelector('[data-loaded="' + source + '"]');
      if (!full) return;
      var href = bare(location.href);
      Array.prototype.forEach.call(full.querySelectorAll('a'), function (a) {
        if (bare(a.href) === href) a.setAttribute('aria-current', 'page');
        else a.removeAttribute('aria-current');
      });
      list.replaceWith(full);
    });
    var label = doc.querySelector('details.side > summary');
    if (label)
      /** @type {HTMLElement} */ (rail.querySelector('summary')).textContent = label.textContent;
    nav.replaceWith(next);
    if (window.jacketSide) window.jacketSide(state);
    sideGroups();
  }
})();
