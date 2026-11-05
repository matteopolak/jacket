/** Generate declarations from the exact schema derived from Rust DTOs.
 * Example: npm run generate:types; npm run check:generated
 */
import { readFile, writeFile } from 'node:fs/promises';
import { compile } from 'json-schema-to-typescript';
import { Ajv2020 } from 'ajv/dist/2020.js';
import { _ } from 'ajv';
import standalone from 'ajv/dist/standalone/index.js';
import { numericFormats } from '../src/lib/validation-formats.js';
const root = new URL('../', import.meta.url);
const schema = JSON.parse(await readFile(new URL('schema/doc-site.schema.json', root), 'utf8'));
const generated = await compile(schema, 'Payload', {
  bannerComment: '/* Generated from Rust doc/export/contract.rs. Run npm run generate:types. */',
  additionalProperties: false,
  unreachableDefinitions: true,
});
const destination = new URL('src/generated/doc-site.d.ts', root);
if (process.argv.includes('--check')) {
  if (await readFile(destination, 'utf8') !== generated) throw new Error('generated documentation types are stale; run npm run generate:types');
} else await writeFile(destination, generated);

const ajv = new Ajv2020({ allErrors: true, strict: true, code: { source: true, esm: true, formats: _`numericFormats` } });
for (const [name, format] of Object.entries(numericFormats)) ajv.addFormat(name, format);
schema.$id = 'urn:jacket:doc-site';
ajv.addSchema(schema);
for (const [index, branch] of schema.oneOf.entries()) {
  const kind = branch.properties.payload.const;
  const validate = ajv.compile({ $ref: `urn:jacket:doc-site#/oneOf/${index}` });
  const code = '/* Generated from Rust doc/export/contract.rs. Run npm run generate:types. */\n'
    + "import { numericFormats } from '../lib/validation-formats.js';\n" + standalone(ajv, validate) + '\n';
  const declarations = `/* Generated from Rust documentation schema. */\nimport type { Payload } from './doc-site.js';\ndeclare const validate: import('ajv').ValidateFunction<Extract<Payload, { payload: '${kind}' }>>;\nexport default validate;\nexport { validate };\n`;
  for (const [suffix, contents] of [['js',code],['d.ts',declarations]]) {
    const path = new URL(`src/generated/validate-${kind}.${suffix}`,root);
    if (process.argv.includes('--check')) {
      if (await readFile(path,'utf8') !== contents) throw new Error(`generated ${kind} validator is stale`);
    } else await writeFile(path,contents);
  }
}
