import { AsyncLocalStorage } from "node:async_hooks";

/**
 * The part of vscode-languageclient's didOpen feature that sends the didOpens it held back.
 * It holds back documents that are open but not visible when the client starts, and every
 * outgoing request first sends those (`sendPendingOpenNotifications`).
 */
export interface PendingOpens {
  sendPendingOpenNotifications(closingDocument?: string): Promise<boolean>;
}

/** Whether `feature` sends held-back didOpens (vscode-languageclient 10 does; its type omits it). */
export function hasPendingOpens(feature: object): feature is PendingOpens {
  return typeof (feature as Partial<PendingOpens>).sendPendingOpenNotifications === "function";
}

/**
 * Makes each send of held-back didOpens wait for the one in progress. vscode-languageclient
 * empties its list before sending, so without this a second request finds it empty and goes
 * out while the first is still sending didOpens: jailsp then gets a request for a document it
 * has not opened, and the rest of the didOpens can follow `shutdown`. Each didOpen sent asks
 * for the held-back ones again; those calls, made from inside a send, go straight through.
 * The function returned resolves once the sends started so far are done and the requests that
 * waited on them have gone out (they continue on the microtask queue, so a turn of the event
 * loop is enough), for a caller about to stop the client: otherwise they would follow
 * `shutdown`.
 */
export function orderPendingOpens(feature: PendingOpens): () => Promise<void> {
  const send = feature.sendPendingOpenNotifications.bind(feature);
  const sending = new AsyncLocalStorage<true>();
  let last: Promise<unknown> = Promise.resolve();
  feature.sendPendingOpenNotifications = (closingDocument?: string) => {
    if (sending.getStore()) return send(closingDocument);
    const run = () => sending.run(true, () => send(closingDocument));
    const next = last.then(run, run);
    last = next.catch(() => undefined);
    return next;
  };
  return async () => {
    let seen: Promise<unknown>;
    do {
      seen = last;
      await seen;
      await new Promise((resolve) => setImmediate(resolve));
    } while (seen !== last);
  };
}
