// Run with the Codex CUA binding against an isolated integration Vite server.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

async function send(endpoint, command, payload) {
  const response = await fetch(endpoint, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }),
  });
  assert.equal(response.status, 200);
  const result = await response.json();
  assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
  return result.data;
}

export async function seedS27ModelRecoveryUI(endpoint) {
  const workspace = await send(endpoint, 'workspace.open', { name: 'S27 model recovery', baseCurrency: 'USD' });
  const workspaceId = workspace.workspaceId;
  await send(endpoint, 'workspace.ready.fixture', { workspaceId });
  const accounts = [];
  for (const providerId of ['binance', 'trading212']) {
    accounts.push(await send(endpoint, 'account.arming.fixture.seed', { workspaceId, providerId, label: `S27 ${providerId} ${randomUUID().slice(0,8)}` }));
  }
  return { workspaceId, accounts };
}

export async function setS27ModelRecoveryUI(endpoint, fixture, status) {
  const payload = { workspaceId: fixture.workspaceId };
  const before = (await send(endpoint, 'account.list', payload)).accounts;
  const modelBefore = await send(endpoint, 'model.get', payload);
  await send(endpoint, 'model.gateway.fixture', { ...payload, status });
  assert.deepEqual((await send(endpoint, 'account.list', payload)).accounts, before);
  const model = await send(endpoint, 'model.get', payload);
  assert.deepEqual(model.defaultRoute, modelBefore.defaultRoute);
  assert.equal(model.automaticFallback, false);
}

export async function checkS27ModelRecoveryUI(tab, browser, { accounts }, status) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  async function navigate(name) {
    const button = ui.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name, exact: true });
    if (!(await button.isVisible())) await ui.getByText('More', { exact: true }).press('Enter');
    await button.press('Enter');
    await tab.getAXState({ emit: false });
  }
  async function noOverflow(width) {
    const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    assert.ok(size.width <= width && size.width >= width - 20, JSON.stringify(size));
    assert.ok(size.scroll <= size.width, JSON.stringify(size));
  }
  try {
    const [label, remediation] = {
      STOPPED: ['Stopped', 'The selected model route is unavailable. Retry its probe or choose another route.'],
      UNAUTHORIZED: ['Unauthorized', 'The owned gateway rejected authentication. Restart to regenerate its local configuration.'],
      PORT_CONFLICT: ['Port conflict', 'Port 8317 is occupied. Close or reconfigure that service, then launch again.'],
      RUNNING: ['Running', null],
    }[status];
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await navigate('Settings');
      await ui.getByRole('button', { name: 'Providers & Models', exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      await ui.locator('.model-settings').getByRole('status').filter({ hasText: label }).first().waitFor({ state: 'visible' });
      if (remediation) assert.equal(await ui.getByText(remediation, { exact: true }).isVisible(), true);
      assert.equal(await ui.getByRole('button', { name: 'Restart', exact: true }).isEnabled(), true);
      await ui.getByRole('button', { name: 'Reload model state', exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      await noOverflow(width);
      await ui.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      for (const account of accounts) {
        const row = ui.locator('.account-row').filter({ hasText: account.label });
        assert.match(await row.innerText(), /CONNECTED · ONLINE/);
        assert.match(await row.innerText(), /Arming: DISARMED/);
        await row.press('Enter');
        await tab.getAXState({ emit: false });
        await ui.getByRole('heading', { name: account.label, exact: true }).waitFor({ state: 'visible' });
        await noOverflow(width);
      }
    }
    observed.push(`${status}: model remediation and Account Health work with Enter at 1280/768/390px.`);
    const errors = await tab.dev.logs({ levels: ['error'], limit: 100 });
    assert.equal(errors.filter(log => !log.message.includes('chrome-extension://') && !log.url?.startsWith('chrome-extension://')).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
