import { createHash } from 'node:crypto';

/** One shared navigation fact shard per module, e.g. navigation/<stable-module-digest>.json. */
export function moduleNavigationPath(id: string): string {
  return `navigation/${createHash('sha256').update(id).digest('hex').slice(0, 16)}.json`;
}


import type {ApiModule} from '../generated/doc-site.js';
const navigation=new WeakMap<ApiModule,{readonly json:string;readonly sha256:string}>();
/** The writer and URL use the exact same compact bytes, memoized per admitted module. */
export function moduleNavigation(module:ApiModule):{readonly json:string;readonly sha256:string} {
  let value=navigation.get(module);if(value)return value;
  const entries=module.declarations.filter(d=>!d.private).map(({id,route,name,kind,binding,experimental,private:visibility,group})=>({id,route,name,kind,binding,experimental,private:visibility,group}));
  const json=JSON.stringify({payload:'navigation',data:{schemaVersion:1,moduleId:module.id,entries}});
  value={json,sha256:createHash('sha256').update(json).digest('hex')};navigation.set(module,value);return value;
}
