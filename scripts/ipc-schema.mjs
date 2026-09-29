import { execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { build } from 'vite';
import { compile } from 'json-schema-to-typescript';
import Ajv2020 from 'ajv/dist/2020.js';
import standaloneCode from 'ajv/dist/standalone/index.js';

const schema = JSON.parse(execFileSync('cargo', ['run', '--quiet', '--bin', 'schema-export'], { encoding: 'utf8' }));
schema.$id = 'https://tradex.local/schema/ipc-v1';
const ajv = new Ajv2020({ strict: false, validateFormats: false, code: { source: true } });
ajv.addSchema(schema);
const definitions = Object.fromEntries(Object.keys(schema.$defs).map(name => [name, `${schema.$id}#/$defs/${name}`]));
const entry = resolve('shared/ipc-validators-input.cjs');
const bundled = await build({
  configFile: false, logLevel: 'silent',
  plugins: [{ name: 'ipc-validators', resolveId: id => id === entry ? entry : undefined, load: id => id === entry ? standaloneCode(ajv, definitions) : undefined }],
  build: { write: false, minify: false, lib: { entry, formats: ['es'] }, commonjsOptions: { include: [/node_modules/, /ipc-validators-input\.cjs$/] } },
});
const validatorCode = (Array.isArray(bundled) ? bundled[0] : bundled).output.find(item => item.type === 'chunk').code;
const files = {
  'shared/ipc-v1.schema.json': JSON.stringify(schema, null, 2) + '\n',
  'shared/ipc-types.ts': await compile(schema, 'IpcSchema', {
    bannerComment: '/* Generated from Rust protocol.rs. Run npm run schema:generate. */',
    unreachableDefinitions: true,
  }),
  'shared/ipc-validators.js': '/* Generated from Rust JSON Schema. No runtime eval. */\n' + validatorCode,
  'shared/ipc-validators.d.ts': '/* Generated from Rust JSON Schema. */\ndeclare const validators: {\n' + Object.keys(definitions).map(name => `  ${name}: (value: unknown) => boolean;`).join('\n') + '\n};\nexport default validators;\n',
};
await mkdir('shared', { recursive: true });
for (const [file, contents] of Object.entries(files)) {
  if (process.argv.includes('--check')) {
    if (await readFile(file, 'utf8') !== contents) throw new Error(`IPC schema drift: ${file}`);
  } else await writeFile(file, contents);
}
console.log(process.argv.includes('--check') ? 'Rust / JSON Schema / TypeScript agree.' : 'Generated IPC schema and TypeScript.');
