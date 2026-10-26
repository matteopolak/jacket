// When a compile-time limit stops code in the language server it sends `jai/compileTimeLimit`
// (an evaluation that ran out of its budget, or a check that ran past its time limit). This
// module decides when to tell the user and what the buttons do; it does not import `vscode`, so
// it runs in the unit tests. `limitUi.ts` is the VS Code side.

/** The settings the limits come from. Both are machine-scoped: a workspace cannot raise them. */
export const BUDGET_SETTING = "compileTime.evaluationBudget";
export const TIME_SETTING = "compileTime.checkTimeLimit";

export const DEFAULT_BUDGET = 200_000_000;
export const DEFAULT_TIME = 10;

export type LimitKind = "budget" | "time";

export interface Position {
  line: number;
  character: number;
}

/** The params of `jai/compileTimeLimit`. */
export interface LimitNotice {
  kind: LimitKind;
  uri: string;
  range: { start: Position; end: Position };
  message: string;
}

export const RAISE = "Raise limit";
export const COSTS = "Show compile-time costs";
export const SETTINGS = "Open settings";
export type Choice = typeof RAISE | typeof COSTS | typeof SETTINGS;

/** What the user sees and what acting on a choice does. */
export interface LimitUi {
  /** Show `text` with `choices` as buttons; the one pressed, or `undefined` when dismissed. */
  ask(text: string, choices: Choice[]): Promise<Choice | undefined>;
  /** Write `value` to the user's (global) settings, which makes the server check again. */
  setLimit(kind: LimitKind, value: number): Promise<void>;
  /** The limit now in force. */
  currentLimit(kind: LimitKind): number;
  /** Open the list of costs of the program the notice is in. */
  showCosts(notice: LimitNotice): Promise<void>;
  openSettings(kind: LimitKind): Promise<void>;
  /** `main.jai:3`, for the text. */
  place(notice: LimitNotice): string;
}

/** Twice `current`; an unlimited setting (`0`) has nothing to raise. */
export function raised(current: number): number | undefined {
  if (current <= 0) return undefined;
  return Math.min(current * 2, Number.MAX_SAFE_INTEGER);
}

/** The text of the message for `notice`. */
export function noticeText(notice: LimitNotice, place: string, limit: number): string {
  return notice.kind === "budget"
    ? `Jai: the compile-time code at ${place} ran past its budget of ${limit.toLocaleString("en-US")} basic blocks and was stopped. The rest of the program was still checked.`
    : `Jai: checking stopped after ${limit} s, so the results are partial. Compile-time code at ${place} was running.`;
}

/** The place an evaluation is: the same one again is the same evaluation. */
export function noticeKey(notice: LimitNotice): string {
  const { line, character } = notice.range.start;
  return `${notice.kind}|${notice.uri}|${line}:${character}`;
}

/**
 * Shows each notice once: not while its message is still open, and not again until the user
 * edits (the server sends the notice again after every check that hits the limit).
 */
export class LimitPopups {
  private readonly open = new Set<string>();
  private readonly seen = new Map<string, LimitNotice>();

  constructor(private readonly ui: LimitUi) {}

  async notice(notice: LimitNotice): Promise<void> {
    const key = noticeKey(notice);
    if (this.open.has(key) || this.seen.has(key)) return;
    this.open.add(key);
    this.seen.set(key, notice);
    try {
      const limit = this.ui.currentLimit(notice.kind);
      const choices: Choice[] = raised(limit) === undefined ? [COSTS, SETTINGS] : [RAISE, COSTS, SETTINGS];
      const choice = await this.ui.ask(noticeText(notice, this.ui.place(notice), limit), choices);
      if (choice === RAISE) {
        const next = raised(this.ui.currentLimit(notice.kind));
        if (next !== undefined) await this.ui.setLimit(notice.kind, next);
        // The raised limit may still not be enough: the server's next notice asks again.
        this.seen.delete(key);
      } else if (choice === COSTS) {
        await this.ui.showCosts(notice);
      } else if (choice === SETTINGS) {
        await this.ui.openSettings(notice.kind);
      }
    } finally {
      this.open.delete(key);
    }
  }

  /** The user edited `uri`: what was shown for it may be shown again. A time limit concerns the whole check, so any edit counts. */
  edited(uri: string): void {
    for (const [key, notice] of this.seen) {
      if (notice.uri === uri || notice.kind === "time") this.seen.delete(key);
    }
  }
}
