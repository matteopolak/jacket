import { pageScope } from './lifetime.ts';
/** Theme and version controls have one abort-owned lifetime for each Astro page. */
export type Theme = 'system' | 'light' | 'dark';
const themes: readonly Theme[] = ['system', 'light', 'dark'];

/** Move to the next stored theme, e.g. dark returns to the operating system preference. */
export function nextTheme(current: string): Theme {
  return themes[(themes.findIndex(theme => theme === current) + 1) % themes.length] ?? 'system';
}

export interface VersionEntry { version: string; url: string; }

/** Validate optional version-menu data before assigning any browser destination. */
export function versionEntries(value: unknown, source: URL): VersionEntry[] {
  if (!Array.isArray(value)) throw new Error('versions manifest must contain an array');
  return value.map((entry: unknown) => {
    if (!entry || typeof entry !== 'object' || !('version' in entry) || !('url' in entry)
      || typeof entry.version !== 'string' || typeof entry.url !== 'string') throw new Error('invalid version entry');
    const url = new URL(entry.url, source);
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) throw new Error('unsupported version destination');
    return { version: entry.version, url: url.href };
  });
}

function setupTheme(signal: AbortSignal): void {
  const button = document.querySelector<HTMLButtonElement>('#theme-toggle');
  if (!button) return;
  const root = document.documentElement;
  const label = (): void => {
    const theme = root.getAttribute('data-theme') ?? 'system';
    const text = `Theme: ${theme} (click to change)`;
    button.setAttribute('aria-label', text); button.title = text;
    const icon = document.querySelector('#theme-icon use');
    if (icon) icon.setAttribute('href', `${icon.getAttribute('href')?.split('#')[0] ?? ''}#${theme}`);
  };
  label();
  button.addEventListener('click', () => {
    const theme = nextTheme(root.getAttribute('data-theme') ?? 'system');
    root.setAttribute('data-theme', theme);
    try {
      if (theme === 'system') localStorage.removeItem('jacket-docs-theme');
      else localStorage.setItem('jacket-docs-theme', theme);
    } catch { /* A blocked store leaves this page's explicit selection intact. */ }
    label();
  }, { signal });
}

function setupVersions(signal: AbortSignal): void {
  const button = document.querySelector<HTMLButtonElement>('#ver-button');
  const menu = document.querySelector<HTMLElement>('#ver-menu');
  if (!button || !menu) return;
  menu.replaceChildren(); menu.hidden = true; button.disabled = false;
  button.setAttribute('aria-expanded', 'false');
  const manifestURL = new URL(`${document.body.dataset['base'] ?? '/'}versions.json`, location.href);
  let pending:Promise<void>|undefined;
  const load=():Promise<void>=>{
    if(pending)return pending;
    let available=false;
    pending=fetch(manifestURL,{signal,cache:'no-cache'}).then(async response=>{
      if(!response.ok)return;
      const raw:unknown=await response.json();
      if(signal.aborted||!button.isConnected)return;
      const entries=versionEntries(raw,manifestURL);
      if(entries.length<2)return;
      const current=button.textContent?.trim();
      for(const entry of entries){
        const anchor=document.createElement('a');
        anchor.href=entry.url;anchor.textContent=entry.version;anchor.setAttribute('data-astro-reload','');
        anchor.setAttribute('role','menuitem');anchor.tabIndex=-1;
        if(entry.version===current)anchor.setAttribute('aria-current','true');
        menu.appendChild(anchor);
      }
      available=true;
    }).catch(()=>{ /* Optional unavailable/offline archives remain disabled for this page. */ }).then(()=>{if(!signal.aborted)button.disabled=!available;});
    return pending;
  };
  button.addEventListener('pointerenter',()=>{void load();},{signal});
  button.addEventListener('focus',()=>{void load();},{signal});
  let opening=0,wanted=false;
  const setOpen = (open: boolean): void => {if(!open)opening++;wanted=open;menu.hidden = !open; button.setAttribute('aria-expanded', String(open)); };
  const openAt = (last = false): void => {
    setOpen(true);
    const links = menu.querySelectorAll<HTMLAnchorElement>('a');
    links[last ? links.length - 1 : 0]?.focus();
  };
  const requestOpen=(last=false):void=>{wanted=true;const intent=++opening;void load().then(()=>{if(intent===opening&&wanted&&!signal.aborted&&!button.disabled)openAt(last);});};
  button.addEventListener('click', event => {event.stopPropagation();if(wanted||!menu.hidden)setOpen(false);else requestOpen();}, {signal});
  button.addEventListener('keydown', event => {
    if (!['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(event.key)) return;
    event.preventDefault();if((event.key==='Enter'||event.key===' ')&&wanted)setOpen(false);else requestOpen(event.key==='ArrowUp');
  }, { signal });
  menu.addEventListener('keydown', event => {
    if (event.key === ' ') { event.preventDefault(); if (document.activeElement instanceof HTMLElement) document.activeElement.click(); return; }
    if (event.key === 'Tab') { setOpen(false); button.focus(); return; }
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const links = [...menu.querySelectorAll<HTMLAnchorElement>('a')];
    const current = links.findIndex(link => link === document.activeElement);
    const index = event.key === 'Home' ? 0 : event.key === 'End' ? links.length - 1
      : (current + (event.key === 'ArrowUp' ? -1 : 1) + links.length) % links.length;
    links[index]?.focus();
  }, { signal });
  const leave=(event:FocusEvent):void=>{const next=event.relatedTarget;if(next instanceof Node&&(button.contains(next)||menu.contains(next)))return;setOpen(false);};
  button.addEventListener('focusout',leave,{signal});menu.addEventListener('focusout',leave,{signal});
  document.addEventListener('click', event => { if (event.target instanceof Node && !menu.contains(event.target)) setOpen(false); }, { signal });
  document.addEventListener('keydown', event => { if (event.key === 'Escape' && (wanted||!menu.hidden)) {const wasOpen=!menu.hidden;setOpen(false);if(wasOpen)button.focus();} }, { signal });
}

const scope = pageScope();
/** Each Astro page load replaces previous controls and cancels stale menu work. */
export function setupControls(parent: AbortSignal): void {
  const signal = scope(parent);
  if (signal.aborted) return;
  setupTheme(signal); setupVersions(signal);
}
