// Open the isolated local integration Vite server on port 1422 first, then run with
// `playwright-cli run-code --filename=tests/s33-keyboard-focus-order-playwright.js`.
// Uses the real React app, a disposable Rust-backed workspace, and passive synthetic state only.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const command = async (name, payload) => {
    const requestId = `s33-focus-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name}: ${JSON.stringify(result.body?.error)}`);
    return result.body.data;
  };
  const readLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);
  const forbidden = new Set([
    'provider.connect', 'provider.probe', 'provider.permissions', 'provider.disconnect', 'account.refresh',
    'model.verify_route', 'data.source.probe', 'data.source.disconnect',
    'alpaca.paper.orders.refresh', 'trading212.demo.orders.refresh', 'binance.testnet.orders.refresh',
    'trade.live_order.refresh', 'trade.resolution_evidence.refresh', 'trade.spot_capacity.refresh',
    'trade.spot_commission.refresh', 'trade.spot_order_intervals.refresh', 'trade.spot_rules.refresh',
    'data.fx.refresh', 'data.fx.disconnect', 'data.calendar.refresh', 'data.calendar.disconnect',
    'data.instrument.refresh', 'data.instrument.disconnect', 'data.actions.refresh', 'data.actions.disconnect',
    'data.binance_rules.refresh', 'data.binance_rules.disconnect', 'data.binance_market.disconnect',
    'paper.quote.refresh', 'market.hot.acquire', 'market.hot.release', 'market.screen',
  ]);
  const auditKeyboardSequence = async (surface, width) => {
    const visibleClosedDialogs = await page.evaluate(() => [...document.querySelectorAll('dialog:not([open])')]
      .filter(dialog => getComputedStyle(dialog).display !== 'none' && dialog.getClientRects().length > 0)
      .map(dialog => ({ label: dialog.getAttribute('aria-labelledby'), text: dialog.innerText })));
    check(visibleClosedDialogs.length === 0,
      `${surface} at ${width}px renders a closed native dialog: ${JSON.stringify(visibleClosedDialogs)}.`);
    const expected = await page.evaluate(() => {
      const selector = 'a[href],button,input,select,textarea,summary,[tabindex]';
      const candidates = [...document.querySelectorAll(selector)].map((element, domIndex) => ({ element, domIndex }))
        .filter(({ element }) => {
          const style = getComputedStyle(element);
          const closedDisclosure = element.closest('details:not([open])');
          return element.tabIndex >= 0 && !element.matches(':disabled')
            && !element.closest('[inert],[aria-hidden="true"]')
            && (!closedDisclosure || element.matches('summary'))
            && style.display !== 'none' && style.visibility !== 'hidden'
            && element.getClientRects().length > 0;
        });
      candidates.sort((left, right) => {
        const leftIndex = left.element.tabIndex;
        const rightIndex = right.element.tabIndex;
        if (leftIndex > 0 || rightIndex > 0) {
          if (leftIndex === 0) return 1;
          if (rightIndex === 0) return -1;
          if (leftIndex !== rightIndex) return leftIndex - rightIndex;
        }
        return left.domIndex - right.domIndex;
      });
      candidates.forEach(({ element }, index) => element.setAttribute('data-s33-focus-audit', String(index)));
      return candidates.map(({ element }) => ({
        index: element.getAttribute('data-s33-focus-audit'),
        tag: element.tagName,
        type: element.getAttribute('type'),
        name: element.getAttribute('aria-label') || element.getAttribute('title') || element.innerText?.trim() || element.getAttribute('name') || '',
      }));
    });
    check(expected.length > 0, `${surface} at ${width}px has at least one visible keyboard control.`);
    const visualOrder = await page.evaluate(() => {
      const read = (name, selector, compare) => {
        const elements = [...document.querySelectorAll(selector)]
          .filter(element => element.hasAttribute('data-s33-focus-audit'));
        const domOrder = elements.map(element => Number(element.getAttribute('data-s33-focus-audit')));
        const visualElements = [...elements].sort(compare);
        const visualOrder = visualElements.map(element => Number(element.getAttribute('data-s33-focus-audit')));
        const describe = element => {
          const rect = element.getBoundingClientRect();
          return { index: Number(element.getAttribute('data-s33-focus-audit')),
            label: element.getAttribute('aria-label') || element.title || element.innerText?.trim() || element.tagName,
            top: rect.top, left: rect.left };
        };
        const matches = JSON.stringify(domOrder) === JSON.stringify(visualOrder);
        return { name, count: elements.length, matches,
          ...(matches ? {} : { domOrder: elements.map(describe), visualOrder: visualElements.map(describe) }) };
      };
      const fieldLabels = new Map();
      for (const label of document.querySelectorAll('label')) {
        if (label.control && !fieldLabels.has(label.control)) fieldLabels.set(label.control, label);
      }
      const top = (left, right, includeFieldLabel = false) => {
        const readingRect = element => {
          const controlRect = element.getBoundingClientRect();
          const label = includeFieldLabel && element.matches('input,select,textarea')
            ? element.closest('label') ?? fieldLabels.get(element)
            : null;
          if (!label) return controlRect;
          const labelRect = label.getBoundingClientRect();
          if (!labelRect.width && !labelRect.height) return controlRect;
          const top = Math.min(labelRect.top, controlRect.top);
          const bottom = Math.max(labelRect.bottom, controlRect.bottom);
          return { top, bottom, left: Math.min(labelRect.left, controlRect.left), height: bottom - top };
        };
        const a = readingRect(left); const b = readingRect(right);
        const verticalOverlap = Math.max(0, Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top));
        const smallerHeight = Math.min(a.height, b.height);
        if (smallerHeight > 0 && verticalOverlap / smallerHeight >= 0.5) return a.left - b.left;
        return Math.abs(a.top - b.top) > 2 ? a.top - b.top : a.left - b.left;
      };
      const left = (a, b) => a.getBoundingClientRect().left - b.getBoundingClientRect().left;
      const groups = [
        read('Primary navigation (top to bottom)', 'nav[aria-label="Primary navigation"] [data-s33-focus-audit]', top),
        read('Top bar controls (left to right)', '.topbar > details.mobile-nav > summary, .topbar > button[data-s33-focus-audit]', left),
      ].filter(group => group.count > 0);
      const content = document.querySelector('.content');
      if (!content) return groups;
      groups.push(read('Main content overall visual reading order', '.content [data-s33-focus-audit]',
        (left, right) => top(left, right, true)));
      const contentControls = [...content.querySelectorAll('[data-s33-focus-audit]')];
      const groupsByContainer = new Map();
      for (const element of contentControls) {
        let container = element.closest('form, section, article, [role="group"], .page-heading');
        if (!container || container === content) {
          container = [...content.children].find(child => child === element || child.contains(element)) ?? content;
        }
        groupsByContainer.set(container, [...(groupsByContainer.get(container) ?? []), element]);
      }
      const describe = element => {
        const rect = element.getBoundingClientRect();
        return { index: Number(element.getAttribute('data-s33-focus-audit')),
          label: element.getAttribute('aria-label') || element.title || element.innerText?.trim() || element.tagName,
          top: rect.top, left: rect.left };
      };
      let groupIndex = 0;
      for (const [container, elements] of groupsByContainer) {
        const domOrder = elements.map(element => Number(element.getAttribute('data-s33-focus-audit')));
        const visualElements = [...elements].sort((a, b) => top(a, b, true));
        const visual = visualElements.map(element => Number(element.getAttribute('data-s33-focus-audit')));
        const name = container.getAttribute('aria-label')
          || document.getElementById(container.getAttribute('aria-labelledby'))?.innerText?.trim()
          || container.querySelector('h1,h2,h3,legend')?.innerText?.trim()
          || container.className?.toString()
          || container.tagName;
        const matches = JSON.stringify(domOrder) === JSON.stringify(visual);
        groups.push({ name: `Main content group ${++groupIndex}: ${name}`, count: elements.length, matches,
          ...(matches ? {} : { domOrder: elements.map(describe), visualOrder: visualElements.map(describe) }) });
      }
      return groups;
    });
    const visualOrderFailures = visualOrder.filter(group => !group.matches);
    check(visualOrderFailures.length === 0,
      `${surface} at ${width}px has a focus sequence that diverges from a measured visual reading group: ${JSON.stringify(visualOrderFailures)}.`);
    check(visualOrder.some(group => group.name === 'Main content overall visual reading order'),
      `${surface} at ${width}px has an auditable main-content reading sequence.`);
    if (surface === 'Mobile navigation drawer') {
      const primaryNavigation = visualOrder.find(group => group.name === 'Primary navigation (top to bottom)');
      check(primaryNavigation?.count === 9,
        `Mobile navigation at ${width}px must expose all nine enabled destinations: ${JSON.stringify(primaryNavigation)}.`);
    }

    await page.evaluate(() => {
      const body = document.body;
      body.setAttribute('tabindex', '-1');
      body.focus();
      body.removeAttribute('tabindex');
    });
    const observed = [];
    let previousIndex = null;
    let previousType = null;
    for (let index = 0; index < expected.length; index += 1) {
      let state;
      let reached = false;
      const maxNativeSegmentTabs = previousType === 'datetime-local' ? 8 : 1;
      for (let attempt = 0; attempt < maxNativeSegmentTabs; attempt += 1) {
        await page.keyboard.press('Tab');
        state = await page.evaluate(() => {
          const active = document.activeElement;
          const style = active ? getComputedStyle(active) : undefined;
          const rect = active?.getBoundingClientRect();
          const visibleWidth = rect ? Math.max(0, Math.min(rect.right, innerWidth) - Math.max(rect.left, 0)) : 0;
          const visibleHeight = rect ? Math.max(0, Math.min(rect.bottom, innerHeight) - Math.max(rect.top, 0)) : 0;
          return {
            index: active?.getAttribute('data-s33-focus-audit') ?? null,
            focusVisible: active?.matches(':focus-visible') ?? false,
            outlineStyle: style?.outlineStyle ?? 'none',
            outlineWidth: Number.parseFloat(style?.outlineWidth ?? '0'),
            left: rect?.left ?? -1,
            right: rect?.right ?? -1,
            top: rect?.top ?? -1,
            bottom: rect?.bottom ?? -1,
            width: document.documentElement.clientWidth,
            height: document.documentElement.clientHeight,
            visibleFraction: rect && rect.width > 0 && rect.height > 0
              ? (visibleWidth * visibleHeight) / (rect.width * rect.height) : 0,
          };
        });
        if (state.index === expected[index].index) { reached = true; break; }
        check(previousType === 'datetime-local' && state.index === previousIndex,
          `${surface} at ${width}px Tab moved outside the native date/time control sequence: ${JSON.stringify(state)}.`);
      }
      check(reached,
        `${surface} at ${width}px keyboard Tab did not reach ${JSON.stringify(expected[index])}: ${JSON.stringify(state)}.`);
      check(state.focusVisible && state.outlineStyle !== 'none' && state.outlineWidth >= 2,
        `${surface} at ${width}px control ${index + 1} lacks a visible keyboard focus indicator: ${JSON.stringify(state)}.`);
      check(state.visibleFraction >= 0.75,
        `${surface} at ${width}px focused control ${index + 1} is not sufficiently visible: ${JSON.stringify(state)}.`);
      observed.push(state.index);
      previousIndex = state.index;
      previousType = expected[index].type;
    }
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
    }));
    check(geometry.scroll <= geometry.viewport, `${surface} at ${width}px has document overflow: ${JSON.stringify(geometry)}.`);
    await page.evaluate(() => document.querySelectorAll('[data-s33-focus-audit]').forEach(element => element.removeAttribute('data-s33-focus-audit')));
    return { surface, width, focusableControls: observed.length, visualOrderGroups: visualOrder.length,
      visualOrderControls: visualOrder.reduce((total, item) => total + item.count, 0),
      mainContentGeometryGroups: visualOrder.filter(item => item.name.startsWith('Main content group ')).length,
      mainContentGeometryControls: visualOrder.filter(item => item.name.startsWith('Main content group '))
        .reduce((total, item) => total + item.count, 0),
      documentOverflow: geometry.scroll - geometry.viewport };
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  const localOrigin = page.url().match(/^(https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?)/)?.[1];
  check(localOrigin, `Refusing to run outside a local integration browser: ${page.url()}.`);
  check(localOrigin.endsWith(':1422'), `Refusing to run outside isolated integration port 1422: ${localOrigin}.`);
  await page.goto(`${localOrigin}/`);
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 keyboard focus ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-focus-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const pages = [
    ['+ New Thread', 'What would you like to research?'],
    ['Threads', 'Threads'], ['Markets', 'Markets'], ['Watchlists', 'Watchlists'],
    ['Order Drafts', 'Order Drafts'], ['Accounts', 'Account connections'],
    ['Strategies', 'Strategies'], ['Artifacts', 'Artifacts'], ['Settings', 'Settings'],
  ];
  const settingsSections = ['Providers & Models', 'Risk & Limits', 'Data & Storage', 'Account Health', 'Appearance', 'About'];
  const widths = [1280, 768, 390];
  const observations = [];
  let mobileNavigationAuditCount = 0;
  let cursor = (await readLog(0)).cursor;
  let forbiddenCommandCount = 0;
  const assertNoForbiddenCommands = async (surface, width) => {
    const log = await readLog(cursor);
    const hits = log.commands.filter(entry => forbidden.has(entry.command));
    forbiddenCommandCount += hits.length;
    check(hits.length === 0,
      `${surface} at ${width}px issued forbidden provider/model/data commands: ${JSON.stringify(hits)}.`);
    cursor = log.cursor;
  };
  for (const width of widths) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    if (width < 900) {
      const mobile = page.locator('details.mobile-nav');
      if (await mobile.count() && !(await mobile.evaluate(element => element.open))) {
        await page.getByText('More', { exact: true }).press('Enter');
      }
      if (width === 390) {
        await page.locator('.mobile-nav nav[aria-label="Primary navigation"]')
          .screenshot({ path: 'docs/implementation/evidence/s33/keyboard-reading-order-mobile-nav-390.png' });
      }
      observations.push(await auditKeyboardSequence('Mobile navigation drawer', width));
      mobileNavigationAuditCount += 1;
      await assertNoForbiddenCommands('Mobile navigation drawer', width);
    }
    for (const [destination, heading] of pages) {
      const mobile = page.locator('details.mobile-nav');
      if (width < 900 && await mobile.count() && !(await mobile.evaluate(element => element.open))) {
        await page.getByText('More', { exact: true }).press('Enter');
      }
      await page.getByRole('navigation', { name: 'Primary navigation' })
        .getByRole('button', { name: destination, exact: true }).press('Enter');
      await page.getByRole('heading', { name: heading, exact: true }).first().waitFor({ state: 'visible' });
      await page.waitForTimeout(100);
      observations.push(await auditKeyboardSequence(destination, width));
      await assertNoForbiddenCommands(destination, width);

      if (destination === 'Settings') {
        const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
        for (const name of settingsSections) {
          await tabs.getByRole('button', { name, exact: true }).press('Enter');
          await page.waitForTimeout(100);
          observations.push(await auditKeyboardSequence(`Settings → ${name}`, width));
          await assertNoForbiddenCommands(`Settings → ${name}`, width);
        }
      }
    }
  }
  return { result: 'PASS', workspace: workspace.workspaceId, viewportWidths: widths,
    surfaceViewportPairs: observations.length,
    tabStopsAcrossSurfaceViewportPairs: observations.reduce((total, item) => total + item.focusableControls, 0),
    visualReadingOrderGroups: observations.reduce((total, item) => total + item.visualOrderGroups, 0),
    visualReadingOrderControls: observations.reduce((total, item) => total + item.visualOrderControls, 0),
    mainContentGeometryGroups: observations.reduce((total, item) => total + item.mainContentGeometryGroups, 0),
    mainContentGeometryControls: observations.reduce((total, item) => total + item.mainContentGeometryControls, 0),
    mobileNavigationVisualReadingOrderPairs: mobileNavigationAuditCount,
    forbiddenCommandCount, observations };
}
