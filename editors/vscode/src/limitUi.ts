// The VS Code side of `limitPopups.ts`: the message with its buttons, and the settings it edits.
import * as vscode from "vscode";
import { BUDGET_SETTING, DEFAULT_BUDGET, DEFAULT_TIME, TIME_SETTING, type Choice, type LimitKind, type LimitNotice, type LimitUi } from "./limitPopups";

const settingOf = (kind: LimitKind): string => (kind === "budget" ? BUDGET_SETTING : TIME_SETTING);

/** The limits as `initializationOptions` carries them to the server. */
export function limitSettings(): { evaluationBudget: number; checkTimeLimit: number; costLensBlocks: number; limitNotices: boolean } {
  const config = vscode.workspace.getConfiguration("jai");
  return {
    // Machine-scoped settings: VS Code does not read a workspace's value for them.
    evaluationBudget: config.get<number>(BUDGET_SETTING, DEFAULT_BUDGET),
    checkTimeLimit: config.get<number>(TIME_SETTING, DEFAULT_TIME),
    costLensBlocks: config.get<number>("compileTime.costLensBlocks", 1_000_000),
    limitNotices: true,
  };
}

export const vscodeLimitUi: LimitUi = {
  async ask(text: string, choices: Choice[]): Promise<Choice | undefined> {
    return vscode.window.showWarningMessage(text, ...choices);
  },
  async setLimit(kind: LimitKind, value: number): Promise<void> {
    await vscode.workspace.getConfiguration("jai").update(settingOf(kind), value, vscode.ConfigurationTarget.Global);
  },
  currentLimit(kind: LimitKind): number {
    const config = vscode.workspace.getConfiguration("jai");
    return kind === "budget" ? config.get<number>(BUDGET_SETTING, DEFAULT_BUDGET) : config.get<number>(TIME_SETTING, DEFAULT_TIME);
  },
  async showCosts(notice: LimitNotice): Promise<void> {
    await vscode.commands.executeCommand("jai.showCompileTimeCosts", { uri: notice.uri, position: notice.range.start });
  },
  async openSettings(kind: LimitKind): Promise<void> {
    await vscode.commands.executeCommand("workbench.action.openSettings", `jai.${settingOf(kind)}`);
  },
  place(notice: LimitNotice): string {
    return `${vscode.workspace.asRelativePath(vscode.Uri.parse(notice.uri))}:${notice.range.start.line + 1}`;
  },
};
