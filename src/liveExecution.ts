import type { ExecutionAttempt } from '../shared/ipc-types.ts';

export function liveExecutionStatus(attempt: ExecutionAttempt): string {
  switch (attempt.state) {
    case 'RESERVED':
      return 'The Order Gateway is validating this attempt. The provider order request has not been sent yet.';
    case 'INVALIDATED':
      return attempt.dispatchDisposition === 'STOPPED_BEFORE_DISPATCH'
        ? 'Stopped before provider dispatch. No order request was sent.'
        : 'This attempt was invalidated; check its saved dispatch disposition before taking further action.';
    case 'SUBMITTING':
      return 'The provider request may have been sent. Capacity remains held; do not retry.';
    case 'ACCEPTED':
      return attempt.operation === 'CANCEL'
        ? 'The provider acknowledged the cancellation request. The order is not confirmed cancelled.'
        : 'The provider accepted the order. This is not a fill.';
    case 'REJECTED':
      return attempt.operation === 'CANCEL'
        ? 'The provider rejected the cancellation request. The original order remains unresolved.'
        : 'The provider rejected the order. TradeX released its reservation.';
    case 'UNKNOWN_RECONCILING':
      return 'The provider outcome is uncertain. Capacity remains held; do not resend this request.';
    case 'CANCEL_PENDING':
      return 'The provider acknowledged cancellation; the order is not confirmed cancelled. The original commitment remains until provider evidence confirms a terminal state or a fill.';
  }
}
