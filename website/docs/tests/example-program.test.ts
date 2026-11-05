import assert from 'node:assert/strict';
import { test } from 'node:test';
import { fill, base64url, playResult } from '../src/lib/example-program.ts';
test('edited Unicode example keeps wrapper imports and blank-line policy',()=>{
  assert.equal(fill({head:'H\n',middle:'M\n',tail:'T\n',wrapped:true,pad:false},'#import "Basic";\nprint("é");\n\n'),'H\n#import "Basic";\nM\n    print("é");\n\nT\n');
  assert.equal(new TextDecoder().decode(Uint8Array.from(atob(base64url('é').replaceAll('-','+').replaceAll('_','/')),c=>c.charCodeAt(0))),'é');
  assert.throws(()=>playResult({exitCode:'0'}),/Invalid browser compiler response/);
});
