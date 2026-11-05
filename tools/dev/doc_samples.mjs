// The header links directly to the first configured guide, not to a guide-card index.
import assert from 'node:assert/strict';

/** Return a direct guide URL from the header, e.g. `/reference/guides/testing`. */
export function firstGuidePath(html) {
  const nav = html.match(/<nav\b[^>]*class="site-nav"[^>]*>([\s\S]*?)<\/nav>/)?.[1];
  const href = [...(nav || '').matchAll(/<a\b[^>]*href="([^"]+)"[^>]*>/g)]
    .map(match => match[1]).find(path => /(?:^|\/)guides\/[^/]+/.test(path));
  assert(href, 'No guide page in the site navigation');
  return href;
}

/** Check a page without leaving an unread body, e.g. `await pageStatus(base + 'guides/testing')`. */
export async function pageStatus(url) {
  return (await fetch(url, { method: 'HEAD' })).status;
}

/** Wait for two browser frames, e.g. `await waitForFrames(expression => tab.evaluate(expression))`. */
export function waitForFrames(evaluate) {
  return evaluate('new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))');
}

/** Select a canonical native-binding page from jacket-doc-site/1 search facts, e.g. api/Native/open. */
export function foreignItemPath(payload) {
  assert(payload?.payload === 'search' && payload.data?.schemaVersion === 1 && Array.isArray(payload.data.entries),
    'Expected versioned documentation search data');
  const foreign = payload.data.entries.find(entry => entry.binding === true && entry.kind === 'declaration');
  assert(foreign?.target?.kind === 'page' && typeof foreign.target.id === 'string', 'No foreign declaration page in the site');
  const id = foreign.target.id;
  assert(id.startsWith('api/') && !/[\\:\0?#]/u.test(id) && !id.split('/').some(part => ['', '.', '..'].includes(part)),
    'Unsafe foreign declaration target');
  return id;
}
