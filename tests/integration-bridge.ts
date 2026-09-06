// Browser QA calls the same Rust dispatcher over inherited stdio. Never included in a desktop build.
import { spawn } from 'node:child_process';
import { mkdtempSync, realpathSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, sep } from 'node:path';
import { createInterface } from 'node:readline';
import type { ServerResponse } from 'node:http';
import type { Plugin } from 'vite';

export function integrationBridge(): Plugin {
  return {
    name: 'tradex-isolated-rust-integration',
    apply: 'serve',
    configureServer(server) {
      const directory = realpathSync(mkdtempSync(join(tmpdir(), 'tradex-browser-')));
      const child = spawn(resolve('target/debug/tradex-ipc'), [join(directory, 'workspace')], { stdio: ['pipe', 'pipe', 'inherit'] });
      const clients = new Set<ServerResponse>();
      const pending = new Map<string, ServerResponse>();
      createInterface({ input: child.stdout }).on('line', line => {
        const frame = JSON.parse(line);
        if (frame.kind === 'event') {
          for (const response of clients) {
            if (response.destroyed || response.writableLength > 65_536) { response.destroy(); clients.delete(response); }
            else response.write(`data: ${JSON.stringify(frame.event)}\n\n`);
          }
        } else {
          const response = pending.get(frame.result.requestId);
          if (response) { response.end(JSON.stringify(frame.result)); pending.delete(frame.result.requestId); }
        }
      });
      child.on('exit', () => {
        for (const response of pending.values()) { response.writeHead(503); response.end(); }
        pending.clear();
        for (const response of clients) response.end();
        clients.clear();
      });
      server.httpServer?.once('close', () => {
        child.once('exit', () => rmSync(directory, { recursive: true, force: true }));
        child.kill();
      });
      server.middlewares.use('/__integration', async (request, response) => {
        const origin = request.headers.origin;
        if (origin && origin !== 'http://127.0.0.1:1420') { response.writeHead(403); response.end(); return; }
        if (request.url === '/disconnect' && request.method === 'POST') {
          // Fault injection for the real UI subscription/retry path; no domain state changes.
          for (const client of clients) client.end();
          clients.clear();
          response.writeHead(204); response.end(); return;
        }
        if (request.url === '/events' && request.method === 'GET') {
          response.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-store', Connection: 'keep-alive' });
          response.write(': connected\n\n');
          clients.add(response);
          request.on('close', () => clients.delete(response));
          return;
        }
        if (request.url !== '/command' || request.method !== 'POST') { response.writeHead(404); response.end(); return; }
        try {
          const parts: Buffer[] = []; let size = 0;
          for await (const chunk of request) {
            size += chunk.length;
            if (size > 60_000) { response.writeHead(413); response.end(); return; }
            parts.push(chunk);
          }
          const envelope = JSON.parse(Buffer.concat(parts).toString());
          if (typeof envelope.requestId !== 'string' || pending.has(envelope.requestId)) { response.writeHead(400); response.end(); return; }
          // The QA bridge owns only its fresh temporary tree, never a real user workspace.
          if (envelope.command === 'workspace.open' && envelope.payload?.path) {
            const path = resolve(envelope.payload.path);
            if (!path.startsWith(directory + sep)) { response.writeHead(403); response.end(); return; }
          }
          response.setHeader('Content-Type', 'application/json');
          response.setHeader('Cache-Control', 'no-store');
          pending.set(envelope.requestId, response);
          response.on('close', () => pending.delete(envelope.requestId));
          child.stdin.write(JSON.stringify(envelope) + '\n');
        } catch { if (!response.headersSent) response.writeHead(400); response.end(); }
      });
    },
  };
}
