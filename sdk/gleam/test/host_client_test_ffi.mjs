import { Buffer } from 'node:buffer';
import { mkdtemp, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import process from 'node:process';
import assert from 'node:assert/strict';
import { request, logs, events, submit_request } from './host_client_ffi.mjs';

export const pause = () => new Promise(resolve => setTimeout(resolve, 25));
const listen = server => new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve(server.address().port)));
const close = server => new Promise(resolve => { server.close(resolve); server.closeAllConnections(); });

export async function fixture(run, traced = false) {
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
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith('OTEL_')));
  const payloads = [];
  let collector;
  if (traced) {
    collector = createServer((req, res) => {
      const chunks = [];
      req.on('data', bytes => chunks.push(bytes));
      req.on('end', () => { payloads.push(Buffer.concat(chunks)); res.writeHead(200); res.end(); });
    });
    const collectorPort = await listen(collector);
    env.OTEL_EXPORTER_OTLP_ENDPOINT = `http://127.0.0.1:${collectorPort}/v1/traces`;
    env.OTEL_EXPORTER_OTLP_PROTOCOL = 'http/protobuf';
  }
  const child = spawn(binary, ['serve-runs', '--listen', `127.0.0.1:${port}`, '--data-dir', join(root, 'server'), '--max-runs', '2'], {
    env: { ...env, FLUENTCI_RUN_TOKEN: token }, stdio: ['ignore', 'ignore', 'pipe'],
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
    if (collector) await close(collector);
    await rm(root, { recursive: true, force: true });
  }
  if (traced) {
    const exported = Buffer.concat(payloads);
    assert.ok(exported.includes(Buffer.from('pipeline.run')));
    assert.ok(exported.includes(Buffer.from('pipeline.step')));
    assert.ok(exported.includes(Buffer.from('1234567890abcdef1234567890abcdef', 'hex')));
    assert.ok(exported.includes(Buffer.from('fedcba9876543210', 'hex')));
    assert.ok(exported.includes(Buffer.from('vendor=gleam')));
  }
}

export const traced_fixture = run => fixture(run, true);

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


export async function log_transport_contract() {
  let header = "4", body = new Uint8Array([0, 255, 195, 169]), calls = 0;
  const server = createServer((req, res) => {
    calls++;
    assert.equal(req.url, '/v1/runs/00000000-0000-0000-0000-000000000001/logs/0/stdout?cursor=0');
    res.writeHead(200, header === null ? {} : { 'X-Next-Cursor': header });
    res.end(body);
  });
  const port = await listen(server);
  const config = { base_url: `http://127.0.0.1:${port}`, token: 'log-token', timeout_ms: 5000 };
  const id = '00000000-0000-0000-0000-000000000001';
  const ok = (bytes, next) => ({ bytes, next });
  const invalid = () => 'decode-error';
  const httpError = () => 'http-error';
  const transport = () => 'transport-error';
  const read = (index = 0, cursor = 0) => logs(config, id, index, 'stdout', cursor, ok, invalid, httpError, transport);
  try {
    assert.deepEqual(await read(), { bytes: body, next: 4 });
    for (const malformed of [null, '', '3', '5', '-1', '4.0', '04', '9007199254740992', 'NaN']) {
      header = malformed;
      assert.equal(await read(), 'decode-error');
    }
    body = new Uint8Array(1024 * 1024 + 1);
    header = String(body.length);
    assert.equal(await read(), 'decode-error');
    const before = calls;
    for (const cursor of [-1, 0.5, Number.MAX_SAFE_INTEGER + 1]) assert.equal(await read(0, cursor), 'transport-error');
    for (const index of [-1, 0.5, 10000]) assert.equal(await read(index), 'transport-error');
    assert.equal(calls, before, 'invalid arguments must not reach the server');
  } finally { await close(server); }
}


export async function event_transport_contract() {
  const text = '{"sequence":0,"elapsed_ms":0,"type":"log","id":"x","stream":"stdout","text":"é"}\n';
  let body = new TextEncoder().encode(text), header = String(body.length), calls = 0;
  const server = createServer((req, res) => {
    calls++;
    assert.equal(req.url, '/v1/runs/00000000-0000-0000-0000-000000000001/events?cursor=0');
    res.writeHead(200, header === null ? {} : { 'X-Next-Cursor': header });
    res.end(body);
  });
  const port = await listen(server);
  const config = { base_url: `http://127.0.0.1:${port}`, token: 'event-token', timeout_ms: 5000 };
  const read = (cursor = 0) => events(config, '00000000-0000-0000-0000-000000000001', cursor,
    (text, next) => ({ text, next }), () => 'decode-error', () => 'http-error', () => 'transport-error');
  try {
    assert.deepEqual(await read(), { text, next: body.length });
    header = String(text.length);
    assert.equal(await read(), 'decode-error', 'cursor counts UTF-8 bytes, not characters');
    header = null;
    assert.equal(await read(), 'decode-error');
    body = new Uint8Array([255, 10]); header = '2';
    assert.equal(await read(), 'decode-error', 'invalid UTF-8 must not silently become replacement characters');
    body = new Uint8Array(4 * 1024 * 1024 + 1); header = String(body.length);
    assert.equal(await read(), 'decode-error');
    body = new Uint8Array(); header = '0';
    assert.deepEqual(await read(), { text: '', next: 0 });
    const before = calls;
    for (const cursor of [-1, 0.5, Number.MAX_SAFE_INTEGER + 1]) assert.equal(await read(cursor), 'transport-error');
    assert.equal(calls, before);
  } finally { await close(server); }
}


export async function trace_transport_contract() {
  const requests = [];
  const server = createServer((req, res) => {
    requests.push(req.headers);
    res.writeHead(202, { 'Idempotency-Key': req.headers['idempotency-key'] ?? '' }); res.end('{}');
  });
  const port = await listen(server);
  const config = { base_url: `http://127.0.0.1:${port}`, token: 'trace-token', timeout_ms: 5000 };
  const parent = '00-1234567890abcdef1234567890abcdef-1234567890abcdef-01';
  const send = (p, state) => submit_request(config, '{}', 'trace-key', p, state, () => 'ok', () => 'error');
  try {
    assert.equal(await send(parent, 'vendor=gleam'), 'ok');
    assert.equal(requests[0].traceparent, parent);
    assert.equal(requests[0].tracestate, 'vendor=gleam');
    assert.equal(requests[0].authorization, 'Bearer trace-token');
    assert.equal(requests[0]['idempotency-key'], 'trace-key');
    assert.equal(await send('', ''), 'ok');
    assert.equal(requests[1].traceparent, undefined);
    assert.equal(requests[1].tracestate, undefined);
    const count = requests.length;
    for (const [p, state] of [[parent + '\r\nX-Extra: bad', ''], [parent, 'bad\nvalue'], [parent, 'x'.repeat(513)], ['', 'vendor=gleam'], [parent, 'é']]) {
      assert.equal(await send(p, state), 'error');
    }
    assert.equal(requests.length, count);
  } finally { await close(server); }
}
