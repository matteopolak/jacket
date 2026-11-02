(function () {
  var s = document.querySelector('details.side');
  if (!s) return;
  var n = s.querySelector('nav'), k = n && n.getAttribute('data-nav'), st = null;
  try { st = JSON.parse(sessionStorage.getItem('jacket-docs-side') || 'null'); } catch (e) {}
  if (st && st.k !== k) st = null;
  if (st) (st.o || []).forEach(function (key) {
    var l = s.querySelectorAll('details.nav-cat');
    for (var i = 0; i < l.length; i++) if (l[i].getAttribute('data-key') === key) l[i].open = true;
  });
  function wide() { return matchMedia('(min-width:900px)').matches; }
  function reveal() {
    if (!wide()) return;
    var c = s.querySelector('[aria-current="page"]');
    if (!c) return;
    var a = c.getBoundingClientRect(), b = s.getBoundingClientRect();
    if (a.bottom <= b.top) s.scrollTop += a.top - b.top;
    else if (a.top >= b.bottom) s.scrollTop += a.bottom - b.bottom;
  }
  if (st && wide()) s.scrollTop = st.t;
  reveal();
  var list = s.querySelector('[data-source]');
  if (!list) return;
  var initialScroll = s.scrollTop, moved = false;
  // A reader can scroll or focus the fallback before the request finishes.
  s.addEventListener('wheel', function () { moved = true; }, { passive: true });
  s.addEventListener('touchstart', function () { moved = true; }, { passive: true });
  s.addEventListener('keydown', function () { moved = true; });
  var base = document.body.getAttribute('data-base') || '/';
  fetch(base + list.getAttribute('data-source')).then(function (response) {
    if (!response.ok) throw new Error('sidebar unavailable');
    return response.text();
  }).then(function (text) {
    var template = document.createElement('template');
    template.innerHTML = text;
    var rows = template.content.querySelectorAll('li');
    var start = +list.getAttribute('data-start'), end = +list.getAttribute('data-end');
    var first = +list.getAttribute('data-first'), count = list.children.length;
    if (rows.length < end) { list.removeAttribute('style'); return; }
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
      if (!moved && s.scrollTop === initialScroll && st) s.scrollTop = st.t;
      else if (current) s.scrollTop += current.getBoundingClientRect().top - top;
      reveal();
    }
  }).catch(function () {
    // The inline window and back link remain usable.
    list.removeAttribute('style');
  });
})();
