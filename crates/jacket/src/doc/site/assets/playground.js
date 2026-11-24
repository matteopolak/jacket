// @ts-check
// The playground: an editor, the options and the results, on the browser build of the compiler.
// The compiler and its language server run in run-worker.js; ending that worker stops a program.
// `mount` starts the page that is in the document; site.js calls it again when a page swap
// brings the playground in.

import { createLayout } from './playground-layout.js';
import { DEFAULTS, STEPS, decode, encode, formatConfig, words } from './playground-state.js';

/**
 * @typedef {{ name: string, title: string, summary: string, main: string, files: string[], assets: string[] }} Example
 * @typedef {{ severity: string, file: string, line: number, column: number, message: string, end?: { line: number, column: number }, raw?: any }} Diagnostic
 * @typedef {{ exitCode: number | null, stdout: string, stderr: string, rendered: string, output: { stream: string, text: string }[], diagnostics: Diagnostic[], failurePhase?: string | null }} Played
 * @typedef {import('./playground-state.js').Options} Options
 * @typedef {import('./playground-state.js').Workspace} Workspace
 * @typedef {{ id?: number, method?: string, result?: any, error?: unknown, params?: any }} LspMessage
 */

const STORE = 'jacket-playground';
const LAYOUT = 'jacket-playground-layout';
const WIDTH = 'jacket-playground-width';
const HELLO = '#import "Basic";\n\nmain :: () {\n    print("Hello, world!\\n");\n}\n';
/** How long a run may take before its worker is stopped, in milliseconds. */
const WALL = 120_000;
const ROOT = 'file:///workspace/';

/** @param {string} s */
const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
/** @param {string} cls @param {string} s */
const tok = (cls, s) => `<span class="${cls}">${esc(s)}</span>`;

/** Colours TOML with the classes the code blocks use. @param {string} text */
function toml(text) {
  return text
    .split('\n')
    .map((line) => {
      const comment = /^(.*?)(#.*)$/.exec(line);
      const body = comment && !/["']/.test(comment[1]) ? comment[1] : line;
      const tail = body === line ? '' : tok('c', /** @type {RegExpExecArray} */ (comment)[2]);
      const table = /^(\s*)(\[\[?[^\]]*\]\]?)(\s*)$/.exec(body);
      if (table) return esc(table[1]) + tok('t', table[2]) + esc(table[3]) + tail;
      const pair = /^(\s*)([A-Za-z0-9_.\-"']+)(\s*=\s*)(.*)$/.exec(body);
      if (!pair) return esc(body) + tail;
      const value = pair[4].replace(
        /("(?:[^"\\]|\\.)*"?|'[^']*'?)|\b(true|false)\b|(-?\d[\w.:-]*)/g,
        (m, s, b, n) => tok(s ? 's' : b ? 'k' : n ? 'n' : '', m),
      );
      return esc(pair[1]) + tok('t', pair[2]) + esc(pair[3]) + value.replace(/<span class="">(.*?)<\/span>/g, '$1') + tail;
    })
    .join('\n');
}

/** Colours Markdown: headings, code, lists, links and emphasis. @param {string} text */
function markdown(text) {
  let fenced = false;
  return text
    .split('\n')
    .map((line) => {
      if (/^\s*(```|~~~)/.test(line)) {
        fenced = !fenced;
        return tok('s', line);
      }
      if (fenced) return tok('s', line);
      if (/^#{1,6}\s/.test(line)) return tok('k', line);
      if (/^\s*>/.test(line)) return tok('c', line);
      const marker = /^(\s*(?:[-*+]|\d+\.)\s)(.*)$/.exec(line);
      const rest = marker ? marker[2] : line;
      const inline = esc(rest)
        .replace(/`[^`]*`/g, (m) => `<span class="s">${m}</span>`)
        .replace(/\[([^\]]*)\]\(([^)]*)\)/g, (m) => `<span class="d">${m}</span>`)
        .replace(/(\*\*|__)(?=\S)[^]*?\1/g, (m) => `<span class="t">${m}</span>`);
      return (marker ? tok('d', marker[1]) : '') + inline;
    })
    .join('\n');
}

/** The coloured HTML for a file, by its extension. @param {string} name @param {string} text */
function colour(name, text) {
  if (name.endsWith('.toml')) return toml(text);
  if (name.endsWith('.md')) return markdown(text);
  return window.jacketHighlight(text);
}

/** Where the site's assets are; set when the page mounts. */
let assets = '';

const ICONS = {
  md: '<path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><path d="M14 3v6h6M8 13h8M8 17h5"/>',
  toml: '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9 7 7M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1"/>',
  file: '<path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><path d="M14 3v6h6"/>',
};

/** A padlock, for the read-only file tab. */
function lockIcon() {
  const svg = glyph('file');
  svg.innerHTML = '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>';
  svg.setAttribute('class', 'pg-ico pg-lock');
  return svg;
}

/** A small icon for a file's kind. @param {string} name */
function icon(name) {
  const kind = name.endsWith('.jai') ? 'jai' : name.endsWith('.md') ? 'md' : name.endsWith('.toml') ? 'toml' : 'file';
  return glyph(kind);
}

/** @param {'jai' | 'md' | 'toml' | 'file'} kind */
function glyph(kind) {
  if (kind === 'jai') {
    const img = document.createElement('img');
    img.src = assets + 'favicon.svg';
    img.width = img.height = 14;
    img.alt = '';
    img.className = 'pg-ico pg-ico-jai';
    return img;
  }
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('width', '14');
  svg.setAttribute('height', '14');
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', '2');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  svg.setAttribute('aria-hidden', 'true');
  svg.setAttribute('class', 'pg-ico pg-ico-' + kind);
  svg.innerHTML = ICONS[kind];
  return svg;
}

/** Ends the page that is mounted, if any. @type {(() => void) | null} */
let dispose = null;

export function mount() {
  const root = document.getElementById('pg');
  if (!root || root.dataset.ready) return;
  root.dataset.ready = '1';
  if (dispose) dispose();
  const page = /** @type {HTMLElement} */ (root);
  const base = /** @type {string} */ (page.dataset.base);
  assets = new URL(base + 'assets/', location.href).href;
  const find = /** @param {string} selector */ (selector) =>
    /** @type {HTMLElement} */ (page.querySelector(selector));

  const area = /** @type {HTMLTextAreaElement} */ (find('#pg-code'));
  const code = find('.pg-ed .code code');
  const gutter = find('.pg-gutter');
  const pre = find('.pg-ed .code');
  const line = find('.pg-line');
  const marksIn = find('.pg-marks > div');
  const bulb = find('.pg-bulb');
  const actionsMenu = find('.pg-actions');
  const tip = find('.pg-tip');
  const fileTabs = find('.pg-edit .pg-tabs');
  const fileList = find('#pg-files');
  const exampleList = find('#pg-examples');
  const status = find('.pg-status');
  const position = find('.pg-pos');
  const spaces = find('.pg-spaces');
  const language = find('.pg-lang');
  const output = find('.pg-output');
  const problems = find('.pg-problems');
  const count = find('.pg-count');
  const runButton = /** @type {HTMLButtonElement} */ (find('[data-act="run"]'));
  const stopButton = /** @type {HTMLButtonElement} */ (find('[data-act="stop"]'));
  const formatButton = /** @type {HTMLButtonElement} */ (find('[data-act="format"]'));

  /** The panels, which the reader can dock and resize; see playground-layout.js. */
  const wrap = /** @type {HTMLElement | null} */ (document.querySelector('body > .wrap'));
  const docking = createLayout(
    find('.pg-body'),
    { files: find('.pg-side'), code: find('.pg-edit'), results: find('.pg-res') },
    {
      main: find('.pg-main'),
      titles: { files: 'Files', code: 'Code', results: 'Output' },
      // Moving the editor in the page resets its scroll, so it is put back.
      around(move) {
        const top = area.scrollTop;
        const left = area.scrollLeft;
        move();
        area.scrollTop = top;
        area.scrollLeft = left;
        pre.scrollTop = top;
        pre.scrollLeft = left;
        gutter.scrollTop = top;
        paint();
      },
      changed(tree, free) {
        wrap?.classList.toggle('pg-free', tree !== null && free);
        try {
          localStorage.setItem(LAYOUT, JSON.stringify({ tree, free }));
        } catch {
          // The layout is not remembered.
        }
      },
    },
  );
  try {
    const saved = JSON.parse(localStorage.getItem(LAYOUT) ?? 'null');
    const tree = saved && saved.tree ? saved.tree : null;
    docking.apply(tree, Boolean(saved && saved.free));
    wrap?.classList.toggle('pg-free', tree !== null && Boolean(saved.free));
  } catch {
    // The page's own layout stays.
  }

  // The edges of the workspace drag outwards (or inwards) past the page's usual width.
  /** @param {number | null} width */
  function sizeWorkspace(width) {
    if (width === null) wrap?.style.removeProperty('--pg-max');
    else wrap?.style.setProperty('--pg-max', width + 'px');
  }
  try {
    const kept = Number(localStorage.getItem(WIDTH));
    if (kept > 0) sizeWorkspace(kept);
  } catch {
    // The page's usual width stays.
  }
  for (const handle of page.querySelectorAll('.pg-reach')) {
    handle.addEventListener('pointerdown', (e) => {
      const side = Number(/** @type {HTMLElement} */ (handle).dataset.reach);
      const start = wrap ? wrap.getBoundingClientRect().width : 0;
      const x0 = /** @type {PointerEvent} */ (e).clientX;
      /** @type {number | null} */
      let width = null;
      e.preventDefault();
      /** @type {HTMLElement} */ (handle).setPointerCapture(/** @type {PointerEvent} */ (e).pointerId);
      handle.classList.add('dragging');
      /** @param {Event} m */
      const move = (m) => {
        const dx = (/** @type {PointerEvent} */ (m).clientX - x0) * side;
        width = Math.round(Math.min(Math.max(start + 2 * dx, 640), window.innerWidth));
        sizeWorkspace(width);
      };
      const up = () => {
        handle.classList.remove('dragging');
        handle.removeEventListener('pointermove', move);
        handle.removeEventListener('pointerup', up);
        handle.removeEventListener('pointercancel', up);
        if (width === null) return;
        try {
          localStorage.setItem(WIDTH, String(width));
        } catch {
          // Not remembered.
        }
      };
      handle.addEventListener('pointermove', move);
      handle.addEventListener('pointerup', up);
      handle.addEventListener('pointercancel', up);
    });
  }

  page.dataset.layout = 'ready';

  /** @type {Options} */
  const options = { ...DEFAULTS };
  /** @type {Workspace} */
  let work = { files: { 'main.jai': HELLO }, active: 'main.jai', example: '' };
  /** @type {Example[]} */
  let examples = [];
  /** What the last run reported, then what the language server says. @type {Diagnostic[]} */
  let found = [];
  /** @type {Record<string, Diagnostic[]>} */
  let live = {};
  /** @type {{ engine: string, wasm: string, formatter: string, examples: string } | null} */
  let runtime = null;
  /** @type {Promise<string> | null} */
  let driver = null;
  /** What the editor held when it was last loaded, to tell whether it was edited. @type {Record<string, string>} */
  let initial = {};
  let running = false;
  /** A read-only file from the standard library, open beside the workspace's files. @type {{ name: string, uri: string, text: string } | null} */
  let viewing = null;

  // Saved state and the address ----------------------------------------------------------------

  /** @param {string} hash Takes the code and options an address fragment holds; false when it has no code. */
  function readFragment(hash) {
    const read = decode(hash, options);
    Object.assign(options, read.options);
    if (read.work) work = read.work;
    return read.work !== null;
  }

  function save() {
    try {
      localStorage.setItem(STORE, JSON.stringify({ work, options }));
    } catch {
      // Private windows refuse storage; the page still works.
    }
  }

  /** @type {number | undefined} */
  let saving;
  function changed() {
    clearTimeout(saving);
    saving = setTimeout(() => {
      save();
      history.replaceState(history.state, '', '#' + encode(work, options));
    }, 300);
  }

  // The editor ---------------------------------------------------------------------------------

  /** The diagnostics to mark: the run's, else the language server's. */
  function marks() {
    return found.length ? found : Object.values(live).flat();
  }

  /** Redraws the coloured code and the line numbers from the text. */
  function paint() {
    const text = area.value;
    code.innerHTML = colour(viewing ? viewing.name : work.active, text) + (text.endsWith('\n') ? ' ' : '');
    const lines = text.split('\n').length;
    /** @type {Map<number, string>} */
    const flagged = new Map();
    const rank = ['hint', 'information', 'warning', 'error'];
    for (const d of marks())
      if (!viewing && d.file === work.active && d.line > 0 && d.severity !== 'hint') {
        const had = flagged.get(d.line);
        if (!had || rank.indexOf(d.severity) > rank.indexOf(had)) flagged.set(d.line, d.severity);
      }
    gutter.textContent = '';
    const frag = document.createDocumentFragment();
    for (let n = 1; n <= lines; n++) {
      const row = document.createElement('span');
      row.textContent = String(n);
      const flag = flagged.get(n);
      if (flag) row.className = 'pg-' + flag;
      frag.append(row);
    }
    gutter.append(frag);
    squiggles();
    cursor();
  }

  /** Underlines each message's text with a wavy line, coloured by how serious it is. */
  function squiggles() {
    marksIn.textContent = '';
    if (viewing || !measure) return;
    const style = getComputedStyle(area);
    measure.font = `${style.fontSize} ${style.fontFamily}`;
    const width = measure.measureText('M').width;
    const height = parseFloat(style.lineHeight) || 20;
    const lines = area.value.split('\n');
    for (const d of marks()) {
      if (d.file !== work.active || d.line < 1) continue;
      const text = lines[d.line - 1] ?? '';
      const from = Math.max(d.column - 1, 0);
      let to = from;
      if (d.end && d.end.line === d.line) to = d.end.column - 1;
      else if (d.end) to = text.length;
      else while (to < text.length && /[A-Za-z0-9_]/.test(text[to])) to++;
      if (to <= from) to = Math.min(from + 1, Math.max(text.length, from + 1));
      const el = document.createElement('div');
      el.className = 'pg-sq ' + d.severity;
      el.style.left = parseFloat(style.paddingLeft) + from * width + 'px';
      el.style.top = parseFloat(style.paddingTop) + (d.line - 1) * height + 'px';
      el.style.width = (to - from) * width + 'px';
      el.style.height = height + 'px';
      marksIn.append(el);
    }
    marksIn.style.transform = `translate(${-area.scrollLeft}px, ${-area.scrollTop}px)`;
  }

  /** The diagnostics at a position, as text. @param {number} row @param {number} column */
  function messagesAt(row, column) {
    if (viewing) return [];
    const lines = area.value.split('\n');
    return marks()
      .filter((d) => {
        if (d.file !== work.active || d.line !== row + 1) return false;
        const text = lines[row] ?? '';
        const from = d.column - 1;
        let to = d.end && d.end.line === d.line ? d.end.column - 1 : from;
        if (!d.end) while (to < text.length && /[A-Za-z0-9_]/.test(text[to])) to++;
        return column >= from && column < Math.max(to, from + 1);
      })
      .map((d) => `${d.severity}: ${d.message}`);
  }

  /** Puts the current-line bar under the caret and says where it is. */
  function cursor() {
    const before = area.value.slice(0, area.selectionStart).split('\n');
    const at = before.length;
    const style = getComputedStyle(area);
    const height = parseFloat(style.lineHeight) || 20;
    line.style.height = height + 'px';
    line.style.transform = `translateY(${parseFloat(style.paddingTop) + (at - 1) * height - area.scrollTop}px)`;
    line.hidden = document.activeElement !== area;
    for (const [n, row] of [...gutter.children].entries()) row.classList.toggle('now', n + 1 === at);
    position.textContent = `Ln ${at}, Col ${before[at - 1].length + 1}`;
  }

  /** Puts the active file's text in the editor. */
  function show() {
    viewing = null;
    area.readOnly = false;
    area.value = work.files[work.active] ?? '';
    area.scrollTop = area.scrollLeft = 0;
    pre.scrollTop = pre.scrollLeft = gutter.scrollTop = 0;
    paint();
    tabs();
  }

  function tabs() {
    const names = Object.keys(work.files);
    fileTabs.textContent = '';
    fileList.textContent = '';
    for (const name of names) {
      const tab = document.createElement('button');
      tab.type = 'button';
      tab.setAttribute('role', 'tab');
      tab.setAttribute('aria-selected', String(!viewing && name === work.active));
      tab.dataset.file = name;
      tab.append(icon(name), name);
      fileTabs.append(tab);
      const item = document.createElement('li');
      const button = document.createElement('button');
      button.type = 'button';
      button.dataset.file = name;
      const label = document.createElement('span');
      label.textContent = name;
      button.append(icon(name), label);
      if (!viewing && name === work.active) button.setAttribute('aria-current', 'true');
      item.append(button);
      fileList.append(item);
    }
    if (viewing) {
      const tab = document.createElement('button');
      tab.type = 'button';
      tab.setAttribute('role', 'tab');
      tab.setAttribute('aria-selected', 'true');
      tab.title = 'Read-only: ' + viewing.uri.replace('file://', '');
      tab.append(icon(viewing.name), viewing.name.replace(/^stdlib\//, ''), lockIcon());
      fileTabs.append(tab);
    }
    const shown = viewing ? viewing.name : work.active;
    language.textContent = shown.endsWith('.md') ? 'Markdown' : shown.endsWith('.toml') ? 'TOML' : shown.endsWith('.jai') ? 'Jai' : 'Plain text';
    markExample();
  }

  /** Marks the example the workspace came from. */
  function markExample() {
    for (const b of exampleList.querySelectorAll('button'))
      if (b.dataset.example === work.example) b.setAttribute('aria-current', 'true');
      else b.removeAttribute('aria-current');
  }

  /** @param {string} name */
  function open(name) {
    if (!viewing) work.files[work.active] = area.value;
    work.active = name;
    hideTip();
    show();
    changed();
  }

  /**
   * Replaces `from..to` of the text, keeping the editor's undo history where the browser allows.
   * @param {number} from @param {number} to @param {string} text
   */
  function replace(from, to, text) {
    area.focus();
    area.setSelectionRange(from, to);
    if (!document.execCommand('insertText', false, text)) {
      area.setRangeText(text, from, to, 'end');
      edited();
    }
  }

  function edited() {
    hideBulb();
    work.files[work.active] = area.value;
    places.clear();
    found = [];
    live[work.active] = [];
    hideTip();
    paint();
    clearProblems();
    changed();
    syncServer();
  }

  /** Moves the caret to a position and brings it into view. @param {number} row @param {number} column */
  function reveal(row, column) {
    const lines = area.value.split('\n');
    const at = Math.min(Math.max(row, 1), lines.length);
    let offset = 0;
    for (let i = 0; i < at - 1; i++) offset += lines[i].length + 1;
    offset += Math.min(Math.max(column - 1, 0), lines[at - 1].length);
    area.focus();
    area.setSelectionRange(offset, offset);
    const height = parseFloat(getComputedStyle(area).lineHeight) || 20;
    area.scrollTop = Math.max(0, (at - 1) * height - area.clientHeight / 3);
    cursor();
  }

  area.addEventListener('input', edited);
  area.addEventListener('scroll', () => {
    pre.scrollTop = gutter.scrollTop = area.scrollTop;
    pre.scrollLeft = area.scrollLeft;
    marksIn.style.transform = `translate(${-area.scrollLeft}px, ${-area.scrollTop}px)`;
    hideTip();
    cursor();
  });
  for (const event of ['keyup', 'click', 'focus', 'blur'])
    area.addEventListener(event, cursor);
  // The caret moves on keydown and while dragging, before any of those events.
  const moved = () => {
    if (document.activeElement === area) {
      cursor();
      refreshBulb();
    }
  };
  document.addEventListener('selectionchange', moved);

  let escaped = false;
  area.addEventListener('blur', () => (escaped = false));
  area.addEventListener('keydown', (e) => {
    hideTip();
    const plain = !e.ctrlKey && !e.metaKey && !e.altKey;
    const text = area.value;
    const start = area.selectionStart;
    const end = area.selectionEnd;
    if (e.key === 'Tab' && plain && !escaped) {
      e.preventDefault();
      const unit = ' '.repeat(Number(options.indent) || 4);
      // The lines a selection touches are indented (or, with Shift, outdented) together.
      const from = text.lastIndexOf('\n', start - 1) + 1;
      if (start === end && !e.shiftKey) return replace(start, end, unit);
      const block = text.slice(from, end);
      const next = block
        .split('\n')
        .map((l) => (e.shiftKey ? l.replace(new RegExp(`^( {1,${unit.length}}|\\t)`), '') : unit + l))
        .join('\n');
      if (next !== block) replace(from, end, next);
    } else if (e.key === 'Enter' && plain && !e.shiftKey) {
      e.preventDefault();
      const before = text.slice(text.lastIndexOf('\n', start - 1) + 1, start);
      const indent = /^[ \t]*/.exec(before)?.[0] ?? '';
      replace(start, end, '\n' + indent + (/\{\s*$/.test(before) ? ' '.repeat(Number(options.indent) || 4) : ''));
    }
    escaped = e.key === 'Escape';
  });

  // Running ------------------------------------------------------------------------------------

  /** @type {Worker | null} */
  let worker = null;
  let serial = 0;
  /** @type {(() => void) | null} */
  let abort = null;

  /**
   * Sends one request to the compiler's worker.
   * @param {Record<string, unknown>} request @param {number} limit
   * @returns {Promise<{ result?: Played, ms?: number, ready?: boolean, lsp?: LspMessage[] }>}
   */
  function ask(request, limit) {
    if (!runtime) return Promise.reject(new Error('The compiler is not available.'));
    const known = runtime;
    return new Promise((resolve, reject) => {
      if (!worker) worker = new Worker(base + 'assets/run-worker.js', { type: 'module' });
      const w = worker;
      const id = ++serial;
      const done = () => {
        clearTimeout(timer);
        w.removeEventListener('message', onMessage);
        w.removeEventListener('error', onError);
        if (abort === stop) abort = null;
      };
      const stop = () => {
        done();
        w.terminate();
        if (worker === w) worker = null;
        lspReady = false;
        reject(new Error('Stopped.'));
      };
      const timer = setTimeout(stop, limit);
      /** @param {MessageEvent} e */
      const onMessage = (e) => {
        if (e.data.id !== id) return;
        done();
        if (e.data.error) {
          if (e.data.crashed && worker === w) {
            w.terminate();
            worker = null;
            lspReady = false;
          }
          reject(new Error(e.data.error));
        } else resolve(e.data);
      };
      /** @param {ErrorEvent} e */
      const onError = (e) => {
        done();
        if (worker === w) worker = null;
        reject(new Error(e.message || 'The compiler could not start in this browser.'));
      };
      w.addEventListener('message', onMessage);
      w.addEventListener('error', onError);
      if (request.files) abort = stop;
      w.postMessage({ id, engine: known.engine, wasm: known.wasm, ...request });
    });
  }

  /** The file a run starts from. */
  function mainFile() {
    return 'main.jai' in work.files
      ? 'main.jai'
      : (Object.keys(work.files).find((n) => n.endsWith('.jai')) ?? 'main.jai');
  }

  /** @param {boolean} on */
  function busy(on) {
    running = on;
    runButton.disabled = on || !runtime;
    formatButton.disabled = on || !runtime;
    stopButton.hidden = !on;
    runButton.hidden = on;
  }

  /** @param {string} text @param {string} [kind] */
  function say(text, kind = '') {
    status.textContent = text;
    status.className = 'pg-status' + (kind ? ' ' + kind : '');
  }

  /** @type {number | undefined} */
  let toasting;
  /**
   * A short message in a popover beside the control that was used.
   * @param {string} text @param {Element | null} anchor @param {string} [kind]
   */
  function toast(text, anchor, kind = '') {
    const box = find('.pg-toast');
    box.textContent = text;
    box.className = 'pg-toast' + (kind ? ' ' + kind : '');
    box.hidden = false;
    const host = page.getBoundingClientRect();
    const at = (anchor ?? formatButton).getBoundingClientRect();
    box.style.left = Math.max(0, Math.min(at.left - host.left, host.width - box.offsetWidth)) + 'px';
    box.style.top = at.bottom - host.top + 6 + 'px';
    clearTimeout(toasting);
    toasting = setTimeout(() => (box.hidden = true), kind === 'bad' ? 5000 : 2000);
  }

  function clearProblems() {
    problems.textContent = '';
    count.hidden = true;
  }

  /** @param {'output' | 'problems'} name */
  function tab(name) {
    for (const t of page.querySelectorAll('.pg-res [data-tab]'))
      t.setAttribute('aria-selected', String(/** @type {HTMLElement} */ (t).dataset.tab === name));
    output.hidden = name !== 'output';
    problems.hidden = name !== 'problems';
  }

  /** @param {Played} r */
  function printed(r) {
    output.textContent = '';
    const chunks =
      r.output && r.output.length
        ? r.output
        : [
            { stream: 'stdout', text: r.stdout || '' },
            { stream: 'stderr', text: r.stderr || '' },
          ];
    for (const c of chunks) {
      if (!c.text) continue;
      const span = document.createElement('span');
      if (c.stream === 'stderr') span.className = 'pg-stderr';
      span.textContent = c.text;
      output.append(span);
    }
    if (r.rendered && r.rendered.trim()) {
      const span = document.createElement('span');
      span.className = 'pg-rendered';
      span.textContent = (output.textContent ? '\n' : '') + r.rendered.trim() + '\n';
      output.append(span);
    }
  }

  /** @param {Diagnostic[]} list */
  function listProblems(list) {
    problems.textContent = '';
    const shown = options.show === 'errors' ? list.filter((d) => d.severity === 'error') : list;
    for (const d of shown) {
      const file = d.file.replace(/^\/workspace\//, '');
      const item = document.createElement('li');
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'pg-problem ' + d.severity;
      const where = document.createElement('span');
      where.className = 'pg-where';
      where.textContent = d.line > 0 ? `${file}:${d.line}:${d.column}` : file;
      const what = document.createElement('span');
      what.textContent = d.message;
      const kind = document.createElement('b');
      kind.textContent = d.severity;
      button.append(kind, where, what);
      if (file in work.files && d.line > 0)
        button.addEventListener('click', () => {
          if (file !== work.active) open(file);
          reveal(d.line, d.column);
        });
      else button.disabled = true;
      item.append(button);
      problems.append(item);
    }
    count.hidden = shown.length === 0;
    count.textContent = String(shown.length);
    count.className = 'pg-count' + (shown.some((d) => d.severity === 'error') ? ' error' : '');
    return shown;
  }

  async function run() {
    if (!runtime || runButton.disabled) return;
    if (!viewing) work.files[work.active] = area.value;
    busy(true);
    found = [];
    view('output');
    say('Running...', 'busy');
    clearProblems();
    const example = examples.find((x) => x.name === work.example);
    try {
      const { result, ms } = await ask(
        {
          files: work.files,
          main: mainFile(),
          play: {
            budget: STEPS[/** @type {keyof typeof STEPS} */ (options.limit)] ?? STEPS['200m'],
            args: ['main', ...words(options.args)],
            stdin: options.stdin || undefined,
          },
          assets:
            example && example.assets.length
              ? new URL(example.name + '/', runtime.engine).href
              : undefined,
        },
        WALL,
      );
      const r = /** @type {Played} */ (result);
      found = r.diagnostics || [];
      printed(r);
      const shown = listProblems(found);
      paint();
      const errors = found.filter((d) => d.severity === 'error').length;
      const time = Math.max(1, Math.round(ms ?? 0)) + ' ms';
      if (r.exitCode === 0 && !errors) say(`Exited 0 in ${time}`, 'ok');
      else if (r.failurePhase === 'compile' || (r.exitCode === null && errors && !output.textContent))
        say(`Did not compile: ${errors} ${errors === 1 ? 'error' : 'errors'}`, 'bad');
      else if (r.exitCode === null) say(`Stopped by a runtime error after ${time}`, 'bad');
      else say(`Exited ${r.exitCode} in ${time}`, 'bad');
      tab(
        shown.some((d) => d.severity === 'error') && !output.textContent.trim()
          ? 'problems'
          : 'output',
      );
    } catch (error) {
      output.textContent = '';
      say(error instanceof Error ? error.message : String(error), 'bad');
    } finally {
      busy(false);
      syncServer();
    }
  }

  /** The driver `jaifmt` runs as, with the file it formats. @param {string} name */
  async function formatter(name) {
    if (!runtime) throw new Error('The compiler is not available.');
    const url = runtime.formatter;
    driver ??= fetch(url).then((r) => {
      if (!r.ok) throw new Error('The formatter is missing from this copy of the docs.');
      return r.text();
    });
    const text = await driver;
    return text.replace(/^TARGET :: ".*";$/m, `TARGET :: ${JSON.stringify(name)};`);
  }

  async function format() {
    if (!runtime || formatButton.disabled) return;
    const name = work.active;
    if (viewing) return toast('The standard library is read-only.', formatButton, 'bad');
    if (!name.endsWith('.jai')) return toast(`${name} is not Jai code, so it is not formatted.`, formatButton, 'bad');
    busy(true);
    try {
      const files = {
        '__jaifmt__.jai': await formatter(name),
        [name]: area.value,
        'jacket.toml': formatConfig(options),
      };
      const { result } = await ask({ files, main: '__jaifmt__.jai' }, 30_000);
      const r = /** @type {Played} */ (result);
      if (r.exitCode === 0 && !(r.diagnostics || []).some((d) => d.severity === 'error')) {
        if (r.stdout !== area.value) replace(0, area.value.length, r.stdout);
        toast('Formatted', formatButton, 'ok');
      } else {
        toast(
          (r.stderr || r.rendered || 'The code could not be formatted.').trim().replace(/^jaifmt: /, ''),
          formatButton,
          'bad',
        );
      }
    } catch (error) {
      toast(error instanceof Error ? error.message : String(error), formatButton, 'bad');
    } finally {
      busy(false);
    }
  }

  // The language server ------------------------------------------------------------------------

  let lspReady = false;
  let lspId = 100;
  /** The version of each file the server has, and the text it was sent. @type {Map<string, { version: number, text: string }>} */
  const served = new Map();
  /** Library files the server has been given to read. @type {Set<string>} */
  const opened = new Set();
  /** @type {number | undefined} */
  let syncing;

  /** @param {string} name */
  const uriOf = (name) => ROOT + name;

  /** @param {Record<string, unknown>[]} messages @returns {Promise<LspMessage[]>} */
  async function lsp(messages) {
    const reply = await ask({ lsp: messages }, 30_000);
    return reply.lsp ?? [];
  }

  /** Makes the server's copy of the files the editor's, and shows what it reports. */
  function syncServer() {
    clearTimeout(syncing);
    if (!runtime) return;
    syncing = setTimeout(async () => {
      if (running) return syncServer();
      try {
        /** @type {Record<string, unknown>[]} */
        const out = [];
        if (!lspReady) {
          const start = await lsp([
            {
              jsonrpc: '2.0',
              id: 1,
              method: 'initialize',
              params: {
                processId: null,
                rootUri: ROOT,
                capabilities: { textDocument: { hover: { contentFormat: ['markdown', 'plaintext'] } } },
              },
            },
            { jsonrpc: '2.0', method: 'initialized', params: {} },
          ]);
          if (!start.some((m) => m.id === 1 && m.result)) return;
          lspReady = true;
          served.clear();
          opened.clear();
        }
        const wanted = Object.entries(work.files).filter(([n]) => n.endsWith('.jai'));
        for (const [name] of [...served]) {
          if (!wanted.some(([n]) => n === name)) {
            out.push({
              jsonrpc: '2.0',
              method: 'textDocument/didClose',
              params: { textDocument: { uri: uriOf(name) } },
            });
            served.delete(name);
          }
        }
        for (const [name, text] of wanted) {
          const had = served.get(name);
          if (had && had.text === text) continue;
          const version = (had?.version ?? 0) + 1;
          served.set(name, { version, text });
          out.push(
            had
              ? {
                  jsonrpc: '2.0',
                  method: 'textDocument/didChange',
                  params: { textDocument: { uri: uriOf(name), version }, contentChanges: [{ text }] },
                }
              : {
                  jsonrpc: '2.0',
                  method: 'textDocument/didOpen',
                  params: { textDocument: { uri: uriOf(name), languageId: 'jai', version, text } },
                },
          );
        }
        if (!out.length) return;
        showLive(await lsp(out));
        refreshBulb();
      } catch {
        // No language server in this build, or the worker was stopped: editing still works.
      }
    }, 400);
  }

  /** @param {LspMessage[]} messages */
  function showLive(messages) {
    let any = false;
    for (const m of messages) {
      if (m.method !== 'textDocument/publishDiagnostics' || !m.params) continue;
      const name = String(m.params.uri).replace(ROOT, '');
      const sent = served.get(name);
      if (!sent || (m.params.version !== undefined && m.params.version !== sent.version)) continue;
      live[name] = /** @type {any[]} */ (m.params.diagnostics).map((d) => ({
        severity: ['error', 'error', 'warning', 'information', 'hint'][d.severity ?? 1] ?? 'error',
        file: name,
        line: d.range.start.line + 1,
        column: d.range.start.character + 1,
        message: d.message,
        end: { line: d.range.end.line + 1, column: d.range.end.character + 1 },
        raw: d,
      }));
      any = true;
    }
    if (!any || found.length || running) return;
    listProblems(Object.values(live).flat());
    paint();
  }

  /** The document the editor shows, as the language server names it. */
  const docUri = () => (viewing ? viewing.uri : uriOf(work.active));
  /** Whether the server has the text the editor shows. */
  const synced = () =>
    viewing ? opened.has(viewing.uri) : served.get(work.active)?.text === area.value;

  /** Shows a library file, read-only, and lets the server answer for it. @param {string} uri @param {string} text */
  async function viewSource(uri, text) {
    if (!viewing) work.files[work.active] = area.value;
    viewing = { name: uri.replace(/^file:\/\/\//, ''), uri, text };
    area.readOnly = true;
    area.value = text;
    area.scrollTop = area.scrollLeft = 0;
    hideTip();
    paint();
    tabs();
    if (!opened.has(uri)) {
      opened.add(uri);
      try {
        await lsp([
          {
            jsonrpc: '2.0',
            method: 'textDocument/didOpen',
            params: { textDocument: { uri, languageId: 'jai', version: 1, text } },
          },
        ]);
      } catch {
        opened.delete(uri);
      }
    }
  }

  /** Where names are declared, by `document|row|column`; cleared when the text changes. @type {Map<string, { uri: string, row: number, column: number } | null>} */
  const places = new Map();
  /** The text of library files already fetched. @type {Map<string, string>} */
  const sources = new Map();

  /** Where the name at a position is declared, or null. @param {number} row @param {number} column */
  async function locate(row, column) {
    if (!lspReady || !synced()) return null;
    const key = `${docUri()}|${row}|${column}`;
    if (places.has(key)) return places.get(key) ?? null;
    const id = ++lspId;
    const answer = await lsp([
      {
        jsonrpc: '2.0',
        id,
        method: 'textDocument/definition',
        params: { textDocument: { uri: docUri() }, position: { line: row, character: column } },
      },
    ]);
    const result = answer.find((m) => m.id === id)?.result;
    const target = Array.isArray(result) ? result[0] : result;
    const range = target && (target.range ?? target.targetSelectionRange ?? target.targetRange);
    const uri = target && (target.uri ?? target.targetUri);
    const place = range && uri ? { uri: String(uri), row: range.start.line + 1, column: range.start.character + 1 } : null;
    places.set(key, place);
    return place;
  }

  /** Where the caret is now, as a history entry. */
  function here() {
    const before = area.value.slice(0, area.selectionStart).split('\n');
    return {
      pg: { uri: viewing ? viewing.uri : uriOf(work.active), row: before.length, column: before[before.length - 1].length + 1 },
    };
  }

  /**
   * Shows a place: a workspace file or a library file, with the caret there.
   * @param {{ uri: string, row: number, column: number }} place
   */
  async function show_(place) {
    if (place.uri.startsWith(ROOT)) {
      const name = place.uri.slice(ROOT.length);
      if (!(name in work.files)) return say(`${name} is not in the workspace.`, 'bad');
      if (viewing || name !== work.active) open(name);
    } else {
      let text = sources.get(place.uri);
      if (text === undefined) {
        const sid = ++lspId;
        const source = await lsp([{ jsonrpc: '2.0', id: sid, method: 'jai/source', params: { uri: place.uri } }]);
        const got = source.find((m) => m.id === sid)?.result;
        if (typeof got !== 'string') return say('That source is not available.', 'bad');
        text = got;
        sources.set(place.uri, text);
      }
      if (!viewing || viewing.uri !== place.uri) await viewSource(place.uri, text);
    }
    reveal(place.row, place.column);
  }

  /** Goes to where the name at a position is declared; Back returns to where you were. @param {number} row @param {number} column */
  async function definition(row, column) {
    if (!viewing && !work.active.endsWith('.jai')) return;
    if (!lspReady || !synced()) return say('Definitions are not ready yet.', 'bad');
    try {
      const place = await locate(row, column);
      if (!place) return say('No definition found.');
      say('');
      history.replaceState(here(), '', location.href);
      await show_(place);
      history.pushState({ pg: place }, '', location.href);
    } catch (error) {
      say(error instanceof Error ? error.message : String(error), 'bad');
    }
  }

  // Back and Forward walk the places jumped to.
  const popped = (/** @type {PopStateEvent} */ e) => {
    if (page.isConnected && e.state && e.state.pg) show_(e.state.pg).catch(() => {});
  };
  window.addEventListener('popstate', popped);

  /** The underline shown under a name that Ctrl or Cmd would follow. */
  const link = find('.pg-link');
  function hideLink() {
    link.hidden = true;
    area.classList.remove('goto');
  }
  /** @type {number} */
  let linking = 0;
  /** @param {MouseEvent} e */
  async function showLink(e) {
    const token = ++linking;
    const at = pointAt(e.clientX, e.clientY);
    if (!at) return hideLink();
    let place = null;
    try {
      place = await locate(at.row, at.column);
    } catch {
      place = null;
    }
    if (token !== linking || !place || !(e.ctrlKey || e.metaKey)) return hideLink();
    const style = getComputedStyle(area);
    measure && (measure.font = `${style.fontSize} ${style.fontFamily}`);
    const width = measure ? measure.measureText('M').width : 8;
    const height = parseFloat(style.lineHeight) || 20;
    link.style.left = parseFloat(style.paddingLeft) + at.from * width - area.scrollLeft + 'px';
    link.style.top = parseFloat(style.paddingTop) + at.row * height - area.scrollTop + 'px';
    link.style.width = (at.to - at.from) * width + 'px';
    link.style.height = height + 'px';
    link.hidden = false;
    area.classList.add('goto');
  }

  /** @type {{ title: string, edit: any }[]} */
  let actions = [];
  let bulbToken = 0;
  /** @type {number | undefined} */
  let bulbTimer;

  function hideBulb() {
    bulbToken++;
    actions = [];
    bulb.hidden = true;
    actionsMenu.hidden = true;
  }

  /** Shows a light bulb on the caret's line when the server offers fixes there. */
  function refreshBulb() {
    clearTimeout(bulbTimer);
    bulb.hidden = true;
    bulbTimer = setTimeout(async () => {
      if (viewing || !lspReady || !synced() || running) return;
      const token = ++bulbToken;
      const before = area.value.slice(0, area.selectionStart).split('\n');
      const row = before.length - 1;
      const character = before[row].length;
      const context = (live[work.active] ?? [])
        .filter((d) => d.raw && d.raw.range.start.line <= row && d.raw.range.end.line >= row)
        .map((d) => d.raw);
      const id = ++lspId;
      try {
        const answer = await lsp([
          {
            jsonrpc: '2.0',
            id,
            method: 'textDocument/codeAction',
            params: {
              textDocument: { uri: docUri() },
              range: { start: { line: row, character }, end: { line: row, character } },
              context: { diagnostics: context, only: ['quickfix'] },
            },
          },
        ]);
        if (token !== bulbToken) return;
        const list = /** @type {any[]} */ (answer.find((m) => m.id === id)?.result ?? []);
        actions = list.filter((a) => a.edit).map((a) => ({ title: String(a.title), edit: a.edit }));
        if (!actions.length) return;
        const style = getComputedStyle(area);
        const height = parseFloat(style.lineHeight) || 20;
        bulb.style.top = parseFloat(style.paddingTop) + row * height - area.scrollTop + (height - 18) / 2 + 'px';
        bulb.hidden = false;
      } catch {
        actions = [];
      }
    }, 350);
  }

  /** Applies a code action's edits to the open file. @param {any} edit */
  function applyEdit(edit) {
    /** @type {any[]} */
    let edits = [];
    for (const [uri, list] of Object.entries(edit.changes ?? {})) if (uri === docUri()) edits.push(...list);
    for (const c of edit.documentChanges ?? [])
      if (c.textDocument && c.textDocument.uri === docUri()) edits.push(...c.edits);
    const lines = area.value.split('\n');
    /** @param {{ line: number, character: number }} p */
    const offset = (p) => {
      let n = 0;
      for (let i = 0; i < p.line && i < lines.length; i++) n += lines[i].length + 1;
      return n + Math.min(p.character, (lines[p.line] ?? '').length);
    };
    edits = edits.map((e) => ({ from: offset(e.range.start), to: offset(e.range.end), text: e.newText }));
    edits.sort((a, b) => b.from - a.from);
    for (const e of edits) replace(e.from, e.to, e.text);
  }

  bulb.addEventListener('click', () => {
    actionsMenu.textContent = '';
    for (const a of actions) {
      const item = document.createElement('button');
      item.type = 'button';
      item.textContent = a.title;
      item.addEventListener('click', () => {
        actionsMenu.hidden = true;
        applyEdit(a.edit);
      });
      actionsMenu.append(item);
    }
    actionsMenu.style.top = bulb.style.top;
    actionsMenu.hidden = false;
  });

  /** The hover text for a position, or null. @param {number} row @param {number} column */
  async function hover(row, column) {
    if (!lspReady) return null;
    const id = ++lspId;
    if (!synced()) return null;
    const answer = await lsp([
      {
        jsonrpc: '2.0',
        id,
        method: 'textDocument/hover',
        params: { textDocument: { uri: docUri() }, position: { line: row, character: column } },
      },
    ]);
    const contents = answer.find((m) => m.id === id)?.result?.contents;
    if (!contents) return null;
    const value = typeof contents === 'string' ? contents : (contents.value ?? '');
    return String(value).trim() || null;
  }

  /** Text with `code` spans as elements. @param {HTMLElement} into @param {string} text */
  function inline(into, text) {
    for (const [n, part] of text.split('`').entries()) {
      if (!part) continue;
      if (n % 2) {
        const code = document.createElement('code');
        code.innerHTML = window.jacketHighlight(part);
        into.append(code);
      } else into.append(part);
    }
  }

  /**
   * Hover text from the server: a declaration, a line of facts, then Markdown notes; several
   * declarations (overloads) follow one another. Each is laid out as a block.
   * @param {string} text
   */
  function renderTip(text) {
    tip.textContent = '';
    const lines = text.split('\n');
    /** @type {HTMLElement | null} */
    let entry = null;
    /** @type {HTMLElement | null} */
    let paragraph = null;
    /** @type {HTMLElement | null} */
    let list = null;
    let fenced = false;
    /** @type {string[]} */
    let fence = [];
    let afterSignature = false;
    const start = () => {
      entry = document.createElement('section');
      tip.append(entry);
      paragraph = list = null;
    };
    const host = () => {
      if (!entry) start();
      return /** @type {HTMLElement} */ (entry);
    };
    for (const raw of lines) {
      const line = raw.trimEnd();
      if (/^```/.test(line)) {
        if (fenced) {
          const pre = document.createElement('pre');
          pre.innerHTML = window.jacketHighlight(fence.join('\n'));
          host().append(pre);
          fence = [];
        }
        fenced = !fenced;
        paragraph = list = null;
        continue;
      }
      if (fenced) {
        fence.push(raw);
        continue;
      }
      const note = /^(error|warning|information|hint): (.*)$/.exec(line);
      if (note) {
        const div = document.createElement('div');
        div.className = 'pg-tip-note ' + note[1];
        div.textContent = note[2];
        tip.append(div);
        entry = null;
        continue;
      }
      if (/^[A-Za-z_$][\w.$]*\s*::?[^=]/.test(line) || /^[A-Za-z_][\w.]*\s*:=/.test(line)) {
        start();
        const sig = document.createElement('pre');
        sig.className = 'pg-sig';
        sig.innerHTML = window.jacketHighlight(line);
        host().append(sig);
        afterSignature = true;
        continue;
      }
      if (!line.trim()) {
        paragraph = list = null;
        afterSignature = false;
        continue;
      }
      if (afterSignature) {
        const facts = document.createElement('div');
        facts.className = 'pg-facts';
        facts.textContent = line;
        host().append(facts);
        continue;
      }
      const item = /^\s*[-*]\s+(.*)$/.exec(line);
      if (item) {
        if (!list) {
          list = document.createElement('ul');
          host().append(list);
          paragraph = null;
        }
        const li = document.createElement('li');
        inline(li, item[1]);
        list.append(li);
        continue;
      }
      list = null;
      if (!paragraph) {
        paragraph = document.createElement('p');
        host().append(paragraph);
      } else paragraph.append(' ');
      inline(paragraph, line.trim());
    }
  }

  /** @type {number | undefined} */
  let dwell;
  let hovered = '';
  function hideTip() {
    clearTimeout(dwell);
    clearTimeout(leaving);
    hovered = '';
    tip.hidden = true;
  }

  const measure = document.createElement('canvas').getContext('2d');

  /** The name under a pointer: its row, column and where it starts, or null. @param {number} x @param {number} y */
  function pointAt(x, y) {
    const style = getComputedStyle(area);
    const box = area.getBoundingClientRect();
    if (!measure) return null;
    measure.font = `${style.fontSize} ${style.fontFamily}`;
    const width = measure.measureText('M').width;
    const height = parseFloat(style.lineHeight) || 20;
    const row = Math.floor((y - box.top - parseFloat(style.paddingTop) + area.scrollTop) / height);
    const lineText = area.value.split('\n')[row];
    if (lineText === undefined) return null;
    const column = Math.floor((x - box.left - parseFloat(style.paddingLeft) + area.scrollLeft) / width);
    if (column < 0 || column >= lineText.length || !/[A-Za-z0-9_#]/.test(lineText[column])) return null;
    let from = column;
    while (from > 0 && /[A-Za-z0-9_#]/.test(lineText[from - 1])) from--;
    let to = column + 1;
    while (to < lineText.length && /[A-Za-z0-9_]/.test(lineText[to])) to++;
    return { row, column, from, to };
  }

  // Ctrl or Cmd with a click, or F12, goes to the definition.
  area.addEventListener('mousedown', (e) => {
    if (e.button !== 0 || !(e.metaKey || e.ctrlKey)) return;
    const at = pointAt(e.clientX, e.clientY);
    if (!at) return;
    e.preventDefault();
    hideTip();
    definition(at.row, at.column);
  });
  area.addEventListener('contextmenu', (e) => {
    if (!e.ctrlKey) return;
    const at = pointAt(e.clientX, e.clientY);
    if (!at) return;
    e.preventDefault();
    definition(at.row, at.column);
  });
  const pointer = (/** @type {KeyboardEvent} */ e) => {
    if (!(e.metaKey || e.ctrlKey)) hideLink();
  };
  document.addEventListener('keydown', pointer);
  document.addEventListener('keyup', pointer);
  area.addEventListener('mousemove', (e) => {
    clearTimeout(dwell);
    if (e.ctrlKey || e.metaKey) {
      hideTip();
      showLink(e);
      return;
    }
    if (linking) hideLink();
    if (!tip.hidden && e.target === area) {
      // Moving off the word hides it; staying on it keeps it.
    }
    const x = e.clientX;
    const y = e.clientY;
    dwell = setTimeout(async () => {
      const at = pointAt(x, y);
      if (!at) return hideTip();
      const { row, column, from } = at;
      const key = `${row}:${from}`;
      if (key === hovered && !tip.hidden) return;
      let text = null;
      try {
        text = await hover(row, column);
      } catch {
        text = null;
      }
      const notes = messagesAt(row, column);
      if (notes.length) text = notes.join('\n\n') + (text ? '\n\n' + text : '');
      if (!text) return hideTip();
      hovered = key;
      renderTip(text);
      tip.hidden = false;
      const host = /** @type {HTMLElement} */ (tip.offsetParent).getBoundingClientRect();
      const left = Math.min(Math.max(8, x - host.left), host.width - tip.offsetWidth - 8);
      const above = y - host.top - tip.offsetHeight - 12;
      tip.style.left = left + 'px';
      tip.style.top = (above > 4 ? above : y - host.top + 18) + 'px';
    }, 350);
  });
  // Leaving the text for the popover keeps it, so its contents can be selected and scrolled.
  /** @type {number | undefined} */
  let leaving;
  area.addEventListener('mouseleave', () => {
    clearTimeout(dwell);
    hideLink();
    leaving = setTimeout(hideTip, 250);
  });
  tip.addEventListener('mouseenter', () => clearTimeout(leaving));
  tip.addEventListener('mouseleave', hideTip);
  area.addEventListener('mouseenter', () => clearTimeout(leaving));

  // Controls -----------------------------------------------------------------------------------

  function syncOptions() {
    for (const el of page.querySelectorAll('[data-opt]')) {
      const field = /** @type {HTMLInputElement} */ (el);
      field.value = options[/** @type {keyof Options} */ (field.dataset.opt)] ?? '';
    }
    spaces.textContent = `Spaces: ${options.indent}`;
  }

  page.addEventListener('change', (e) => {
    const field = /** @type {HTMLInputElement} */ (e.target);
    const key = /** @type {keyof Options | undefined} */ (field.dataset.opt);
    if (!key) return;
    options[key] = field.value;
    if (key === 'show') {
      listProblems(marks());
      paint();
    }
    spaces.textContent = `Spaces: ${options.indent}`;
    changed();
  });

  for (const field of page.querySelectorAll('[data-opt]'))
    field.addEventListener('input', () => {
      const el = /** @type {HTMLInputElement} */ (field);
      options[/** @type {keyof Options} */ (el.dataset.opt)] = el.value;
      changed();
    });

  /** Opens the menu a toolbar button names, or closes them all. @param {string} which @param {boolean} open */
  function menu(which, open) {
    for (const b of page.querySelectorAll('.pg-menu > [data-act]')) {
      const on = open && /** @type {HTMLElement} */ (b).dataset.act === which;
      b.setAttribute('aria-expanded', String(on));
      const panel = /** @type {HTMLElement | null} */ (b.nextElementSibling);
      if (panel) panel.hidden = !on;
    }
  }

  /** On a phone one pane shows at a time. @param {string} name */
  function view(name) {
    page.dataset.view = name;
    for (const b of page.querySelectorAll('.pg-views button'))
      if (/** @type {HTMLElement} */ (b).dataset.view === name) b.setAttribute('aria-current', 'true');
      else b.removeAttribute('aria-current');
  }

  page.addEventListener('click', async (e) => {
    const target = /** @type {HTMLElement} */ (e.target);
    const file = target.closest('[data-file]');
    if (file instanceof HTMLElement && file.dataset.file) {
      open(file.dataset.file);
      view('code');
    }
    const act = target.closest('[data-act]')?.getAttribute('data-act');
    const result = target.closest('[data-tab]');
    if (result instanceof HTMLElement) tab(/** @type {'output' | 'problems'} */ (result.dataset.tab));
    if (act === 'run') run();
    else if (act === 'stop' && abort) abort();
    else if (act === 'format') format();
    else if (act === 'options' || act === 'examples')
      menu(act, target.closest('[data-act]')?.getAttribute('aria-expanded') !== 'true');
    else if (act === 'reset-layout') {
      menu('', false);
      docking.reset();
      sizeWorkspace(null);
      try {
        localStorage.removeItem(WIDTH);
      } catch {
        // Nothing was kept.
      }
    } else if (act === 'spaces') {
      options.indent = options.indent === '4' ? '8' : options.indent === '8' ? '2' : '4';
      syncOptions();
      changed();
    } else if (act === 'clear') {
      output.textContent = '';
      say('');
    } else if (act === 'share') {
      save();
      history.replaceState(history.state, '', '#' + encode(work, options));
      try {
        await navigator.clipboard.writeText(location.href);
        toast('Link copied', target.closest('[data-act]'), 'ok');
      } catch {
        toast('Copy the link from the address bar', target.closest('[data-act]'), 'bad');
      }
    }
    const switcher = target.closest('[data-view]');
    if (switcher instanceof HTMLElement && switcher.closest('.pg-views'))
      view(switcher.dataset.view ?? 'code');
    if (!target.closest('.pg-menu')) menu('', false);
  });

  exampleList.addEventListener('click', async (e) => {
    const button = /** @type {HTMLElement} */ (e.target).closest('button');
    if (!button) return;
    const name = button.dataset.example ?? '';
    menu('', false);
    if (name === work.example) return view('code');
    const dirty = Object.entries(work.files).some(([n, t]) => t !== (initial[n] ?? null));
    if (dirty && !(await confirmReplace())) return;
    if (!name) {
      load({ 'main.jai': HELLO }, 'main.jai', '');
      return view('code');
    }
    const entry = examples.find((x) => x.name === name);
    if (!entry || !runtime) return;
    say('Loading the example...', 'busy');
    try {
      const dir = new URL(entry.name + '/', runtime.examples);
      const texts = await Promise.all(
        entry.files.map(async (f) => {
          const r = await fetch(new URL(f, dir));
          if (!r.ok) throw new Error(`Could not load ${f}.`);
          return /** @type {[string, string]} */ ([f, await r.text()]);
        }),
      );
      load(Object.fromEntries(texts), entry.main, name);
      say('');
      view('code');
    } catch (error) {
      say(error instanceof Error ? error.message : String(error), 'bad');
    }
  });

  /** Asks before the editor's files are replaced. */
  function confirmReplace() {
    const dialog = /** @type {HTMLDialogElement} */ (find('.pg-modal'));
    return new Promise((resolve) => {
      dialog.returnValue = '';
      dialog.addEventListener('close', () => resolve(dialog.returnValue === 'ok'), { once: true });
      dialog.showModal();
    });
  }

  /** @param {Record<string, string>} files @param {string} active @param {string} example */
  function load(files, active, example) {
    work = { files, active: active in files ? active : Object.keys(files)[0], example };
    initial = { ...files };
    places.clear();
    found = [];
    live = {};
    output.textContent = '';
    clearProblems();
    show();
    changed();
    syncServer();
  }

  /** @param {KeyboardEvent} e */
  const keys = (e) => {
    if (!page.isConnected) return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key === 'Enter') {
      e.preventDefault();
      run();
    } else if (
      (e.altKey && e.shiftKey && e.code === 'KeyF') ||
      (mod && !e.shiftKey && !e.altKey && e.key === 's')
    ) {
      e.preventDefault();
      format();
    } else if (e.key === 'F12' && document.activeElement === area) {
      e.preventDefault();
      const before = area.value.slice(0, area.selectionStart).split('\n');
      definition(before.length - 1, before[before.length - 1].length);
    } else if (e.key === 'Escape') {
      menu('', false);
      hideTip();
    }
  };
  document.addEventListener('keydown', keys);

  const onHash = () => {
    if (!page.isConnected || history.state?.pg || !readFragment(location.hash)) return;
    syncOptions();
    initial = { ...work.files };
    show();
    syncServer();
  };
  window.addEventListener('hashchange', onHash);

  // Start --------------------------------------------------------------------------------------

  /** @type {{ work?: Workspace, options?: Options }} */
  let saved = {};
  try {
    saved = JSON.parse(localStorage.getItem(STORE) ?? '{}');
  } catch {
    // Without storage the page starts from the example.
  }
  if (saved.options) Object.assign(options, saved.options);
  if (saved.work && saved.work.files && Object.keys(saved.work.files).length) work = saved.work;
  // Zulip's code playgrounds pass the raw text as `?code=`.
  const raw = new URLSearchParams(location.search).get('code');
  if (!readFragment(location.hash) && raw !== null)
    work = { files: { 'main.jai': raw }, active: 'main.jai', example: '' };
  initial = { ...work.files };
  syncOptions();
  area.disabled = false;
  show();
  tab('output');

  // A page swap that leaves the playground ends it.
  const watcher = new MutationObserver(() => {
    if (!page.isConnected) dispose?.();
  });
  watcher.observe(document.body, { childList: true, subtree: true });
  dispose = () => {
    dispose = null;
    watcher.disconnect();
    document.removeEventListener('keydown', keys);
    document.removeEventListener('selectionchange', moved);
    window.removeEventListener('popstate', popped);
    docking.destroy();
    wrap?.classList.remove('pg-free');
    wrap?.style.removeProperty('--pg-max');
    clearTimeout(bulbTimer);
    document.removeEventListener('keydown', pointer);
    document.removeEventListener('keyup', pointer);
    window.removeEventListener('hashchange', onHash);
    clearTimeout(saving);
    clearTimeout(syncing);
    clearTimeout(dwell);
    worker?.terminate();
    worker = null;
  };

  (async () => {
    try {
      const response = await fetch(/** @type {string} */ (page.dataset.runtime));
      if (!response.ok) throw new Error('runtime.json is missing from this copy of the docs.');
      const spec = await response.json();
      const resolve = /** @param {string} path */ (path) =>
        new URL(path, new URL(base, location.href)).href;
      runtime = {
        engine: resolve(spec.engine),
        wasm: resolve(spec.wasm),
        formatter: resolve(spec.formatter),
        examples: resolve(spec.examples),
      };
    } catch (error) {
      say(error instanceof Error ? error.message : String(error), 'bad');
      return;
    }
    busy(false);
    say('Loading the compiler...', 'busy');
    fetch(runtime.examples)
      .then((r) => r.json())
      .then((index) => {
        examples = index.examples;
        for (const x of [{ name: '', title: 'Hello, world' }, ...examples]) {
          const item = document.createElement('li');
          const button = document.createElement('button');
          button.type = 'button';
          button.dataset.example = x.name;
          button.textContent = x.title;
          if ('summary' in x) button.title = x.summary;
          item.append(button);
          exampleList.append(item);
        }
        markExample();
      })
      .catch(() => {});
    try {
      await ask({ warm: true }, WALL);
      say('');
      syncServer();
    } catch (error) {
      say(error instanceof Error ? error.message : String(error), 'bad');
    }
  })();
}

if (document.getElementById('pg')) mount();
