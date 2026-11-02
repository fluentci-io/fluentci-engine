import { mkdtemp, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import process from 'node:process';
import assert from 'node:assert/strict';
import { request } from './host_client_ffi.mjs';

export const pause = () => new Promise(resolve => setTimeout(resolve, 25));
const listen = server => new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve(server.address().port)));
const close = server => new Promise(resolve => { server.close(resolve); server.closeAllConnections(); });

export async function fixture(run) {
  const binary = process.env.FLUENTCI_ENGINE_BIN;
  if (!binary) {
    if (process.env.FLUENTCI_REQUIRE_ENGINE === '1') throw Error('FLUENTCI_ENGINE_BIN required');
    console.error('Gleam HTTP integration skipped: set FLUENTCI_ENGINE_BIN');
    return;
  }
  if (binary.split('/').includes('debug')) throw Error('release engine required');
  const root = await mkdtemp(join(tmpdir(), 'gleam-http-'));
  const reservation = createServer();
  const port = await listen(reservation);
  await close(reservation);
  const base = `http://127.0.0.1:${port}`, token = 'gleam-integration-secret';
  const child = spawn(binary, ['serve-runs', '--listen', `127.0.0.1:${port}`, '--data-dir', join(root, 'server'), '--max-runs', '2'], {
    env: { ...process.env, FLUENTCI_RUN_TOKEN: token }, stdio: ['ignore', 'ignore', 'pipe'],
  });
  let diagnostics = '', launchError;
  child.stderr.on('data', value => { diagnostics = (diagnostics + value).slice(-8192); });
  child.on('error', error => { launchError = error; });
  const stopped = new Promise(resolve => child.once('close', resolve));
  try {
    let ready = false;
    for (let i = 0; i < 200; i++) {
      if (launchError) throw launchError;
      if (child.exitCode !== null) throw Error(`server exited: ${diagnostics}`);
      try {
        const response = await fetch(`${base}/v1/runs`, { headers: { Authorization: `Bearer ${token}` }, signal: AbortSignal.timeout(200) });
        await response.arrayBuffer();
        if (response.ok) { ready = true; break; }
      } catch {}
      await pause();
    }
    assert.ok(ready, `server did not become ready: ${diagnostics}`);
    await run(base, token, root);
    assert.equal(await readFile(join(root, 'output'), 'utf8'), 'executed', 'replayed submission must execute once');
  } finally {
    child.kill('SIGINT');
    await stopped;
    await rm(root, { recursive: true, force: true });
  }
}

export async function transport_contract() {
  let mode = 'ok', calls = 0;
  const id = '00000000-0000-0000-0000-000000000001';
  const server = createServer((req, res) => {
    calls++;
    assert.equal(req.headers.authorization, 'Bearer hidden-token');
    if (mode === 'redirect') { res.writeHead(302, { Location: '/target' }); res.end(); }
    else if (mode === 'slow') { res.writeHead(200); res.write('{'); }
    else if (mode === 'large') { res.end('x'.repeat(16 * 1024 * 1024 + 1)); }
    else { res.writeHead(202, { 'Idempotency-Key': req.headers['idempotency-key'] ?? '' }); res.end('payload'); }
  });
  const port = await listen(server);
  const config = { base_url: `http://127.0.0.1:${port}`, token: 'hidden-token', timeout_ms: 5000 };
  const ok = (status, body, key) => ({ status, body, key });
  const error = message => ({ error: message });
  try {
    const value = await request(config, 'POST', '', '', '{}', 'retry-key', ok, error);
    assert.deepEqual(value, { status: 202, body: 'payload', key: 'retry-key' });
    mode = 'redirect';
    const before = calls;
    assert.ok((await request(config, 'GET', id, '', '', '', ok, error)).error);
    assert.equal(calls, before + 1, 'must not follow redirects');
    mode = 'slow';
    assert.match((await request({ ...config, timeout_ms: 50 }, 'GET', id, '', '', '', ok, error)).error, /timed out/);
    mode = 'large';
    assert.ok((await request(config, 'GET', id, '', '', '', ok, error)).error);
    const noRequests = calls;
    for (const invalid of [{ ...config, base_url: config.base_url + '/prefix' }, { ...config, timeout_ms: 0 }, { ...config, timeout_ms: 2147483648 }]) {
      const value = await request(invalid, 'GET', id, '', '', '', ok, error);
      assert.ok(value.error);
      assert.ok(!value.error.includes('hidden-token'));
    }
    assert.ok((await request(config, 'GET', '../escape', '', '', '', ok, error)).error);
    assert.ok((await request(config, 'POST', '', '', '{}', 'bad key', ok, error)).error);
    assert.equal(calls, noRequests);
  } finally { await close(server); }
}

export async function mock_fixture(run) {
  const server = createServer((req, res) => {
    const mode = req.headers.authorization;
    const state = { id: '00000000-0000-0000-0000-000000000001', status: 'running', created_at_ms: 1 };
    if (mode === 'Bearer unknown-status') state.status = 'unknown';
    if (mode === 'Bearer wrong-id') state.id = '00000000-0000-0000-0000-000000000002';
    if (mode === 'Bearer negative-time') state.created_at_ms = -1;
    res.writeHead(req.method === 'POST' ? 202 : 200);
    res.end(mode === 'Bearer malformed' ? '{' : JSON.stringify(state));
  });
  const port = await listen(server);
  try { await run(`http://127.0.0.1:${port}`); }
  finally { await close(server); }
}
