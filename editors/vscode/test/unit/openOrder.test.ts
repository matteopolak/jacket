import assert from "node:assert/strict";
import { it } from "node:test";
import { type PendingOpens, hasPendingOpens, orderPendingOpens } from "../../src/openOrder";

/**
 * Behaves like vscode-languageclient's didOpen feature: takes the held-back list, then sends
 * each, and sending a notification asks for the held-back didOpens first.
 */
class Feature implements PendingOpens {
  constructor(
    private pending: string[],
    private sent: string[],
  ) {}

  async sendPendingOpenNotifications(): Promise<boolean> {
    const documents = this.pending;
    this.pending = [];
    for (const document of documents) {
      await this.sendPendingOpenNotifications();
      await new Promise((resolve) => setTimeout(resolve, 5));
      this.sent.push(`didOpen ${document}`);
    }
    return false;
  }
}

/** Two requests at once, as VS Code makes right after a server starts. */
async function twoRequests(order: boolean): Promise<string[]> {
  const sent: string[] = [];
  const feature = new Feature(["a.jai", "b.jai"], sent);
  const settled = order ? orderPendingOpens(feature) : async () => {};
  const request = async (document: string) => {
    await feature.sendPendingOpenNotifications();
    sent.push(`request ${document}`);
  };
  await Promise.all([request("a.jai"), request("b.jai")]);
  await settled();
  sent.push("shutdown");
  return sent;
}

it("sends no request before the didOpens held back for it", async () => {
  // Unordered, the second request overtakes both didOpens.
  assert.deepEqual((await twoRequests(false)).slice(0, 1), ["request b.jai"]);
  assert.deepEqual(await twoRequests(true), ["didOpen a.jai", "didOpen b.jai", "request a.jai", "request b.jai", "shutdown"]);
});

it("waits for the didOpens in progress before a stop", async () => {
  const sent: string[] = [];
  const feature = new Feature(["a.jai"], sent);
  const settled = orderPendingOpens(feature);
  void feature.sendPendingOpenNotifications();
  await settled();
  sent.push("shutdown");
  assert.deepEqual(sent, ["didOpen a.jai", "shutdown"]);
});

it("lets the requests that waited on the didOpens go out before a stop", async () => {
  const sent: string[] = [];
  const feature = new Feature(["a.jai"], sent);
  const settled = orderPendingOpens(feature);
  // Like vscode-languageclient, a request takes a few more steps after the didOpens are sent.
  const request = async (document: string) => {
    await feature.sendPendingOpenNotifications();
    for (let step = 0; step < 5; step += 1) await Promise.resolve();
    sent.push(`request ${document}`);
  };
  void request("a.jai");
  void request("b.jai");
  await settled();
  sent.push("shutdown");
  assert.deepEqual(sent, ["didOpen a.jai", "request a.jai", "request b.jai", "shutdown"]);
});

it("keeps going after a failed send", async () => {
  let calls = 0;
  const feature: PendingOpens = {
    sendPendingOpenNotifications: async () => {
      calls += 1;
      if (calls === 1) throw new Error("connection closed");
      return false;
    },
  };
  orderPendingOpens(feature);
  await assert.rejects(feature.sendPendingOpenNotifications());
  assert.equal(await feature.sendPendingOpenNotifications(), false);
});

it("recognizes a feature that holds didOpens back", () => {
  assert.ok(hasPendingOpens(new Feature([], [])));
  assert.ok(!hasPendingOpens({}));
});
