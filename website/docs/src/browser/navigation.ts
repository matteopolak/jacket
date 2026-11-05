import { pageScope, frame } from './lifetime.ts';
import { prefetch } from 'astro:prefetch';
/** Astro page enhancement: sidebar data, outline state and accessible content controls. */
import { peerWindow } from '../lib/render.ts';
import { fetchPayload } from '../lib/client-data.ts';
import type { NavigationDeclaration } from '../generated/doc-site.js';

function pageURL(route: string): string {
  if (/[\\:\0?#]/u.test(route) || route.split('/').some(part => ['', '.', '..'].includes(part))) throw new Error('invalid canonical navigation route');
  return new URL(`${document.body.dataset['base'] ?? '/'}${route}`, location.origin).href;
}

function revealCurrent(side: HTMLElement): void {
  if (!matchMedia('(min-width:900px)').matches) return;
  const current = side.querySelector('[aria-current="page"]');
  if (!current) return;
  const item = current.getBoundingClientRect(), rail = side.getBoundingClientRect();
  if (item.bottom <= rail.top) side.scrollTop += item.top - rail.top;
  else if (item.top >= rail.bottom) side.scrollTop += item.bottom - rail.bottom;
}

function navRow(item: NavigationDeclaration): HTMLLIElement {
  const row = document.createElement('li'), anchor = document.createElement('a');
  anchor.href = pageURL(item.route); anchor.dataset['astroPrefetch'] = 'hover'; anchor.textContent = item.name; row.appendChild(anchor);
  if (item.experimental) {
    const dot = document.createElement('span'), label = document.createElement('span');
    dot.className = 'dot exp'; dot.setAttribute('role', 'img');
    dot.title = 'Experimental: may change or go away'; dot.setAttribute('aria-label', dot.title);
    label.className = 'sr'; label.textContent = 'Experimental'; dot.appendChild(label); anchor.appendChild(dot);
  }
  return row;
}

async function loadSidebar(side: HTMLElement, signal: AbortSignal): Promise<void> {
  const list = side.querySelector<HTMLUListElement>('[data-source]');
  const source = list?.dataset['source'];
  if (!list || !source) return;
  let moved = side.dataset['readerMoved'] === 'true';
  for (const name of ['wheel', 'touchstart', 'keydown', 'focusin']) side.addEventListener(name, () => { moved = true; }, { passive: true, once: true, signal });
  const initialScroll = side.scrollTop;
  try {
    const payload = await fetchPayload(source, 'navigation', signal);
    if (payload.payload !== 'navigation') throw new Error('sidebar payload is not navigation data');
    if (signal.aborted || !list.isConnected) return;
    const declaration = payload.data.entries.find(item => item.id === list.dataset['current']);
    if (!declaration) throw new Error('sidebar current declaration is absent');
    const range = peerWindow(payload.data.entries, declaration);
    if (range.start !== Number(list.dataset['start']) || range.end !== Number(list.dataset['end'])
      || range.first !== Number(list.dataset['first']) || list.children.length !== range.visible.length) throw new Error('sidebar placeholder does not match module data');
    const current = list.querySelector('[aria-current="page"]');
    const top = current?.getBoundingClientRect().top;
    const before = document.createDocumentFragment(), after = document.createDocumentFragment();
    for (let i = range.start; i < range.end; i++) {
      const item = range.peers[i];
      if (!item) throw new Error('sidebar peer window exceeds module data');
      if (i < range.first) before.appendChild(navRow(item));
      else if (i >= range.last) after.appendChild(navRow(item));
    }
    list.prepend(before); list.append(after);
    list.dataset['loaded'] = source; delete list.dataset['source'];
    list.style.removeProperty('--before'); list.style.removeProperty('--after');
    if (matchMedia('(min-width:900px)').matches) {
      let restored = false;
      if (!moved && side.scrollTop === initialScroll) {
        try {
          const state: unknown = JSON.parse(sessionStorage.getItem('jacket-docs-side') ?? 'null');
          if (state && typeof state === 'object' && 'k' in state && 't' in state
            && state.k === side.querySelector('nav')?.getAttribute('data-nav') && typeof state.t === 'number' && Number.isFinite(state.t)) {
            side.scrollTop = state.t; restored = true;
          }
        } catch { /* Preserve the visible current row when storage is unavailable. */ }
      }
      if (!restored && current && top !== undefined) side.scrollTop += current.getBoundingClientRect().top - top;
      revealCurrent(side);
    }
  } catch (error) {
    if (signal.aborted) return;
    list.style.removeProperty('--before'); list.style.removeProperty('--after');
    console.warn('Documentation sidebar unavailable; use the module back link.', error);
  }
}

function setupSidebar(signal: AbortSignal): void {
  const side = document.querySelector<HTMLDetailsElement>('details.side');
  if (!side) return;
  side.querySelectorAll<HTMLDetailsElement>('details.nav-cat').forEach((group, index) => {
    const summary = group.querySelector('summary'), list = group.querySelector<HTMLElement>('.nav-list');
    if (!summary || !list) return;
    list.id = `nav-group-${index}`; summary.setAttribute('aria-controls', list.id);
    const state = (): void => summary.setAttribute('aria-expanded', String(group.open));
    group.addEventListener('toggle', state, { signal }); state();
  });
  void loadSidebar(side, signal);
}

interface OutlineGroup { list: HTMLElement; button: HTMLButtonElement; user: boolean | null; seen: boolean; open: boolean; }
interface Mark { anchor: HTMLAnchorElement; heading: HTMLElement; group?: OutlineGroup; item: boolean; }

function setupOutline(signal: AbortSignal): void {
  const rail = document.querySelector<HTMLElement>('.toc');
  if (!rail) return;
  const groupOf = new Map<Element, OutlineGroup>();
  rail.querySelectorAll<HTMLElement>('li.items').forEach(list => {
    const head = list.previousElementSibling, button = head?.querySelector<HTMLButtonElement>('.tog');
    if (!head || !button) return;
    const open = list.classList.contains('open');
    const group: OutlineGroup = { list, button, user: null, seen: open, open };
    const set = (): void => {
      group.open = group.user ?? group.seen;
      list.classList.toggle('open', group.open); button.setAttribute('aria-expanded', String(group.open));
    };
    button.addEventListener('click', () => { group.user = !group.open; set(); }, { signal });
    groupOf.set(head, group);
    for (const item of list.querySelector('ul')?.children ?? []) groupOf.set(item, group);
  });
  const marks: Mark[] = [];
  for (const anchor of rail.querySelectorAll<HTMLAnchorElement>('a[href^="#"]')) {
    const heading = document.getElementById(anchor.getAttribute('href')?.slice(1) ?? '');
    const item = anchor.closest('li');
    if (!heading || !item) continue;
    const group = groupOf.get(item);
    const individual = !group || item.parentElement?.parentElement === group.list;
    if (individual) anchor.dataset['t'] = anchor.textContent ?? '';
    marks.push(group ? { anchor, heading, group, item: individual } : { anchor, heading, item: individual });
  }
  if (!marks.length) return;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let from = -1, to = -1, first = true, pending = false;
  const below = (y: number): number => {
    let low = 0, high = marks.length;
    while (low < high) {
      const middle = (low + high) >> 1;
      if ((marks[middle]?.heading.getBoundingClientRect().top ?? Infinity) > y) high = middle;
      else low = middle + 1;
    }
    return low;
  };
  const centre = (start: number, end: number): number => {
    const top = marks[start]?.anchor.getBoundingClientRect().top ?? 0;
    const bottom = marks[end]?.anchor.getBoundingClientRect().bottom ?? top;
    const bounds = rail.getBoundingClientRect();
    return Math.round(rail.scrollTop + (top + bottom) / 2 - (bounds.top + bounds.bottom) / 2);
  };
  const glide = (position: number): void => {
    if (Math.abs(position - rail.scrollTop) > 1) rail.scrollTo({ top: position, behavior: first || reduced.matches ? 'instant' : 'smooth' });
    first = false;
  };
  const mark = (): void => {
    pending = false;
    const start = Math.max(0, below(70) - 1), end = Math.max(start, below(innerHeight - 1) - 1);
    if (start === from && end === to) return;
    const position = centre(start, end);
    const affected = new Set<OutlineGroup>(), seen = new Set<OutlineGroup>();
    for (let index = from; index >= 0 && index <= to; index++) {
      const item = marks[index];
      if (!item) continue;
      if (index < start || index > end) item.anchor.classList.remove('active');
      if (item.group) affected.add(item.group);
    }
    marks[from]?.anchor.removeAttribute('aria-current');
    for (let index = start; index <= end; index++) {
      const item = marks[index];
      if (!item) continue;
      if (item.item) item.anchor.classList.add('active');
      if (item.group) { affected.add(item.group); seen.add(item.group); }
    }
    marks[start]?.anchor.setAttribute('aria-current', 'location');
    let changed = false;
    for (const group of affected) {
      group.seen = seen.has(group);
      if (!group.seen && group.user === false) group.user = null;
      const open = group.user ?? group.seen;
      if (open === group.open) continue;
      group.open = open; group.list.classList.toggle('open', open);
      group.button.setAttribute('aria-expanded', String(open)); changed = true;
    }
    from = start; to = end;
    if (!rail.classList.contains('ready')) frame(signal, () => rail.classList.add('ready'));
    if (changed) frame(signal, () => glide(centre(from, to))); else glide(position);
  };
  window.addEventListener('scroll', () => { if (!pending) { pending = true; frame(signal, mark); } }, { passive: true, signal });
  mark();
}

function setupFolds(signal: AbortSignal): void {
  document.querySelectorAll<HTMLElement>('main .fold').forEach((fold, index) => {
    const button = fold.nextElementSibling;
    if (!(button instanceof HTMLButtonElement) || !button.classList.contains('fold-btn')) return;
    if (fold.classList.contains('open')) { button.remove(); return; }
    if (!fold.id) fold.id = `fold-${index}`;
    button.setAttribute('aria-controls', fold.id);
    const open = (): void => { fold.classList.add('open'); button.remove(); };
    button.addEventListener('click', () => { fold.tabIndex = -1; open(); fold.focus({ preventScroll: true }); }, { signal });
    fold.addEventListener('focusin', () => { if (!fold.classList.contains('open')) open(); }, { signal });
  });
}

function openTarget(): void {
  let id: string;
  try { id = decodeURIComponent(location.hash.slice(1)); } catch { return; }
  if (!id) return;
  const target = document.getElementById(id);
  if (!target) {
    for (const anchor of document.querySelectorAll<HTMLAnchorElement>('main li a[href]')) {
      const url = new URL(anchor.href, location.href);
      if (url.origin === location.origin && !url.hash && url.pathname.split('/').at(-1) === id) { location.replace(url.href); return; }
    }
    return;
  }
  let opened = false;
  for (let node: HTMLElement | null = target; node; node = node.parentElement) {
    if (node instanceof HTMLDetailsElement && !node.open) { node.open = true; opened = true; }
    if (node.classList.contains('fold') && !node.classList.contains('open')) {
      node.classList.add('open'); opened = true;
      if (node.nextElementSibling?.classList.contains('fold-btn')) node.nextElementSibling.remove();
    }
  }
  if (opened) target.scrollIntoView({ behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth' });
}

function setupCards(signal: AbortSignal): void {
  const card = (target: EventTarget | null): HTMLAnchorElement | null => {
    if (!(target instanceof Element) || target.closest('a,button,summary,input,textarea,select,option,label,[contenteditable],.ex')) return null;
    const row = target.closest('.row'), selection = getSelection();
    if (!row || selection && !selection.isCollapsed && row.contains(selection.anchorNode)) return null;
    return row.querySelector<HTMLAnchorElement>('.row-head a.name');
  };
  document.addEventListener('click', event => {
    if (event.button !== 0) return;
    const link = card(event.target);
    if (!link) return;
    if (event.ctrlKey || event.metaKey || event.shiftKey) window.open(link.href, '_blank', 'noopener'); else link.click();
  }, { signal });
  document.addEventListener('auxclick', event => { const link = event.button === 1 ? card(event.target) : null; if (link) window.open(link.href, '_blank', 'noopener'); }, { signal });
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element) || event.target.closest('a') || String(getSelection())) return;
    const line = event.target.closest<HTMLElement>('.decl .fl[id]');
    if (line && location.hash !== `#${line.id}`) location.hash = line.id;
  }, { signal });
}

function setupScrollRegions(signal: AbortSignal): void {
  const regions = [...document.querySelectorAll<HTMLElement>('main pre,main table,main .defs')];
  let pending = false;
  const size = (): void => {
    pending = false;
    const overflow = regions.map(region => region.scrollWidth > region.clientWidth);
    regions.forEach((region, index) => { if (overflow[index]) region.tabIndex = 0; else region.removeAttribute('tabindex'); });
  };
  window.addEventListener('resize', () => { if (!pending) { pending = true; frame(signal, size); } }, { signal });
  size();
}

/** Delegation covers lazy sidebar/search anchors and replacement bodies without eager catalogue fetches. */
function setupPrefetch(signal: AbortSignal): void {
  let pending: number | undefined;
  const cancel = (): void => { if (pending !== undefined) window.clearTimeout(pending); pending = undefined; };
  const anchorOf = (target: EventTarget | null): HTMLAnchorElement | null => target instanceof Element
    ? target.closest<HTMLAnchorElement>('a[data-astro-prefetch="hover"]') : null;
  const begin = (event: Event): void => {
    const anchor = anchorOf(event.target);
    if (!anchor) return;
    if (event instanceof MouseEvent && event.relatedTarget instanceof Node && anchor.contains(event.relatedTarget)) return;
    cancel();
    pending = window.setTimeout(() => { pending = undefined; if (!signal.aborted && anchor.isConnected) void prefetch(anchor.href); }, 80);
  };
  const end = (event: Event): void => {
    const anchor = anchorOf(event.target);
    if (!anchor) return;
    if ((event instanceof MouseEvent || event instanceof FocusEvent) && event.relatedTarget instanceof Node && anchor.contains(event.relatedTarget)) return;
    cancel();
  };
  document.addEventListener('mouseover', begin, { signal, passive: true });
  document.addEventListener('mouseout', end, { signal, passive: true });
  document.addEventListener('focusin', begin, { signal });
  document.addEventListener('focusout', end, { signal });
  signal.addEventListener('abort', cancel, { once: true });
}

const scope = pageScope();
/** Enhance the current Astro document; replacement aborts every old listener and layout callback. */
export function setupNavigation(parent: AbortSignal): void {
  const signal = scope(parent);
  if (signal.aborted) return;
  setupSidebar(signal); setupOutline(signal); setupFolds(signal); setupCards(signal); setupScrollRegions(signal); setupPrefetch(signal);
  window.addEventListener('hashchange', openTarget, { signal }); openTarget();
}
