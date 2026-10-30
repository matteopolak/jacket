import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { COSTS, type Choice, LimitPopups, type LimitKind, type LimitNotice, type LimitUi, RAISE, SETTINGS, noticeText, raised } from "../../src/limitPopups";

const notice = (kind: LimitKind = "budget", line = 1): LimitNotice => ({
  kind,
  uri: "file:///p/main.jai",
  range: { start: { line, character: 0 }, end: { line, character: 12 } },
  message: "error during compile-time execution: execution budget exhausted (66000000 steps)",
});

/** A user who presses `answers` in turn, and the settings and commands that follow. */
class FakeUi implements LimitUi {
  readonly asked: { text: string; choices: Choice[] }[] = [];
  readonly written: [LimitKind, number][] = [];
  readonly costs: LimitNotice[] = [];
  readonly opened: LimitKind[] = [];
  private answers: (Choice | undefined)[];
  limits: Record<LimitKind, number> = { budget: 1000, time: 10 };

  constructor(...answers: (Choice | undefined)[]) {
    this.answers = answers;
  }

  async ask(text: string, choices: Choice[]): Promise<Choice | undefined> {
    this.asked.push({ text, choices });
    await new Promise((resolve) => setTimeout(resolve, 5));
    return this.answers.shift();
  }

  async setLimit(kind: LimitKind, value: number): Promise<void> {
    this.written.push([kind, value]);
    this.limits[kind] = value;
  }

  currentLimit(kind: LimitKind): number {
    return this.limits[kind];
  }

  async showCosts(n: LimitNotice): Promise<void> {
    this.costs.push(n);
  }

  async openSettings(kind: LimitKind): Promise<void> {
    this.opened.push(kind);
  }

  place(n: LimitNotice): string {
    return `main.jai:${n.range.start.line + 1}`;
  }
}

function popups(...answers: (Choice | undefined)[]): { ui: FakeUi; popups: LimitPopups } {
  const ui = new FakeUi(...answers);
  return { ui, popups: new LimitPopups(ui) };
}

describe("the compile-time limit popup", () => {
  it("names the place of the evaluation and offers the three buttons", async () => {
    const { ui, popups: p } = popups(undefined);
    await p.notice(notice());
    assert.equal(ui.asked.length, 1);
    assert.match(ui.asked[0].text, /main\.jai:2/);
    assert.deepEqual(ui.asked[0].choices, [RAISE, COSTS, SETTINGS]);
  });

  it("doubles the setting for Raise limit", async () => {
    const { ui, popups: p } = popups(RAISE);
    await p.notice(notice());
    assert.deepEqual(ui.written, [["budget", 2000]]);
  });

  it("raises the time limit when the check ran out of time", async () => {
    const { ui, popups: p } = popups(RAISE);
    await p.notice(notice("time"));
    assert.deepEqual(ui.written, [["time", 20]]);
    assert.match(noticeText(notice("time"), "main.jai:2", 10), /checking stopped after 10 s/);
  });

  it("offers no raise when the limit is already off", async () => {
    const { ui, popups: p } = popups(undefined);
    ui.limits.budget = 0;
    await p.notice(notice());
    assert.deepEqual(ui.asked[0].choices, [COSTS, SETTINGS]);
    assert.equal(raised(0), undefined);
  });

  it("opens the cost list and the settings", async () => {
    const { ui, popups: p } = popups(COSTS, SETTINGS);
    await p.notice(notice());
    await p.notice(notice("time"));
    assert.equal(ui.costs.length, 1);
    assert.equal(ui.costs[0].range.start.line, 1);
    assert.deepEqual(ui.opened, ["time"]);
  });

  it("does not ask again for the same evaluation until the user edits", async () => {
    const { ui, popups: p } = popups(undefined, undefined);
    await p.notice(notice());
    await p.notice(notice());
    assert.equal(ui.asked.length, 1);
    // Another evaluation is another popup.
    await p.notice(notice("budget", 7));
    assert.equal(ui.asked.length, 2);
    p.edited("file:///p/main.jai");
    await p.notice(notice());
    assert.equal(ui.asked.length, 3);
  });

  it("does not stack a second popup while the first is open", async () => {
    const { ui, popups: p } = popups(undefined);
    await Promise.all([p.notice(notice()), p.notice(notice())]);
    assert.equal(ui.asked.length, 1);
  });

  it("is not reset by an edit of another file, except for the time limit", async () => {
    const { ui, popups: p } = popups(undefined, undefined);
    await p.notice(notice());
    await p.notice(notice("time"));
    p.edited("file:///p/other.jai");
    await p.notice(notice());
    await p.notice(notice("time"));
    assert.equal(ui.asked.length, 3);
  });
});
