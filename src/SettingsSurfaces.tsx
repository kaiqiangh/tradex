import { getVersion } from '@tauri-apps/api/app';
import { useEffect, useState } from 'react';
import type { RuntimeStatus } from '../shared/ipc-types.ts';
import tauriConfig from '../src-tauri/tauri.conf.json';
import { desktop } from './client.ts';
import { ErrorRecoveryPanel } from './ErrorRecoveryPanel.tsx';

export function AppearanceSettings() {
  return <section className="appearance-settings" aria-labelledby="appearance-title">
    <div className="page-heading"><div><h3 id="appearance-title">Theme</h3><p className="muted">Prototype preference surface. TradeX currently uses the RevC light theme.</p></div></div>
    <div className="appearance-options" role="group" aria-label="Theme preference">
      <div className="appearance-option selected" aria-current="true"><strong>System / Light prototype</strong><span>Current</span></div>
      <div className="appearance-option future" aria-disabled="true"><strong>Dark workspace</strong><span>Future</span></div>
    </div>
  </section>;
}

export function AboutPrivacySettings({
  runtime,
  runtimeLoading,
  runtimeError,
  onRefresh,
}: {
  runtime?: RuntimeStatus;
  runtimeLoading: boolean;
  runtimeError: unknown;
  onRefresh: () => void;
}) {
  const [version, setVersion] = useState<string>(tauriConfig.version);

  useEffect(() => {
    if (!desktop) return;
    let active = true;
    void getVersion().then(current => { if (active) setVersion(current); }).catch(() => {});
    return () => { active = false; };
  }, []);

  return <div className="about-settings">
    <section className="card" aria-labelledby="about-title">
      <h3 id="about-title">About TradeX</h3>
      <dl className="about-facts">
        <div><dt>Application version</dt><dd>{version}</dd></div>
        <div><dt>Application</dt><dd>Native TradeX desktop workspace</dd></div>
        <div><dt>Documentation baseline</dt><dd>TradeX v1.0 · Revision C</dd></div>
        <div><dt>Prototype</dt><dd>Historical clickable fixture; it is not the running application.</dd></div>
      </dl>
    </section>
    <section className="card" aria-labelledby="runtime-title">
      <div className="about-section-heading"><h3 id="runtime-title">Local runtime</h3><button type="button" onClick={onRefresh} disabled={runtimeLoading}>Refresh runtime status</button></div>
      {runtimeLoading && !runtime ? <p role="status">Checking local runtime…</p> : runtimeError ?
        <ErrorRecoveryPanel error={runtimeError} /> :
        <ul className="component-list">{(runtime?.components ?? []).map(component => <li key={component.id}>
          <div><strong>{component.id === 'cliproxyapi' ? 'CLIProxyAPI' : component.id === 'codex' ? 'Codex App Server' : component.id === 'control-plane' ? 'Control Plane' : component.id === 'order-gateway' ? 'Order Gateway' : component.id}</strong><p>{component.message}</p></div>
          <span className="badge">{component.status.replaceAll('_', ' ')}</span>
        </li>)}</ul>}
      {!runtimeLoading && !runtimeError && !runtime?.components.length && <p className="muted">No local runtime status is available.</p>}
    </section>
    <section className="card privacy-status" aria-labelledby="privacy-title">
      <h3 id="privacy-title">Privacy, updates &amp; crash reporting</h3>
      <dl className="privacy-facts">
        <div><dt>External telemetry</dt><dd><span className="badge">Disabled</span><p>No telemetry collector is configured, and TradeX sends no diagnostic telemetry.</p></dd></div>
        <div><dt>Crash reporting</dt><dd><span className="badge">Not configured</span><p>This build does not create or upload crash reports.</p></dd></div>
        <div><dt>Automatic updates</dt><dd><span className="badge">Not configured</span><p>This build has no automatic update service.</p></dd></div>
      </dl>
      <p className="muted">Any future external telemetry requires a real collector and your explicit opt-in before data is sent.</p>
    </section>
  </div>;
}
