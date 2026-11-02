import { mkdtemp, rm, readFile } from 'node:fs/promises';
import { readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import process from 'node:process';
export async function fixture(test) {
  const binary = process.env.FLUENTCI_ENGINE_BIN;
  if (!binary) {
    if (process.env.FLUENTCI_REQUIRE_ENGINE === '1') throw Error('FLUENTCI_ENGINE_BIN required');
    console.error('Gleam local integration skipped: set FLUENTCI_ENGINE_BIN');
    return;
  }
  if (binary.split('/').includes('debug')) throw Error('release engine required');
  const root = await mkdtemp(join(tmpdir(), 'gleam-local-'));
  try { await test(binary, root); }
  finally { await rm(root, { recursive: true, force: true }); }
}
export async function ready(root) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    try { if (/^\d+\s*$/.test(await readFile(join(root, 'pid'), 'utf8'))) return true; } catch {}
    await new Promise(resolve => setTimeout(resolve, 10));
  }
  return false;
}
export function dead(root) {
  const pid = Number(readFileSync(join(root, 'pid'), 'utf8').trim());
  try { process.kill(pid, 0); return false; }
  catch (error) { if (error.code === 'ESRCH') return true; throw error; }
}
