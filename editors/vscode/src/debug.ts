// "Jai" debug configurations: build the file with `jaic -O0` (DWARF is on by default) and hand the
// executable to CodeLLDB (`vadimcn.vscode-lldb`), which reads jaic's debug info.
import { execFile } from "node:child_process";
import * as path from "node:path";
import * as vscode from "vscode";
import { buildArguments, programPath } from "./debugBuild";

export const LLDB_EXTENSION = "vadimcn.vscode-lldb";

/** The CodeLLDB launch configuration for a built Jai program. */
export function lldbConfiguration(name: string, program: string, args: readonly string[], cwd: string): vscode.DebugConfiguration {
  return { type: "lldb", request: "launch", name, program, args: [...args], cwd };
}

export interface DebugHost {
  /** The `jaic` to build with and its environment; undefined (after telling the user) when there is none. */
  compiler(): { path: string; env: Record<string, string> } | undefined;
}

/** Resolves `{ "type": "jai", "request": "launch", "file": "${file}" }` to a CodeLLDB configuration. */
export class JaiDebugProvider implements vscode.DebugConfigurationProvider {
  constructor(private readonly host: DebugHost) {}

  async resolveDebugConfiguration(folder: vscode.WorkspaceFolder | undefined, config: vscode.DebugConfiguration): Promise<vscode.DebugConfiguration | undefined> {
    if (!vscode.workspace.isTrusted) {
      void vscode.window.showWarningMessage("Jai: trust this workspace to debug: building runs its compile-time code.");
      return undefined;
    }
    if (!vscode.extensions.getExtension(LLDB_EXTENSION)) {
      void vscode.window
        .showErrorMessage(`Jai: debugging needs the CodeLLDB extension (${LLDB_EXTENSION}). Install it, then start again.`, "Install CodeLLDB")
        .then((choice) => {
          if (choice) void vscode.commands.executeCommand("workbench.extensions.installExtension", LLDB_EXTENSION);
        });
      return undefined;
    }
    const file: string | undefined = config.file ?? vscode.window.activeTextEditor?.document.fileName;
    if (!file || !file.endsWith(".jai")) {
      void vscode.window.showWarningMessage('Jai: name the file to debug: set "file" in the launch configuration, or open a .jai file.');
      return undefined;
    }
    const compiler = this.host.compiler();
    if (!compiler) return undefined;
    const program: string = config.program ?? programPath(file, process.platform);
    const extra = vscode.workspace.getConfiguration("jai", folder?.uri).get<string[]>("run.arguments", []);
    const built = await build(compiler.path, buildArguments(file, program, extra), compiler.env, path.dirname(file));
    if (built) {
      void vscode.window.showErrorMessage(`Jai: the build failed.\n${built}`, { modal: true });
      return undefined;
    }
    return lldbConfiguration(config.name ?? `Debug ${path.basename(file)}`, program, config.args ?? [], config.cwd ?? path.dirname(file));
  }
}

/** Runs the build; the compiler's output when it fails, otherwise undefined. */
function build(compiler: string, args: string[], env: Record<string, string>, cwd: string): Promise<string | undefined> {
  return new Promise((resolve) => {
    execFile(compiler, args, { cwd, env: { ...process.env, ...env } }, (error, stdout, stderr) => {
      resolve(error ? `${stdout}${stderr}`.trim() || error.message : undefined);
    });
  });
}
