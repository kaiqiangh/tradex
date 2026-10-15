// Run against the isolated Rust-backed browser harness on port 1423.
// Appearance is a read-only prototype surface; this test activates no theme,
// provider, model, or financial action.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-appearance-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1423/');
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 appearance ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-appearance-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
  const cursor = (await commandLog(0)).cursor;
  const appearanceTab = tabs.getByRole('button', { name: 'Appearance', exact: true });
  await appearanceTab.press('Enter');
  await page.getByRole('heading', { name: 'Appearance', exact: true }).waitFor({ state: 'visible' });
  check(await appearanceTab.getAttribute('aria-pressed') === 'true', 'The Appearance section is selected through keyboard Enter.');

  const surface = page.locator('.appearance-settings');
  await page.getByRole('heading', { name: 'Theme', exact: true }).waitFor({ state: 'visible' });
  await page.getByText('Prototype preference surface. TradeX currently uses the RevC light theme.', { exact: true }).waitFor({ state: 'visible' });
  const group = page.getByRole('group', { name: 'Theme preference', exact: true });
  check(await group.locator('.appearance-option').count() === 2, 'The prototype presents exactly the current and future theme choices.');
  const current = group.locator('.appearance-option.selected');
  check(await current.getAttribute('aria-current') === 'true'
    && await current.getByText('System / Light prototype', { exact: true }).isVisible()
    && await current.getByText('Current', { exact: true }).isVisible(),
  'System / Light is truthfully marked as the current prototype surface.');
  const future = group.locator('.appearance-option.future');
  check(await future.getAttribute('aria-disabled') === 'true'
    && await future.getByText('Dark workspace', { exact: true }).isVisible()
    && await future.getByText('Future', { exact: true }).isVisible(),
  'Dark workspace is exposed as a disabled future surface.');
  check(await surface.locator('button, input, select, textarea, [role="button"]').count() === 0,
    'The prototype preference display provides no active theme control.');

  await page.emulateMedia({ colorScheme: 'dark' });
  const darkPreference = await page.evaluate(() => ({
    prefersDark: matchMedia('(prefers-color-scheme: dark)').matches,
    colorScheme: getComputedStyle(document.documentElement).colorScheme,
  }));
  check(darkPreference.prefersDark && darkPreference.colorScheme === 'light',
    `The prototype remains on the documented RevC light theme under a dark system preference: ${JSON.stringify(darkPreference)}`);
  check(await current.getAttribute('aria-current') === 'true' && await future.getAttribute('aria-disabled') === 'true',
    'Changing the emulated system preference does not imply a persisted theme selection.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => {
      const root = document.documentElement;
      const surface = document.querySelector('.appearance-settings');
      const options = document.querySelector('.appearance-options');
      const surfaceBox = surface?.getBoundingClientRect();
      const optionsBox = options?.getBoundingClientRect();
      return {
        viewport: root.clientWidth, documentScroll: root.scrollWidth,
        surfaceLeft: surfaceBox?.left, surfaceRight: surfaceBox?.right,
        optionsLeft: optionsBox?.left, optionsRight: optionsBox?.right,
      };
    });
    check(geometry.documentScroll <= geometry.viewport
      && geometry.surfaceLeft >= -1 && geometry.surfaceRight <= width + 1
      && geometry.optionsLeft >= -1 && geometry.optionsRight <= width + 1,
    `Appearance stays within ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await surface.screenshot({ path: 'docs/implementation/evidence/s33/appearance-390.png' });
    }
    responsive.push({ width, noHorizontalOverflow: true, currentAndFutureLabelsVisible: true });
  }

  const finalLog = await commandLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const forbidden = commands.filter(name => name.startsWith('provider.') || name === 'account.refresh'
    || name === 'model.verify_route' || name.startsWith('data.') || name.startsWith('market.hot.')
    || name.startsWith('trade.') || name.startsWith('paper.order.') || name.startsWith('order.')
    || name.startsWith('gateway.') || name.startsWith('live.gateway.'));
  check(forbidden.length === 0, `Appearance opened without provider/model/financial actions: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenId: 'J6', workspace: workspace.workspaceId, darkPreference,
    currentTheme: 'System / Light prototype', futureTheme: 'Dark workspace', responsive,
    commands, forbiddenCommands: forbidden, externalHttpRequests: externalRequests.length,
    consoleErrors, pageErrors, themeControlActivated: false,
  };
}
