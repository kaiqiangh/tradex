import { spawnSync } from 'node:child_process';

// Each independent external-provider scenario owns an application process.
// Production process/IP quotas remain intact inside each scenario, including
// the dedicated cross-workspace/source-generation quota regression.
const compiled = spawnSync('cargo', [
  'test', '-p', 'tradex', '--features', 'integration-test',
  '--test', 'binance_market_hot', '--no-run', '--message-format=json',
], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
process.stderr.write(compiled.stderr ?? '');
if (compiled.error || compiled.status !== 0) {
  if (compiled.error) console.error(compiled.error);
  process.exit(compiled.status ?? 1);
}
const artifacts = compiled.stdout.split('\n').filter(Boolean).map(line => JSON.parse(line));
const binary = artifacts.find(item => item.reason === 'compiler-artifact'
  && item.target?.name === 'binance_market_hot' && item.profile?.test && item.executable)?.executable;
if (!binary) throw new Error('Cargo did not report the Binance Hot test executable');
const listing = spawnSync(binary, ['--list', '--format=terse'], { encoding: 'utf8' });
if (listing.error || listing.status !== 0) throw new Error('Cannot enumerate Binance Hot scenarios');
const names = listing.stdout.split('\n').filter(line => line.endsWith(': test'))
  .map(line => line.slice(0, -6));
if (!names.length) throw new Error('No Binance Hot scenarios were discovered');
for (const [index, name] of names.entries()) {
  console.log(`Scenario ${index + 1}/${names.length}: ${name}`);
  const result = spawnSync(binary, [name, '--exact', '--nocapture'], { stdio: 'inherit' });
  if (result.error || result.status !== 0) {
    if (result.error) console.error(result.error);
    process.exit(result.status ?? 1);
  }
}
console.log(`PASS: ${names.length} Binance Hot scenarios, serial isolated processes`);
