// Run with `playwright-cli run-code --filename=tests/s33-qa12-narrow-navigation-playwright.js`
// against the Rust-backed `npm run dev:browser` integration harness.
// Uses one disposable workspace and one local Thread; provider actions are never activated.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const command = async (name, payload) => {
    const requestId = `s33-qa12-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name} failed: ${JSON.stringify(result.body?.error)}`);
    return result.body.data;
  };
  const commandLog = async cursor => page.evaluate(async after =>
    (await (await fetch(`/__integration/command-log?after=${after}`)).json()), cursor);
  const pages = [
    ['+ New Thread', 'What would you like to research?'],
    ['Threads', 'Threads'], ['Markets', 'Markets'], ['Watchlists', 'Watchlists'],
    ['Order Drafts', 'Order Drafts'], ['Accounts', 'Account connections'],
    ['Strategies', 'Strategies'], ['Artifacts', 'Artifacts'], ['Settings', 'Settings'],
  ];
  const openMobileNavigation = async () => {
    const details = page.locator('details.mobile-nav');
    const summary = details.locator('summary');
    if (!(await details.evaluate(element => element.open))) await summary.press('Enter');
    check(await details.evaluate(element => element.open), 'More opens with the keyboard.');
    const nav = details.getByRole('navigation', { name: 'Primary navigation', exact: true });
    await nav.waitFor({ state: 'visible' });
    return nav;
  };
  const navigateMobile = async destination => {
    const nav = await openMobileNavigation();
    await nav.getByRole('button', { name: destination, exact: true }).press('Enter');
  };
  const assertSurface = async (width, heading) => {
    try {
      await page.getByRole('heading', { name: heading, exact: true }).first().waitFor({ state: 'visible', timeout: 5000 });
    } catch {
      const visibleState = await page.evaluate(() => ({
        headings: [...document.querySelectorAll('h1,h2,h3')].map(node => node.textContent?.trim()),
        detail: document.querySelector('.thread-detail')?.innerText,
        selectedHistory: document.querySelector('.thread-history-row[aria-current="true"]')?.innerText,
        appWidth: document.documentElement.clientWidth,
      }));
      throw new Error(`Expected ${heading} at ${width}px; observed ${JSON.stringify(visibleState)}`);
    }
    const viewport = await page.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(viewport.scroll <= viewport.width, `${heading} overflows at ${width}px: ${JSON.stringify(viewport)}`);
    const context = page.getByRole('complementary', { name: 'Workspace', exact: true });
    const identity = context.getByText(workspaceName, { exact: true });
    await identity.scrollIntoViewIfNeeded();
    const identityBox = await identity.boundingBox();
    check(identityBox && identityBox.x >= 0 && identityBox.x + identityBox.width <= width
      && identityBox.y >= 0 && identityBox.y + identityBox.height <= 844,
    `Full workspace identity can be brought into view at ${width}px: ${JSON.stringify(identityBox)}`);
    check(await context.getByText('USD', { exact: true }).isVisible(),
      `The workspace base currency remains in its identity panel at ${width}px.`);
    await page.evaluate(() => window.scrollTo(0, 0));
    const workspaceAction = page.getByRole('button', { name: 'Workspace', exact: true });
    const readOnly = page.getByText('Read only', { exact: true });
    check(await workspaceAction.isVisible() && await readOnly.isVisible(),
      `${heading} retains the workspace control and read-only disclosure at ${width}px.`);
    const headerGeometry = await page.evaluate(() => {
      const button = document.querySelector('.workspace-button')?.getBoundingClientRect();
      const badge = document.querySelector('.topbar .badge.readonly')?.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth,
        workspace: button && { left: button.left, right: button.right, top: button.top, bottom: button.bottom },
        readOnly: badge && { left: badge.left, right: badge.right, top: badge.top, bottom: badge.bottom } };
    });
    check(headerGeometry.workspace && headerGeometry.workspace.left >= 0 && headerGeometry.workspace.right <= width
      && headerGeometry.readOnly && headerGeometry.readOnly.left >= 0 && headerGeometry.readOnly.right <= width,
    `Workspace identity control and read-only disclosure fit in the top bar at ${width}px: ${JSON.stringify(headerGeometry)}`);
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 QA12 ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-qa12-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('navigation', { name: 'Primary navigation' })
    .getByRole('button', { name: '+ New Thread', exact: true }).press('Enter');
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill('S33 QA-12 mobile history');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'S33 QA-12 mobile history', exact: true }).waitFor({ state: 'visible' });
  let cursor = (await commandLog(0)).cursor;

  const observed = [];
  let providerActionsVisible = false;
  let providerActionsUnobscured = false;
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await navigateMobile('Threads');
    await assertSurface(width, 'Threads');
    const nav = await openMobileNavigation();
    for (const [destination] of pages) {
      const button = nav.getByRole('button', { name: destination, exact: true });
      check(await button.count() === 1 && await button.isVisible() && await button.isEnabled(),
        `${destination} is reachable from More at ${width}px.`);
    }
    check(await nav.getByRole('button', { name: 'Threads', exact: true }).getAttribute('aria-current') === 'page',
      `The current Threads destination is identified at ${width}px.`);
    observed.push({ width, destinations: pages.length, surfaceOverflow: false, headerIdentity: true });

    await nav.getByRole('button', { name: '+ New Thread', exact: true }).press('Enter');
    await assertSurface(width, 'What would you like to research?');
    check(await page.getByRole('button', { name: 'Create Thread', exact: true }).isVisible(),
      `New Thread creation remains reachable at ${width}px.`);

    await navigateMobile('Threads');
    await assertSurface(width, 'Threads');
    const history = page.getByRole('heading', { name: 'History', exact: true });
    check(await history.isVisible(), `Thread history is visible at ${width}px.`);
    const historyEntry = page.getByRole('button', { name: 'S33 QA-12 mobile history', exact: true });
    check(await historyEntry.isVisible() && await historyEntry.isEnabled(),
      `The saved Thread can be resumed at ${width}px.`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/qa12-mobile-thread-history-390.png' });
    await historyEntry.press('Enter');
    check(await historyEntry.getAttribute('aria-current') === 'true',
      `Enter resumes the saved Thread at ${width}px.`);
    const detail = page.locator('.thread-detail');
    for (let attempt = 0; attempt < 100 && !(await detail.count()); attempt += 1) await page.waitForTimeout(50);
    const resumeEvidence = await page.evaluate(() => ({
      headings: [...document.querySelectorAll('h1,h2,h3')].map(heading => heading.textContent?.trim()),
      detailText: document.querySelector('.thread-detail')?.innerText,
      alertText: [...document.querySelectorAll('[role="alert"], [role="status"]')].map(node => node.textContent?.trim()),
    }));
    check(await detail.count() === 1 && await detail.isVisible()
      && await page.getByRole('heading', { name: 'S33 QA-12 mobile history', exact: true }).isVisible(),
    `Resuming history should render the selected Thread at ${width}px: ${JSON.stringify(resumeEvidence)}`);

    for (const [destination, heading] of pages.slice(2, 8)) {
      await navigateMobile(destination);
      await assertSurface(width, heading);
    }
    await navigateMobile('Settings');
    await assertSurface(width, 'Settings');
    for (const tab of ['Providers & Models', 'Risk & Limits', 'Data & Storage', 'Account Health', 'Appearance', 'About']) {
      const tabButton = page.getByRole('button', { name: tab, exact: true });
      check(await tabButton.isVisible() && await tabButton.isEnabled(),
        `${tab} is reachable from Settings at ${width}px.`);
      await tabButton.press('Enter');
      await page.getByRole('heading', { name: tab, exact: true }).waitFor({ state: 'visible' });
      await assertSurface(width, 'Settings');
      if (tab === 'Providers & Models') {
        const models = page.locator('.model-settings');
        await models.waitFor({ state: 'visible' });
        await models.getByRole('button', { name: /^(Install & Launch|Launch)$/ }).waitFor({ state: 'visible' });
        const requiredProviderActions = [
          models.getByRole('button', { name: /^(Install & Launch|Launch)$/ }),
          models.getByRole('button', { name: 'Probe', exact: true }),
          models.getByRole('button', { name: 'Restart', exact: true }),
          models.getByRole('button', { name: 'Stop', exact: true }),
          models.getByRole('button', { name: /^(Login ChatGPT|Re-login ChatGPT)$/ }),
          models.getByRole('button', { name: /^(Configure DeepSeek key|Replace DeepSeek key)$/ }),
        ];
        for (const action of requiredProviderActions) {
          check(await action.count() === 1 && await action.isVisible(),
            `Provider setup action ${await action.innerText().catch(() => 'unknown')} stays visible at ${width}px.`);
        }
        let actionGeometry = await models.locator('.model-actions button, .model-provider-actions button').evaluateAll(nodes =>
          nodes.map(node => {
            const rect = node.getBoundingClientRect();
            const style = getComputedStyle(node);
            return { label: node.innerText.trim(), display: style.display, visibility: style.visibility,
              left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom,
              width: innerWidth, rendered: node.getClientRects().length > 0 };
          }));
        check(actionGeometry.length >= requiredProviderActions.length && actionGeometry.every(action =>
          action.display !== 'none' && action.visibility !== 'hidden' && action.rendered
          && action.left >= 0 && action.right <= action.width),
        `Provider setup actions are rendered inside ${width}px: ${JSON.stringify(actionGeometry)}`);
        if (width === 390) {
          providerActionsVisible = true;
          for (const action of requiredProviderActions) {
            await action.scrollIntoViewIfNeeded();
            const geometry = await action.evaluate(node => {
              const rect = node.getBoundingClientRect();
              const topbarBottom = document.querySelector('.topbar')?.getBoundingClientRect().bottom ?? 0;
              return { label: node.innerText.trim(), top: rect.top, bottom: rect.bottom,
                topbarBottom, viewportHeight: innerHeight };
            });
            check(geometry.top >= geometry.topbarBottom && geometry.bottom <= geometry.viewportHeight,
              `Provider action ${geometry.label} is reachable below the sticky top bar at 390px: ${JSON.stringify(geometry)}`);
          }
          await models.locator('.model-actions').scrollIntoViewIfNeeded();
          // Keep the action row below the sticky top bar in the captured viewport.
          await page.evaluate(() => window.scrollBy(0, -80));
          const topbarBottom = await page.locator('.topbar').evaluate(node => node.getBoundingClientRect().bottom);
          const primaryActionGeometry = await models.locator('.model-actions button').evaluateAll(nodes =>
            nodes.map(node => {
              const rect = node.getBoundingClientRect();
              return { label: node.innerText.trim(), top: rect.top, bottom: rect.bottom, viewportHeight: innerHeight };
            }));
          check(primaryActionGeometry.length === 4 && primaryActionGeometry.every(action => action.top >= topbarBottom
            && action.bottom <= action.viewportHeight),
          `Primary Provider actions remain unobscured at 390px: ${JSON.stringify({ topbarBottom, primaryActionGeometry })}`);
          providerActionsUnobscured = true;
          await page.screenshot({ path: 'docs/implementation/evidence/s33/qa12-provider-actions-390.png' });
        }
      }
    }
    observed.push({ width, newThread: true, historyResume: true, secondaryPages: 6, settingsSections: 6 });
  }

  const finalLog = await commandLog(cursor);
  const forbidden = finalLog.commands.filter(entry => (entry.command.startsWith('provider.')
    && entry.command !== 'provider.list_definitions')
    || entry.command.startsWith('alpaca.') || entry.command.startsWith('binance.')
    || entry.command.startsWith('bitget.') || entry.command.startsWith('trading212.')
    || entry.command === 'account.refresh' || entry.command.startsWith('model.verify')
    || entry.command.startsWith('trade.execution.') || entry.command.startsWith('live.gateway.')
    || entry.command.startsWith('trade.approval.') || entry.command.startsWith('trade.prepare'));
  check(forbidden.length === 0, `Navigation and resume issued no provider or financial command: ${JSON.stringify(forbidden)}`);
  return { result: 'PASS', boundary: 'one disposable workspace and local Thread; no provider action',
    widths: [768, 390], navigationDestinations: pages.length, newThread: true, historyResume: true,
    settingsSections: 6, zeroDocumentOverflow: true, forbiddenCommands: forbidden.length,
    providerActionsVisible, providerActionsUnobscured, observed };
}
