/** Semantic checks shared by imported standalone examples and their embedded document form. */
import type {Example} from '../generated/doc-site.js';
import {fill} from './example-program.ts';
import {browserResolver} from './urls.ts';

/** A Run button may execute only the program described by its visible source and wrapper. */
export function validateExample(example:Example):void {
  if(!/^[a-f0-9]{12}$/u.test(example.id))throw new Error('invalid example ID');
  if(!Number.isSafeInteger(example.timeoutMs)||example.timeoutMs<=0)throw new Error('example timeout must be a positive safe integer');
  if(example.runnable&&(!example.program||!example.runTemplate||!example.capabilities.includes('browser_runtime')||example.reason!=null))throw new Error('runnable example lacks program/template/runtime capability or has a disabled reason');
  if(example.runTemplate&&(!example.runnable||fill(example.runTemplate,example.source.source)!==example.program))throw new Error('example template does not match its program');
  if(example.playgroundUrl)browserResolver('/')({kind:'external',url:example.playgroundUrl});
}
