// @ts-check
// What the playground keeps in the address and in storage, and the small text rules around it.
// Nothing here touches the page, so tools/browser/test_playground_state.ts runs it under Node.

/**
 * @typedef {{ limit: string, args: string, stdin: string, show: string, indent: string, braces: string }} Options
 * @typedef {{ files: Record<string, string>, active: string, example: string }} Workspace
 */

/** What each option is until a reader changes it. @type {Options} */
export const DEFAULTS = {
  limit: '200m',
  args: '',
  stdin: '',
  show: 'all',
  indent: '4',
  braces: 'same_line',
};

/** Interpreter steps for each step-limit choice. */
export const STEPS = { '20m': 20e6, '200m': 200e6, '2b': 2e9, '20b': 20e9 };

/** @param {string} text */
export function toBase64(text) {
  const bytes = new TextEncoder().encode(text);
  let bin = '';
  for (let i = 0; i < bytes.length; i += 0x8000)
    bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/** @param {string} text */
export function fromBase64(text) {
  const bin = atob(text.replace(/-/g, '+').replace(/_/g, '/'));
  return new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)));
}

/**
 * The address fragment for the code and the options that differ from the defaults. A single
 * `main.jai` is `code=` (the form the docs' example links use); more files are `files=`.
 * @param {Workspace} work @param {Options} options
 */
export function encode(work, options) {
  const names = Object.keys(work.files);
  /** @type {string[]} */
  const parts = [];
  if (names.length === 1 && names[0] === 'main.jai')
    parts.push('code=' + toBase64(work.files['main.jai']));
  else
    parts.push(
      'files=' + toBase64(JSON.stringify(work.files)),
      'file=' + encodeURIComponent(work.active),
    );
  if (work.example) parts.push('example=' + encodeURIComponent(work.example));
  for (const key of /** @type {(keyof Options)[]} */ (Object.keys(DEFAULTS)))
    if (options[key] !== DEFAULTS[key]) parts.push(key + '=' + encodeURIComponent(options[key]));
  return parts.join('&');
}

/**
 * The options and code a fragment holds. `work` is null when it holds no code (or it is broken);
 * the options it names are still returned.
 * @param {string} hash @param {Options} from
 * @returns {{ work: Workspace | null, options: Options }}
 */
export function decode(hash, from) {
  /** @type {Record<string, string>} */
  const values = {};
  for (const part of hash.replace(/^#/, '').split('&')) {
    const at = part.indexOf('=');
    if (at > 0) values[part.slice(0, at)] = part.slice(at + 1);
  }
  const options = { ...from };
  try {
    for (const key of /** @type {(keyof Options)[]} */ (Object.keys(DEFAULTS)))
      if (key in values) options[key] = decodeURIComponent(values[key]);
    /** @type {Workspace | null} */
    let work = null;
    if (values.files) {
      const files = JSON.parse(fromBase64(values.files));
      if (files && typeof files === 'object' && Object.keys(files).length) {
        const wanted = values.file && decodeURIComponent(values.file);
        work = {
          files,
          active: wanted && wanted in files ? wanted : Object.keys(files)[0],
          example: '',
        };
      }
    } else if (values.code !== undefined) {
      work = { files: { 'main.jai': fromBase64(values.code) }, active: 'main.jai', example: '' };
    }
    if (work && values.example) work.example = decodeURIComponent(values.example);
    return { work, options };
  } catch {
    return { work: null, options: from };
  }
}

/** The words of an argument line, with quotes grouping. @param {string} line */
export function words(line) {
  return [...line.matchAll(/"([^"]*)"|'([^']*)'|(\S+)/g)].map((m) => m[1] ?? m[2] ?? m[3]);
}

/** The `jacket.toml` Format runs with. @param {Options} options */
export function formatConfig(options) {
  const indent = [2, 4, 8].includes(Number(options.indent)) ? Number(options.indent) : 4;
  const braces = options.braces === 'preserve' ? 'preserve' : 'same_line';
  return `[format]\nindent_width = ${indent}\nbrace_style = "${braces}"\n`;
}
