import assert from 'node:assert/strict';
import test from 'node:test';
import { approvalReviewMoneyCopy } from '../src/approval-review-copy.ts';

test('SELL LIMIT review shows a limit-price proceeds scenario without inventing a maximum', () => {
  assert.deepEqual(approvalReviewMoneyCopy('SELL', 'LIMIT'), {
    expectedLabel: 'Gross proceeds at limit price (scenario)',
    disclosure: 'A sell limit sets the minimum sale price. This scenario is not a maximum on proceeds or fees.',
  });
});

test('SELL MARKET review identifies the ceiling as TradeX pre-submission authorization', () => {
  assert.deepEqual(approvalReviewMoneyCopy('SELL', 'MARKET'), {
    expectedLabel: 'Estimated gross proceeds',
    maximumLabel: 'Maximum authorized trade value',
    disclosure: 'TradeX checks this gross-value ceiling before submission. The broker does not enforce it as a market-order bound, and it does not cap final proceeds or fees.',
  });
});

test('BUY review retains its spend and authorization labels', () => {
  assert.deepEqual(approvalReviewMoneyCopy('BUY', 'LIMIT'), {
    expectedLabel: 'Expected spend',
    maximumLabel: 'Maximum authorized spend',
  });
});
