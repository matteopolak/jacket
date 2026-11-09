// Browser verification of reviewed contracts and independent regressions.
// Explicit canonical target unavailability is reported
// separately, never as passing browser execution. No legacy failure baseline is used.
import { Worker, isMainThread, parentPort, workerData } from 'node:worker_threads';
import { playgroundCaseResult } from './playground_case_result.ts';
import { buildPlayRequest } from './playground_request.ts';
import { readFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import type { CaseResult, CatalogueCase, EngineModule } from './play_types.ts';

/** What one case run reports to the summary. */
interface Outcome {
  pass?: boolean;
  why?: string;
  unavailable?: string;
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const python = process.env.PYTHON || (process.platform === 'win32' ? 'python' : 'python3');
const catalogue: CatalogueCase[] = isMainThread
  ? JSON.parse(
      execFileSync(python, [path.join(root, 'tools/browser/playground_policy.py')], {
        encoding: 'utf8',
      }),
    )
  : [];

if (!isMainThread) {
  const port = parentPort;
  if (!port) throw new Error('worker has no parent port');
  const { directory, test }: { directory: string; test: CatalogueCase } = workerData;
  const { createEngine } = (await import(
    pathToFileURL(path.join(directory, 'engine.mjs')).href
  )) as EngineModule;
  const { files, main, request } = await buildPlayRequest(root, test);
  const engine = await createEngine(await readFile(path.join(directory, 'jai_wasm.wasm')));
  try {
    port.postMessage({ ...engine.play(files, main), request } satisfies CaseResult);
  } catch (error) {
    const failure = error as Error & { compilerCrashed?: boolean };
    port.postMessage({
      requestFailure: String(failure.message),
      compilerCrashed:
        failure.compilerCrashed === true || error instanceof WebAssembly.RuntimeError,
      request,
    } satisfies CaseResult);
  }
} else {
  const [staged, ...only] = process.argv.slice(2);
  if (!staged || only.some((a) => a.startsWith('--')))
    throw new Error(
      'usage: node tools/browser/check_playground_stdlib.ts <staged-dir> [case-id...]',
    );
  const directory = path.resolve(staged);
  const tests = catalogue.filter((c) => !only.length || only.includes(c.id));
  for (const test of tests.slice()) {
    if (test.positive_baseline) {
      const baseline = catalogue.find((c) => c.id === test.positive_baseline_id);
      if (!baseline) throw new Error('missing positive baseline for ' + test.id);
      if (!tests.includes(baseline)) tests.push(baseline);
    }
  }
  const runOne = (test: CatalogueCase) =>
    new Promise<Outcome>((resolve) => {
      if (test.browser_unavailable_reason) {
        resolve({ unavailable: test.browser_unavailable_reason });
        return;
      }
      const worker = new Worker(fileURLToPath(import.meta.url), {
        workerData: { directory, test },
        resourceLimits: { maxOldGenerationSizeMb: 4096 },
      });
      let finished = false;
      const started = Date.now();
      const done = (value: Outcome) => {
        if (finished) return;
        finished = true;
        clearTimeout(timer);
        if (process.env.PLAYGROUND_PROGRESS)
          console.log(
            `${Date.now() - started} ms ${test.id}${value.pass === false ? ' (' + value.why + ')' : ''}`,
          );
        resolve(value);
      };
      const timer = setTimeout(() => {
        done({ pass: false, why: 'timeout' });
        worker.terminate();
      }, 120000);
      worker.on('message', (r: CaseResult) => {
        const result = playgroundCaseResult(test, r);
        if (process.env.PLAYGROUND_VERBOSE) console.log(test.id + '\n' + result.output);
        done({ pass: result.pass, why: result.why });
        worker.terminate();
      });
      worker.on('error', (e) => done({ pass: false, why: 'crash: ' + e.message }));
      worker.on('exit', () => done({ pass: false, why: 'worker exited without result' }));
    });
  const results = new Map<string, Outcome>();
  let next = 0;
  await Promise.all(
    Array.from({ length: 4 }, async () => {
      while (next < tests.length) {
        const c = tests[next++];
        results.set(c.id, await runOne(c));
      }
    }),
  );
  for (const test of tests) {
    if (test.positive_baseline && !results.get(test.positive_baseline_id ?? '')?.pass) {
      results.set(test.id, { pass: false, why: 'positive public baseline did not pass' });
    }
  }
  const failed = tests.filter((c) => !results.get(c.id)?.pass && !results.get(c.id)?.unavailable);
  const unavailable = tests.filter((c) => results.get(c.id)?.unavailable);
  const passed = tests.filter((c) => results.get(c.id)?.pass);
  for (const c of failed) console.log('FAIL ' + c.id + ': ' + results.get(c.id)?.why);
  if (process.env.PLAYGROUND_VERBOSE)
    for (const c of unavailable)
      console.log('UNAVAILABLE ' + c.id + ': ' + results.get(c.id)?.unavailable);
  const missing = only.filter((id) => !catalogue.some((c) => c.id === id));
  if (missing.length) console.log('no such case: ' + missing.join(', '));
  console.log(
    passed.length +
      ' passed (including expected precise rejections), ' +
      failed.length +
      ' failed, ' +
      unavailable.length +
      ' target/mode unavailable; ' +
      tests.length +
      ' selected',
  );
  if (failed.length || missing.length || !tests.length) process.exit(1);
}
