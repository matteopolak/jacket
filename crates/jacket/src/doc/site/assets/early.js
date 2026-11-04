// Inlined into every page, so what it does is in place before the first paint.
//
// A temporary head observer applies sidebar state as the parser adds nodes, before they paint.
// jacketSide finishes scroll restoration and disconnects it after the sidebar is complete.
// A long group shows only the rows around the current one until
// its full list arrives. site.js calls jacketSide again when it swaps in another page's sidebar.
//
// jacketPage runs after the content and the outline: it opens the folds that fit, puts a reload
// or Back back where it was, and opens the outline lists of the sections on screen. site.js
// calls it again for a swapped-in page.
(function () {
  function wide() { return matchMedia('(min-width:900px)').matches; }
  function reveal(s) {
    if (!wide()) return;
    var c = s.querySelector('[aria-current="page"]');
    if (!c) return;
    var a = c.getBoundingClientRect(), b = s.getBoundingClientRect();
    if (a.bottom <= b.top) s.scrollTop += a.top - b.top;
    else if (a.top >= b.bottom) s.scrollTop += a.bottom - b.bottom;
  }
  function setup(st, restoreGroups) {
    var s = document.querySelector('details.side');
    if (!s) return;
    var n = s.querySelector('nav'), k = n && n.getAttribute('data-nav');
    if (st && st.k !== k) st = null;
    if (restoreGroups && st) (st.o || []).forEach(function (key) {
      var l = s.querySelectorAll('details.nav-cat');
      for (var i = 0; i < l.length; i++) if (l[i].getAttribute('data-key') === key) l[i].open = true;
    });
    if (st && wide()) s.scrollTop = st.t;
    reveal(s);
    var list = s.querySelector('[data-source]');
    if (!list) return;
    var initialScroll = s.scrollTop;
    // A reader can scroll or focus the fallback before the request finishes.
    s.__moved = false;
    if (!s.__watched) {
      s.__watched = true;
      var moved = function () { s.__moved = true; };
      s.addEventListener('wheel', moved, { passive: true });
      s.addEventListener('touchstart', moved, { passive: true });
      s.addEventListener('keydown', moved);
    }
    var base = document.body.getAttribute('data-base') || '/';
    fetch(base + list.getAttribute('data-source')).then(function (response) {
      if (!response.ok) throw new Error('sidebar unavailable');
      return response.text();
    }).then(function (text) {
      if (!list.isConnected) return;
      // Measured while the placeholder padding still stands in for the missing rows.
      var current = list.querySelector('[aria-current="page"]');
      var top = current && current.getBoundingClientRect().top;
      var template = document.createElement('template');
      template.innerHTML = text;
      var rows = template.content.querySelectorAll('li');
      var start = +list.getAttribute('data-start'), end = +list.getAttribute('data-end');
      var first = +list.getAttribute('data-first'), count = list.children.length;
      var source = list.getAttribute('data-source');
      list.removeAttribute('data-source');
      if (rows.length < end) { list.removeAttribute('style'); return; }
      list.setAttribute('data-loaded', source);
      var before = document.createDocumentFragment(), after = document.createDocumentFragment();
      for (var i = start; i < end; i++) {
        if (i < first) before.appendChild(rows[i]);
        else if (i >= first + count) after.appendChild(rows[i]);
      }
      // Keep the existing nodes, including keyboard focus and aria-current.
      list.insertBefore(before, list.firstChild);
      list.appendChild(after);
      list.removeAttribute('style');
      if (wide()) {
        if (!s.__moved && s.scrollTop === initialScroll && st) s.scrollTop = st.t;
        else if (current) s.scrollTop += current.getBoundingClientRect().top - top;
        reveal(s);
      }
    }).catch(function () {
      // The inline window and back link remain usable.
      list.removeAttribute('style');
    });
  }

  // The footer is parsed after jacketPage, so a position near the bottom is cut short there.
  // jacketRestore runs once the page is whole and puts it right, unless the reader has moved.
  var restore = null;
  window.jacketRestore = function () {
    if (restore && Math.abs(restore.y - restore.at) > 1 && Math.abs(window.scrollY - restore.at) < 1) {
      window.scrollTo({ top: restore.y, behavior: 'instant' });
    }
    restore = null;
  };

  window.jacketPage = function (load) {
    // Measure every fold first, then change them, so the page lays out once.
    var shut = Array.prototype.slice.call(document.querySelectorAll('main .fold:not(.open)'));
    shut.filter(function (f) {
      return f.scrollHeight <= (+f.getAttribute('data-fold') || 440);
    }).forEach(function (f) { f.classList.add('open'); });
    // A fold that fits has no use for its button. Dropping it here, not when site.js loads, matters:
    // `.members:last-child` rules would otherwise change the block's padding a moment after paint.
    Array.prototype.forEach.call(document.querySelectorAll('main .fold.open + .fold-btn'), function (b) { b.remove(); });
    if (load) {
      var nav = performance.getEntriesByType && performance.getEntriesByType('navigation')[0];
      var y = history.state && history.state.y;
      if (y && nav && nav.type !== 'navigate') {
        window.scrollTo({ top: y, behavior: 'instant' });
        restore = { y: y, at: window.scrollY };
      }
    }
    tocLists();
  };
  // Adds the fold buttons to the outline and opens the lists of the sections on screen, the same
  // ones site.js keeps open while scrolling.
  function tocLists() {
    var toc = document.querySelector('.toc');
    if (!toc) return;
    var marks = [];
    Array.prototype.forEach.call(toc.querySelectorAll('li.items'), function (l, n) {
      var head = l.previousElementSibling;
      if (!head || head.classList.contains('items')) return;
      var list = l.querySelector('ul');
      list.id = 'outline-list-' + n;
      var b = document.createElement('button');
      b.type = 'button';
      b.className = 'tog';
      b.setAttribute('aria-controls', list.id);
      b.setAttribute('aria-expanded', 'false');
      b.setAttribute('aria-label', 'Show ' + head.textContent.trim());
      head.classList.add('has-list');
      head.appendChild(b);
    });
    Array.prototype.forEach.call(toc.querySelectorAll('a[href^="#"]'), function (a) {
      var h = document.getElementById(a.getAttribute('href').slice(1));
      if (h) marks.push({ a: a, top: h.getBoundingClientRect().top });
    });
    if (!marks.length) return;
    var a = 0, b = -1;
    marks.forEach(function (m, i) {
      if (m.top <= 70) a = i;
      if (m.top <= window.innerHeight - 1) b = i;
    });
    // The first section is current even before its heading enters the viewport, as in site.js.
    b = Math.max(a, b);
    for (var i = a; i <= b; i++) {
      var li = marks[i].a.closest('li');
      var l = li.classList.contains('has-list') ? li.nextElementSibling : li.closest('li.items');
      if (l && !l.classList.contains('open')) {
        l.classList.add('open');
        var t = l.previousElementSibling.querySelector('.tog');
        if (t) t.setAttribute('aria-expanded', 'true');
      }
    }
  }
  var initialState = null;
  try { initialState = JSON.parse(sessionStorage.getItem('jacket-docs-side') || 'null'); } catch (e) {}
  // Mutation notifications run before rendering, even if parsing yields before the next
  // inline script. Initialize each added node once rather than correcting a painted menu.
  function initialize(records) {
    records.forEach(function (record) {
      Array.prototype.forEach.call(record.addedNodes, function (node) {
        if (node.nodeType !== 1) return;
        if (node.matches('details.side') && !wide()) node.open = false;
        if (node.matches('details.nav-cat') && initialState) {
          var nav = node.closest('nav[data-nav]');
          if (nav && initialState.k === nav.getAttribute('data-nav') &&
              (initialState.o || []).indexOf(node.getAttribute('data-key')) !== -1) node.open = true;
        }
      });
    });
  }
  var parsing = new MutationObserver(initialize);
  parsing.observe(document.documentElement, { childList: true, subtree: true });
  window.jacketSide = function (state) {
    if (parsing) {
      initialize(parsing.takeRecords());
      parsing.disconnect(); parsing = null;
      setup(initialState, false);
    } else setup(state, true);
  };
})();
