import { createHash } from 'node:crypto';
import { readFile, writeFile, rename } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const action = process.argv[2];
if (action !== 'dev' && action !== 'build' && action !== 'browser') {
  throw new Error('usage: node scripts/desktop-order-gateway.mjs <dev|build|browser>');
}

function run(command, args, env = process.env) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const metadata = spawnSync('cargo', [
  'metadata', '--manifest-path', 'src-tauri/Cargo.toml', '--format-version', '1', '--no-deps',
], { cwd: root, encoding: 'utf8' });
if (metadata.error) throw metadata.error;
if (metadata.status !== 0) process.exit(metadata.status ?? 1);
const targetDirectory = JSON.parse(metadata.stdout).target_directory;
const profile = action === 'build' ? 'release' : 'debug';
const features = action === 'browser' ? 'integration-test order-gateway-runtime' : 'order-gateway-runtime';
const buildArgs = ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--features', features, '--bin', 'tradex-order-gateway'];
if (action === 'browser') buildArgs.push('--bin', 'tradex-ipc');
if (profile === 'release') buildArgs.push('--release');
run('cargo', buildArgs);

const gatewayPath = join(targetDirectory, profile, 'tradex-order-gateway');
const gatewayBytes = await readFile(gatewayPath);
const sha256 = createHash('sha256').update(gatewayBytes).digest('hex');
if (action === 'browser') {
  run(join(root, 'node_modules', '.bin', 'vite'), ['--host', '127.0.0.1', '--mode', 'integration'], {
    ...process.env,
    TRADEX_ORDER_GATEWAY_SHA256: sha256,
    TRADEX_INTEGRATION_ORDER_GATEWAY: '1',
  });
} else {
  const tauri = join(root, 'node_modules', '.bin', 'tauri');
  const tauriArgs = action === 'dev'
    ? ['dev', '--features', 'desktop']
    : ['build', '--features', 'desktop', '--no-bundle'];
  run(tauri, tauriArgs, { ...process.env, TRADEX_ORDER_GATEWAY_SHA256: sha256 });
  if (action === 'build') {
    // Tauri's build compiles all binaries with desktop features and may replace
    // the Gateway whose exact bytes were pinned before that build. Restore the
    // pinned executable atomically so the delivered pair uses the same bytes.
    const stagedGatewayPath = `${gatewayPath}.pinned`;
    await writeFile(stagedGatewayPath, gatewayBytes, { mode: 0o755 });
    await rename(stagedGatewayPath, gatewayPath);
  }
}
