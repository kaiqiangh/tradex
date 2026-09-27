import assert from 'node:assert/strict';
import test from 'node:test';
import type { ExecutionAttempt } from '../shared/ipc-types.ts';
import { liveExecutionStatus } from '../src/liveExecution.ts';

function attempt(state: ExecutionAttempt['state'], operation: ExecutionAttempt['operation'] = 'PLACE_ORDER', dispatchDisposition?: ExecutionAttempt['dispatchDisposition']): ExecutionAttempt {
  return { state, operation, dispatchDisposition } as ExecutionAttempt;
}

test('Live attempt status never describes acknowledgement as a fill or cancellation', () => {
  assert.match(liveExecutionStatus(attempt('ACCEPTED')), /accepted.*not a fill/i);
  assert.match(liveExecutionStatus(attempt('CANCEL_PENDING', 'CANCEL')), /not confirmed cancelled/i);
  assert.match(liveExecutionStatus(attempt('UNKNOWN_RECONCILING')), /do not resend/i);
  assert.match(liveExecutionStatus(attempt('SUBMITTING')), /may have been sent.*do not retry/i);
});

test('Live attempt status only claims no dispatch with persisted stop evidence', () => {
  assert.match(liveExecutionStatus(attempt('INVALIDATED', 'PLACE_ORDER', 'STOPPED_BEFORE_DISPATCH')), /no order request was sent/i);
  assert.match(liveExecutionStatus(attempt('INVALIDATED')), /check its saved dispatch disposition/i);
  assert.match(liveExecutionStatus(attempt('REJECTED')), /released its reservation/i);
});
