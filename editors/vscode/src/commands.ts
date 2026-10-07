// "Jai: Run File", "Build File" and "Check File": `jaic <command> <file>` as a task, so the output
// shows in a terminal and the task can be rerun.
import * as path from "node:path";
import * as vscode from "vscode";

export type JaicCommand = "run" | "build" | "check";

/** The `jaic` arguments for a command on `file`. */
export function jaicArguments(command: JaicCommand, file: string, extra: readonly string[]): string[] {
  return [command, file, ...extra];
}

export async function runJaic(
  command: JaicCommand,
  compiler: string | undefined,
  env: Record<string, string>,
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  const document = editor?.document;
  if (!document || document.languageId !== "jai") {
    void vscode.window.showWarningMessage("Jai: open a .jai file first.");
    return;
  }
  if (document.isUntitled) {
    void vscode.window.showWarningMessage("Jai: save the file first; jaic works on files on disk.");
    return;
  }
  if (!compiler) {
    void vscode.window
      .showErrorMessage("Jai: jaic was not found. Set jai.compiler.path, put jaic on PATH, or run \"Jai: Download Toolchain\".", "Open Settings")
      .then((choice) => {
        if (choice) void vscode.commands.executeCommand("workbench.action.openSettings", "jai.compiler.path");
      });
    return;
  }
  if (document.isDirty) await document.save();
  const file = document.uri.fsPath;
  const extra = vscode.workspace.getConfiguration("jai", document.uri).get<string[]>("run.arguments", []);
  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  const definition: vscode.TaskDefinition = { type: "jai", command, file };
  const task = new vscode.Task(
    definition,
    folder ?? vscode.TaskScope.Workspace,
    `${command} ${path.basename(file)}`,
    "jai",
    new vscode.ProcessExecution(compiler, jaicArguments(command, file, extra), {
      cwd: path.dirname(file),
      env,
    }),
  );
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    clear: true,
    focus: command === "run",
  };
  await vscode.tasks.executeTask(task);
}
