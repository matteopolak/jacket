/** Central destination policy keeps page, static-asset and external origins separate. */
import type { SiteData, Target, PageRoute } from '../generated/doc-site.js';

export interface HostAdapter { assetBase?: string; navigate?: 'same_window' | 'new_window' }

/** Normalize an explicitly supplied host asset base; content cannot choose this setting. */
export function assetBase(site: SiteData, host: HostAdapter = {}): string {
  const value = host.assetBase ?? site.pageBase;
  if (!(value.startsWith('/') && !value.startsWith('//')) && !/^https?:\/\//u.test(value)) throw new Error('asset base must be a root path or HTTP(S) URL');
  if (/[\\?#]/u.test(value)) throw new Error('asset base cannot contain backslashes, query or fragment');
  return `${value.replace(/\/+$/u, '')}/`;
}

/** A file digest changes only that request's cache identity; it is not historical storage. */
export function versionedURL(url:string,sha256:string):string {
  if(!/^[a-f0-9]{64}$/u.test(sha256))throw new Error('invalid file digest');
  const [path,fragment]=url.split('#');
  return `${path}${path?.includes('?')?'&':'?'}v=${sha256}${fragment===undefined?'':`#${fragment}`}`;
}
const assetIndexes=new WeakMap<SiteData,ReadonlyMap<string,string>>();
/** Bind once after bundle admission, avoiding repeated inventory scans on static pages. */
export function bindAssetIndex(site:SiteData,assets:readonly {path:string;sha256:string}[]):void {
  if(assetIndexes.has(site))throw new Error('asset index already bound');
  assetIndexes.set(site,new Map(assets.map(asset=>[asset.path,asset.sha256])));
}
/** Only publication-owned local images acquire an asset digest; external metadata stays exact. */
export function publicationImageURL(site:SiteData,image:string):string {
  for(const [path,sha]of assetIndexes.get(site)??[]) {
    const local=`${assetBase(site)}${path}`;
    if(image===local||(site.canonicalOrigin&&image===`${site.canonicalOrigin.replace(/\/$/u,'')}${local}`))return versionedURL(image,sha);
  }
  return image;
}

export function assetURL(site: SiteData, path: string, host: HostAdapter = {}): string {
  if (!path || path.split('/').some(part => ['', '.', '..'].includes(part)) || /[\\:\0]/u.test(path)) throw new Error(`invalid asset destination ${path}`);
  const url=`${assetBase(site, host)}${path}`,sha=assetIndexes.get(site)?.get(path);
  return sha?versionedURL(url,sha):url;
}

/** Imported URLs are absolute HTTP(S) or mail destinations, never executable/native schemes. */
export function externalURL(value:string):string {
  let url:URL;
  try {url=new URL(value);} catch {throw new Error(`unsupported external URL ${value}`);}
  if(!['http:','https:','mailto:'].includes(url.protocol)||value.trim()!==value||/[\u0000-\u001f\u007f\\]/u.test(value))throw new Error(`unsupported external URL ${value}`);
  return value;
}

/** Every internal destination comes from the admitted route table, never a display-name slug. */
const routeIndexes=new WeakMap<SiteData,Map<string,PageRoute>>();
/** One compact route index is shared by validation and all static pages of an admitted site. */
export function routeIndex(site:SiteData):ReadonlyMap<string,PageRoute> {
  let routes=routeIndexes.get(site);if(!routes){routes=new Map(site.routes.map(route=>[route.id,route]));routeIndexes.set(site,routes);}return routes;
}

export function resolver(site: SiteData, host: HostAdapter = {}): (target: Target) => string {
  const routes = routeIndex(site);
  return target => {
    if (target.kind === 'asset') return assetURL(site, target.path, host);
    if (target.kind === 'external') {
      return externalURL(target.url);
    }
    const route = routes.get(target.id);
    if (!route) throw new Error(`missing destination ${target.id}`);
    if (target.anchor && !route.anchors.includes(target.anchor)) throw new Error(`missing fragment ${target.id}#${target.anchor}`);
    return `${site.pageBase}${route.route}${target.anchor ? `#${target.anchor}` : ''}`;
  };
}

/** Browser payloads carry canonical route IDs, so no declaration-name slugger is needed. */
export function browserResolver(base:string):(target:Target)=>string {
  if(!base.startsWith('/')||!base.endsWith('/')||base.startsWith('//')||/[\\?#]/u.test(base))throw new Error('invalid browser page base');
  return target=>{
    if(target.kind==='external')return externalURL(target.url);
    const path=target.kind==='asset'?target.path:target.id;
    if(/[\\:\0?#]/u.test(path)||path!==''&&path.split('/').some(part=>['','.','..'].includes(part)))throw new Error('invalid canonical browser destination');
    return `${base}${path}${target.kind==='page'&&target.anchor?`#${encodeURIComponent(target.anchor)}`:''}`;
  };
}
