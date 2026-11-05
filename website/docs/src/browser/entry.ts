/** Astro owns routing; each actual body receives one cancellable enhancement lifetime. */
import { navigate, swapFunctions } from 'astro:transitions/client';
import { setupControls } from './controls.ts';
import { setupNavigation } from './navigation.ts';
import { initExamples } from './examples.ts';
import { initSearch } from './search.ts';
import { browserResolver } from '../lib/urls.ts';
import { compatibleShell, observeTransition } from './lifetime.ts';

const lifecycle = new AbortController();
let page: AbortController | undefined;
let body: HTMLElement | undefined;
function cleanup(): void { page?.abort(); page = undefined; body = undefined; }
function setup(): void {
  if (body === document.body && page && !page.signal.aborted) return;
  cleanup(); body = document.body; page = new AbortController();
  const signal = page.signal, base = body.dataset['base'] ?? '/';
  setupControls(signal); setupNavigation(signal); initExamples(base, signal, document.body.dataset['runtimeEngine']&&document.body.dataset['runtimeWasm']?{engine:document.body.dataset['runtimeEngine'],wasm:document.body.dataset['runtimeWasm']}:undefined); initSearch(document.body.dataset['searchSource']??'', browserResolver(base), signal, (url: string) => { void navigate(url); });
  // A legacy slash URL probes only its same-origin slashless candidate, without following redirects.
  if (body.dataset['page'] === '404.html' && location.pathname.startsWith(base)
      && location.pathname.length > base.length && location.pathname.endsWith('/')) {
    const candidate = new URL(location.href); candidate.pathname = candidate.pathname.slice(0, -1);
    void fetch(candidate, { method: 'HEAD', redirect: 'manual', signal: AbortSignal.any([signal, AbortSignal.timeout(5000)]) })
      .then(response => { if (!signal.aborted && response.status === 200) location.replace(candidate.href); }).catch(() => {});
  }
}
// Astro retains its router, fetching, history, scripts and native main transition.
// Its supported custom swap keeps live chrome attached, avoiding the default persist lift.
document.addEventListener('astro:before-preparation', event => {
  const load = event.loader;
  event.loader = async () => { await load(); if (!event.signal.aborted && !compatibleShell(document, event.newDocument)) event.preventDefault(); };
}, { signal: lifecycle.signal });
document.addEventListener('astro:before-swap', event => {
  void observeTransition(event.viewTransition); cleanup();
  event.swap = () => {
    const next = event.newDocument;
    document.body.querySelectorAll(':scope > .astro-route-announcer').forEach(node => node.remove());
    swapFunctions.deselectScripts(next); swapFunctions.swapRootAttributes(next); swapFunctions.swapHeadElements(next);
    const main = document.querySelector<HTMLElement>('.wrap > main')!, incoming = next.querySelector<HTMLElement>('.wrap > main')!;
    const wrap = main.parentElement!, nextWrap = incoming.parentElement!;
    const outline = wrap.querySelector(':scope > .toc'), nextOutline = nextWrap.querySelector(':scope > .toc');
    main.replaceWith(incoming);
    if (outline && nextOutline) outline.replaceWith(nextOutline);
    else if (outline) outline.remove();
    else if (nextOutline) incoming.after(nextOutline);
    for (const attribute of [...document.body.attributes]) document.body.removeAttribute(attribute.name);
    for (const attribute of [...next.body.attributes]) document.body.setAttribute(attribute.name, attribute.value);
    if (nextWrap.hasAttribute('data-toc')) wrap.setAttribute('data-toc', ''); else wrap.removeAttribute('data-toc');
    // Fold geometry must settle before Astro restores the destination scroll position.
    (window as Window & { jacketFinishLayout?: () => void }).jacketFinishLayout?.();
    const nextLinks = [...next.querySelectorAll<HTMLAnchorElement>('.top a[aria-current]')].map(anchor => anchor.getAttribute('href'));
    document.querySelectorAll<HTMLAnchorElement>('.top a').forEach(anchor => {
      if (nextLinks.includes(anchor.getAttribute('href'))) anchor.setAttribute('aria-current', 'page'); else anchor.removeAttribute('aria-current');
    });
  };
}, { signal: lifecycle.signal });
document.addEventListener('astro:page-load', setup, { signal: lifecycle.signal });
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', setup, { once: true, signal: lifecycle.signal });
else setup();
if (import.meta.hot) import.meta.hot.dispose(() => { cleanup(); lifecycle.abort(); });
