// Runs in the extension host of the VS Code window being photographed: sets up each scene with
// editor commands, then captures the window with `screencapture -l<window id>`.
import { execFileSync } from "node:child_process";
import { copyFileSync, statSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";

const out = process.env.JAI_SHOTS_OUT!;
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const workspace = () => vscode.workspace.workspaceFolders![0].uri.fsPath;

/**
 * This VS Code instance's processes: the extension host and its ancestors up to the app's main
 * process, and no further, so no other application's window can be captured.
 */
function ancestors(): string[] {
  const app = path.dirname(process.execPath).split("/Contents/")[0];
  const pids = [String(process.pid)];
  for (let pid = process.ppid; pid > 1; ) {
    const command = execFileSync("ps", ["-o", "comm=", "-p", String(pid)]).toString().trim();
    if (!command.startsWith(app)) break;
    pids.push(String(pid));
    pid = Number(execFileSync("ps", ["-o", "ppid=", "-p", String(pid)]).toString().trim());
  }
  if (pids.length < 2) throw new Error(`no VS Code process found above ${process.pid} (app ${app})`);
  return pids;
}

// Height of the macOS title bar at the 1200px capture width.
const TITLE_BAR = 28;

let windowId: string | undefined;
async function capture(name: string): Promise<void> {
  await vscode.commands.executeCommand("notifications.clearAll");
  await sleep(300);
  windowId ??= execFileSync("swift", [process.env.JAI_SHOTS_WINDOW_ID!, ...ancestors()]).toString().trim();
  const file = path.join(out, `${name}.png`);
  execFileSync("screencapture", ["-o", "-x", `-l${windowId}`, file]);
  // Retina captures are twice the window size; the Marketplace page is about 900px wide.
  execFileSync("sips", ["--resampleWidth", "1200", file, "--out", file], { stdio: "ignore" });
  // Drop the title bar: the development host names itself and the temp folder there.
  execFileSync("swift", [process.env.JAI_SHOTS_CROP_TOP!, file, String(TITLE_BAR)]);
  console.log(`captured ${file} (${Math.round(statSync(file).size / 1024)} KiB)`);
}

async function open(name: string, column = vscode.ViewColumn.One): Promise<vscode.TextEditor> {
  const document = await vscode.workspace.openTextDocument(path.join(workspace(), name));
  return vscode.window.showTextDocument(document, { viewColumn: column, preview: false });
}

async function waitForDiagnostics(uri: vscode.Uri, predicate: (d: vscode.Diagnostic[]) => boolean): Promise<void> {
  for (let i = 0; i < 300 && !predicate(vscode.languages.getDiagnostics(uri)); i++) await sleep(100);
}

function place(editor: vscode.TextEditor, line: number, character: number): void {
  const at = new vscode.Position(line, character);
  editor.selection = new vscode.Selection(at, at);
  editor.revealRange(new vscode.Range(at, at), vscode.TextEditorRevealType.InCenterIfOutsideViewport);
}

async function reset(): Promise<void> {
  await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  await vscode.commands.executeCommand("workbench.action.closeSidebar");
  await vscode.commands.executeCommand("workbench.action.closePanel");
  await vscode.commands.executeCommand("workbench.action.closeAuxiliaryBar");
  await sleep(300);
}

async function shots(): Promise<void> {
  await vscode.extensions.getExtension("matteopolak.jai-toolchain")!.activate();
  await reset();

  // Hover: a struct's type and memory layout.
  let editor = await open("creatures.jai");
  await waitForDiagnostics(editor.document.uri, (d) => d.some((x) => x.source === "jailint"));
  await sleep(1500);
  place(editor, 3, 2);
  await vscode.commands.executeCommand("editor.action.showHover");
  await sleep(2000);
  await capture("hover");

  // Completion on a struct value.
  await vscode.commands.executeCommand("editor.action.hideHover");
  await editor.edit((edit) => edit.insert(new vscode.Position(17, 0), "    knight.\n"));
  place(editor, 17, 11);
  await sleep(1500);
  await vscode.commands.executeCommand("editor.action.triggerSuggest");
  await sleep(2500);
  await capture("completion");
  await vscode.commands.executeCommand("hideSuggestWidget");
  await vscode.commands.executeCommand("undo");
  await sleep(1500);

  // A jailint finding and its quick fix.
  await waitForDiagnostics(editor.document.uri, (d) => d.some((x) => x.source === "jailint"));
  place(editor, 17, 9);
  await sleep(500);
  // The finding's hover: the message, the rule (linked to its docs) and the Quick Fix link.
  await vscode.commands.executeCommand("editor.action.showHover");
  await sleep(2000);
  await capture("lint");
  await vscode.commands.executeCommand("editor.action.hideHover");
  await vscode.commands.executeCommand("editor.action.quickFix");
  await sleep(2000);
  await capture("quick-fix");
  await vscode.commands.executeCommand("workbench.action.closeQuickOpen");
  await vscode.commands.executeCommand("hideCodeActionWidget");
  await reset();

  // A compiler error, shown inline with the problem peek.
  editor = await open("errors.jai");
  await waitForDiagnostics(editor.document.uri, (d) => d.length > 0);
  await sleep(1000);
  place(editor, 0, 0);
  await vscode.commands.executeCommand("editor.action.marker.next");
  await sleep(1500);
  await capture("error");
  await reset();

  // Instruction completion inside `#asm AVX2 {`, with the selected instruction's forms.
  editor = await open("simd.jai");
  await waitForDiagnostics(editor.document.uri, () => true);
  await sleep(1500);
  await editor.edit((edit) => edit.replace(new vscode.Range(9, 8, 9, 28), "vpad"));
  place(editor, 9, 12);
  await sleep(1500);
  await vscode.commands.executeCommand("editor.action.triggerSuggest");
  await sleep(2500);
  // vpaddb comes first; select vpaddd, the instruction the block uses.
  await vscode.commands.executeCommand("selectNextSuggestion");
  await vscode.commands.executeCommand("toggleSuggestionDetails");
  await sleep(1000);
  await capture("asm-completion");
  await vscode.commands.executeCommand("hideSuggestWidget");
  await vscode.commands.executeCommand("undo");
  await reset();

  // Formatting: the file before, and formatted beside it.
  copyFileSync(path.join(workspace(), "messy.jai"), path.join(workspace(), "formatted.jai"));
  await open("messy.jai");
  editor = await open("formatted.jai", vscode.ViewColumn.Two);
  await vscode.commands.executeCommand("editor.action.formatDocument");
  await sleep(1500);
  place(editor, 0, 0);
  await capture("formatting");
}

export function run(): Promise<void> {
  return shots();
}
