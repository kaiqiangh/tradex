type RecoveryVariant = { title: string; message: string; action: string };

const variants: Record<string, RecoveryVariant> = {
  AUTH_ERROR: { title: 'Authentication needs attention', message: 'Review the credential or authorization state before retrying.', action: 'Review authentication' },
  PERMISSION_ERROR: { title: 'Permission needs attention', message: 'Review the provider permissions before continuing.', action: 'Review permissions' },
  RATE_LIMITED: { title: 'Provider rate limit reached', message: 'Wait for the provider cooldown before retrying.', action: 'Retry later' },
  NETWORK_ERROR: { title: 'Network source unavailable', message: 'The last trusted observation is preserved while the source recovers.', action: 'Retry source' },
  UNSUPPORTED_CAPABILITY: { title: 'Capability unavailable', message: 'This operation is outside the current provider or workspace capability.', action: 'Review capability' },
  MARKET_CLOSED: { title: 'Market session is closed', message: 'Live authority remains blocked until the next authoritative session boundary.', action: 'View market state' },
  INSTRUMENT_HALTED: { title: 'Instrument is halted', message: 'Live authority remains blocked until the venue reports recovery.', action: 'View market state' },
  INVALID_ORDER: { title: 'Order needs correction', message: 'Review the order fields and retry after the validation issue is resolved.', action: 'Review order' },
  INSUFFICIENT_FUNDS: { title: 'Funds are insufficient', message: 'Review the account capacity before preparing another proposal.', action: 'Review account' },
  RISK_REJECTED: { title: 'Risk policy rejected this action', message: 'Review the current risk policy and its blocking reason.', action: 'Review risk policy' },
  SUBMISSION_REJECTED: { title: 'Submission was rejected', message: 'The provider rejected the submission; review the preserved response.', action: 'Review submission' },
  SUBMISSION_AMBIGUOUS: { title: 'Submission needs reconciliation', message: 'Query authoritative provider state before retrying.', action: 'Review reconciliation' },
  STREAM_DISCONNECTED: { title: 'Stream disconnected', message: 'Reload the authoritative snapshot before continuing.', action: 'Reload state' },
  STATE_STALE: { title: 'State is stale', message: 'Reload authoritative state before continuing.', action: 'Reload state' },
  RECONCILIATION_REQUIRED: { title: 'Reconciliation required', message: 'Resolve the provider state before another authority decision.', action: 'Review reconciliation' },
  MODEL_UNAVAILABLE: { title: 'Model is unavailable', message: 'Verify a supported route before retrying.', action: 'Review model' },
  QUOTA_EXCEEDED: { title: 'Model quota reached', message: 'Wait for the known cooldown before retrying.', action: 'Retry later' },
  OAUTH_EXPIRED: { title: 'OAuth authorization expired', message: 'Re-authorize the provider before retrying.', action: 'Re-authorize' },
  INTERNAL_ERROR: { title: 'TradeX needs attention', message: 'Reload authoritative state before continuing.', action: 'Reload state' },
};

export function ErrorRecoveryPanel({ code, onAction }: { code?: string; onAction?: () => void }) {
  const variant = code ? variants[code] : undefined;
  if (!variant) return null;
  return <section className="error-recovery-panel" role="alert" aria-live="polite">
    <div><p className="eyebrow">Error recovery</p><h3>{variant.title}</h3><p>{variant.message}</p></div>
    {onAction && <button type="button" onClick={onAction}>{variant.action}</button>}
  </section>;
}
