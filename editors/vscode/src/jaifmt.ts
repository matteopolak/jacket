// Running jaifmt on a document's text. Pure apart from spawning the process, so the unit tests
// cover the ignore query and the edit computation without VS Code.
import { spawn } from "node:child_process";

export const CONFIG_FILE = "jaifmt.toml";

/**
 * Whether the `ignore` list of the config that applies to `file` covers it, asked of
 * `jaifmt --is-ignored` (exit 0 ignored, 1 formatted; it finds the config from the
 * absolute path) so the globs have one implementation. Any
 * other outcome (a config error, no executable) counts as not ignored; formatting then reports it.
 */
export function isIgnored(executable: string, file: string): Promise<boolean> {
  return new Promise((resolve) => {
    const child = spawn(executable, ["--is-ignored", file], { windowsHide: true, stdio: "ignore" });
    child.on("error", () => resolve(false));
    child.on("close", (code) => resolve(code === 0));
  });
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
