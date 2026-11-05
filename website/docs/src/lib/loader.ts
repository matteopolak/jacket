/** The single JSON trust boundary. Production components receive only validated generated types. */
import schema from '../../schema/doc-site.schema.json' with { type: 'json' };
import { createHash } from 'node:crypto';
import { lstat, readFile, realpath } from 'node:fs/promises';
import { resolve, sep } from 'node:path';
import { numericFormats } from './validation-formats.js';
import { Ajv2020 } from 'ajv/dist/2020.js';
import {bindAssetIndex,routeIndex} from './urls.ts';
import { graphValidator } from './graph.ts';
import type { Payload, Manifest, SiteData, PageRoute } from '../generated/doc-site.js';

const validator = new Ajv2020({ allErrors: false, strict: true });
for (const [name, format] of Object.entries(numericFormats)) validator.addFormat(name, format);
const validate = validator.compile<Payload>(schema);

/** Limits bound import work; the full-library acceptance report records their measured margin. */
export const limits = Object.freeze({ manifestBytes: 8 * 1024 * 1024, shardBytes: 256 * 1024 * 1024, totalBytes: 2 * 1024 * 1024 * 1024, files: 100_000 });

/** Reject native path syntax and non-canonical segments before joining an export destination. */
export function safeRelativePath(path: string): string {
  if (!path || /[\\:\0<>"|?*]/u.test(path) || path.split('/').some(part => ['', '.', '..'].includes(part) || /[ .]$/u.test(part) || /^(?:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\.|$)/iu.test(part))) throw new Error(`unsafe relative path ${JSON.stringify(path)}`);
  return path;
}

function digest(bytes: Uint8Array): string { return createHash('sha256').update(bytes).digest('hex'); }

async function readSafe(root: string, path: string, maximum: number): Promise<Buffer> {
  safeRelativePath(path);
  let current = root;
  for (const part of path.split('/')) {
    current = resolve(current, part);
    const info = await lstat(current);
    if (info.isSymbolicLink()) throw new Error(`${path}: symlink import is forbidden`);
  }
  const info = await lstat(current);
  if (!info.isFile() || info.size > maximum) throw new Error(`${path}: expected regular file of at most ${maximum} bytes`);
  const actual = await realpath(current);
  if (!actual.startsWith(`${root}${sep}`)) throw new Error(`${path}: import escapes the export root`);
  return readFile(current);
}

function decode(path: string, bytes: Uint8Array): Payload {
  let raw: unknown;
  try { raw = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes)); }
  catch (error) { throw new Error(`${path}: invalid UTF-8 JSON`, { cause: error }); }
  if (!validate(raw)) {
    const errors = validate.errors?.slice(0, 12).map(error => `${error.instancePath || '/'}: ${error.message}`).join('; ');
    throw new Error(`${path}: ${errors}`);
  }
  return raw;
}

function validDigest(hash: string): void {
  if (!/^[a-f0-9]{64}$/u.test(hash)) throw new Error(`invalid SHA-256 digest ${JSON.stringify(hash)}`);
}

/** A complete immutable import keeps only the shared route map and four current shards. */
export class Bundle {
  readonly root: string;
  readonly manifest: Manifest;
  readonly identity: string;
  readonly site: SiteData;
  readonly routes: ReadonlyMap<string, PageRoute>;
  private readonly inventory: ReadonlyMap<string, { sha256: string; bytes: number }>;
  private readonly cache = new Map<string, Payload>();

  constructor(root: string, manifest: Manifest, site: SiteData, inventory: ReadonlyMap<string, { sha256: string; bytes: number }>) {
    this.root = root;
    this.manifest = manifest;
    this.identity = digest(new TextEncoder().encode(JSON.stringify(manifest)));
    this.site = site;
    this.routes = routeIndex(site);
    this.inventory = inventory;
    bindAssetIndex(site,manifest.assets);
  }

  /** Return an admitted file identity without hashing or reading it again. */
  fileDigest(path:string):string {
    const file=this.inventory.get(path);if(!file)throw new Error(`missing inventoried file ${path}`);return file.sha256;
  }

  /** Shards were schema-checked during admission; rereads verify immutable bytes before reuse. */
  async shard(path: string): Promise<Payload> {
    const cached = this.cache.get(path);
    if (cached) { this.cache.delete(path); this.cache.set(path, cached); return cached; }
    const expected = this.inventory.get(path);
    if (!expected) throw new Error(`${path}: shard absent from manifest`);
    const bytes = await readSafe(this.root, path, limits.shardBytes);
    if (bytes.length !== expected.bytes || digest(bytes) !== expected.sha256) throw new Error(`${path}: shard changed after validation`);
    const payload = decode(path, bytes);
    this.cache.set(path, payload);
    if (this.cache.size > 4) {
      const oldest = this.cache.keys().next().value;
      if (oldest !== undefined) this.cache.delete(oldest);
    }
    return payload;
  }
}

/** Capabilities describe an executable bundle, including its two required runtime assets. */
function validateRuntimeAssets(manifest:Manifest):void {
  const runtime=manifest.capabilities.includes('browser_runtime');
  if(runtime!==manifest.capabilities.includes('editable_examples')||new Set(manifest.capabilities).size!==manifest.capabilities.length)throw new Error('manifest capabilities: browser runtime and editable examples must occur together without duplicates');
  const assets=manifest.assets.filter(asset=>asset.provenance==='browser_runtime'||asset.path.startsWith('runtime/'));
  if(runtime){
    const required=[['runtime/engine.mjs','text/javascript'],['runtime/jai_wasm.wasm','application/wasm']];
    if(assets.length!==2||!required.every(([path,media])=>assets.some(asset=>asset.path===path&&asset.mediaType===media&&asset.provenance==='browser_runtime')))throw new Error('manifest runtime capability requires exactly its engine and Wasm assets with matching provenance/media types');
  }else if(assets.length)throw new Error('manifest runtime assets require browser runtime capability');
}

/** Validate every inventoried payload and asset before returning any data to a renderer. */
export async function loadBundle(directory: string): Promise<Bundle> {
  const root = await realpath(directory);
  const envelope = decode('manifest.json', await readSafe(root, 'manifest.json', limits.manifestBytes));
  if (envelope.payload !== 'manifest') throw new Error('manifest.json: expected manifest payload');
  const manifest = envelope.data;
  if (manifest.schemaVersion !== 1) throw new Error('manifest.json /schemaVersion: unsupported jacket-doc-site version; expected 1');
  validateRuntimeAssets(manifest);
  const formatter=manifest.producer.formatter;
  if(formatter){
    validDigest(formatter.sha256);
    if(formatter.profile!=='documentation-declarations/1'||!formatter.version.trim()||formatter.maxWidth===0||formatter.indentWidth===0)throw new Error('manifest.json /producer/formatter: unsupported profile or invalid identity/layout options');
  }
  validDigest(manifest.source.configurationDigest);
  const inventory = new Map<string, { sha256: string; bytes: number }>();
  const folded = new Set<string>();
  let total = 0;
  for (const entry of [...manifest.shards, ...manifest.assets]) {
    safeRelativePath(entry.path); validDigest(entry.sha256);
    if (folded.has(entry.path.toLowerCase())) throw new Error(`${entry.path}: duplicate case-folded inventory path`);
    folded.add(entry.path.toLowerCase()); total += entry.bytes;
    if (folded.size > limits.files || total > limits.totalBytes) throw new Error('export exceeds documented import limits');
    inventory.set(entry.path, entry);
  }
  for (const input of manifest.source.inputs) { safeRelativePath(input.path); validDigest(input.sha256); }
  const siteEntry = inventory.get('site.json');
  if (!siteEntry) throw new Error('export has no site.json payload');
  const siteBytes = await readSafe(root, 'site.json', limits.shardBytes);
  if (siteBytes.length !== siteEntry.bytes || digest(siteBytes) !== siteEntry.sha256) throw new Error('site.json: inventory size or SHA-256 mismatch');
  const siteEnvelope = decode('site.json', siteBytes);
  if (siteEnvelope.payload !== 'site') throw new Error('site.json: expected site payload');
  const site = siteEnvelope.data;
  const routeIds = new Set<string>();
  const outputs = new Set<string>();
  for (const route of site.routes) {
    safeRelativePath(route.output);
    if (route.route !== route.id || route.route.startsWith('/') || route.route.includes('..')) throw new Error(`${route.id}: invalid canonical route`);
    if (!inventory.has(route.shard)) throw new Error(`${route.id}: unlisted shard ${route.shard}`);
    if (routeIds.has(route.id) || outputs.has(route.output.toLowerCase())) throw new Error(`${route.id}: duplicate route ID or case-folded output`);
    routeIds.add(route.id); outputs.add(route.output.toLowerCase());
    if (new Set(route.anchors).size !== route.anchors.length) throw new Error(`${route.id}: duplicate anchor`);
  }
  const validateGraph = graphValidator(site, new Set(manifest.assets.map(asset=>asset.path)),new Set(manifest.capabilities));
  validateGraph(siteEnvelope, 'site.json');
  for (const entry of manifest.shards) {
    if (entry.path === 'site.json') continue;
    const bytes = await readSafe(root, entry.path, limits.shardBytes);
    if (bytes.length !== entry.bytes || digest(bytes) !== entry.sha256) throw new Error(`${entry.path}: inventory size or SHA-256 mismatch`);
    const payload = decode(entry.path, bytes);
    if (payload.payload === 'site' || payload.payload === 'manifest') throw new Error(`${entry.path}: unexpected envelope payload`);
    validateGraph(payload, entry.path);
  }
  for (const entry of manifest.assets) {
    const bytes = await readSafe(root, entry.path, limits.shardBytes);
    if (bytes.length !== entry.bytes || digest(bytes) !== entry.sha256) throw new Error(`${entry.path}: asset size or SHA-256 mismatch`);
  }
  return new Bundle(root, manifest, site, inventory);
}

let loaded: Promise<Bundle> | undefined;
/** Site builds share one admitted export; schema validation cannot be overridden by page code. */
export function bundle(): Promise<Bundle> {
  const path = process.env['JACKET_DOC_EXPORT'];
  if (!path) throw new Error('JACKET_DOC_EXPORT must name a complete `jacket doc --export-only` bundle');
  return loaded ??= loadBundle(resolve(path));
}
