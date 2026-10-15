import assert from 'node:assert/strict';
import test from 'node:test';
import { calculateExactSpread } from '../src/marketSpread.ts';

test('calculates a venue spread using exact decimal arithmetic', () => {
  assert.deepEqual(calculateExactSpread('50000.00', '50000.25'), { status: 'AVAILABLE', value: '0.25' });
  assert.deepEqual(calculateExactSpread('0.10000000000000001', '0.10000000000000002'), {
    status: 'AVAILABLE', value: '0.00000000000000001',
  });
});

test('keeps the spread unavailable when quote sides are incomplete, malformed, or crossed', () => {
  assert.equal(calculateExactSpread('10.00', undefined).status, 'UNAVAILABLE');
  assert.equal(calculateExactSpread('1e-8', '0.00000002').status, 'UNAVAILABLE');
  assert.equal(calculateExactSpread('10.01', '10.00').status, 'UNAVAILABLE');
  assert.equal(calculateExactSpread('0', '10.00').status, 'UNAVAILABLE');
});
