// What the Jai debug configuration builds: pure, so the unit tests can load it without the editor.
import * as path from "node:path";

/** The `jaic` arguments that build `file` for debugging into `program`. */
export function buildArguments(file: string, program: string, extra: readonly string[]): string[] {
  return [file, "-O0", "-exe", program, ...extra];
}

/** Where the debug build of `file` goes: next to it, named after it. */
export function programPath(file: string, platform: NodeJS.Platform): string {
  const stem = path.basename(file, path.extname(file));
  return path.join(path.dirname(file), platform === "win32" ? `${stem}.exe` : stem);
}
