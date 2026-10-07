// Running jaifmt on a document's text. Pure apart from spawning the process, so the unit tests
// cover the config lookup, ignore globs and the edit computation without VS Code.
import { spawn } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

export const CONFIG_FILE = "jaifmt.toml";

/** The nearest `jaifmt.toml` in `dir` or above it. */
export function findConfig(dir: string, isFile: (file: string) => boolean = defaultIsFile): string | undefined {
  let current = path.resolve(dir);
  for (;;) {
    const candidate = path.join(current, CONFIG_FILE);
    if (isFile(candidate)) return candidate;
    const parent = path.dirname(current);
    if (parent === current) return undefined;
    current = parent;
  }
}

function defaultIsFile(file: string): boolean {
  try {
    return fs.statSync(file).isFile();
  } catch {
    return false;
  }
}

/** The `ignore = [...]` globs of a `jaifmt.toml` (the one-line string array jaifmt accepts). */
export function ignoreGlobs(toml: string): string[] {
  for (const raw of toml.split(/\r?\n/)) {
    const line = raw.replace(/^\s+/, "");
    const match = /^ignore\s*=\s*\[(.*)\]\s*(?:#.*)?$/.exec(line);
    if (!match) continue;
    return [...match[1].matchAll(/"((?:[^"\\]|\\.)*)"/g)].map((m) => m[1].replace(/\\(.)/g, "$1"));
  }
  return [];
}

/**
 * jaifmt's glob semantics: `*` and `?` stay within a path component, `**` crosses components,
 * and a pattern that matches a directory covers everything below it.
 */
export function globToRegExp(glob: string): RegExp {
  let source = "";
  for (let i = 0; i < glob.length; i++) {
    const c = glob[i];
    if (c === "*" && glob[i + 1] === "*") {
      i += 1;
      if (glob[i + 1] === "/") {
        i += 1;
        source += "(?:.*/)?";
      } else {
        source += ".*";
      }
    } else if (c === "*") {
      source += "[^/]*";
    } else if (c === "?") {
      source += "[^/]";
    } else {
      source += c.replace(/[.+^${}()|[\]\\]/g, "\\$&");
    }
  }
  return new RegExp(`^${source}(?:/.*)?$`);
}

/** Whether `file` is ignored by the globs of the config at `configPath`. */
export function isIgnored(configPath: string, globs: string[], file: string): boolean {
  const relative = path.relative(path.dirname(configPath), file).split(path.sep).join("/");
  if (relative.startsWith("../")) return false;
  return globs.some((glob) => globToRegExp(glob.replace(/^\.\//, "")).test(relative));
}

export interface Replacement {
  /** Offsets into the original text. */
  start: number;
  end: number;
  text: string;
}

/**
 * The smallest single replacement turning `before` into `after`: the common prefix and suffix
 * stay, so the cursor and folds outside the changed region do not move.
 */
export function minimalReplacement(before: string, after: string): Replacement | undefined {
  if (before === after) return undefined;
  let start = 0;
  const limit = Math.min(before.length, after.length);
  while (start < limit && before.charCodeAt(start) === after.charCodeAt(start)) start++;
  let endBefore = before.length;
  let endAfter = after.length;
  while (endBefore > start && endAfter > start && before.charCodeAt(endBefore - 1) === after.charCodeAt(endAfter - 1)) {
    endBefore--;
    endAfter--;
  }
  // Do not split a surrogate pair.
  if (start > 0 && isSurrogate(before.charCodeAt(start - 1), 0xd800)) start--;
  if (endBefore < before.length && isSurrogate(before.charCodeAt(endBefore), 0xdc00)) {
    endBefore++;
    endAfter++;
  }
  return { start, end: endBefore, text: after.slice(start, endAfter) };
}

function isSurrogate(code: number, base: number): boolean {
  return code >= base && code <= base + 0x3ff;
}

export interface FormatResult {
  ok: boolean;
  stdout: string;
  stderr: string;
  code: number | null;
}

/**
 * Formats `text` with `jaifmt --stdin`, started in `cwd` so jaifmt finds the `jaifmt.toml` that
 * applies to the document (it looks in the working directory and its parents).
 */
export function runJaifmt(executable: string, text: string, cwd: string, token?: { onCancellationRequested(listener: () => void): unknown }): Promise<FormatResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, ["--stdin"], { cwd, windowsHide: true });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (code) =>
      resolve({
        ok: code === 0,
        code,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      }),
    );
    token?.onCancellationRequested(() => child.kill());
    child.stdin.on("error", () => {});
    child.stdin.end(text, "utf8");
  });
}
