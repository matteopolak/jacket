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

export async function buildPlayRequest(root, test) {
  const parts = test.path.split("/");
  const moduleWorkspace = test.family === "modules";
  const base = path.join(root, ...parts.slice(0, moduleWorkspace ? 1 : 2));
  const fileBase = moduleWorkspace ? root : base;
  const files = {};
  await collectPlayFiles(base, fileBase, files, { moduleWorkspace });
  const main = path.relative(fileBase, path.join(root, test.path)).replaceAll("\\", "/");
  const libraryRoot = moduleWorkspace ? path.relative(root, base).replaceAll("\\", "/") : undefined;
  return { files, main, libraryRoot, request: playRequestSize(files, main, { libraryRoot }) };
}

export function playRequestSize(files, main, { libraryRoot } = {}) {
  let pathBytes = 0, sourceBytes = 0;
  for (const [name, text] of Object.entries(files)) {
    pathBytes += Buffer.byteLength(name, "utf8");
    sourceBytes += Buffer.byteLength(text, "utf8");
  }
  const mainBytes = Buffer.byteLength(main, "utf8");
  const libraryRootBytes = libraryRoot === undefined ? 0 : Buffer.byteLength(libraryRoot, "utf8");
  return { ...(libraryRoot === undefined ? {} : { libraryRootBytes }), fileCount: Object.keys(files).length, pathBytes, sourceBytes, mainBytes,
    totalBytes: pathBytes + sourceBytes + mainBytes + libraryRootBytes };
}
