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
    assert.equal(await ui.getByText('Mode: RESEARCH', { exact: true }).isVisible(), true);
    assert.equal(await ui.getByText('Execution: NONE_READ_ONLY', { exact: true }).isVisible(), true);
    assert.equal(await ui.getByText('No turns have started. Send remains unavailable until the Codex runtime slice is complete.', { exact: true }).isVisible(), true);
    observed.push('Thread create persists its title and mode/context defaults before any Turn exists.');

    await tab.reload();
    await ui.getByRole('button', { name: 'Threads', exact: true }).click();
    await ui.getByRole('button', { name: 'Earnings timeline', exact: true }).last().waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Earnings timeline', exact: true }).last().click();
    await ui.getByRole('heading', { name: 'Earnings timeline', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('alert').count(), 0);
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
