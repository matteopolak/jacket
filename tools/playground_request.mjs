// Build one browser request from a source workspace. No compiler or import parsing.
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";

const textFile = /\.(jai|txt|json|md)$/;

export async function collectPlayFiles(dir, base, files, { moduleWorkspace = false } = {}) {
  const entries = await readdir(dir, { withFileTypes: true });
  entries.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);
  for (const entry of entries) {
    // AppleDouble files are filesystem sidecars, not checked-in inputs.
    if (entry.name.startsWith("._")) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) await collectPlayFiles(full, base, files, { moduleWorkspace });
    else if (entry.isFile() && textFile.test(entry.name)) {
      const relative = path.relative(base, full).replaceAll("\\", "/");
      // Module workspaces retain every source (including relative #load inputs)
      // and every test fixture. Library audit/documentation payloads outside test
      // directories are not run inputs and must not be resent as user files.
      if (moduleWorkspace && !entry.name.endsWith(".jai") && !relative.split("/").includes("tests")) continue;
      files[relative] = await readFile(full, "utf8");
    }
  }
}

export function playRequestSize(files, main) {
  let pathBytes = 0, sourceBytes = 0;
  for (const [name, text] of Object.entries(files)) {
    pathBytes += Buffer.byteLength(name, "utf8");
    sourceBytes += Buffer.byteLength(text, "utf8");
  }
  const mainBytes = Buffer.byteLength(main, "utf8");
  return { fileCount: Object.keys(files).length, pathBytes, sourceBytes, mainBytes,
    totalBytes: pathBytes + sourceBytes + mainBytes };
}
