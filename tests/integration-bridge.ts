// Browser QA calls the same Rust dispatcher over inherited stdio. Never included in a desktop build.
import { spawn } from 'node:child_process';
import { randomUUID } from 'node:crypto';
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
      const child = spawn(resolve('target/debug/tradex-ipc'), [join(directory, 'workspace')], {
        stdio: ['pipe', 'pipe', 'inherit'],
        env: { ...process.env, TRADEX_BACKTEST_FIXTURE: '1', TRADEX_MARKET_FIXTURE: '1', TRADEX_PORTFOLIO_FIXTURE: '1', TRADEX_RESEARCH_FIXTURE: '1', TRADEX_SCREENER_FIXTURE: '1', TRADEX_STRATEGY_FIXTURE: '1' },
      });
      const blockedDirectory = realpathSync(mkdtempSync(join(tmpdir(), 'tradex-browser-blocked-')));
      const blockedEnv = { ...process.env };
      delete blockedEnv.TRADEX_SCREENER_FIXTURE;
      const blockedChild = spawn(resolve('target/debug/tradex-ipc'), [join(blockedDirectory, 'workspace')], {
        stdio: ['pipe', 'pipe', 'inherit'],
        env: blockedEnv,
      });
      let blockedWorkspaceId: string | undefined;
      let blockedChildAvailable = true;
      let screenerBlocked = false;
      let backtestMode: 'normal' | 'delay' | 'list-error' | 'compare-error' = 'normal';
      const blockedBootstrapRequestId = randomUUID();
      let resolveBlockedBootstrap!: () => void;
      const blockedBootstrapReady = new Promise<void>(resolve => { resolveBlockedBootstrap = resolve; });
      const clients = new Set<ServerResponse>();
      const pending = new Map<string, { response: ServerResponse; command: string }>();
      const blockedPending = new Map<string, { response: ServerResponse; command: string }>();
      createInterface({ input: child.stdout }).on('line', line => {
        const frame = JSON.parse(line);
        if (frame.kind === 'event') {
          for (const response of clients) {
            if (response.destroyed || response.writableLength > 65_536) { response.destroy(); clients.delete(response); }
            else response.write(`data: ${JSON.stringify(frame.event)}\n\n`);
          }
        } else {
          const pendingResult = pending.get(frame.result.requestId);
          if (pendingResult) {
            const finish = () => {
              if (pending.get(frame.result.requestId) !== pendingResult) return;
              pending.delete(frame.result.requestId);
              if (!pendingResult.response.destroyed) pendingResult.response.end(JSON.stringify(frame.result));
            };
            if (backtestMode === 'delay' && pendingResult.command === 'backtest.list') setTimeout(finish, 800);
            else finish();
          }
        }
      });
      child.on('exit', () => {
        for (const { response } of pending.values()) { response.writeHead(503); response.end(); }
        pending.clear();
        for (const response of clients) response.end();
        clients.clear();
      });
      createInterface({ input: blockedChild.stdout }).on('line', line => {
        const frame = JSON.parse(line);
        if (frame.kind !== 'result') return;
        if (frame.result.requestId === blockedBootstrapRequestId && frame.result.ok) {
          blockedWorkspaceId = frame.result.data.workspaceId;
          resolveBlockedBootstrap();
          return;
        }
        const pendingResult = blockedPending.get(frame.result.requestId);
        if (pendingResult) {
          if (pendingResult.command === 'workspace.open' && frame.result.ok) blockedWorkspaceId = frame.result.data.workspaceId;
          pendingResult.response.end(JSON.stringify(frame.result));
          blockedPending.delete(frame.result.requestId);
        }
      });
      blockedChild.on('exit', () => {
        blockedChildAvailable = false;
        blockedWorkspaceId = undefined;
        for (const { response } of blockedPending.values()) { response.writeHead(503); response.end(); }
        blockedPending.clear();
      });
      blockedChild.on('error', () => {
        blockedChildAvailable = false;
        blockedWorkspaceId = undefined;
      });
      const cleanup = (path: string) => () => rmSync(path, { recursive: true, force: true });
      child.once('exit', cleanup(directory));
      child.once('error', cleanup(directory));
      blockedChild.once('exit', cleanup(blockedDirectory));
      blockedChild.once('error', cleanup(blockedDirectory));
      blockedChild.stdin.write(JSON.stringify({
        requestId: blockedBootstrapRequestId,
        schemaVersion: 1,
        command: 'workspace.open',
        payload: {},
      }) + '\n');
      server.httpServer?.once('close', () => {
        child.kill();
        blockedChild.kill();
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
        if (request.url === '/screener-mode' && request.method === 'POST') {
          try {
            const parts: Buffer[] = []; let size = 0;
            for await (const chunk of request) {
              size += chunk.length;
              if (size > 100) { response.writeHead(413); response.end(); return; }
              parts.push(chunk);
            }
            const payload = JSON.parse(Buffer.concat(parts).toString());
            if (typeof payload.enabled !== 'boolean') { response.writeHead(400); response.end(); return; }
            if (payload.enabled) {
              await Promise.race([
                blockedBootstrapReady,
                new Promise(resolve => setTimeout(resolve, 3000)),
              ]);
              if (!blockedChildAvailable || !blockedWorkspaceId) { response.writeHead(503); response.end(); return; }
            }
            screenerBlocked = payload.enabled;
            response.writeHead(204); response.end();
          } catch { response.writeHead(400); response.end(); }
          return;
        }
        if (request.url === '/backtest-mode' && request.method === 'POST') {
          try {
            const parts: Buffer[] = []; let size = 0;
            for await (const chunk of request) {
              size += chunk.length;
              if (size > 100) { response.writeHead(413); response.end(); return; }
              parts.push(chunk);
            }
            const payload = JSON.parse(Buffer.concat(parts).toString());
            if (!['normal', 'delay', 'list-error', 'compare-error'].includes(payload.mode)) { response.writeHead(400); response.end(); return; }
            backtestMode = payload.mode;
            response.writeHead(204); response.end();
          } catch { response.writeHead(400); response.end(); }
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
          if (typeof envelope.requestId !== 'string' || pending.has(envelope.requestId) || blockedPending.has(envelope.requestId)) { response.writeHead(400); response.end(); return; }
          // The QA bridge owns only its fresh temporary tree, never a real user workspace.
          if (envelope.command === 'workspace.open' && envelope.payload?.path) {
            const path = resolve(envelope.payload.path);
            if (!path.startsWith(directory + sep)) { response.writeHead(403); response.end(); return; }
          }
          if (screenerBlocked && envelope.command === 'market.screen') {
            if (!blockedChildAvailable || !blockedWorkspaceId) { response.writeHead(503); response.end(); return; }
            envelope.payload = { ...envelope.payload, workspaceId: blockedWorkspaceId };
            response.setHeader('Content-Type', 'application/json');
            response.setHeader('Cache-Control', 'no-store');
            blockedPending.set(envelope.requestId, { response, command: envelope.command });
            response.on('close', () => blockedPending.delete(envelope.requestId));
            blockedChild.stdin.write(JSON.stringify(envelope) + '\n');
            return;
          }
          response.setHeader('Content-Type', 'application/json');
          response.setHeader('Cache-Control', 'no-store');
          const failedBacktestQuery = (backtestMode === 'list-error' && envelope.command === 'backtest.list')
            || (backtestMode === 'compare-error' && envelope.command === 'backtest.compare');
          if (failedBacktestQuery) {
            response.end(JSON.stringify({
              requestId: envelope.requestId,
              schemaVersion: 1,
              ok: false,
              error: {
                category: 'RUNTIME_ERROR',
                code: 'BACKTEST_RUNTIME_UNAVAILABLE',
                message: 'The backtest runtime failed closed without producing synthetic results.',
                retryable: true,
                blocking: true,
                remediationActions: [{ id: 'retry_backtest_run', label: 'Retry backtest' }],
              },
            }));
            return;
          }
          pending.set(envelope.requestId, { response, command: envelope.command });
          response.on('close', () => pending.delete(envelope.requestId));
          child.stdin.write(JSON.stringify(envelope) + '\n');
        } catch { if (!response.headersSent) response.writeHead(400); response.end(); }
      });
    },
  };
}
