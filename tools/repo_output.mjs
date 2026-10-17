// Where the repo's tools write build products and scratch output: `<repo>/target/`, or
// `JACKET_OUTPUT_DIR` when set. Mirrors tools/repo_output.py.
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export function outputDir(...parts) {
  const base = process.env.JACKET_OUTPUT_DIR
    ? path.resolve(process.env.JACKET_OUTPUT_DIR)
    : path.join(root, "target");
  return path.join(base, ...parts);
}
