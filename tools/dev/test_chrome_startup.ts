import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { PassThrough } from 'node:stream';
import test from 'node:test';
import { waitForDevToolsPort } from './chrome_startup.ts';
import type { DevToolsChild } from './chrome_startup.ts';

function child(): DevToolsChild & { stderr: PassThrough } {
  return Object.assign(new EventEmitter(), {
    exitCode: null,
    signalCode: null,
    stderr: new PassThrough(),
  });
}

test('waits through empty and partial port records before using the complete port', async () => {
  const browser = child();
  const records = ['', '12', '12345\n/devtools/browser/test'];
  let reads = 0;
  const port = await waitForDevToolsPort(browser, '/test-profile', {
    readRecord: async () => records[reads++],
  });
  assert.equal(port, 12345);
  assert.equal(reads, 3);
  assert.equal(browser.listenerCount('error'), 0);
  assert.equal(browser.stderr.listenerCount('data'), 0);
});

test('missing file is pending but another read failure is reported immediately', async () => {
  const browser = child();
  let reads = 0;
  assert.equal(
    await waitForDevToolsPort(browser, '/test-profile', {
      readRecord: async () => {
        if (reads++ === 0) throw Object.assign(new Error('not written'), { code: 'ENOENT' });
        return '54321\n/devtools/browser/test';
      },
    }),
    54321,
  );
  await assert.rejects(
    waitForDevToolsPort(child(), '/denied-profile', {
      readRecord: async () => {
        throw Object.assign(new Error('permission denied'), { code: 'EACCES' });
      },
    }),
    /EACCES.*permission denied/,
  );
});

test('deadline reports the last invalid record, elapsed time, process and stderr', async () => {
  const browser = child();
  await assert.rejects(
    waitForDevToolsPort(browser, '/test-profile', {
      timeoutMs: 20,
      readRecord: async () => {
        browser.stderr.write('DevTools policy diagnostic\n');
        return '65536\n/devtools/browser/test';
      },
    }),
    (error: Error) => {
      assert.match(error.message, /after \d+ms/);
      assert.match(error.message, /last record: "65536\\n\/devtools\/browser\/test"/);
      assert.match(error.message, /exit=null, signal=null/);
      assert.match(error.message, /DevTools policy diagnostic/);
      return true;
    },
  );
});

test('spawn failure includes its cause and captured stderr', async () => {
  const browser = child();
  await assert.rejects(
    waitForDevToolsPort(browser, '/test-profile', {
      readRecord: async () => {
        browser.stderr.write('launcher diagnostic');
        browser.emit('error', Object.assign(new Error('missing executable'), { code: 'ENOENT' }));
        throw Object.assign(new Error('not written'), { code: 'ENOENT' });
      },
    }),
    /missing executable[\s\S]*launcher diagnostic/,
  );
});

test('exit during the read cannot publish a port from a dead browser', async () => {
  const browser = child();
  await assert.rejects(
    waitForDevToolsPort(browser, '/test-profile', {
      readRecord: async () => {
        browser.stderr.write('browser crash');
        browser.exitCode = 7;
        return '12345\n/devtools/browser/test';
      },
    }),
    /exit=7, signal=null[\s\S]*browser crash/,
  );
});

test('signal termination is diagnosed and helper listeners are released', async () => {
  const browser = child();
  await assert.rejects(
    waitForDevToolsPort(browser, '/test-profile', {
      readRecord: async () => {
        browser.signalCode = 'SIGTERM';
        return '';
      },
    }),
    /signal=SIGTERM/,
  );
  assert.equal(browser.listenerCount('error'), 0);
  assert.equal(browser.stderr.listenerCount('data'), 0);
});

test('a stalled record read cannot extend the startup deadline', { timeout: 1000 }, async () => {
  await assert.rejects(
    waitForDevToolsPort(child(), '/test-profile', {
      timeoutMs: 20,
      readRecord: () => new Promise(() => {}),
    }),
    /after \d+ms[\s\S]*last record: \(not read\)/,
  );
});
