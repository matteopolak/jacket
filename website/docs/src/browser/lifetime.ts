/** Replaceable page lifetimes own listeners, requests and queued layout work across Astro swaps. */
export function pageScope(): (parent: AbortSignal) => AbortSignal {
  let active: AbortController | undefined;
  return parent => {
    active?.abort();
    const current = new AbortController(); active = current;
    if (parent.aborted) current.abort();
    else parent.addEventListener('abort', () => current.abort(), { once: true, signal: current.signal });
    return current.signal;
  };
}

/** A discarded page never measures, scrolls or mutates its replacement on the next frame. */
export function frame(signal: AbortSignal, callback: () => void): void {
  if (signal.aborted) return;
  const cancel = (): void => cancelAnimationFrame(id);
  const id = requestAnimationFrame(() => {
    signal.removeEventListener('abort', cancel);
    if (!signal.aborted) callback();
  });
  signal.addEventListener('abort', cancel, { once: true });
}

/** Observe Astro's actual transition result without hiding failed DOM updates.
 * Example: void observeTransition(event.viewTransition) in astro:before-swap.
 * A skipped visual animation is expected only when its DOM callback succeeds.
 */
export async function observeTransition(
  transition: Pick<ViewTransition, 'ready' | 'updateCallbackDone'>,
  report: (reason: unknown) => void = reason => reportError(reason),
): Promise<void> {
  try { await transition.ready; }
  catch (visualError) {
    try { await transition.updateCallbackDone; }
    catch (swapError) { report(swapError); return; }
    if (!(visualError instanceof DOMException && visualError.name === 'AbortError')) report(visualError);
  }
}

/** A live shell is shared only by documents with the same protocol, base and version.
 * Example: a version-menu destination with a different version uses ordinary document navigation.
 */
export function compatibleShell(current: Document, incoming: Document): boolean {
  if (!['docShell', 'base', 'docVersion', 'docIdentity', 'docState', 'docIcons'].every(key => current.body.dataset[key] !== undefined
    && current.body.dataset[key] === incoming.body.dataset[key])
    || current.body.dataset['docShell'] !== 'jacket-docs/1'
    || !current.querySelector('.wrap > main') || !incoming.querySelector('.wrap > main')) return false;
  const bootstrap = current.querySelector('script[data-doc-bootstrap]')?.textContent;
  if (!bootstrap || bootstrap !== incoming.querySelector('script[data-doc-bootstrap]')?.textContent) return false;
  const assets = (doc: Document): (string | null)[] => [
    ...[...doc.querySelectorAll<HTMLScriptElement>('script[type="module"][src]')].filter(script => !script.closest('main')).map(script => script.getAttribute('src')),
    ...[...doc.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"][href]')].map(link => link.getAttribute('href')),
  ];
  const previous = assets(current);
  return previous.length > 0 && JSON.stringify(previous) === JSON.stringify(assets(incoming));
}
