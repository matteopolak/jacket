import assert from 'node:assert/strict';
import {test} from 'node:test';
import validate from '../src/generated/validate-module.js';
import {apiModule} from './fixtures.ts';

test('module import paths are required independently of selected page IDs',()=>{
  const module=apiModule();
  module.id='Widget';
  module.importPath='Extensions/Widget';
  assert.equal(validate({payload:'module',data:module}),true,JSON.stringify(validate.errors));
  assert.equal(module.id,'Widget');
  for(const invalid of [undefined,17,null]){
    const data:Record<string,unknown>={...module};
    if(invalid===undefined)delete data['importPath'];else data['importPath']=invalid;
    assert.equal(validate({payload:'module',data}),false,'invalid import identity admitted');
    assert.ok(validate.errors?.some(error=>error.instancePath.endsWith('/importPath')||error.params['missingProperty']==='importPath'));
  }
});

test('external nested import paths do not collapse to compiler link basenames',()=>{
  const module=apiModule();
  module.id='Acme/Widget';
  module.importPath='Acme/Widget';
  assert.equal(validate({payload:'module',data:module}),true,JSON.stringify(validate.errors));
});
