// Run with `playwright-cli run-code --filename=tests/s33-reduced-motion-playwright.js`
// against the Rust-backed `npm run dev:browser` integration harness.
// The sentinel is local DOM state; this check never calls a provider or financial command.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });

  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  await page.waitForFunction(() => Boolean(document.querySelector('#root')?.children.length));
  const before = await page.evaluate(() => fetch('/__integration/command-log?after=0').then(response => response.json()));

  const normalMotion = await page.evaluate(() => {
    const indicator = document.createElement('div');
    indicator.id = 's33-reduced-motion-progress';
    indicator.setAttribute('role', 'progressbar');
    indicator.setAttribute('aria-label', 'Local fixture progress');
    indicator.setAttribute('aria-valuemin', '0');
    indicator.setAttribute('aria-valuemax', '100');
    indicator.setAttribute('aria-valuenow', '50');
    indicator.textContent = 'Local fixture progress 50%';
    indicator.style.cssText = 'display:block;position:fixed;top:8px;left:8px;z-index:99999;opacity:0.8;transition:opacity 2s linear;animation:s33-progress-pulse 2s infinite;';
    const main = document.querySelector('main');
    if (main) main.append(indicator);
    else document.body.append(indicator);
    const style = getComputedStyle(indicator);
    return {
      reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches,
      transitionProperty: style.transitionProperty,
      transitionDuration: style.transitionDuration,
      animationName: style.animationName,
      animationDuration: style.animationDuration,
    };
  });
  check(!normalMotion.reducedMotion, `Expected no-preference before emulation: ${JSON.stringify(normalMotion)}`);
  check(normalMotion.transitionProperty === 'opacity' && normalMotion.transitionDuration === '2s'
    && normalMotion.animationName === 's33-progress-pulse' && normalMotion.animationDuration === '2s',
    `Sentinel must have motion before reduce is enabled: ${JSON.stringify(normalMotion)}`);

  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.waitForFunction(() => matchMedia('(prefers-reduced-motion: reduce)').matches);
  const reducedMotion = await page.locator('#s33-reduced-motion-progress').evaluate(indicator => {
    const style = getComputedStyle(indicator);
    const bounds = indicator.getBoundingClientRect();
    return {
      transitionProperty: style.transitionProperty,
      transitionDuration: style.transitionDuration,
      animationName: style.animationName,
      animationDuration: style.animationDuration,
      scrollBehavior: getComputedStyle(document.documentElement).scrollBehavior,
      text: indicator.textContent,
      value: indicator.getAttribute('aria-valuenow'),
      visible: style.display !== 'none' && style.visibility !== 'hidden' && Number(style.opacity) > 0
        && bounds.width > 0 && bounds.height > 0 && bounds.left >= 0 && bounds.top >= 0
        && bounds.right <= innerWidth && bounds.bottom <= innerHeight,
    };
  });
  check(reducedMotion.transitionProperty === 'none' && reducedMotion.transitionDuration === '0s',
    `Reduced motion must remove transitions: ${JSON.stringify(reducedMotion)}`);
  check(reducedMotion.animationName === 'none' && reducedMotion.animationDuration === '0s',
    `Reduced motion must remove animations: ${JSON.stringify(reducedMotion)}`);
  check(reducedMotion.scrollBehavior === 'auto', `Reduced motion must disable smooth scrolling: ${JSON.stringify(reducedMotion)}`);
  check(reducedMotion.visible && reducedMotion.text === 'Local fixture progress 50%' && reducedMotion.value === '50',
    `Progress must remain visible and semantically available: ${JSON.stringify(reducedMotion)}`);

  const after = await page.evaluate(cursor => fetch(`/__integration/command-log?after=${cursor}`).then(response => response.json()), before.cursor);
  const isForbidden = name => name.startsWith('provider.') || name === 'account.refresh'
    || name === 'model.verify_route' || name.startsWith('data.') || name.startsWith('market.hot.')
    || name.startsWith('trade.') || name.startsWith('paper.order.') || name.startsWith('order.')
    || name.startsWith('gateway.');
  const forbidden = after.commands.filter(entry => isForbidden(entry.command));
  check(forbidden.length === 0, `Reduced-motion check issued a forbidden command: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  await page.locator('#s33-reduced-motion-progress').evaluate(indicator => indicator.remove());
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  return {
    result: 'PASS',
    normalMotion,
    reducedMotion,
    forbiddenCommands: forbidden.length,
    externalRequests: externalRequests.length,
  };
}
