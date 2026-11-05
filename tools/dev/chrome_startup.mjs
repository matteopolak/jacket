// Shared readiness observation for the documentation checks; each caller owns its child/profile.
import { spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

/** Locate Chrome for the docs checks, honoring CHROME_PATH on every platform.
 * Example: const child = spawn(findChrome(), ownedProfileArguments);
 */
export function findChrome({env=process.env,platform=process.platform,locate=name=>spawnSync('which',[name],{stdio:'ignore'}).status===0}={}) {
  if(env.CHROME_PATH)return env.CHROME_PATH;
  if(platform==='darwin')return '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  const found=['google-chrome','google-chrome-stable','chromium','chromium-browser'].find(locate);
  if(!found)throw new Error('Install Chrome or set CHROME_PATH');
  return found;
}

/** Wait for Chrome's complete port line, preserving the 10s startup deadline and diagnostics.
 *
 *   const port = await waitForDevToolsPort(child, profile);
 *
 * readRecord is injectable so tests can expose partial writes without timing-dependent sleeps.
 * Callers must stop/reap the child and remove its profile in their existing finally block.
 */
export async function waitForDevToolsPort(child, profile, {
  timeoutMs = 10_000,
  readRecord = signal => readFile(path.join(profile, 'DevToolsActivePort'), { encoding: 'utf8', signal }),
} = {}) {
  const started = performance.now();
  let launchError, stderr = '', lastRecord;
  const onError = error => { launchError = error; };
  const onStderr = data => { stderr = (stderr + data).slice(-8000); };
  child.on('error', onError);
  child.stderr.on('data', onStderr);
  const fail = reason => new Error(`${reason} after ${Math.round(performance.now() - started)}ms: `
    + `profile=${profile}, exit=${child.exitCode}, signal=${child.signalCode}; `
    + `last record: ${lastRecord === undefined ? '(not read)' : JSON.stringify(lastRecord)}\n${stderr}`);
  const controller = new AbortController();
  let timeoutError, timer;
  const deadline = new Promise((_, reject) => {
    timer = setTimeout(() => {
      timeoutError = fail('Chrome did not publish a valid DevTools port');
      reject(timeoutError);
      controller.abort();
    }, timeoutMs);
  });
  const checkChild = () => {
    if (launchError) throw fail(`Chrome launch failed: ${launchError}`);
    if (child.exitCode !== null || child.signalCode !== null) throw fail('Chrome exited before readiness');
  };
  try {
    while (true) {
      checkChild();
      try {
        lastRecord = (await Promise.race([readRecord(controller.signal), deadline])).slice(0, 1024);
      } catch (error) {
        if (timeoutError) throw timeoutError;
        if (error.code !== 'ENOENT') throw fail(`Cannot read DevToolsActivePort: ${error.code}: ${error.message}`);
      }
      // readFile yields: a child that died during that read cannot publish a usable port.
      checkChild();
      const remaining = timeoutMs - (performance.now() - started);
      if (remaining <= 0) throw fail('Chrome did not publish a valid DevTools port');
      // File creation is not readiness. The numeric line itself may still be being written.
      const line = /^(\d+)\r?\n/.exec(lastRecord ?? '');
      const port = line && Number(line[1]);
      if (port >= 1 && port <= 65535) return port;
      await delay(Math.min(50, remaining));
    }
  } finally {
    clearTimeout(timer);
    controller.abort();
    child.off('error', onError);
    child.stderr.off('data', onStderr);
  }
}
