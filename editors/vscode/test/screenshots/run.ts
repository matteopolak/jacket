// Captures the README screenshots from a real VS Code window (macOS only):
//
//   JAILSP=... JAIFMT=... VSCODE_EXECUTABLE=".../Visual Studio Code.app/Contents/MacOS/Code" \
//     node esbuild.mjs && node esbuild.mjs --tests && node out/test/screenshots/run.js
//
// VS Code runs with a throwaway profile (--user-data-dir, --extensions-dir in a temp folder), so
// the user's own settings and extensions are untouched. Shots land in images/.
import { runTests } from "@vscode/test-electron";
import { cpSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";

async function main(): Promise<void> {
  const root = path.resolve(__dirname, "..", "..", "..");
  const { JAILSP, JAIFMT } = process.env;
  if (!JAILSP || !JAIFMT) throw new Error("set JAILSP and JAIFMT");
  const workspace = mkdtempSync(path.join(tmpdir(), "jai-shots-"));
  cpSync(path.join(root, "test", "screenshots", "workspace"), workspace, { recursive: true });
  mkdirSync(path.join(workspace, ".vscode"));
  writeFileSync(
    path.join(workspace, ".vscode", "settings.json"),
    JSON.stringify(
      {
        "jai.server.path": JAILSP,
        "jai.formatter.path": JAIFMT,
        "jai.toolchain.autoDownload": "never",
        "workbench.colorTheme": "Default Dark Modern",
        "workbench.startupEditor": "none",
        "workbench.activityBar.location": "hidden",
        "workbench.secondarySideBar.defaultVisibility": "hidden",
        "workbench.layoutControl.enabled": false,
        "workbench.editor.enablePreview": false,
        "window.commandCenter": false,
        "chat.disableAIFeatures": true,
        "chat.commandCenter.enabled": false,
        "editor.minimap.enabled": false,
        "editor.fontSize": 14,
        "editor.inlayHints.enabled": "on",
        "editor.lightbulb.enabled": "on",
        "breadcrumbs.enabled": false,
        "files.exclude": { ".vscode": true },
        "security.workspace.trust.enabled": false,
        "update.mode": "none",
        "telemetry.telemetryLevel": "off",
      },
      null,
      2,
    ),
  );
  const profile = mkdtempSync(path.join(tmpdir(), "jai-vsc-"));
  await runTests({
    extensionDevelopmentPath: root,
    extensionTestsPath: path.join(__dirname, "suite", "index.js"),
    vscodeExecutablePath: process.env.VSCODE_EXECUTABLE || undefined,
    launchArgs: [
      workspace,
      "--user-data-dir",
      path.join(profile, "user"),
      "--extensions-dir",
      path.join(profile, "extensions"),
      "--skip-welcome",
      "--skip-release-notes",
    ],
    extensionTestsEnv: {
      JAI_SHOTS_OUT: path.join(root, "images"),
      JAI_SHOTS_WINDOW_ID: path.join(root, "test", "screenshots", "window-id.swift"),
      JAI_SHOTS_CROP_TOP: path.join(root, "test", "screenshots", "crop-top.swift"),
    },
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
