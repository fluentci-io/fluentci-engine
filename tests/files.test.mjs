import assert from 'node:assert/strict';
import test from 'node:test';
import * as ts from '../sdk/typescript/src/files.ts';
import * as rs from '../sdk/rescript/src/files_ffi.mjs';
for (const [name, sdk] of [['TypeScript', ts], ['ReScript', rs]]) {
  test(`${name}: native file builders are pure and preserve literal data`, () => {
    const original = sdk.writeText('write', "path ' 世界", '$(touch injected)\0', false, '.');
    assert.equal(original.program, '');
    assert.deepEqual(original.file_operation, {op:'write_text', path:"path ' 世界", text:'$(touch injected)\0', overwrite:false});
    const bytes = [0,255];
    const binary = sdk.writeBytes('binary', 'bytes', bytes, false, '.');
    bytes[0] = 123;
    assert.deepEqual(binary.file_operation.bytes, [0,255]);
    const configured = sdk.envFile(original, 'settings.env');
    assert.equal(original.env_files, undefined);
    assert.deepEqual(configured.env_files, ['settings.env']);
    assert(Object.isFrozen(configured));
    for (const value of [sdk.readText('a','path',10,'.'),sdk.readBytes('b','path',10,'.'),sdk.writeBytes('c','path',[0,255],false,'.'),sdk.copyFile('d','a','b',false,'.'),sdk.copyTree('e','a','b',true,'.'),sdk.move('f','a','b',false,'.'),sdk.mkdir('g','dir',true,'.'),sdk.remove('h','dir',true,true,'.'),sdk.stat('i','file','.'),sdk.exists('j','file','.'),sdk.list('k','dir','.'),sdk.glob('l','**/*.txt','.'),sdk.hash('m','file','.'),sdk.chmod('n','file',0o700,'.'),sdk.symlink('o','target','link','.')]) {
      assert.equal(value.program,''); assert(Object.isFrozen(value.file_operation));
      assert.equal(JSON.parse(JSON.stringify(value)).file_operation.op, value.file_operation.op);
    }
  });
}
