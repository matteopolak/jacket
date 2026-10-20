// Launches VS Code with the extension and runs test/e2e/suite in its extension host.
//
//   JAILSP=/path/to/jailsp [JAIFMT=/path/to/jaifmt] [VSCODE_EXECUTABLE=...] npm run test:e2e
//
// Without JAIFMT a stand-in formatter (fake-jaifmt.cjs) checks the extension's side of
// formatting: the process, its working directory and applying the edit. VS Code is downloaded
// into .vscode-test/ unless VSCODE_EXECUTABLE points at an installed one.
import { runTests } from "@vscode/test-electron";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";

// Passes stdout and stderr through and remembers whether `text` was ever written to either.
function watchOutputFor(text: string): { seen(): boolean } {
  let found = false;
  for (const stream of [process.stdout, process.stderr]) {
    const write = stream.write.bind(stream) as (chunk: unknown, ...rest: unknown[]) => boolean;
    stream.write = ((chunk: unknown, ...rest: unknown[]) => {
      if (String(chunk).includes(text)) found = true;
      return write(chunk, ...rest);
    }) as typeof stream.write;
  }
  return { seen: () => found };
}

async function main(): Promise<void> {
  const root = path.resolve(__dirname, "..", "..", "..");
  const jailsp = process.env.JAILSP;
  if (!jailsp) throw new Error("set JAILSP to a jailsp executable (cargo build -p jai-language-server)");

  const workspace = mkdtempSync(path.join(tmpdir(), "jai-e2e-"));
  cpSync(path.join(root, "test", "e2e", "fixture"), workspace, { recursive: true });
  let jaifmt = process.env.JAIFMT;
  if (!jaifmt) {
    const fake = path.join(__dirname, "fake-jaifmt.cjs");
    if (process.platform === "win32") {
      jaifmt = path.join(workspace, "fake-jaifmt.cmd");
      writeFileSync(jaifmt, `@node "${fake}" %*\r\n`);
    } else {
      jaifmt = path.join(workspace, "fake-jaifmt");
      writeFileSync(jaifmt, `#!/bin/sh\nexec node "${fake}" "$@"\n`);
      chmodSync(jaifmt, 0o755);
    }
  }
  mkdirSync(path.join(workspace, ".vscode"), { recursive: true });
  writeFileSync(
    path.join(workspace, ".vscode", "settings.json"),
    JSON.stringify(
      {
        "jai.server.path": path.resolve(jailsp),
        "jai.formatter.path": jaifmt,
        "jai.toolchain.autoDownload": "never",
        "jai.trace.server": "messages",
      },
      null,
      2,
    ),
  );

  const options = {
    extensionDevelopmentPath: root,
    extensionTestsPath: path.join(__dirname, "suite", "index.cjs"),
    vscodeExecutablePath: process.env.VSCODE_EXECUTABLE || undefined,
    // A short user data folder: its IPC socket path must fit in 103 bytes on macOS. No GPU: under
    // xvfb the window's software GL once stalled startup long enough to be killed as unresponsive.
    launchArgs: [workspace, "--user-data-dir", mkdtempSync(path.join(tmpdir(), "jai-vsc-")), "--disable-extensions", "--skip-welcome", "--skip-release-notes", "--disable-workspace-trust", "--disable-gpu"],
  };

  // A window that stalls during startup is killed by VS Code before any test runs, which says
  // nothing about the extension: launch once more for that, and only that.
  const unresponsive = watchOutputFor("detected unresponsive");
  try {
    await runTests(options);
  } catch (error) {
    if (!unresponsive.seen()) throw error;
    console.error("VS Code reported an unresponsive window during startup; launching it once more");
    await runTests(options);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
