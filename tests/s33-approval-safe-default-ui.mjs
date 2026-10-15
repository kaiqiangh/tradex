// Run with a Codex CUA browser binding against `npm run dev:browser`.
// This regression uses only a disposable local fixture account; it does not connect a provider.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';

async function command(tab, name, payload) {
  const endpoint = new URL('/__integration/command', await tab.url());
  const requestId = randomUUID();
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Origin: endpoint.origin },
    body: JSON.stringify({ requestId, schemaVersion: 1, command: name, payload }),
  });
  assert.equal(response.status, 200, `${name} is available in the local integration bridge`);
  const result = await response.json();
  assert.equal(result.requestId, requestId);
  assert.equal(result.ok, true, `${name}: ${JSON.stringify(result.error)}`);
  return result.data;
}

export async function checkS33ApprovalSafeDefaultUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  try {
    await viewport.set({ width: 1280, height: 900 });
    const bootstrap = await command(tab, 'workspace.open', {});
    const workspaceName = 'S33 Approval Focus';
    const workspacePath = join(dirname(bootstrap.path), `s33-approval-focus-${randomUUID()}`);
    const workspace = await command(tab, 'workspace.open', {
      name: workspaceName, baseCurrency: 'USD', path: workspacePath,
    });
    await command(tab, 'workspace.ready.fixture', { workspaceId: workspace.workspaceId });
    if (!(await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible())) {
      await ui.getByRole('button', { name: 'Workspace', exact: true }).press('Enter');
    }
    await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
    await ui.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
    await ui.getByLabel('Local storage', { exact: true }).fill(workspacePath);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

    const workspaceId = workspace.workspaceId;
    const risk = await command(tab, 'risk.get_policy', { workspaceId });
    await command(tab, 'risk.save_policy', {
      workspaceId,
      expectedStateVersion: risk.stateVersion,
      policy: {
        ...risk.policy,
        marketOrdersEnabled: true,
        maxMarketOrderSlippagePercent: '5',
        staleQuoteThresholdSeconds: 120,
        maxSingleInstrumentExposurePercent: null,
      },
    });
    const account = await command(tab, 'account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: 'S33 local synthetic account',
    });
    await command(tab, 'time.revalidate', { workspaceId });
    await command(tab, 'account.arm', {
      workspaceId, connectionId: account.connectionId,
      expectedStateVersion: account.stateVersion, confirmed: true,
    });

    const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).first();
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('510');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const proposals = await command(tab, 'trade.proposal.list', { workspaceId });
    assert.equal(proposals.proposals.length, 1);
    await ui.locator('.order-proposal-row').first().press('Enter');

    const reviewButton = ui.getByRole('button', { name: 'Review Live approval', exact: true });
    await reviewButton.press('Enter');
    const dialog = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await dialog.waitFor({ state: 'visible' });
    const proposal = proposals.proposals[0];
    const focusedControl = await ui.evaluate(() => document.activeElement?.textContent?.trim());
    assert.equal(focusedControl, 'Keep reviewing', 'Approval dialog opens on the neutral dismissal action');
    assert.equal(await dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).isEnabled(), false,
      'Current RevC evidence gates remain authoritative');
    const beforeEnter = await command(tab, 'trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
    assert.equal(beforeEnter.approvals.length, 0, 'Opening the blocked review creates no approval');

    const bridgeOrigin = new URL(await tab.url()).origin;
    const logBeforeDismiss = await (await fetch(`${bridgeOrigin}/__integration/command-log?after=0`)).json();
    await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
    await dialog.waitFor({ state: 'hidden' });
    const afterEnter = await command(tab, 'trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
    assert.equal(afterEnter.approvals.length, 0, 'Enter on the safe control dismisses without approval or rejection');
    const later = await (await fetch(`${bridgeOrigin}/__integration/command-log?after=${logBeforeDismiss.cursor}`)).json();
    const sideEffects = later.commands.filter(entry => /^(provider\.(connect|probe)|account\.refresh|trade\.execution\.|live\.gateway\.fixture\.)/.test(entry.command));
    assert.equal(sideEffects.length, 0, `Safe dismissal issues no provider, gateway, or execution command: ${JSON.stringify(later.commands)}`);
    return [
      'A blocked synthetic Live approval review opens with focus on Keep reviewing while approval remains disabled by RevC gates.',
      'Enter dismisses the review with no approval, rejection, provider, fake-gateway, or execution command.',
    ];
  } finally {
    await viewport.reset();
  }
}
