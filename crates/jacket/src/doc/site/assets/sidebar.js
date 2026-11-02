// Inlined right after the sidebar, so the sidebar is put back before the first paint: its open
// groups and scroll from the last page, then the current link in view. A long group shows only
// the rows around the current one until its full list arrives. site.js calls jacketSide again
// when it swaps in another page's sidebar.
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
  function setup(st) {
    var s = document.querySelector('details.side');
    if (!s) return;
    var n = s.querySelector('nav'), k = n && n.getAttribute('data-nav');
    if (st && st.k !== k) st = null;
    if (st) (st.o || []).forEach(function (key) {
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
      var template = document.createElement('template');
      template.innerHTML = text;
      var rows = template.content.querySelectorAll('li');
      var start = +list.getAttribute('data-start'), end = +list.getAttribute('data-end');
      var first = +list.getAttribute('data-first'), count = list.children.length;
      var source = list.getAttribute('data-source');
      list.removeAttribute('data-source');
      if (rows.length < end) { list.removeAttribute('style'); return; }
      list.setAttribute('data-loaded', source);
      var current = list.querySelector('[aria-current="page"]');
      var top = current && current.getBoundingClientRect().top;
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
  window.jacketSide = setup;
  var st = null;
  try { st = JSON.parse(sessionStorage.getItem('jacket-docs-side') || 'null'); } catch (e) {}
  setup(st);
})();
