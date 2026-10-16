import { test } from 'node:test';
import assert from 'node:assert/strict';
import { observeImports } from './wasi_file_boundary_observer.mjs';

test('real import results, arguments and exceptions are forwarded once', () => {
  const memory = new WebAssembly.Memory({ initial: 1 });
  const events = [];
  let calls = 0;
  const original = { wasi_snapshot_preview1: {
    fd_write: (...args) => { calls++; assert.deepEqual(args, [1, 20, 1, 40]); return 8; },
    proc_exit: code => { throw new Error(`exit ${code}`); },
  } };
  const observed = observeImports(original, () => memory, events).wasi_snapshot_preview1;
  assert.equal(observed.fd_write(1, 20, 1, 40), 8);
  assert.equal(calls, 1);
  assert.equal(events[0].result, 8);
  assert.throws(() => observed.proc_exit(7), /exit 7/);
  assert.equal(events.length, 2);
});

test('argument observation reads the actual WASI32 pointer table without editing memory', () => {
  const memory = new WebAssembly.Memory({ initial: 1 });
  const bytes = new Uint8Array(memory.buffer);
  const view = new DataView(memory.buffer);
  const events = [];
  const original = { wasi_snapshot_preview1: {
    args_sizes_get: (count, size) => { view.setUint32(count, 3, true); view.setUint32(size, 80, true); return 0; },
    args_get: table => {
      ['module.wasm', 'caller-data.bin', 'boundary-token'].forEach((arg, i) => {
        const address = 128 + i * 32;
        bytes.set(new TextEncoder().encode(arg + '\0'), address);
        view.setUint32(table + i * 4, address, true);
      });
      return 0;
    },
  } };
  const observed = observeImports(original, () => memory, events).wasi_snapshot_preview1;
  assert.equal(observed.args_sizes_get(16, 20), 0);
  assert.equal(observed.args_get(32, 128), 0);
  assert.deepEqual(events[1].argv, ['module.wasm', 'caller-data.bin', 'boundary-token']);
  assert.equal(view.getUint32(32, true), 128);
  assert.equal(bytes[128], 'm'.charCodeAt(0));
});

test('file observation preserves BigInt rights and the real opened descriptor', () => {
  const memory = new WebAssembly.Memory({ initial: 1 });
  const bytes = new Uint8Array(memory.buffer);
  const view = new DataView(memory.buffer);
  bytes.set(new TextEncoder().encode('caller-data.bin'), 128);
  const events = [];
  const original = { wasi_snapshot_preview1: {
    path_open: (...args) => {
      assert.equal(args[5], 0xffffn);
      view.setUint32(args[8], 9, true);
      return 0;
    },
  } };
  const observed = observeImports(original, () => memory, events).wasi_snapshot_preview1;
  assert.equal(observed.path_open(3, 1, 128, 15, 9, 0xffffn, 0n, 0, 32), 0);
  assert.equal(events[0].path, 'caller-data.bin');
  assert.equal(events[0].opened_fd, 9);
  assert.equal(events[0].args[5], '65535');
});
