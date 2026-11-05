/** Lazy typed browser admission uses validators generated from the same Rust schema as builds. */
import {validateExample} from './example-validation.ts';
import type { Payload } from '../generated/doc-site.js';

export async function fetchPayload(url: string, expectedKind: 'module' | 'search' | 'example' | 'navigation', signal?: AbortSignal): Promise<Payload> {
  const response=await fetch(url, signal ? { signal } : undefined);
  if(!response.ok)throw new Error(`Could not load documentation data (${response.status})`);
  const raw:unknown=await response.json();
  if(expectedKind==='navigation') {
    const {default:validate}=await import('../generated/validate-navigation.js');
    if(validate(raw)&&raw.data.schemaVersion===1)return raw;
    throw new Error('Invalid or unsupported navigation data');
  }
  if(expectedKind==='module') {
    const {default:validate}=await import('../generated/validate-module.js');
    if(validate(raw))return raw;
    throw new Error(`Invalid module data: ${validate.errors?.[0]?.instancePath}`);
  }
  if(expectedKind==='search') {
    const {default:validate}=await import('../generated/validate-search.js');
    if(validate(raw) && raw.data.schemaVersion===1)return raw;
    throw new Error('Invalid or unsupported search data');
  }
  const {default:validate}=await import('../generated/validate-example.js');
  if(validate(raw)){validateExample(raw.data);return raw;}
  throw new Error(`Invalid example data: ${validate.errors?.[0]?.instancePath}`);
}
