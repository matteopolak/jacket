// Detecting a PATH toolchain older than the one this extension expects. Pure apart from the
// injectable probe, so the unit tests need no executables.
import { execFile } from "node:child_process";
import { compareVersions } from "./toolchain";

/** The version in `jailsp --version` / `jaic --version` output (`jailsp 0.25.0`). */
export function parseVersionOutput(output: string): string | undefined {
  return /\b(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)\b/.exec(output)?.[1];
}

/** Asks an executable for its version; undefined when it cannot be run or says nothing usable. */
export function probeVersion(executable: string): Promise<string | undefined> {
  return new Promise((resolve) => {
    execFile(executable, ["--version"], { timeout: 5000, windowsHide: true }, (error, stdout) => {
      resolve(error ? undefined : parseVersionOutput(stdout));
    });
  });
}

/** The key under which "keep using it" is remembered: a newer version on the same path warns again. */
export function acceptedKey(executable: string, version: string): string {
  return `${executable}@${version}`;
}

export interface StaleCheck {
  /** The executable discovery picked. */
  found: { path: string; source: string };
  /** The version the extension expects. */
  expected: string;
  accepted: readonly string[];
  /** Whether `jai.toolchain.preferDownloaded` is already on. */
  preferDownloaded: boolean;
  probe: (executable: string) => Promise<string | undefined>;
}

export interface StaleToolchain {
  path: string;
  version: string;
  expected: string;
}

/** The PATH executable that is older than expected and not yet accepted, if any. */
export async function staleToolchain(check: StaleCheck): Promise<StaleToolchain | undefined> {
  if (check.found.source !== "path" || check.preferDownloaded) return undefined;
  const version = await check.probe(check.found.path);
  if (!version || compareVersions(version, check.expected) >= 0) return undefined;
  if (check.accepted.includes(acceptedKey(check.found.path, version))) return undefined;
  return { path: check.found.path, version, expected: check.expected };
}

export function staleMessage(stale: StaleToolchain): string {
  return (
    `Jai: ${stale.path} is version ${stale.version}, but this extension expects ${stale.expected}. ` +
    "PATH takes precedence over the toolchain the extension downloads, so the older one is in use."
  );
}
