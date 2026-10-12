export const canonicalRecoveryCodes = [
  'AUTH_ERROR', 'PERMISSION_ERROR', 'RATE_LIMITED', 'NETWORK_ERROR', 'UNSUPPORTED_CAPABILITY',
  'MARKET_CLOSED', 'INSTRUMENT_HALTED', 'INVALID_ORDER', 'INSUFFICIENT_FUNDS', 'RISK_REJECTED',
  'RISK_EVIDENCE_UNAVAILABLE', 'SUBMISSION_REJECTED', 'SUBMISSION_AMBIGUOUS', 'STREAM_DISCONNECTED',
  'STATE_STALE', 'RECONCILIATION_REQUIRED', 'MODEL_UNAVAILABLE', 'QUOTA_EXCEEDED', 'OAUTH_EXPIRED',
  'INTERNAL_ERROR',
] as const;

export type CanonicalRecoveryCode = typeof canonicalRecoveryCodes[number];

export type RecoveryVariant = { title: string; message: string; action: string };

const variants: Record<CanonicalRecoveryCode, RecoveryVariant> = {
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
  RISK_EVIDENCE_UNAVAILABLE: { title: 'Risk evidence is unavailable', message: 'No order was submitted. Review the saved RiskDecision for missing or untrusted inputs.', action: 'Review risk decision' },
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

const recognizedCodes = new Set<string>(canonicalRecoveryCodes);

export function getRecoveryVariant(code: string): RecoveryVariant {
  return variants[recognizedCodes.has(code) ? code as CanonicalRecoveryCode : 'INTERNAL_ERROR'];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function stringField(value: Record<string, unknown> | undefined, key: string): string | undefined {
  return typeof value?.[key] === 'string' ? value[key] as string : undefined;
}

export function resolveCanonicalRecoveryCode(error: unknown): CanonicalRecoveryCode {
  const envelope = isRecord(error) && isRecord(error.detail) ? error.detail : isRecord(error) ? error : undefined;
  const code = typeof error === 'string' ? error : stringField(envelope, 'code');
  const category = stringField(envelope, 'category');

  if (code && recognizedCodes.has(code)) return code as CanonicalRecoveryCode;
  if (category && recognizedCodes.has(category)) return category as CanonicalRecoveryCode;

  if (code === 'MODEL_OAUTH_EXPIRED' || code === 'MODEL_LOGIN_FAILED' || code === 'MODEL_LOGIN_TIMEOUT') return 'OAUTH_EXPIRED';
  if (code === 'MODEL_QUOTA_EXCEEDED' || code === 'MODEL_QUOTA_COOLDOWN') return 'QUOTA_EXCEEDED';
  if (code?.startsWith('GATEWAY_') || code?.startsWith('MODEL_') || code?.startsWith('CODEX_')) return 'MODEL_UNAVAILABLE';
  if (code === 'PROVIDER_AUTH_FAILED' || code?.startsWith('CREDENTIAL_')) return 'AUTH_ERROR';
  if (code === 'PROVIDER_PERMISSION_BLOCKED' || code === 'ORDER_SUBMIT_FORBIDDEN') return 'PERMISSION_ERROR';
  if (code === 'PROVIDER_RATE_LIMITED' || code === 'PROVIDER_BACKPRESSURE') return 'RATE_LIMITED';
  if (code === 'PROVIDER_UNAVAILABLE' || code === 'DATA_SOURCE_PROBE_FAILED' || code === 'MARKET_HISTORY_UNAVAILABLE') return 'NETWORK_ERROR';
  if (code === 'PROVIDER_UNSUPPORTED' || code === 'UNSUPPORTED_CAPABILITY' || code?.startsWith('ORDER_CAPABILITY_')) return 'UNSUPPORTED_CAPABILITY';
  if (code === 'ORDER_BUYING_POWER_INSUFFICIENT') return 'INSUFFICIENT_FUNDS';
  if (code === 'PROVIDER_ORDER_REJECTED' || code === 'ORDER_SUBMISSION_REJECTED') return 'SUBMISSION_REJECTED';
  if (code === 'ORDER_SUBMISSION_AMBIGUOUS' || code === 'ORDER_SUBMISSION_UNKNOWN') return 'SUBMISSION_AMBIGUOUS';
  if (code === 'ORDER_STATUS_UNKNOWN' || code === 'ORDER_RECONCILIATION_REQUIRED') return 'RECONCILIATION_REQUIRED';
  if (code === 'STATE_VERSION_CONFLICT' || code === 'CLOCK_SKEW' || code === 'IPC_REPLAY_UNAVAILABLE') return 'STATE_STALE';
  if (code === 'RISK_EVIDENCE_UNAVAILABLE' || code === 'RISK_DECISION_UNAVAILABLE') return 'RISK_EVIDENCE_UNAVAILABLE';
  if (code === 'RISK_REJECTED' || code?.startsWith('RISK_')) return 'RISK_REJECTED';
  if (category === 'PROVIDER_ORDER_REJECTED') return 'SUBMISSION_REJECTED';
  if (category === 'POLICY_ERROR') return code?.includes('EVIDENCE') ? 'RISK_EVIDENCE_UNAVAILABLE' : 'RISK_REJECTED';
  if (category === 'RATE_LIMITED') return 'RATE_LIMITED';
  if (category === 'NETWORK_ERROR') return 'NETWORK_ERROR';
  if (category === 'UNSUPPORTED_CAPABILITY') return 'UNSUPPORTED_CAPABILITY';
  return 'INTERNAL_ERROR';
}

export function describeCanonicalRecovery(error: unknown): string {
  const code = resolveCanonicalRecoveryCode(error);
  const variant = getRecoveryVariant(code);
  return `${code} · ${variant.title}. Next step: ${variant.action}`;
}
