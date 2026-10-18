// Formatter test inputs are text. Only a named, structurally complete AppleDouble
// v2 sidecar is transport metadata (the same boundary used by objc_selectors.rs).
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

export function isAppleDoubleMetadata(name, bytes) {
  if (!name.startsWith("._") || bytes.length < 26) return false;
  if (!bytes.subarray(0, 8).equals(Buffer.from([0, 5, 0x16, 7, 0, 2, 0, 0]))) return false;
  const count = bytes.readUInt16BE(24);
  const tableEnd = 26 + count * 12;
  if (tableEnd > bytes.length) return false;
  for (let i = 26; i < tableEnd; i += 12) {
    const offset = bytes.readUInt32BE(i + 4);
    const length = bytes.readUInt32BE(i + 8);
    if (length > 0 && (offset < tableEnd || offset + length > bytes.length)) return false;
  }
  return true;
}

export function goldenCaseNames(directory) {
  return readdirSync(directory)
    .filter(name => name.endsWith(".in.jai") &&
      !isAppleDoubleMetadata(name, readFileSync(path.join(directory, name))))
    .map(name => name.slice(0, -".in.jai".length)).sort();
}

export function readUtf8(file) {
  // Preserve a leading BOM, and reject invalid bytes rather than replacing them.
  return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(readFileSync(file));
}

export function optionalConfig(file) {
  try {
    return readUtf8(file);
  } catch (error) {
    if (error.code === "ENOENT") return undefined;
    throw error;
  }
}
