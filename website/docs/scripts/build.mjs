/** Build a complete static tree in staging, validate it, then replace only Jacket-owned output.
 * Example: npm run build -- --export /tmp/library-export --out /tmp/library-site
 */
import { spawn } from 'node:child_process';
import { cp, lstat, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadBundle } from '../src/lib/loader.ts';
import { moduleNavigationPath, moduleNavigation } from '../src/lib/navigation-path.ts';
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
  for (const asset of bundle.manifest.assets) {
    const output = resolve(staging,asset.path);
    await mkdir(dirname(output),{recursive:true});
    await cp(resolve(bundle.root,asset.path),output,{errorOnExist:true,force:false});
  }
  const exampleFiles=new Map();
  async function examples(value) {
    if(!value || typeof value!=='object')return;
    if(value.kind==='fence' && value.example.runnable) {
      const example=value.example;
      if(!/^[a-f0-9]{12}$/u.test(example.id))throw new Error('invalid example asset ID');
      const bytes=JSON.stringify({payload:'example',data:example});
      const previous=exampleFiles.get(example.id);
      if(previous && previous!==bytes)throw new Error(`example ID collision ${example.id}`);
      if(!previous){exampleFiles.set(example.id,bytes);const path=resolve(staging,`examples/${example.id}.json`);await mkdir(dirname(path),{recursive:true});await writeFile(path,bytes);}
    }
    for(const child of Object.values(value))if(child && typeof child==='object') {
      if(Array.isArray(child)){for(const item of child)await examples(item);}else await examples(child);
    }
  }
  for (const shard of bundle.manifest.shards) {
    const payload = await bundle.shard(shard.path);
    if (payload.payload === 'module') {
      await examples(payload.data);
      const path = resolve(staging,moduleNavigationPath(payload.data.id));
      await mkdir(dirname(path),{recursive:true});
      await writeFile(path,moduleNavigation(payload.data).json);
    } else if (payload.payload === 'search') await writeFile(resolve(staging,'search-index.json'),JSON.stringify(payload));
  }
  await writeFile(resolve(staging,'versions.json'),JSON.stringify(bundle.site.version ? [{version:bundle.site.version,url:'./'}] : []));
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
