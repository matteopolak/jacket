/** Astro serves the same admitted publication files in development as a production build. */
import type {AstroIntegration} from 'astro';
import {mkdir,mkdtemp,readFile,rm} from 'node:fs/promises';
import {resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';

/** Use an owned generated publicDir; e.g. JACKET_DOC_EXPORT=/tmp/export npm run dev. */
export function documentationPublication():AstroIntegration{
 let directory:string|undefined,detachSignal=():void=>{};
 const cleanup=async():Promise<void>=>{const owned=directory;directory=undefined;if(owned)await rm(owned,{recursive:true,force:true});};
 return{name:'jacket-doc-publication',hooks:{
  'astro:config:setup':async({command,config,updateConfig})=>{
   if(command!=='dev')return;
   const {bundle}=await import('./loader.ts'),{writePublication}=await import('./publication.ts');
   const source=await bundle(),configured=process.env['JACKET_DOC_BASE'];
   if(configured!==undefined&&configured!==source.site.pageBase)throw new Error('JACKET_DOC_BASE differs from the admitted export base');
   const cache=resolve(fileURLToPath(config.root),'.astro');await mkdir(cache,{recursive:true});
   directory=await mkdtemp(resolve(cache,'jacket-doc-public-'));
   try{
    await writePublication(source,directory);
    const publicDirectory=directory,assets=new Map(source.manifest.assets.map(asset=>[source.site.pageBase+asset.path,asset]));
    updateConfig({trailingSlash:'ignore',publicDir:pathToFileURL(directory+'/'),vite:{plugins:[{
     name:'jacket-doc-publication',
     configureServer(server){
      // Vite's extension MIME table can differ from the admitted publication contract.
      server.middlewares.use((request,response,next)=>{
       if(request.method!=='GET'&&request.method!=='HEAD'){next();return;}
       let path:string;try{path=decodeURIComponent(new URL(request.url??'/','http://docs.invalid').pathname);}catch{next();return;}
       const asset=assets.get(path);if(!asset){next();return;}
       void readFile(resolve(publicDirectory,asset.path)).then(bytes=>{
        if(bytes.length!==asset.bytes)throw new Error(`${asset.path}: generated development asset changed`);
        response.statusCode=200;response.setHeader('Content-Type',asset.mediaType);response.setHeader('Content-Length',bytes.length);response.setHeader('Cache-Control','no-cache');
        response.end(request.method==='HEAD'?undefined:bytes);
       }).catch(next);
      });
     },
     async closeServer(){detachSignal();await cleanup();},
    }]}});
   }
   catch(error){await cleanup();throw error;}
  },
  'astro:server:setup':({server})=>{
   // The foreground CLI leaves SIGINT to Node; close Vite so its awaited cleanup runs.
   const interrupt=():void=>{process.exitCode=130;void server.close().catch(error=>{console.error(error);process.exitCode=1;});};
   process.once('SIGINT',interrupt);detachSignal=():void=>{process.removeListener('SIGINT',interrupt);};
  },
  'astro:server:done':async()=>{detachSignal();await cleanup();},
 }};
}
