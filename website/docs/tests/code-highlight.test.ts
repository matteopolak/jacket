import assert from 'node:assert/strict';
import test from 'node:test';
import {codeLanguage,codeTheme,codePresentation} from '../src/lib/code-highlight.ts';
test('only existing non-Jai fence aliases select the bundled highlighter',()=>{
 for(const alias of ['sh','bash','shell','zsh','SH'])assert.equal(codeLanguage(alias),'shellscript');
 for(const alias of ['powershell','ps1','pwsh'])assert.equal(codeLanguage(alias),'powershell');
 assert.equal(codeLanguage('console'),'shellscript');assert.equal(codeLanguage('toml'),'toml');
 for(const unknown of ['jai','diagnostic','<script>','unknown-fixture-language',''])assert.equal(codeLanguage(unknown),undefined);
});
test('non-Jai theme uses the live palette without introducing language-specific layout',()=>{
 assert.equal(codeTheme.colors['editor.foreground'],'var(--fg)');
 for(const token of codeTheme.tokenColors){assert.match(token.settings.foreground,/^var\(--(?:muted|kw|str|type)\)$/u);assert.deepEqual(Object.keys(token.settings).filter(key=>key!=='foreground'&&key!=='fontStyle'),[]);}
});

test('highlighted fence wrappers retain the existing CSS and focus behavior',()=>{
 const node={properties:{style:'background-color:transparent',tabindex:0,class:'astro-code',dataLanguage:'toml'}};
 codePresentation[0]!.pre(node);
 assert.deepEqual(node.properties,{class:'code wrap',dataLanguage:'toml'});
});
