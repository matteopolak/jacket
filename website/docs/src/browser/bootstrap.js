/** Synchronous prepaint chrome setup, served as one Vite-hashed classic asset.
 * Shell loads this in its head without async/defer; e.g. the next document shares its cached URL.
 */
(() => {
  if (window.jacketBootstrap) { window.jacketBootstrap.prepare(document); return; }
  // Native full-document restoration must run before the deferred ClientRouter starts.
  document.documentElement.setAttribute('data-doc-restoring', '');
  if (window.history) window.history.scrollRestoration = 'auto';
  const finishDocumentScroll = () => {
    if (window.history) window.history.scrollRestoration = 'manual';
    document.documentElement.removeAttribute('data-doc-restoring');
  };
  document.addEventListener('astro:page-load', finishDocumentScroll);
  window.addEventListener('pageshow', event => { if (event.persisted) finishDocumentScroll(); });
  let earlyFolds = 0;
  // The supported custom swap leaves this exact rail attached. Only a navigation context may change its contents.
  const railContexts = new Map();
  let liveRail = null;
  const contextKey = nav => nav?.getAttribute('data-context') || nav?.getAttribute('data-nav');
  const currentLink = nav => nav?.querySelector('[aria-current="page"]')?.getAttribute('href');
  const markCurrent = (nav, href) => {
    nav.querySelectorAll('a[aria-current="page"]').forEach(anchor => anchor.removeAttribute('aria-current'));
    if (href) nav.querySelectorAll('a[href]').forEach(anchor => { if (anchor.getAttribute('href') === href) anchor.setAttribute('aria-current', 'page'); });
  };
  const transferRail = doc => {
    const side = document.querySelector('details.side'), incoming = doc.querySelector('details.side');
    const oldNav = side?.querySelector('nav'), nextNav = incoming?.querySelector('nav');
    if (!side || !incoming || !oldNav || !nextNav) return;
    const oldKey = contextKey(oldNav), nextKey = contextKey(nextNav), href = currentLink(nextNav);
    const focus = side.contains(document.activeElement) ? document.activeElement : null;
    railContexts.set(oldKey, { nav: oldNav, scroll: side.scrollTop, summary: side.querySelector('summary')?.textContent });
    let nav = oldNav;
    if (oldKey !== nextKey) {
      const cached = railContexts.get(nextKey);
      nav = cached?.nav || nextNav;
      oldNav.replaceWith(nav);
      side.querySelector('summary').textContent = incoming.querySelector('summary').textContent;
      side.scrollTop = cached?.scroll ?? 0;
      // Retain the global tree and latest peer context, never a growing page cache.
      if (nextKey !== 'modules') for (const key of railContexts.keys()) if (key !== 'modules' && key !== nextKey) railContexts.delete(key);
    }
    if (nextKey !== 'modules') {
      const oldList = nav.querySelector('.nav-list.mono'), nextList = nextNav.querySelector('.nav-list.mono');
      if (oldList && nextList && oldList !== nextList) {
        const outline = []; for (let node = nextList.nextSibling; node; node = node.nextSibling) outline.push(node);
        if (!oldList.dataset.loaded) {
          const rows = new Map([...oldList.children].map(row => [row.querySelector('a')?.getAttribute('href'), row]));
          for (const row of [...nextList.children]) { const existing = rows.get(row.querySelector('a')?.getAttribute('href')); if (existing) row.replaceWith(existing); }
          oldList.replaceWith(nextList);
        } else oldList.dataset.current = nextList.dataset.current;
        const list = nav.querySelector('.nav-list.mono');
        while (list.nextSibling) list.nextSibling.remove();
        for (const node of outline) nav.appendChild(node);
      }
    }
    markCurrent(nav, href);
    if (oldKey !== nextKey && nextKey !== 'modules' && !railContexts.has(nextKey) && wide()) {
      const current = nav.querySelector('[aria-current="page"]');
      if (current) { const item = current.getBoundingClientRect(), rail = side.getBoundingClientRect(); if (item.bottom <= rail.top) side.scrollTop += item.top - rail.top; else if (item.top >= rail.bottom) side.scrollTop += item.bottom - rail.bottom; }
    }
    liveRail = side;
    // An unchanged live node retains focus naturally. Context changes
    // restore only an equivalent link when that link belongs to the new context.
    if (focus && !side.contains(focus)) {
      const href = focus.getAttribute('href');
      const replacement = href && [...nav.querySelectorAll('a[href]')].find(anchor => anchor.getAttribute('href') === href);
      replacement?.focus({ preventScroll: true });
    }
  };
  let readerScope;
  const wide = () => matchMedia('(min-width:900px)').matches;
  let railWide = wide(), mobileRailOpen = false;
  // Failed writes retain live choices; a changed backing value still takes precedence.
  let sidebarState = null, sideStored, sideStoreKnown = false, sidePending = false;
  const saved = () => {
    try {
      const stored = sessionStorage.getItem('jacket-docs-side');
      const state = JSON.parse(stored || 'null');
      const changed = sideStoreKnown && stored !== sideStored;
      sideStored = stored; sideStoreKnown = true;
      if (!sidePending || changed) { sidebarState = state; sidePending = false; }
    } catch {}
    return sidebarState;
  };
  sidebarState = saved();
  const save = () => {
    const side = document.querySelector('details.side');
    if (!side) return;
    sidebarState = { k: side.querySelector('nav')?.getAttribute('data-nav'), t: side.scrollTop, o: [...side.querySelectorAll('details.nav-cat[open]')].map(group => group.getAttribute('data-key')) };
    sidePending = true;
    try {
      const stored = JSON.stringify(sidebarState);
      sessionStorage.setItem('jacket-docs-side', stored);
      sideStored = stored; sideStoreKnown = true; sidePending = false;
    } catch {}
  };
  const themeValue = value => value === 'light' || value === 'dark' ? value : 'system';
  let themeState = themeValue(document.documentElement.getAttribute('data-theme'));
  let themeStored, themeStoreKnown = false, themePending = false;
  const savedTheme = () => {
    const current = themeValue(document.documentElement.getAttribute('data-theme'));
    if (current !== themeState) { themeState = current; themePending = true; }
    try {
      const stored = localStorage.getItem('jacket-docs-theme');
      const changed = themeStoreKnown && stored !== themeStored;
      themeStored = stored; themeStoreKnown = true;
      if (!themePending || changed) themeState = themeValue(stored);
      if (themeState === themeValue(stored)) themePending = false;
    } catch {}
    return themeState;
  };
  // The attached header border box is the rail offset, including wrapping and font changes.
  let headerOffset, observedHeader;
  const measureHeader = () => {
    const side = document.querySelector('details.side'), nextWide = wide();
    if (side && nextWide !== railWide) {
      if (nextWide) { mobileRailOpen = side.open; side.open = true; }
      else side.open = mobileRailOpen;
      railWide = nextWide;
    }
    const header = document.querySelector('.top');
    if (header !== observedHeader) {
      headerObserver.disconnect(); observedHeader = header;
      if (header) headerObserver.observe(header, { box: 'border-box' });
    }
    if (!header) return;
    const height = header.getBoundingClientRect().height;
    if (!Number.isFinite(height) || height <= 0) return;
    headerOffset = `${height}px`;
    if (document.documentElement.style.getPropertyValue('--header-offset') !== headerOffset) document.documentElement.style.setProperty('--header-offset', headerOffset);
  };
  const headerObserver = new ResizeObserver(measureHeader);
  const prepare = doc => {
    if (headerOffset !== undefined) doc.documentElement.style.setProperty('--header-offset', headerOffset);
    doc.documentElement.classList.add('js');
    doc.documentElement.setAttribute('data-theme', savedTheme());
    const state = sidebarState = saved();
    doc.querySelectorAll('details.side').forEach(side => { if (side !== liveRail) side.open = wide(); });
    doc.querySelectorAll('details.nav-cat').forEach(group => {
      if (liveRail?.contains(group)) return;
      if (state && state.k === group.closest('nav[data-nav]')?.getAttribute('data-nav') && Array.isArray(state.o) && state.o.includes(group.getAttribute('data-key'))) group.open = true;
    });
  };
  window.fold = function () {
    if (++earlyFolds > 6) return;
    const fold = document.currentScript?.parentElement;
    if (fold && fold.scrollHeight <= Number(fold.getAttribute('data-fold') || 440)) fold.classList.add('open');
  };
  const initialized = new WeakSet();
  const records = changes => {
    for (const change of changes) for (const node of change.addedNodes) {
      if (!(node instanceof Element)) continue;
      const state = sidebarState;
      const apply = element => {
        if (initialized.has(element)) return;
        if (element.matches('details.side') || element.matches('details.nav-cat')) initialized.add(element);
        if (element.matches('details.side')) element.open = wide();
        if (element.matches('details.nav-cat') && state && state.k === element.closest('nav[data-nav]')?.getAttribute('data-nav') && Array.isArray(state.o) && state.o.includes(element.getAttribute('data-key'))) element.open = true;
      };
      apply(node); node.querySelectorAll('details.side,details.nav-cat').forEach(apply);
    }
  };
  const observer = new MutationObserver(records);
  observer.observe(document.documentElement, { childList: true, subtree: true });
  window.jacketFinishSidebar = () => {
    measureHeader();
    records(observer.takeRecords()); observer.disconnect();
    readerScope?.abort(); readerScope = new AbortController();
    const side = document.querySelector('details.side'), state = sidebarState = saved();
    if (!side) return;
    const retained = side === liveRail;
    const moved = () => { side.dataset.readerMoved = 'true'; };
    for (const event of ['wheel', 'touchstart', 'keydown', 'focusin']) side.addEventListener(event, moved, { passive: true, signal: readerScope.signal });
    if (retained) return;
    if (side.dataset.readerMoved !== 'true' && wide() && state?.k === side.querySelector('nav')?.getAttribute('data-nav') && Number.isFinite(state.t)) side.scrollTop = state.t;
    const current = side.querySelector('[aria-current="page"]');
    if (current && wide() && side.dataset.readerMoved !== 'true') {
      const itemBox = current.getBoundingClientRect(), sideBox = side.getBoundingClientRect();
      if (itemBox.bottom <= sideBox.top) side.scrollTop += itemBox.top - sideBox.top;
      else if (itemBox.top >= sideBox.bottom) side.scrollTop += itemBox.bottom - sideBox.bottom;
    }
  };
  window.jacketFinishLayout = () => {
    measureHeader();
    const folds = [...document.querySelectorAll('main .fold:not(.open)')];
    folds.filter(fold => fold.scrollHeight <= Number(fold.getAttribute('data-fold') || 440)).forEach(fold => fold.classList.add('open'));
    document.querySelectorAll('main .fold.open + .fold-btn').forEach(button => button.remove());
    const toc = document.querySelector('.toc');
    if (!toc) return;
    const marks = [...toc.querySelectorAll('a[href^="#"]')].flatMap(anchor => {
      const heading = document.getElementById(anchor.getAttribute('href').slice(1));
      return heading ? [{ anchor, top: heading.getBoundingClientRect().top }] : [];
    });
    if (!marks.length) return;
    let first = 0, last = -1;
    marks.forEach((mark, index) => { if (mark.top <= 70) first = index; if (mark.top <= innerHeight - 1) last = index; });
    for (let index = first; index <= Math.max(first, last); index++) {
      const item = marks[index].anchor.closest('li');
      const list = item.classList.contains('has-list') ? item.nextElementSibling : item.closest('li.items');
      if (!list) continue;
      list.classList.add('open'); list.previousElementSibling?.querySelector('.tog')?.setAttribute('aria-expanded', 'true');
    }
  };
  document.addEventListener('astro:before-swap', event => {
    save(); readerScope?.abort(); earlyFolds = 0;
    headerObserver.disconnect(); observedHeader = null;
    prepare(event.newDocument); transferRail(event.newDocument);
    if (event.navigationType === 'traverse') event.newDocument.documentElement.setAttribute('data-doc-restoring', '');
  });
  document.addEventListener('astro:after-swap', () => {
    window.jacketFinishSidebar(); window.jacketFinishLayout();
  });
  window.addEventListener('pagehide', () => {
    save();
    if (window.history) window.history.scrollRestoration = 'auto';
  });
  window.addEventListener('resize', measureHeader, { passive: true });
  window.jacketBootstrap = { prepare };
  prepare(document);
})();
