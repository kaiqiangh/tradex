import assert from 'node:assert/strict';

export async function checkThreadUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 860 });
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Workspace', exact: true }).click();
    await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill('S04 Thread Research');
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).click();
    await ui.getByRole('button', { name: 'Threads', exact: true }).click();
    await ui.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('textbox', { name: 'Thread title', exact: true }).fill('Earnings timeline');
    await ui.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
    await ui.getByRole('button', { name: 'Create Thread', exact: true }).click();
    await ui.getByRole('heading', { name: 'Earnings timeline', exact: true }).waitFor({ state: 'visible' });
    assert.ok(await ui.getByRole('button', { name: 'Earnings timeline', exact: true }).count() >= 2, 'New Thread should appear in sidebar and history immediately');
    assert.equal(await ui.getByText('Mode: RESEARCH', { exact: true }).isVisible(), true);
    assert.equal(await ui.getByText('Execution: NONE_READ_ONLY', { exact: true }).isVisible(), true);
    assert.equal(await ui.getByText('No turns have started. Send a request to begin the read-only timeline.', { exact: true }).isVisible(), true);
    observed.push('Thread create persists its title and mode/context defaults before any Turn exists.');

    await ui.getByLabel('Turn request', { exact: true }).fill('Summarize the evidence');
    await ui.getByRole('button', { name: 'Send', exact: true }).click();
    const turnStatus = ui.getByRole('status', { name: 'Turn 1 status', exact: true });
    await turnStatus.waitFor({ state: 'visible' });
    let sawRunning = (await turnStatus.innerText()) === 'RUNNING';
    for (let attempt = 0; attempt < 40 && (await turnStatus.innerText()) !== 'COMPLETED'; attempt += 1) {
      await ui.waitForTimeout(25);
      sawRunning ||= (await turnStatus.innerText()) === 'RUNNING';
    }
    assert.equal(sawRunning, true, 'Turn should expose RUNNING before completion');
    assert.equal(await ui.getByText('user message', { exact: true }).isVisible(), true);
    await ui.getByText('Read-only response for: Summarize the evidence', { exact: true }).waitFor({ state: 'visible' });
    assert.equal(await turnStatus.innerText(), 'COMPLETED');
    assert.match(await ui.getByText('Provider attempt:', { exact: false }).innerText(), /SUCCEEDED/);
    observed.push('A fake App Server handshake produces a persisted running Turn, typed user/agent timeline and completed provider attempt.');

    await tab.reload();
    await ui.getByRole('button', { name: 'Threads', exact: true }).click();
    await ui.getByRole('button', { name: 'Earnings timeline', exact: true }).last().waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Earnings timeline', exact: true }).last().click();
    await ui.getByRole('heading', { name: 'Earnings timeline', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('alert').count(), 0);
    assert.equal(await ui.getByText('Read-only response for: Summarize the evidence', { exact: true }).isVisible(), true);
    observed.push('Thread history and its selected detail survive renderer/workspace reload.');

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 860 });
      await tab.getAXState({ emit: false });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.scroll <= size.width, `Horizontal overflow at ${width}: ${JSON.stringify(size)}`);
      assert.equal(await ui.getByRole('heading', { name: 'Earnings timeline', exact: true }).isVisible(), true);
      assert.equal(await ui.getByText('Mode: RESEARCH', { exact: true }).isVisible(), true);
      observed.push(`Thread identity, context and focusable history remain visible at ${width}px.`);
    }
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
