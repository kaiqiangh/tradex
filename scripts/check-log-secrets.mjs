import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const sensitivePatterns = [
  /\b(?:authorization|api[_-]?key|client[_-]?secret|access[_-]?token|refresh[_-]?token|password|private[_-]?key|credential)s?\s*[:=]\s*(?!\[redacted\])[^\s,;"'{}]+/i,
  /\bbearer\s+(?!\[redacted\])[^\s,;"'{}]+/i,
  /\b(?:sk-(?:proj-)?[a-z0-9_-]{12,}|ghp_[a-z0-9]{20,})\b/i,
];

function findSensitiveMarkers(corpus) {
  const findings = [];
  for (const [index, line] of corpus.split(/\r?\n/).entries()) {
    for (const pattern of sensitivePatterns) {
      if (pattern.test(line)) findings.push({ line: index + 1, marker: pattern.source });
    }
    try {
      const value = JSON.parse(line);
      const inspect = (node, path = '$') => {
        if (Array.isArray(node)) node.forEach((child, childIndex) => inspect(child, `${path}[${childIndex}]`));
        else if (node && typeof node === 'object') {
          for (const [key, child] of Object.entries(node)) {
            if (/^(authorization|api[_-]?key|apikey|client[_-]?secret|secret|access[_-]?token|refresh[_-]?token|password|private[_-]?key|credential|credentials)$/i.test(key)) {
              findings.push({ line: index + 1, marker: `sensitive JSON field ${path}.${key}` });
            }
            inspect(child, `${path}.${key}`);
          }
        }
      };
      inspect(value);
    } catch { /* Non-JSON diagnostic lines are scanned by the marker patterns above. */ }
  }
  return findings;
}

for (const canary of [
  'api_key=synthetic-api-canary',
  'authorization: Bearer synthetic-bearer-canary',
  '{"credential":"synthetic-credential-canary"}',
  'sk-proj-syntheticcredentialcanary12345',
]) assert.ok(findSensitiveMarkers(canary).length > 0, `scanner failed to detect ${canary}`);

const directory = mkdtempSync(join(tmpdir(), 'tradex-s32-log-scan-'));
try {
  const workspacePath = join(directory, 'workspace');
  const probe = {
    requestId: 's32-diagnostic-scan',
    schemaVersion: 1,
    command: 's32.synthetic.diagnostic_probe',
    payload: {
      description: 'api_key=synthetic-api-canary',
      authorization: 'Bearer synthetic-bearer-canary',
      credential: 'synthetic-credential-canary',
    },
  };
  const metadata = spawnSync('cargo', ['metadata', '--no-deps', '--format-version=1'], {
    cwd: root,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
  });
  if (metadata.error) throw metadata.error;
  assert.equal(metadata.status, 0, `isolated diagnostic metadata failed:\n${metadata.stderr}`);
  const targetDirectory = JSON.parse(metadata.stdout).target_directory;
  const build = spawnSync('cargo', [
    'build', '--quiet', '--manifest-path', 'src-tauri/Cargo.toml', '--bin', 'tradex-ipc',
  ], { cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  if (build.error) throw build.error;
  assert.equal(build.status, 0, `isolated diagnostic build failed:\n${build.stderr}`);
  const executable = join(
    targetDirectory,
    'debug',
    process.platform === 'win32' ? 'tradex-ipc.exe' : 'tradex-ipc',
  );
  const run = spawnSync(executable, [workspacePath], {
    cwd: root,
    input: `${JSON.stringify(probe)}\n`,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
  });
  if (run.error) throw run.error;
  assert.equal(run.status, 0, `isolated diagnostic probe failed:\n${run.stderr}`);
  const corpus = `${run.stdout}\n${run.stderr}`;
  assert.match(corpus, /IPC_COMMAND_UNKNOWN/, 'the real dispatcher must emit a bounded diagnostic result');
  const findings = findSensitiveMarkers(corpus);
  assert.deepEqual(findings, [], `secret-like values reached representative dispatcher diagnostics: ${JSON.stringify(findings)}`);
  process.stdout.write('Secret scan passed: real isolated dispatcher diagnostics had zero findings; all synthetic canaries were detected.\n');
} finally {
  rmSync(directory, { recursive: true, force: true });
}
