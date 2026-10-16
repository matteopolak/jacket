// Opt-in observation of the real Node WASI wrapper, without replacing providers.
// node --import ./tools/wasi_file_boundary_observer.mjs tools/wasi_run.mjs ...
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, realpathSync } from 'node:fs';
import { resolve } from 'node:path';
import { WASI } from 'node:wasi';

export function artifact(path) {
  const bytes = readFileSync(path);
  return { path, size: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex'),
    magic: bytes.subarray(0, 8).toString('hex'), all_zero: !bytes.some(byte => byte !== 0) };
}

export function observeImports(imports, getMemory, events) {
  let argc = 0;
  const view = () => new DataView(getMemory().buffer);
  const text = (offset, count) => new TextDecoder().decode(
    new Uint8Array(getMemory().buffer, Number(offset), Number(count)));
  const cString = offset => {
    const bytes = new Uint8Array(getMemory().buffer);
    const end = bytes.indexOf(0, Number(offset));
    if (end < 0) throw new Error('unterminated observed argument');
    return text(offset, end - Number(offset));
  };
  return Object.fromEntries(Object.entries(imports).map(([namespace, functions]) =>
    [namespace, Object.fromEntries(Object.entries(functions).map(([name, fn]) => {
      if (typeof fn !== 'function') return [name, fn];
      return [name, (...args) => {
        const event = { name, args: args.map(value => typeof value === 'bigint' ? value.toString() : value) };
        events.push(event);
        if (name === 'path_open') event.path = text(args[2], args[3]);
        const result = fn(...args);
        event.result = result;
        if (name === 'args_sizes_get' && result === 0) argc = view().getUint32(args[0], true);
        if (name === 'args_get' && result === 0) {
          event.argv = Array.from({ length: argc }, (_, i) => cString(view().getUint32(args[0] + i * 4, true)));
        }
        if (name === 'path_open' && result === 0) event.opened_fd = view().getUint32(args[8], true);
        return result;
      }];
    }))]));
}

const receipt = process.env.FOUNDATIONS_WASI_TRACE;
if (receipt) {
  const options = process.argv.slice(2);
  const modulePath = options[0] === '--unknown-imports-trap' ? options[1] : options[0];
  if (!modulePath) throw new Error('observer requires the actual wrapper module argument');
  const actualModule = realpathSync(modulePath);
  const protectedPaths = [actualModule, ...options.filter(path => path !== '--unknown-imports-trap').slice(1).map(path => resolve(path))];
  if (protectedPaths.includes(resolve(receipt))) throw new Error('receipt aliases module or caller data');
  const events = [];
  const report = { command: process.argv, before: artifact(actualModule), events };
  let memory;
  const originalImports = WASI.prototype.getImportObject;
  const originalStart = WASI.prototype.start;
  WASI.prototype.getImportObject = function () {
    return observeImports(originalImports.call(this), () => memory, events);
  };
  WASI.prototype.start = function (instance) {
    memory = instance.exports.memory;
    report.before_start = artifact(actualModule);
    try { return originalStart.call(this, instance); }
    finally { report.after_start = artifact(actualModule); }
  };
  process.on('exit', code => {
    report.exit_code = code;
    report.after_process = artifact(actualModule);
    writeFileSync(receipt, JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
  });
}
