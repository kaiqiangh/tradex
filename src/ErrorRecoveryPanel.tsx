import { getRecoveryVariant, getRecoveryVariantForError, resolveCanonicalRecoveryCode } from './errorRecovery.ts';

export function ErrorRecoveryPanel({ code, error, onAction }: { code?: string; error?: unknown; onAction?: () => void }) {
  const recoveryCode = code ?? (error == null ? undefined : resolveCanonicalRecoveryCode(error));
  if (!recoveryCode) return null;
  const canonicalCode = resolveCanonicalRecoveryCode(recoveryCode);
  const variant = code == null && error != null
    ? getRecoveryVariantForError(error)
    : getRecoveryVariant(canonicalCode);
  return <section className="error-recovery-panel" role="alert" aria-live="polite" aria-label={`${canonicalCode} error recovery`} data-error-code={canonicalCode}>
    <div><p className="eyebrow">Error recovery · <code>{canonicalCode}</code></p><h3>{variant.title}</h3><p>{variant.message}</p>
      {!onAction && <p className="error-recovery-next-step">Next step: {variant.action}</p>}
    </div>
    {onAction && <button type="button" onClick={onAction}>{variant.action}</button>}
  </section>;
}
