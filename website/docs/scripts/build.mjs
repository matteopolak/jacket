/** Build a complete static tree in staging, validate it, then replace only Jacket-owned output.
 * Example: npm run build -- --export /tmp/library-export --out /tmp/library-site
 */
import { spawn } from 'node:child_process';
import { lstat, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadBundle } from '../src/lib/loader.ts';
import { writePublication } from '../src/lib/publication.ts';
import { checkSite } from './check-site.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const astroPackage = JSON.parse(await readFile(resolve(root,'node_modules/astro/package.json'),'utf8'));
const astroEntry = astroPackage.bin?.astro;
if(typeof astroEntry !== 'string') throw new Error('installed Astro package has no CLI entry');
const args = process.argv.slice(2);
let source = process.env['JACKET_DOC_EXPORT'];
let destination = resolve(root, 'dist');
let check = false;
for (let i = 0; i < args.length; i++) {
  const flag = args[i];
  if (flag === '--export') source = args[++i];
  else if (flag === '--out') { const path = args[++i]; if (!path) throw new Error('--out needs a path'); destination = resolve(path); }
  else if (flag === '--check') check = true;
  else throw new Error(`unknown build option ${flag}`);
}
if (!source) throw new Error('pass --export DIR (or JACKET_DOC_EXPORT) naming a complete semantic export');
const bundle = await loadBundle(resolve(source));
try {
  const info = await lstat(destination);
  if (info.isSymbolicLink() || !info.isDirectory()) throw new Error('output is not a regular directory');
  if ((await readdir(destination)).length > 0) await lstat(resolve(destination, '.jacket-doc'));
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
  // An absent marker in a nonempty existing directory is rejected separately below.
  try { if ((await readdir(destination)).length > 0) throw new Error(`${destination} exists and Jacket did not create it; choose another --out`); }
  catch (nested) { if (nested.code !== 'ENOENT') throw nested; }
}
await mkdir(dirname(destination), {recursive:true});
const staging = await mkdtemp(resolve(dirname(destination), '.jacket-doc-build-'));
try {
  await new Promise((accept,reject) => {
    const child = spawn(process.execPath, [resolve(root,'node_modules/astro',astroEntry),'build'], {
      cwd:root, stdio:'inherit', env:{...process.env,JACKET_DOC_EXPORT:bundle.root,JACKET_DOC_BASE:bundle.site.pageBase,JACKET_DOC_OUT:staging},
    });
    child.on('error',reject);
    child.on('exit',code => code === 0 ? accept() : reject(new Error(`Astro exited with status ${code}`)));
  });
  await writePublication(bundle,staging);
  await writeFile(resolve(staging,'.jacket-doc'),'');
  await checkSite(staging,bundle.site.pageBase,{name:bundle.site.name,homeTitle:bundle.site.socialSiteName,socialName:bundle.site.socialSiteName});
  if (!check) {
    const previous = `${staging}.previous`;
    let replaced = false;
    try { await rename(destination,previous); replaced=true; } catch(error) {if(error.code !== 'ENOENT')throw error;}
    try { await rename(staging,destination); }
    catch(error) {if(replaced)await rename(previous,destination);throw error;}
    if(replaced)await rm(previous,{recursive:true});
  }
} finally { await rm(staging,{recursive:true,force:true}); }
