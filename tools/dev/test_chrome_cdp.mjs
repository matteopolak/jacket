import assert from 'node:assert/strict';
import test from 'node:test';
import { runtimeException } from './chrome_cdp.mjs';

test('every browser runtime exception remains visible including aborted view transitions',()=>{
  const messages=['ordinary failure','ViewTransition promise rejected','Transition was aborted'];
  const events=messages.map(description=>({method:'Runtime.exceptionThrown',params:{exceptionDetails:{exception:{description},text:'fallback'}}}));
  assert.deepEqual(events.map(runtimeException),messages);
  assert.equal(runtimeException({method:'Network.requestWillBeSent',params:{}}),undefined);
  assert.equal(runtimeException({method:'Runtime.exceptionThrown',params:{exceptionDetails:{text:'Syntax error'}}}),'Syntax error');
});
