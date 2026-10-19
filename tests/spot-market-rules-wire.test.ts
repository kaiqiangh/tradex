import assert from 'node:assert/strict';
import test from 'node:test';
import { decode } from '../src/projection.ts';

const sourceRow = { scope: 'SYMBOL', origin: 'EXCHANGE_INFO', ruleType: 'MAX_POSITION', applicable: true, outcome: 'UNAVAILABLE', reasonCode: 'CURRENT_FREE_LOCKED_AND_OPEN_BUY_POSITION_REQUIRED', unit: 'BTC' };
const comparison = { observed: '5', increment: '0.001', total: '5.001', limit: '5.001' };

test('old source rows remain valid without a local comparison', () => {
  assert.deepEqual(decode('SpotRuleEvaluation', sourceRow), sourceRow);
  const composed = { ...sourceRow, outcome: 'PASS', reasonCode: 'LOCAL_MARKET_RULE_SATISFIED', localComparison: comparison };
  assert.deepEqual(decode('SpotRuleEvaluation', composed), composed);
  assert.throws(() => decode('SpotRuleEvaluation', { ...composed, approved: true }));
});

test('local comparisons retain exact strings and bounded output without renderer authority', () => {
  assert.deepEqual(decode('SpotMarketRuleComparison', comparison), comparison);
  for (const field of ['observed', 'increment', 'total', 'limit']) {
    assert.doesNotThrow(() => decode('SpotMarketRuleComparison', { ...comparison, [field]: '9'.repeat(64) }));
    for (const invalid of ['', '9'.repeat(65), 5, null]) {
      assert.throws(() => decode('SpotMarketRuleComparison', { ...comparison, [field]: invalid }));
    }
  }
  assert.throws(() => decode('SpotMarketRuleComparison', { ...comparison, allowed: true }));
});

test('MARKET composition bounds input versions, admission reason and carried limitations', () => {
  const composition = { capacityVersion: `sha256:${'a'.repeat(64)}`, intervalVersion: `sha256:${'b'.repeat(64)}`, admissionOutcome: 'PASS', admissionReason: 'Complete exact inputs.', carriedLimitations: ['NON_ATOMIC_PROVIDER_SNAPSHOT'] };
  assert.deepEqual(decode('SpotMarketRuleComposition', composition), composition);
  for (const invalid of [
    { ...composition, capacityVersion: 'short' },
    { ...composition, intervalVersion: 'x'.repeat(72) },
    { ...composition, admissionOutcome: 'APPROVED' },
    { ...composition, admissionReason: '' },
    { ...composition, admissionReason: 'x'.repeat(257) },
    { ...composition, carriedLimitations: Array(6).fill('LIMITATION') },
    { ...composition, carriedLimitations: ['x'.repeat(129)] },
    { ...composition, armed: true },
  ]) assert.throws(() => decode('SpotMarketRuleComposition', invalid));
});
