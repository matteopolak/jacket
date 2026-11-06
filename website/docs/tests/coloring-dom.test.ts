import assert from 'node:assert/strict';
import test from 'node:test';
import {recolor} from '../src/browser/coloring.ts';
class Node{
 children:Node[]=[];className='';value='';tag:string;
 constructor(tag='#text'){this.tag=tag;}
 set textContent(value:string){this.value=value;this.children=[];}
 get textContent():string{return this.value+this.children.map(node=>node.textContent).join('');}
 append(node:Node):void{this.children.push(node);}
 replaceChildren(fragment:Node):void{this.children=fragment.children;this.value='';}
}
const document={createDocumentFragment:()=>new Node('#fragment'),createElement:(tag:string)=>new Node(tag),createTextNode:(text:string)=>{const node=new Node();node.textContent=text;return node;}};
test('edited source is colored through text nodes and never inserted as author HTML',()=>{
 const code=Object.assign(new Node('code'),{ownerDocument:document});
 const source='/* outer /* nested */ literal */\nname := "</span><script>owned()</script>雪";';
 recolor(code as unknown as HTMLElement,source);
 assert.equal(code.textContent,source);assert(code.children.some(node=>node.className==='c'));assert(code.children.some(node=>node.className==='s'));
 assert(code.children.every(node=>node.tag==='#text'||node.tag==='span'));assert(!code.children.some(node=>node.tag==='script'||node.tag==='a'));
 recolor(code as unknown as HTMLElement,'return 1');assert.equal(code.textContent,'return 1');assert.equal(code.children[0]?.className,'k');
});
test('plain source runs do not allocate one DOM node per punctuation or character',()=>{
 const code=Object.assign(new Node('code'),{ownerDocument:document}),source='name   < > / = : ; '.repeat(1000);
 recolor(code as unknown as HTMLElement,source);assert.equal(code.textContent,source);assert.equal(code.children.length,1,'adjacent plain lexical spans should share one text node');
});
