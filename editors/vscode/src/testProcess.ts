// Starting `jacket test` and reading its JSON events as they arrive. Pure of VS Code.
import { spawn } from "node:child_process";
import * as path from "node:path";
import { type Event, parseEvent } from "./testEvents";

/** `jacket test --list --format json`: every test, one `list` event each. */
export function listArguments(extra: readonly string[]): string[] {
  return ["test", "--list", "--format", "json", ...extra];
}

export interface Selection {
  /** Flags that choose the tests' roots and filter (a code lens carries them). */
  flags?: readonly string[];
  /** Whole test names to run; none (and no flags) runs everything. */
  names?: readonly string[];
}

/** `jacket test --format json` for `selection`. `extra` comes last, so `--` in it still ends the options. */
export function runArguments(selection: Selection, extra: readonly string[]): string[] {
  const names = selection.names ?? [];
  return ["test", "--format", "json", ...(selection.flags ?? []), ...(names.length > 0 ? ["--exact", ...names] : []), ...extra];
}

export interface Finish {
  /** Null when the process was killed. */
  code: number | null;
  /** Everything it wrote to standard error: root notices, warnings, build errors. */
  stderr: string;
}

export interface RunOptions {
  cwd: string;
  env: Record<string, string>;
  onEvent(event: Event): void;
  signal?: AbortSignal;
}

/** Runs `executable` and calls `onEvent` for each event line as soon as the line is complete. */
export function runJacket(executable: string, args: readonly string[], options: RunOptions): Promise<Finish> {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, args, { cwd: options.cwd, env: options.env, stdio: ["ignore", "pipe", "pipe"], signal: options.signal });
    let pending = "";
    let stderr = "";
    const line = (text: string) => {
      const event = parseEvent(text);
      if (event) options.onEvent(event);
    };
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      const lines = (pending + chunk).split("\n");
      pending = lines.pop() ?? "";
      for (const text of lines) line(text);
    });
    child.stderr.setEncoding("utf8");
    child.stderr.on("data", (chunk: string) => (stderr += chunk));
    child.on("error", (error: Error) => {
      // Aborting ends the process; that is a result, not a failure.
      if (error.name === "AbortError") resolve({ code: null, stderr });
      else reject(error);
    });
    child.on("close", (code) => {
      line(pending);
      resolve({ code, stderr });
    });
  });
}

/** `PATH` with the folder of `tool` first, so `jacket` finds the `jaic` the editor is set to use. */
export function withToolFolder(env: Record<string, string>, tool: string | undefined, platform: NodeJS.Platform): Record<string, string> {
  if (!tool) return env;
  const key = platform === "win32" ? (Object.keys(env).find((k) => k.toLowerCase() === "path") ?? "Path") : "PATH";
  const separator = platform === "win32" ? ";" : ":";
  const folder = (platform === "win32" ? path.win32 : path.posix).dirname(tool);
  return { ...env, [key]: env[key] ? `${folder}${separator}${env[key]}` : folder };
}
